//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! CRM 回收站服务：软删数据的统一查看 / 还原 / 彻底删除 / 超期定时清理。
//! 设计规格见 docs/CRM数据删除与作废策略-规划方案.md 6.5：
//! - 可见性：普通用户仅见自己删除的（delete_by = 当前用户），超管见全部
//! - 还原：deleted 1→0 并清空 delete_by / delete_time，本人或管理员可操作，不做查重
//! - 彻底删除：仅超管，物理 DELETE，级联清理已软删子记录（避免孤儿数据）
//! - 定时清理：删除 delete_time 超过保留期（30 天）的数据

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::recycle::{RecycleItemVO, RecycleListQuery};
use crate::modules::crm::service::delete_guard_service::{
    is_manager, is_super_admin, RECYCLE_RETENTION_DAYS,
};
use crate::modules::system::entity::admin::{self, Entity as Admin};
use chrono::{Duration, NaiveDateTime};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, EntityTrait, QueryFilter, Statement, TransactionTrait,
    Value,
};

/// 支持回收站的业务模块 -> 物理表名（白名单，防 SQL 注入）
const MODULE_TABLES: &[(&str, &str)] = &[
    ("customer", "mxx_crm_customer"),
    ("opportunity", "mxx_crm_opportunity"),
    ("followup", "mxx_crm_followup"),
    ("contact", "mxx_crm_contact"),
    ("lead", "mxx_crm_lead"),
];

/// 五表软删数据 UNION 视图（静态 SQL，无外部参数）
const UNION_SQL: &str = "SELECT id, 'customer' AS module, '客户' AS module_label, COALESCE(NULLIF(company_name, ''), '未命名客户') AS title, delete_by, create_time, delete_time FROM mxx_crm_customer WHERE deleted = 1 \
    UNION ALL SELECT id, 'opportunity', '商机', COALESCE(NULLIF(title, ''), '未命名商机'), delete_by, create_time, delete_time FROM mxx_crm_opportunity WHERE deleted = 1 \
    UNION ALL SELECT id, 'followup', '跟进', COALESCE(NULLIF(content, ''), '跟进记录'), delete_by, create_time, delete_time FROM mxx_crm_followup WHERE deleted = 1 \
    UNION ALL SELECT id, 'contact', '联系人', COALESCE(NULLIF(name, ''), '未命名联系人'), delete_by, create_time, delete_time FROM mxx_crm_contact WHERE deleted = 1 \
    UNION ALL SELECT id, 'lead', '线索', COALESCE(NULLIF(company_name, ''), NULLIF(contact_name, ''), '未命名线索'), delete_by, create_time, delete_time FROM mxx_crm_lead WHERE deleted = 1";

/// 模块中文名（审计摘要使用）
pub fn module_label(module: &str) -> &'static str {
    match module {
        "customer" => "客户",
        "opportunity" => "商机",
        "followup" => "跟进",
        "contact" => "联系人",
        "lead" => "线索",
        _ => "数据",
    }
}

/// 模块白名单校验，返回物理表名
fn resolve_table(module: &str) -> Result<&'static str> {
    MODULE_TABLES
        .iter()
        .find(|(m, _)| *m == module)
        .map(|(_, t)| *t)
        .ok_or_else(|| Error::from("无效的数据模块"))
}

/// 执行单参数 SQL（DELETE），返回受影响行数
async fn exec_sql<C: ConnectionTrait>(conn: &C, sql: &str, value: Value) -> Result<u64> {
    let stmt = Statement::from_sql_and_values(conn.get_database_backend(), sql, vec![value]);
    let res = conn
        .execute_raw(stmt)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(res.rows_affected())
}

