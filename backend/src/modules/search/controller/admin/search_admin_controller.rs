//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
use actix_web::{HttpResponse, web};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
#[cfg(feature = "enable-es")]
use crate::modules::search::service::search_service;

pub async fn create_index(state: web::Data<AppState>) -> HttpResponse {
    #[cfg(feature = "enable-es")]
    {
        if let Ok(status) = search_service::create_index_mappings().await {
            if status {
                HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::fail(200, "创建成功", "local"))
            } else {
                HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "创建失败", "local"))
            }
        } else {
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "创建失败", "local"))
        }
    }
    
    #[cfg(not(feature = "enable-es"))]
    {
        let _ = state;
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "Elasticsearch 功能未启用", "local"))
    }
}



pub async fn delete_index(state: web::Data<AppState>) -> HttpResponse {
    #[cfg(feature = "enable-es")]
    {
        match search_service::delete_index().await {
            Ok(status) => {
                if status {
                    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::fail(200, "删除成功", "local"))
                } else {
                    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除失败", "local"))
                }
            }
            Err(_) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未知错误，删除失败", "local")),
        }
    }
    
    #[cfg(not(feature = "enable-es"))]
    {
        let _ = state;
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "Elasticsearch 功能未启用", "local"))
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册搜索索引模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(search_admin_controller::register)` 注册。
///
/// 注意：本模块原 handler 未使用 `#[protect]` 宏，因此 route 不挂 `require_permission`。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search/index")
            // GET /search/index/create - 创建索引
            .route(
                "/create",
                web::get().to(create_index),
            )
            // DELETE /search/index/delete - 删除索引
            .route(
                "/delete",
                web::delete().to(delete_index),
            ),
    );
}
