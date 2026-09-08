//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait,
};

use crate::modules::system::entity::admin_preference::{ActiveModel, Column, Entity};
use crate::modules::system::model::admin_preference::{
    AdminPreferenceVO, QuickNavItem, SavePreferenceRequest,
};

/// 快捷导航偏好键
pub const PREF_KEY_QUICK_NAV: &str = "quick_nav";

/// 销售简易模式偏好键
pub const PREF_KEY_SALE_SIMPLE_MODE: &str = "sale_simple_mode";

/// 查询用户偏好（按 admin_id + pref_key）
pub async fn find_by_admin_and_key(
    db: &DatabaseConnection,
    admin_id: i64,
    pref_key: &str,
) -> Result<Option<AdminPreferenceVO>, sea_orm::DbErr> {
    let m = Entity::find()
        .filter(Column::AdminId.eq(admin_id))
        .filter(Column::PrefKey.eq(pref_key))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?;
    Ok(m.map(|m| AdminPreferenceVO {
        id: m.id,
        admin_id: m.admin_id,
        pref_key: m.pref_key,
        pref_value: m.pref_value,
    }))
}

/// 保存偏好（upsert：存在则更新，不存在则插入，用事务）
pub async fn save(
    db: &DatabaseConnection,
    admin_id: i64,
    req: &SavePreferenceRequest,
) -> Result<i64, sea_orm::DbErr> {
    let now = Local::now().naive_local();
    let existing = Entity::find()
        .filter(Column::AdminId.eq(admin_id))
        .filter(Column::PrefKey.eq(&req.pref_key))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 闭包要求 'static，先将引用值 clone 为 owned
    let pref_key = req.pref_key.clone();
    let pref_value = req.pref_value.clone();

    let result_id = db
        .transaction::<_, i64, sea_orm::DbErr>(|txn| {
            Box::pin(async move {
                let id = if let Some(m) = existing {
                    // 更新
                    let mut active: ActiveModel = m.into();
                    active.pref_value = Set(Some(pref_value.clone()));
                    active.update_time = Set(Some(now));
                    let updated = active.update(txn).await?;
                    updated.id
                } else {
                    // 插入
                    let active = ActiveModel {
                        admin_id: Set(admin_id),
                        pref_key: Set(Some(pref_key.clone())),
                        pref_value: Set(Some(pref_value.clone())),
                        create_time: Set(Some(now)),
                        update_time: Set(Some(now)),
                        deleted: Set(Some(0)),
                        ..Default::default()
                    };
                    let inserted = Entity::insert(active).exec(txn).await?;
                    inserted.last_insert_id
                };
                Ok(id)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(err) => err,
            sea_orm::TransactionError::Transaction(err) => err,
        })?;
    Ok(result_id)
}

/// 获取快捷导航配置（无配置返回空 Vec）
pub async fn find_quick_nav(
    db: &DatabaseConnection,
    admin_id: i64,
) -> Result<Vec<QuickNavItem>, sea_orm::DbErr> {
    let pref = find_by_admin_and_key(db, admin_id, PREF_KEY_QUICK_NAV).await?;
    match pref {
        Some(p) => match p.pref_value {
            Some(value) => {
                let items: Vec<QuickNavItem> =
                    serde_json::from_value(value).unwrap_or_default();
                Ok(items)
            }
            None => Ok(Vec::new()),
        },
        None => Ok(Vec::new()),
    }
}

/// 保存快捷导航配置
pub async fn save_quick_nav(
    db: &DatabaseConnection,
    admin_id: i64,
    items: &Vec<QuickNavItem>,
) -> Result<i64, sea_orm::DbErr> {
    let value = serde_json::to_value(items).unwrap_or(serde_json::Value::Array(vec![]));
    let req = SavePreferenceRequest {
        pref_key: PREF_KEY_QUICK_NAV.to_string(),
        pref_value: value,
    };
    save(db, admin_id, &req).await
}

/// 获取销售简易模式开关（无配置返回 false）
pub async fn find_sale_simple_mode(
    db: &DatabaseConnection,
    admin_id: i64,
) -> Result<bool, sea_orm::DbErr> {
    let pref = find_by_admin_and_key(db, admin_id, PREF_KEY_SALE_SIMPLE_MODE).await?;
    match pref {
        Some(p) => match p.pref_value {
            Some(serde_json::Value::Bool(b)) => Ok(b),
            _ => Ok(false),
        },
        None => Ok(false),
    }
}

/// 保存销售简易模式开关
pub async fn save_sale_simple_mode(
    db: &DatabaseConnection,
    admin_id: i64,
    enabled: bool,
) -> Result<i64, sea_orm::DbErr> {
    let req = SavePreferenceRequest {
        pref_key: PREF_KEY_SALE_SIMPLE_MODE.to_string(),
        pref_value: serde_json::Value::Bool(enabled),
    };
    save(db, admin_id, &req).await
}
