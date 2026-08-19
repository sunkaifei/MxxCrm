//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::entity::{
    admin, admin_dept_merge, admin_post_merge, dept, hr_profile_log, post,
};
use crate::modules::system::model::profile::{
    HrArchiveDetailVO, HrArchiveListVO, HrArchiveUpdateRequest,
    ProfileLogQuery, ProfileLogVO,
};
use crate::modules::system::service::profile_service;

/// HR 档案列表（分页）
/// filled：None=全部 / Some(true)=仅看资料完善 / Some(false)=仅看有缺项
pub async fn get_archive_page(
    db: &DbConn,
    keyword: Option<String>,
    filled: Option<bool>,
    page: u32,
    page_size: u32,
) -> Result<ResultPage<Vec<HrArchiveListVO>>> {
    let mut qr = admin::Entity::find()
        .filter(admin::Column::Deleted.eq(0)); // 全部账号（含超管），列表用 tag 区分

    if let Some(kw) = keyword {
        if !kw.trim().is_empty() {
            let like = format!("%{}%", kw.trim());
            qr = qr.filter(
                Condition::any()
                    .add(admin::Column::NickName.like(&like))
                    .add(admin::Column::UserName.like(&like))
                    .add(admin::Column::Mobile.like(&like))
                    .add(admin::Column::Email.like(&like)),
            );
        }
    }
    if let Some(only_filled) = filled {
        // 完善筛选：六项（身份证/银行卡/邮箱/入职日期/简历/紧急联系人）在 SQL 层先按
        // 基础四项粗筛（简历/紧急联系人数量在行内计算），行内再精确过滤
        if only_filled {
            qr = qr
                .filter(admin::Column::IdCardNo.is_not_null())
                .filter(admin::Column::BankCardNo.is_not_null())
                .filter(admin::Column::Email.is_not_null())
                .filter(admin::Column::HireDate.is_not_null());
        }
    }

    let total = qr.clone().count(db).await?;
    let list = qr
        .order_by_asc(admin::Column::Id)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;
    let mut vos = Vec::with_capacity(list.len());
    for a in list {
        let dept_names = profile_service::dept_names_pub(db, a.id).await;
        let post_names = profile_service::post_names_pub(db, a.id).await;
        let resume_count = crate::modules::system::entity::hr_resume::Entity::find()
            .filter(crate::modules::system::entity::hr_resume::Column::AdminId.eq(a.id))
            .filter(crate::modules::system::entity::hr_resume::Column::Deleted.eq(0))
            .count(db)
            .await?;
        let contact_count = crate::modules::system::entity::hr_emergency_contact::Entity::find()
            .filter(crate::modules::system::entity::hr_emergency_contact::Column::AdminId.eq(a.id))
            .filter(crate::modules::system::entity::hr_emergency_contact::Column::Deleted.eq(0))
            .count(db)
            .await?;

        // 完整度六项：身份证/银行卡/邮箱/入职日期/简历/紧急联系人
        let id_filled = a.id_card_no.as_deref().map_or(false, |s| !s.is_empty());
        let bank_filled = a.bank_card_no.as_deref().map_or(false, |s| !s.is_empty());
        let email_filled = a.email.as_deref().map_or(false, |s| !s.is_empty());
        let hire_filled = a.hire_date.is_some();
        let resume_filled = resume_count > 0;
        let contact_filled = contact_count > 0;
        let score = u32::from(id_filled)
            + u32::from(bank_filled)
            + u32::from(email_filled)
            + u32::from(hire_filled)
            + u32::from(resume_filled)
            + u32::from(contact_filled);

        // 简历/紧急联系人在行内精确过滤
        if let Some(only_filled) = filled {
            if only_filled && (!resume_filled || !contact_filled) {
                continue;
            }
        }

        vos.push(HrArchiveListVO {
            id: a.id,
            user_name: a.user_name.clone(),
            nick_name: a.nick_name.clone(),
            avatar: a.avatar.clone(),
            mobile: a.mobile.clone(),
            email: a.email.clone(),
            dept_names,
            post_names,
            hire_date: a.hire_date,
            id_locked: a.id_locked.unwrap_or(0) == 1,
            bank_locked: a.bank_locked.unwrap_or(0) == 1,
            user_type: a.user_type,
            status: a.status,
            id_filled,
            bank_filled,
            email_filled,
            hire_filled,
            resume_filled,
            contact_filled,
            resume_count: resume_count as i64,
            contact_count: contact_count as i64,
            completeness: (score * 100 / 6) as i32,
        });
    }

    Ok(ResultPage::new(vos, total as i64, page as i64, page_size as i64))
}

