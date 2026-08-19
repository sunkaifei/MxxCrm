//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::{
    admin, admin_dept_merge, admin_post_merge, hr_emergency_contact, hr_resume,
    hr_profile_log,
};
use crate::modules::system::model::profile::{
    BankRequest, BasicUpdateRequest, CardVO, EmergencyContactItem, IdCardBlock,
    IdCardRequest, MyProfileVO, ResumeItem, VisibilityConfig, BasicBlock, BankBlock,
    EmployBlock,
};

/// 手机号脱敏：139****1234
pub fn mask_mobile(m: &str) -> String {
    if m.len() >= 7 {
        format!("{}****{}", &m[..3], &m[m.len() - 4..])
    } else {
        m.to_string()
    }
}

/// 身份证脱敏：前4后2
pub fn mask_id_card(s: &str) -> String {
    if s.len() >= 8 {
        format!("{}{}{}", &s[..4], "*".repeat(s.len() - 6), &s[s.len() - 2..])
    } else {
        s.to_string()
    }
}

/// 银行卡脱敏：前4后4
pub fn mask_bank_card(s: &str) -> String {
    if s.len() >= 12 {
        format!("{}****{}", &s[..4], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}

fn default_visibility() -> VisibilityConfig {
    VisibilityConfig {
        show_mobile: false,
        show_wechat: false,
        show_skills: false,
        show_birthday: false,
    }
}

/// 从 JSONB 解析公开配置（兼容空/坏数据）
fn parse_visibility(v: &Option<serde_json::Value>) -> VisibilityConfig {
    let mut c = default_visibility();
    if let Some(serde_json::Value::Object(map)) = v {
        if let Some(serde_json::Value::Bool(b)) = map.get("showMobile") {
            c.show_mobile = *b;
        }
        if let Some(serde_json::Value::Bool(b)) = map.get("showWechat") {
            c.show_wechat = *b;
        }
        if let Some(serde_json::Value::Bool(b)) = map.get("showSkills") {
            c.show_skills = *b;
        }
        if let Some(serde_json::Value::Bool(b)) = map.get("showBirthday") {
            c.show_birthday = *b;
        }
    }
    c
}

/// 本人档案聚合（脱敏输出）
pub async fn get_my_profile(db: &DbConn, admin_id: i64) -> Result<MyProfileVO> {
    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工信息不存在"))?;

    let dept_names = dept_names_of(db, admin_id).await;
    let post_names = post_names_of(db, admin_id).await;

    let direct_manager_name = match a.direct_manager_id {
        Some(mid) if mid > 0 => admin::Entity::find_by_id(mid)
            .one(db)
            .await?
            .and_then(|m| m.nick_name.or(m.user_name)),
        _ => None,
    };

    let resume = resume_list(db, admin_id).await?;
    let contacts = contact_list(db, admin_id).await?;

    let vis = parse_visibility(&a.public_profile_config);

    Ok(MyProfileVO {
        basic: BasicBlock {
            nick_name: a.nick_name.clone(),
            gender: a.gender,
            email: a.email.clone(),
            avatar: a.avatar.clone(),
            intro: a.remark.clone(),
            mobile_masked: a.mobile.as_deref().map(mask_mobile),
        },
        employ: EmployBlock {
            user_name: a.user_name.clone(),
            dept_names,
            post_names,
            direct_manager_id: a.direct_manager_id,
            direct_manager_name,
            hire_date: a.hire_date,
            probation_months: a.probation_months,
        },
        id_card: IdCardBlock {
            masked: a.id_card_no.as_deref().map(mask_id_card),
            locked: a.id_locked.unwrap_or(0) == 1,
        },
        bank: BankBlock {
            masked_card_no: a.bank_card_no.as_deref().map(mask_bank_card),
            bank_name: a.bank_name.clone(),
            masked_account_name: a.bank_account_name.clone(),
            locked: a.bank_locked.unwrap_or(0) == 1,
        },
        visibility: vis,
        resume,
        emergency_contacts: contacts,
    })
}

async fn dept_names_of<C: ConnectionTrait>(db: &C, admin_id: i64) -> Vec<String> {
    use crate::modules::system::entity::dept;
    let merges = admin_dept_merge::Entity::find()
        .filter(admin_dept_merge::Column::AdminId.eq(admin_id))
        .all(db)
        .await
        .unwrap_or_default();
    let mut names = Vec::new();
    for m in merges {
        if let Some(d) = dept::Entity::find_by_id(m.dept_id.unwrap_or_default())
            .one(db)
            .await
            .unwrap_or(None)
        {
            if let Some(n) = d.dept_name {
                names.push(n);
            }
        }
    }
    names
}

async fn post_names_of<C: ConnectionTrait>(db: &C, admin_id: i64) -> Vec<String> {
    use crate::modules::system::entity::post;
    let merges = admin_post_merge::Entity::find()
        .filter(admin_post_merge::Column::AdminId.eq(admin_id))
        .all(db)
        .await
        .unwrap_or_default();
    let mut names = Vec::new();
    for m in merges {
        if let Some(p) = post::Entity::find_by_id(m.post_id.unwrap_or_default())
            .one(db)
            .await
            .unwrap_or(None)
        {
            if let Some(n) = p.post_name {
                names.push(n);
            }
        }
    }
    names
}

/// 本人基本信息更新（白名单：昵称/性别/邮箱/简介/公开配置；雇佣事实字段一律忽略）
pub async fn update_basic(
    db: &DbConn,
    admin_id: i64,
    req: BasicUpdateRequest,
) -> Result<()> {
    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工信息不存在"))?;

    let mut am: admin::ActiveModel = a.clone().into();
    let mut changed = false;
    if req.nick_name.is_some() {
        am.nick_name = Set(req.nick_name.clone());
        changed = true;
    }
    if req.gender.is_some() {
        am.gender = Set(req.gender);
        changed = true;
    }
    if req.email.is_some() {
        am.email = Set(req.email.clone());
        changed = true;
    }
    if req.intro.is_some() {
        am.remark = Set(req.intro.clone());
        changed = true;
    }
    if let Some(vis) = req.visibility {
        let old = parse_visibility(&a.public_profile_config);
        am.public_profile_config = Set(Some(serde_json::json!({
            "showMobile": vis.show_mobile,
            "showWechat": vis.show_wechat,
            "showSkills": vis.show_skills,
            "showBirthday": vis.show_birthday,
            // 保留名片扩展数据
            "wechat": old_wechat(&a),
            "birthday": old_birthday(&a),
        })));
        changed = true;
        if old.show_mobile != vis.show_mobile {
            write_log(db, admin_id, "visibility.showMobile",
                Some(old.show_mobile.to_string()), Some(vis.show_mobile.to_string()),
                2, admin_id, None).await?;
        }
    }
    // wechat / birthday 存入 public_profile_config（名片专属数据）
    if req.wechat.is_some() || req.birthday.is_some() {
        let mut cfg = a.public_profile_config.clone().unwrap_or(serde_json::json!({}));
        if let serde_json::Value::Object(ref mut map) = cfg {
            if let Some(w) = req.wechat {
                map.insert("wechat".into(), serde_json::Value::String(w));
            }
            if let Some(b) = req.birthday {
                map.insert("birthday".into(), serde_json::Value::String(b.to_string()));
            }
        }
        am.public_profile_config = Set(Some(cfg));
        changed = true;
    }

    if changed {
        am.update_time = Set(Some(Utc::now().naive_utc()));
        am.update(db).await.map_err(|e| Error::from(e.to_string()))?;
    }
    Ok(())
}

fn old_wechat(a: &admin::Model) -> serde_json::Value {
    a.public_profile_config
        .as_ref()
        .and_then(|v| v.get("wechat").cloned())
        .unwrap_or(serde_json::Value::Null)
}

fn old_birthday(a: &admin::Model) -> serde_json::Value {
    a.public_profile_config
        .as_ref()
        .and_then(|v| v.get("birthday").cloned())
        .unwrap_or(serde_json::Value::Null)
}

/// 身份证首填（事务：写入+锁定+日志）
pub async fn submit_id_card(db: &DbConn, admin_id: i64, req: IdCardRequest) -> Result<()> {
    let no = req.id_card_no.trim().to_string();
    if no.len() != 18 {
        return Err(Error::from("身份证格式不正确"));
    }
    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工信息不存在"))?;
    if a.id_locked.unwrap_or(0) == 1 {
        return Err(Error::from("身份证已锁定，请联系 HR 修改"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: admin::ActiveModel = a.into();
    am.id_card_no = Set(Some(no.clone()));
    am.id_locked = Set(Some(1));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;

    insert_log(&txn, admin_id, "id_card_no", None, Some(no), 1, admin_id, None)
        .await?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 工资卡首填（事务：写入+锁定+日志）
pub async fn submit_bank(db: &DbConn, admin_id: i64, req: BankRequest) -> Result<()> {
    let no = req.bank_card_no.trim().to_string();
    if no.len() < 12 || no.len() > 25 {
        return Err(Error::from("银行卡号格式不正确"));
    }
    let a = admin::Entity::find_by_id(admin_id)
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工信息不存在"))?;
    if a.bank_locked.unwrap_or(0) == 1 {
        return Err(Error::from("工资卡已锁定，请联系 HR 修改"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: admin::ActiveModel = a.into();
    am.bank_card_no = Set(Some(no.clone()));
    if req.bank_name.is_some() {
        am.bank_name = Set(req.bank_name.clone());
    }
    if req.bank_account_name.is_some() {
        am.bank_account_name = Set(req.bank_account_name.clone());
    }
    am.bank_locked = Set(Some(1));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;

    insert_log(&txn, admin_id, "bank_card_no", None, Some(no), 1, admin_id, None)
        .await?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 简历列表
pub async fn resume_list(db: &DbConn, admin_id: i64) -> Result<Vec<ResumeItem>> {
    let list = hr_resume::Entity::find()
        .filter(hr_resume::Column::AdminId.eq(admin_id))
        .filter(hr_resume::Column::Deleted.eq(0))
        .order_by_asc(hr_resume::Column::Kind)
        .order_by_desc(hr_resume::Column::StartDate)
        .all(db)
        .await?;
    Ok(list
        .into_iter()
        .map(|m| ResumeItem {
            id: Some(m.id),
            kind: m.kind.unwrap_or(0),
            title: m.title,
            org: m.org,
            start_date: m.start_date,
            end_date: m.end_date,
            remark: m.remark,
            is_public: m.is_public,
        })
        .collect())
}

/// 简历新增（事务）
pub async fn resume_save(db: &DbConn, admin_id: i64, item: ResumeItem, op: &str) -> Result<i64> {
    if item.kind < 1 || item.kind > 3 {
        return Err(Error::from("简历条目类型不合法"));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let am = hr_resume::ActiveModel {
        admin_id: Set(admin_id),
        kind: Set(Some(item.kind)),
        title: Set(item.title.clone()),
        org: Set(item.org.clone()),
        start_date: Set(item.start_date),
        end_date: Set(item.end_date),
        remark: Set(item.remark.clone()),
        is_public: Set(Some(item.is_public.unwrap_or(0))),
        deleted: Set(Some(0)),
        create_by: Set(Some(op.to_string())),
        create_time: Set(Some(Utc::now().naive_utc())),
        ..Default::default()
    };
    let inserted = am.insert(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(inserted.id)
}

/// 简历修改（校验归属，事务）
pub async fn resume_update(db: &DbConn, admin_id: i64, item: ResumeItem, op: &str) -> Result<()> {
    let id = item.id.ok_or_else(|| Error::from("条目ID不能为空"))?;
    let exist = hr_resume::Entity::find_by_id(id)
        .filter(hr_resume::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("简历条目不存在"))?;
    if exist.admin_id != admin_id {
        return Err(Error::from("无权修改该条目"));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: hr_resume::ActiveModel = exist.into();
    am.kind = Set(Some(item.kind));
    am.title = Set(item.title);
    am.org = Set(item.org);
    am.start_date = Set(item.start_date);
    am.end_date = Set(item.end_date);
    am.remark = Set(item.remark);
    am.is_public = Set(Some(item.is_public.unwrap_or(0)));
    am.update_by = Set(Some(op.to_string()));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 简历删除（软删，事务）
pub async fn resume_delete(db: &DbConn, admin_id: i64, id: i64) -> Result<()> {
    let exist = hr_resume::Entity::find_by_id(id)
        .filter(hr_resume::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("简历条目不存在"))?;
    if exist.admin_id != admin_id {
        return Err(Error::from("无权删除该条目"));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: hr_resume::ActiveModel = exist.into();
    am.deleted = Set(Some(1));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 紧急联系人列表
pub async fn contact_list(db: &DbConn, admin_id: i64) -> Result<Vec<EmergencyContactItem>> {
    let list = hr_emergency_contact::Entity::find()
        .filter(hr_emergency_contact::Column::AdminId.eq(admin_id))
        .filter(hr_emergency_contact::Column::Deleted.eq(0))
        .order_by_asc(hr_emergency_contact::Column::Sort)
        .all(db)
        .await?;
    Ok(list
        .into_iter()
        .map(|m| EmergencyContactItem {
            id: Some(m.id),
            name: m.name.unwrap_or_default(),
            relation: m.relation,
            mobile: m.mobile.unwrap_or_default(),
            sort: m.sort,
        })
        .collect())
}

const MAX_CONTACTS: i64 = 3;

/// 紧急联系人新增（上限3条，事务）
pub async fn contact_save(
    db: &DbConn,
    admin_id: i64,
    item: EmergencyContactItem,
    op: &str,
) -> Result<i64> {
    if item.name.trim().is_empty() || item.mobile.trim().is_empty() {
        return Err(Error::from("姓名与电话不能为空"));
    }
    let count = hr_emergency_contact::Entity::find()
        .filter(hr_emergency_contact::Column::AdminId.eq(admin_id))
        .filter(hr_emergency_contact::Column::Deleted.eq(0))
        .count(db)
        .await?;
    if count as i64 >= MAX_CONTACTS {
        return Err(Error::from("紧急联系人最多 3 条"));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let am = hr_emergency_contact::ActiveModel {
        admin_id: Set(admin_id),
        name: Set(Some(item.name)),
        relation: Set(item.relation),
        mobile: Set(Some(item.mobile)),
        sort: Set(item.sort.or(Some(count as i32 + 1))),
        deleted: Set(Some(0)),
        create_by: Set(Some(op.to_string())),
        create_time: Set(Some(Utc::now().naive_utc())),
        ..Default::default()
    };
    let inserted = am.insert(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(inserted.id)
}

/// 紧急联系人修改（校验归属，事务）
pub async fn contact_update(
    db: &DbConn,
    admin_id: i64,
    item: EmergencyContactItem,
    op: &str,
) -> Result<()> {
    let id = item.id.ok_or_else(|| Error::from("条目ID不能为空"))?;
    let exist = hr_emergency_contact::Entity::find_by_id(id)
        .filter(hr_emergency_contact::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("联系人不存在"))?;
    if exist.admin_id != admin_id {
        return Err(Error::from("无权修改该联系人"));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: hr_emergency_contact::ActiveModel = exist.into();
    am.name = Set(Some(item.name));
    am.relation = Set(item.relation);
    am.mobile = Set(Some(item.mobile));
    am.sort = Set(item.sort);
    am.update_by = Set(Some(op.to_string()));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 紧急联系人删除（软删，事务）
pub async fn contact_delete(db: &DbConn, admin_id: i64, id: i64) -> Result<()> {
    let exist = hr_emergency_contact::Entity::find_by_id(id)
        .filter(hr_emergency_contact::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("联系人不存在"))?;
    if exist.admin_id != admin_id {
        return Err(Error::from("无权删除该联系人"));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let mut am: hr_emergency_contact::ActiveModel = exist.into();
    am.deleted = Set(Some(1));
    am.update_time = Set(Some(Utc::now().naive_utc()));
    am.update(&txn).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 同事名片（隐私档字段在 SQL 层即不查询）
pub async fn get_card(db: &DbConn, target_id: i64) -> Result<CardVO> {
    let a = admin::Entity::find_by_id(target_id)
        .filter(admin::Column::Deleted.eq(0))
        .filter(admin::Column::Status.eq(1))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("目标员工不存在或已停用"))?;

    let vis = parse_visibility(&a.public_profile_config);
    let dept_names = dept_names_of(db, target_id).await;
    let post_names = post_names_of(db, target_id).await;
    let direct_manager_name = match a.direct_manager_id {
        Some(mid) if mid > 0 => admin::Entity::find_by_id(mid)
            .one(db)
            .await?
            .and_then(|m| m.nick_name.or(m.user_name)),
        _ => None,
    };

    // 授权档：手机号
    let mobile = if vis.show_mobile { a.mobile.clone() } else { None };
    // 授权档：微信（存于公开配置 JSON）
    let wechat = if vis.show_wechat {
        a.public_profile_config
            .as_ref()
            .and_then(|v| v.get("wechat"))
            .and_then(|w| w.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };
    // 授权档：技能标签（公开证书标题）
    let skills = if vis.show_skills {
        hr_resume::Entity::find()
            .filter(hr_resume::Column::AdminId.eq(target_id))
            .filter(hr_resume::Column::Deleted.eq(0))
            .filter(hr_resume::Column::Kind.eq(3))
            .filter(hr_resume::Column::IsPublic.eq(1))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|r| r.title)
            .collect()
    } else {
        Vec::new()
    };
    // 授权档：生日（仅月日）
    let birthday: Option<NaiveDate> = if vis.show_birthday {
        a.public_profile_config
            .as_ref()
            .and_then(|v| v.get("birthday"))
            .and_then(|b| b.as_str())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    } else {
        None
    };

    Ok(CardVO {
        admin_id: target_id,
        nick_name: a.nick_name.clone(),
        avatar: a.avatar.clone(),
        dept_names,
        post_names,
        direct_manager_name,
        email: a.email.clone(),
        intro: a.remark.clone(),
        mobile,
        wechat,
        skills,
        birthday,
        online: false,
    })
}

/// 写变更日志（独立小事务，best-effort 之外仍保证准确）
pub async fn write_log(
    db: &DbConn,
    admin_id: i64,
    field: &str,
    old: Option<String>,
    new: Option<String>,
    operate_type: i32,
    operator_id: i64,
    operator_name: Option<String>,
) -> Result<()> {
    insert_log(db, admin_id, field, old, new, operate_type, operator_id, operator_name)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

pub async fn insert_log<C: ConnectionTrait>(
    db: &C,
    admin_id: i64,
    field: &str,
    old: Option<String>,
    new: Option<String>,
    operate_type: i32,
    operator_id: i64,
    operator_name: Option<String>,
) -> std::result::Result<(), DbErr> {
    let am = hr_profile_log::ActiveModel {
        admin_id: Set(admin_id),
        field: Set(Some(field.to_string())),
        old_value: Set(old),
        new_value: Set(new),
        operate_type: Set(Some(operate_type)),
        operator_id: Set(Some(operator_id)),
        operator_name: Set(operator_name),
        create_time: Set(Some(Utc::now().naive_utc())),
        ..Default::default()
    };
    hr_profile_log::Entity::insert(am).exec(db).await?;
    Ok(())
}

/// 公开包装：部门名列表（供 hr_archive_service 复用）
pub async fn dept_names_pub(db: &DbConn, admin_id: i64) -> Vec<String> {
    dept_names_of(db, admin_id).await
}

/// 公开包装：岗位名列表
pub async fn post_names_pub(db: &DbConn, admin_id: i64) -> Vec<String> {
    post_names_of(db, admin_id).await
}

/// 公开包装：简历列表
pub async fn resume_list_pub(db: &DbConn, admin_id: i64) -> Vec<ResumeItem> {
    resume_list(db, admin_id).await.unwrap_or_default()
}

/// 公开包装：紧急联系人列表
pub async fn contact_list_pub(db: &DbConn, admin_id: i64) -> Vec<EmergencyContactItem> {
    contact_list(db, admin_id).await.unwrap_or_default()
}
