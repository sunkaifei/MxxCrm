//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::HttpRequest;
use sea_orm::{ConnectionTrait, EntityTrait};

use crate::core::web::base_controller::get_current_user;
use crate::modules::system::entity::audit_event;

/// 业务审计埋点统一入口（append-only）
///
/// 使用约定：
/// - 在业务事务**提交后**调用（best-effort，失败仅 warn 日志，不影响业务）
/// - before/after 仅放关键字段快照（金额/状态/负责人等），不放全量
/// - 系统代码对本表只有 INSERT，禁止 UPDATE/DELETE（append-only 审计语义）
pub async fn record<C: ConnectionTrait>(
    db: &C,
    req: &HttpRequest,
    module: &str,
    action: &str,
    target_type: &str,
    target_id: i64,
    summary: String,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
) {
    let (user_id, user_name) = get_current_user(req);
    let ip = req
        .connection_info()
        .peer_addr()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let mut am = audit_event::ActiveModel {
        user_id: sea_orm::Set(user_id),
        user_name: sea_orm::Set(user_name),
        module: sea_orm::Set(module.to_string()),
        action: sea_orm::Set(action.to_string()),
        target_type: sea_orm::Set(target_type.to_string()),
        target_id: sea_orm::Set(target_id),
        summary: sea_orm::Set(summary.chars().take(200).collect()),
        before_json: sea_orm::Set(before),
        after_json: sea_orm::Set(after),
        ip: sea_orm::Set(ip),
        create_time: sea_orm::Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };
    am.id = sea_orm::NotSet;

    if let Err(e) = audit_event::Entity::insert(am).exec(db).await {
        log::warn!("[audit] 审计写入失败(不阻断业务): module={} action={} target={} err={}", module, action, target_id, e);
    }
}

/// 快照辅助：构造关键字段 JSON
pub fn snap(pairs: Vec<(&str, serde_json::Value)>) -> Option<serde_json::Value> {
    if pairs.is_empty() {
        return None;
    }
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    Some(serde_json::Value::Object(map))
}