/// HR 档案详情（完整字段）
pub async fn get_archive_detail(db: &DbConn, admin_id: i64) -> Result<HrArchiveDetailVO> {
    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工不存在"))?;

    let dept_merges = admin_dept_merge::Entity::find()
        .filter(admin_dept_merge::Column::AdminId.eq(admin_id))
        .all(db)
        .await?;
    let mut dept_ids = Vec::new();
    let mut dept_names = Vec::new();
    for m in dept_merges {
        let did = m.dept_id.unwrap_or_default();
        if did > 0 {
            if let Some(d) = dept::Entity::find_by_id(did).one(db).await? {
                if let Some(n) = d.dept_name {
                    dept_names.push(n);
                }
            }
            dept_ids.push(did);
        }
    }

    let post_merges = admin_post_merge::Entity::find()
        .filter(admin_post_merge::Column::AdminId.eq(admin_id))
        .all(db)
        .await?;
    let mut post_ids = Vec::new();
    let mut post_names = Vec::new();
    for m in post_merges {
        let pid = m.post_id.unwrap_or_default();
        if pid > 0 {
            if let Some(p) = post::Entity::find_by_id(pid).one(db).await? {
                if let Some(n) = p.post_name {
                    post_names.push(n);
                }
            }
            post_ids.push(pid);
        }
    }

    let direct_manager_name = match a.direct_manager_id {
        Some(mid) if mid > 0 => admin::Entity::find_by_id(mid)
            .one(db)
            .await?
            .and_then(|m| m.nick_name.or(m.user_name)),
        _ => None,
    };

    let resume = crate::modules::system::service::profile_service::resume_list_pub(db, admin_id).await;
    let contacts = crate::modules::system::service::profile_service::contact_list_pub(db, admin_id).await;

    Ok(HrArchiveDetailVO {
        id: a.id,
        user_name: a.user_name.clone(),
        nick_name: a.nick_name.clone(),
        gender: a.gender,
        email: a.email.clone(),
        mobile: a.mobile.clone(),
        avatar: a.avatar.clone(),
        dept_ids,
        dept_names,
        post_ids,
        post_names,
        direct_manager_id: a.direct_manager_id,
        direct_manager_name,
        hire_date: a.hire_date,
        probation_months: a.probation_months,
        probation_ratio: a.probation_ratio.map(|d| d.to_f64_ret()),
        id_card_no: a.id_card_no.clone(),
        id_locked: a.id_locked.unwrap_or(0) == 1,
        bank_card_no: a.bank_card_no.clone(),
        bank_name: a.bank_name.clone(),
        bank_account_name: a.bank_account_name.clone(),
        bank_locked: a.bank_locked.unwrap_or(0) == 1,
        status: a.status,
        resume,
        emergency_contacts: contacts,
    })
}

