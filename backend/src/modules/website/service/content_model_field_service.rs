//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, EntityTrait, QueryFilter, ColumnTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::website::model::content_model_field::{ListQuery, PageWhere, FieldDetailVO, FieldListVO, ContentModelFieldModel, FieldSaveDTO};
use crate::modules::website::entity::content_model;
use crate::modules::website::service::dynamic_table_service::DynamicTableService;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 同步字段到动态物理表（T-P0.2）：
/// 取模型编码后，对 `mxx_model_{code}` 执行 ADD COLUMN IF NOT EXISTS。
/// 勾选「唯一」时同步创建唯一索引。
/// 表不存在 / 加列失败不阻断字段记录写入（记 warning）。
async fn sync_column_to_dynamic_table(db: &DbConn, model_id: i64, dto: &FieldSaveDTO) {
    let name = match dto.field_name.as_deref() {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return,
    };
    let ft = dto.field_type.unwrap_or(1);
    if let Ok(Some(model)) = content_model::Entity::find_by_id(model_id)
        .filter(content_model::Column::Deleted.eq(0))
        .one(db)
        .await
    {
        if let Some(code) = model.model_code {
            if let Err(e) = DynamicTableService::add_column_if_not_exists(db, &code, &name, ft).await {
                log::warn!("[content_model_field] 动态表加列失败: {:?}", e);
            }
            // 唯一约束：勾选「唯一」时创建唯一索引（已有重复数据会失败并记日志）
            if dto.is_unique.unwrap_or(0) == 1 {
                if let Err(e) =
                    DynamicTableService::add_unique_index_if_not_exists(db, &code, &name).await
                {
                    log::warn!("[content_model_field] 唯一索引创建失败: {:?}", e);
                }
            }
        }
    }
}

/// 可搜索字段白名单查询共用的「启用」条件：status=1 或 NULL（旧数据未设状态视为启用）
pub fn status_active_condition() -> sea_orm::Condition {
    use crate::modules::website::entity::content_model_field;
    sea_orm::Condition::any()
        .add(content_model_field::Column::Status.eq(1))
        .add(content_model_field::Column::Status.is_null())
}

/// 保留字段校验：id/title/status 等系统列在建模时自动生成，
/// 用户手动创建同名字段会与固定列重名（数据落在固定列上，语义混乱），必须禁止以防搞错
fn validate_reserved_name(name: &str) -> Result<()> {
    if DynamicTableService::is_fixed_column(name) {
        return Err(Error::from(format!(
            "「{}」是系统保留字段（创建模型时已自动生成），无需也不能手动添加",
            name
        )));
    }
    Ok(())
}

pub async fn insert(db: &DbConn, form_data: &FieldSaveDTO) -> Result<i64> {
    let model_id = form_data.model_id.ok_or_else(|| Error::from("模型ID不能为空"))?;
    // T-P1.4：同模型内字段名唯一校验 + 保留字段校验
    if let Some(name) = form_data.field_name.as_deref() {
        validate_reserved_name(name)?;
        let existing = ContentModelFieldModel::find_by_model_id(db, &Some(model_id)).await?;
        if existing.iter().any(|f| f.field_name.as_deref() == Some(name)) {
            return Err(Error::from(format!("字段名「{}」在该模型下已存在", name)));
        }
    }
    let result = ContentModelFieldModel::insert(db, form_data).await?;
    if result > 0 {
        // status 未传时兜底为启用（1），避免 NULL 行被「启用」过滤排除
        if form_data.status.is_none() {
            use crate::modules::website::entity::content_model_field;
            use sea_orm::EntityTrait;
            let _ = content_model_field::Entity::update_many()
                .col_expr(content_model_field::Column::Status, sea_orm::sea_query::Expr::value(1))
                .filter(content_model_field::Column::Id.eq(result))
                .exec(db)
                .await;
        }
        sync_column_to_dynamic_table(db, model_id, form_data).await;
    }
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = ContentModelFieldModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, form_data: &FieldSaveDTO) -> Result<i64> {
    // T-P1.4：更新时字段名唯一校验（排除自身）+ 保留字段校验
    if let (Some(mid), Some(name), Some(self_id)) =
        (form_data.model_id, form_data.field_name.as_deref(), form_data.id)
    {
        validate_reserved_name(name)?;
        let existing = ContentModelFieldModel::find_by_model_id(db, &Some(mid)).await?;
        if existing
            .iter()
            .any(|f| f.id != self_id && f.field_name.as_deref() == Some(name))
        {
            return Err(Error::from(format!("字段名「{}」在该模型下已存在", name)));
        }
    }
    let result = ContentModelFieldModel::update_by_id(&db, &form_data.id, form_data).await?;
    if result > 0 {
        if let Some(mid) = form_data.model_id {
            sync_column_to_dynamic_table(db, mid, form_data).await;
        }
    }
    Ok(result)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<FieldDetailVO> {
    let result = ContentModelFieldModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!("{}={}", "模型字段不存在，id".to_string(), &id.unwrap_or_default()))
    })?;
    let result = FieldDetailVO::from(result);
    Ok(result)
}

/// 根据模型ID查询所有字段
pub async fn get_by_model_id(db: &DbConn, model_id: &Option<i64>) -> Result<Vec<FieldListVO>> {
    let list = ContentModelFieldModel::find_by_model_id(&db, model_id).await?;
    let list_data: Vec<FieldListVO> = list.into_iter().map(|item| FieldListVO::from(item)).collect();
    Ok(list_data)
}

pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<FieldListVO>>> {
    let select_where = PageWhere {
        model_id: query.model_id,
        field_name: query.field_name,
        status: query.status,
    };
    let select_where = select_where.format();
    let page_num = std::cmp::max(query.page_num.unwrap_or(1), 1);
    let (list, _num_pages) = ContentModelFieldModel::select_in_page(&db, page_num - 1, query.page_size.unwrap_or(10), select_where.clone()).await?;
    let list_data: Vec<FieldListVO> = list.into_iter().map(|item| FieldListVO::from(item)).collect();
    let count = ContentModelFieldModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}