/// 回收站分页列表：UNION 五表软删数据，普通用户仅见自己删除的，超管见全部
pub async fn list(
    db: &DbConn,
    query: &RecycleListQuery,
    current_user_id: i64,
) -> Result<ResultPage<Vec<RecycleItemVO>>> {
    let is_admin = is_super_admin(db, current_user_id).await?;

    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    let mut idx = 1;

    if let Some(m) = query.module.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        resolve_table(m)?;
        conditions.push(format!("t.module = ${}", idx));
        values.push(m.to_string().into());
        idx += 1;
    }
    if let Some(kw) = query.keywords.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        conditions.push(format!("t.title LIKE ${}", idx));
        values.push(format!("%{}%", kw).into());
        idx += 1;
    }
    if !is_admin {
        conditions.push(format!("t.delete_by = ${}", idx));
        values.push(current_user_id.into());
        idx += 1;
    }
    let where_clause = if conditions.is_empty() {
        "1 = 1".to_string()
    } else {
        conditions.join(" AND ")
    };

    let count_sql = format!(
        "SELECT COUNT(*) AS total FROM ({}) t WHERE {}",
        UNION_SQL, where_clause
    );
    let count_stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        &count_sql,
        values.clone(),
    );
    let total: i64 = db
        .query_one_raw(count_stmt)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .and_then(|r| r.try_get::<i64>("", "total").ok())
        .unwrap_or(0);

    let page_num = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);
    let offset = (page_num - 1) * page_size;

    let list_sql = format!(
        "SELECT t.id, t.module, t.module_label, t.title, t.delete_by, t.create_time, t.delete_time FROM ({}) t WHERE {} ORDER BY t.delete_time DESC LIMIT ${} OFFSET ${}",
        UNION_SQL, where_clause, idx, idx + 1
    );
    let mut list_values = values;
    list_values.push(page_size.into());
    list_values.push(offset.into());
    let list_stmt = Statement::from_sql_and_values(db.get_database_backend(), &list_sql, list_values);
    let rows = db
        .query_all_raw(list_stmt)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut items: Vec<RecycleItemVO> = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(RecycleItemVO {
            id: row.try_get("", "id").unwrap_or_default(),
            module: row.try_get("", "module").unwrap_or_default(),
            module_label: row.try_get("", "module_label").unwrap_or_default(),
            title: row.try_get("", "title").unwrap_or_default(),
            delete_by: row.try_get("", "delete_by").unwrap_or(None),
            delete_by_name: None,
            create_time: row.try_get("", "create_time").unwrap_or(None),
            delete_time: row.try_get("", "delete_time").unwrap_or(None),
        });
    }

    let mut admin_ids: Vec<i64> = items.iter().filter_map(|it| it.delete_by).collect();
    admin_ids.sort_unstable();
    admin_ids.dedup();
    if !admin_ids.is_empty() {
        let admins = Admin::find()
            .filter(admin::Column::Id.is_in(admin_ids))
            .filter(admin::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let name_map: std::collections::HashMap<i64, String> = admins
            .into_iter()
            .map(|a| {
                let name = a
                    .nick_name
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .or(a.user_name.clone())
                    .unwrap_or_else(|| format!("用户{}", a.id));
                (a.id, name)
            })
            .collect();
        for it in items.iter_mut() {
            if let Some(uid) = it.delete_by {
                it.delete_by_name = name_map.get(&uid).cloned();
            }
        }
    }

    Ok(ResultPage::new(items, total, page_num, page_size))
}

/// 还原：deleted 1→0 并清空 delete_by / delete_time。
/// 权限：本人可还原自己删的，管理员（超管或持有 crm:recycle:restore 权限码）可还原任何人的；不做查重（G2）。
pub async fn restore(db: &DbConn, module: &str, id: i64, current_user_id: i64) -> Result<()> {
    let table = resolve_table(module)?;

    let chk_sql = format!("SELECT deleted, delete_by FROM \"{}\" WHERE id = $1", table);
    let row = db
        .query_one_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            &chk_sql,
            vec![id.into()],
        ))
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("回收站中不存在该记录"))?;
    let deleted: i32 = row.try_get("", "deleted").unwrap_or(0);
    if deleted != 1 {
        return Err(Error::from("该记录不在回收站中"));
    }
    let delete_by: Option<i64> = row.try_get("", "delete_by").unwrap_or(None);
    if delete_by != Some(current_user_id) && !is_manager(db, current_user_id, "crm:recycle:restore").await? {
        return Err(Error::from("只能还原自己删除的数据"));
    }

    let upd_sql = format!(
        "UPDATE \"{}\" SET deleted = 0, delete_by = NULL, delete_time = NULL WHERE id = $1 AND deleted = 1",
        table
    );
    let affected = exec_sql(db, &upd_sql, id.into()).await?;
    if affected == 0 {
        return Err(Error::from("还原失败，请刷新后重试"));
    }
    Ok(())
}