/// HR 代改（逐字段写日志，事务）
pub async fn hr_update(
    db: &DbConn,
    admin_id: i64,
    req: HrArchiveUpdateRequest,
    operator_id: i64,
    operator_name: &str,
) -> Result<()> {
    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工不存在"))?;

    let mut am: admin::ActiveModel = a.clone().into();
    let mut changes: Vec<(&str, Option<String>, Option<String>)> = Vec::new();

    fn opt_s<T: ToString>(v: &Option<T>) -> Option<String> {
        v.as_ref().map(|x| x.to_string())
    }

    if let Some(v) = req.nick_name.clone() {
        if a.nick_name.as_ref() != Some(&v) {
            changes.push(("nick_name", a.nick_name.clone(), Some(v.clone())));
            am.nick_name = Set(Some(v));
        }
    }
    if let Some(v) = req.gender {
        if a.gender != Some(v) {
            changes.push(("gender", opt_s(&a.gender), Some(v.to_string())));
            am.gender = Set(Some(v));
        }
    }
    if let Some(v) = req.email.clone() {
        if a.email.as_ref() != Some(&v) {
            changes.push(("email", a.email.clone(), Some(v.clone())));
            am.email = Set(Some(v));
        }
    }
    if let Some(v) = req.mobile.clone() {
        if a.mobile.as_ref() != Some(&v) {
            changes.push(("mobile", a.mobile.clone(), Some(v.clone())));
            am.mobile = Set(Some(v));
        }
    }
    if let Some(v) = req.hire_date {
        if a.hire_date != Some(v) {
            changes.push(("hire_date", opt_s(&a.hire_date), Some(v.to_string())));
            am.hire_date = Set(Some(v));
        }
    }
    if let Some(v) = req.probation_months {
        if a.probation_months != Some(v) {
            changes.push(("probation_months", opt_s(&a.probation_months), Some(v.to_string())));
            am.probation_months = Set(Some(v));
        }
    }
    if let Some(v) = req.direct_manager_id {
        if a.direct_manager_id != Some(v) {
            changes.push(("direct_manager_id", opt_s(&a.direct_manager_id), Some(v.to_string())));
            am.direct_manager_id = Set(Some(v));
        }
    }
    if let Some(v) = req.bank_card_no.clone() {
        if a.bank_card_no.as_ref() != Some(&v) {
            changes.push(("bank_card_no", a.bank_card_no.clone(), Some(v.clone())));
            am.bank_card_no = Set(Some(v));
        }
    }
    if let Some(v) = req.bank_name.clone() {
        if a.bank_name.as_ref() != Some(&v) {
            changes.push(("bank_name", a.bank_name.clone(), Some(v.clone())));
            am.bank_name = Set(Some(v));
        }
    }
    if let Some(v) = req.bank_account_name.clone() {
        if a.bank_account_name.as_ref() != Some(&v) {
            changes.push(("bank_account_name", a.bank_account_name.clone(), Some(v.clone())));
            am.bank_account_name = Set(Some(v));
        }
    }
    if let Some(v) = req.status {
        if a.status != Some(v) {
            changes.push(("status", opt_s(&a.status), Some(v.to_string())));
            am.status = Set(Some(v));
        }
    }

    if changes.is_empty() {
        return Ok(());
    }

    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    am.update_time = Set(Some(now));
    am.update_by = Set(Some(operator_name.to_string()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    for (field, old, new) in changes {
        crate::modules::system::service::profile_service::insert_log(
            &txn, admin_id, field, old, new, 3, operator_id, Some(operator_name.to_string()),
        )
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    }
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// HR 解锁身份证/银行卡（写日志）
pub async fn hr_unlock(
    db: &DbConn,
    admin_id: i64,
    field: &str,
    operator_id: i64,
    operator_name: &str,
) -> Result<()> {
    if field != "id_card" && field != "bank" {
        return Err(Error::from("不支持的字段，仅 id_card / bank"));
    }

    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工不存在"))?;

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: admin::ActiveModel = a.into();
    let log_field = if field == "id_card" {
        am.id_locked = Set(Some(0));
        "id_locked"
    } else {
        am.bank_locked = Set(Some(0));
        "bank_locked"
    };
    am.update_by = Set(Some(operator_name.to_string()));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    crate::modules::system::service::profile_service::insert_log(
        &txn,
        admin_id,
        log_field,
        Some("1".to_string()),
        Some("0".to_string()),
        4,
        operator_id,
        Some(operator_name.to_string()),
    )
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 变更日志查询（分页）
pub async fn get_log_page(
    db: &DbConn,
    q: ProfileLogQuery,
) -> Result<ResultPage<Vec<ProfileLogVO>>> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let mut qr = hr_profile_log::Entity::find();
    if let Some(aid) = q.admin_id {
        qr = qr.filter(hr_profile_log::Column::AdminId.eq(aid));
    }
    let total = qr.clone().count(db).await?;
    let list = qr
        .order_by_desc(hr_profile_log::Column::Id)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    let vos = list
        .into_iter()
        .map(|m| ProfileLogVO {
            id: m.id,
            admin_id: m.admin_id,
            field: m.field,
            old_value: m.old_value,
            new_value: m.new_value,
            operate_type: m.operate_type,
            operator_name: m.operator_name,
            create_date: m.create_time.map(|t| t.date()),
            create_time: m.create_time.map(|t| t.time()),
        })
        .collect();

    Ok(ResultPage::new(vos, total as i64, page as i64, page_size as i64))
}

trait DecimalToF64 {
    fn to_f64_ret(&self) -> f64;
}
impl DecimalToF64 for sea_orm::prelude::Decimal {
    fn to_f64_ret(&self) -> f64 {
        use rust_decimal::prelude::ToPrimitive;
        self.to_f64().unwrap_or(0.0)
    }
}