/// 彻底删除：仅超管；物理 DELETE，并按模块级联清理已软删子记录与标签关联。
pub async fn purge(db: &DbConn, module: &str, id: i64, current_user_id: i64) -> Result<()> {
    if !is_super_admin(db, current_user_id).await? {
        return Err(Error::from("仅超级管理员可彻底删除"));
    }
    let table = resolve_table(module)?;

    let chk_sql = format!("SELECT deleted FROM \"{}\" WHERE id = $1", table);
    let row = db
        .query_one_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            &chk_sql,
            vec![id.into()],
        ))
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("回收站中不存在该记录"))?;
    let deleted: i32 = row.try_get("", "deleted").unwrap_or(0);
    if deleted != 1 {
        return Err(Error::from("该记录不在回收站中"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    match module {
        "customer" => {
            exec_sql(&txn, "DELETE FROM mxx_system_tag_merge WHERE entity_type = 'customer' AND entity_id = $1", id.into()).await?;
            exec_sql(&txn, "DELETE FROM mxx_crm_contact WHERE customer_id = $1 AND deleted = 1", id.into()).await?;
            exec_sql(&txn, "DELETE FROM mxx_crm_followup WHERE customer_id = $1 AND deleted = 1", id.into()).await?;
            exec_sql(&txn, "DELETE FROM mxx_crm_opportunity WHERE customer_id = $1 AND deleted = 1", id.into()).await?;
        }
        "lead" => {
            exec_sql(&txn, "DELETE FROM mxx_system_tag_merge WHERE entity_type = 'lead' AND entity_id = $1", id.into()).await?;
            exec_sql(&txn, "DELETE FROM mxx_crm_followup WHERE lead_id = $1 AND deleted = 1", id.into()).await?;
        }
        "opportunity" => {
            exec_sql(&txn, "DELETE FROM mxx_crm_followup WHERE opportunity_id = $1 AND deleted = 1", id.into()).await?;
        }
        "contact" => {
            exec_sql(&txn, "DELETE FROM mxx_crm_customer_contact_merge WHERE contact_id = $1", id.into()).await?;
        }
        "followup" => {}
        _ => return Err(Error::from("无效的数据模块")),
    }
    let del_sql = format!("DELETE FROM \"{}\" WHERE id = $1 AND deleted = 1", table);
    let affected = exec_sql(&txn, &del_sql, id.into()).await?;
    if affected == 0 {
        return Err(Error::from("彻底删除失败，请刷新后重试"));
    }
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 定时清理入口（scheduler 处理器 recycle_purge 调用）：
/// 物理删除五表 delete_time 超过保留期（30 天）的软删数据，并同步清理对应标签关联。
pub async fn purge_expired(db: &DbConn) -> Result<u64> {
    let cutoff: NaiveDateTime =
        chrono::Local::now().naive_local() - Duration::days(RECYCLE_RETENTION_DAYS);

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut total: u64 = 0;
    // 先清理将被物理删除的 lead / customer 的标签关联（避免孤儿标签）
    total += exec_sql(
        &txn,
        "DELETE FROM mxx_system_tag_merge WHERE entity_type = 'lead' AND entity_id IN (SELECT id FROM mxx_crm_lead WHERE deleted = 1 AND delete_time < $1)",
        cutoff.into(),
    )
    .await?;
    total += exec_sql(
        &txn,
        "DELETE FROM mxx_system_tag_merge WHERE entity_type = 'customer' AND entity_id IN (SELECT id FROM mxx_crm_customer WHERE deleted = 1 AND delete_time < $1)",
        cutoff.into(),
    )
    .await?;
    for (_, table) in MODULE_TABLES {
        let sql = format!("DELETE FROM \"{}\" WHERE deleted = 1 AND delete_time < $1", table);
        total += exec_sql(&txn, &sql, cutoff.into()).await?;
    }
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(total)
}
