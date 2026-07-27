//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::system::model::notice::{ListQuery, NoticeSaveDTO, NoticeSaveRequest, NoticeUpdateRequest};
use crate::modules::system::service::notice_service;
use crate::validate;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn add_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Json<NoticeSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;

    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let mut form_data = NoticeSaveDTO::from(item.into_inner());
    form_data.publish_status = Some(0);
    form_data.create_by = admin_token.id;
    form_data.update_by = admin_token.id;
    let result = notice_service::insert(db, &form_data).await?;
    if result > 0 {
        // 返回新创建的公告 ID，前端"保存并发布"流程依赖此 ID 调用发布接口
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}


pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = notice_service::batch_delete_by_ids(db, &ids_vec).await;
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_by_id(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<NoticeUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let mut form_data = NoticeSaveDTO::from(item.into_inner());
    form_data.id = Some(id.into_inner());
    form_data.update_by = admin_token.id;

    let result = notice_service::update_by_id(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("更新成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "更新失败", "local")))
    }
}

pub async fn user_read_all(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let result = notice_service::update_by_read_all(&db, &admin_token.id).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "已设置全部为阅读状态", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "阅读设置失败", "local")))
    }
}

pub async fn revoke_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let result= notice_service::update_by_id_revoke(&db, &item.id, &admin_token.id).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "撤销成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "撤销失败", "local")))
    }
}

pub async fn publish_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    let result=notice_service::update_by_id_publish(&db, &item.id, &admin_token.id).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("发布成功", "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发布失败", "local")))
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, _req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    match notice_service::get_by_detail(&db, &item.id).await {
        Ok(Some(notice_vo)) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(notice_vo, "local")))
        }
        Ok(None) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "该公告信息不存在或者已删除", "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}


pub async fn get_by_user_detail(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    match notice_service::get_by_user_detail(&db, &item.id, &admin_token.id).await {
        Ok(Some(notice_vo)) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(notice_vo, "local")))
        }
        Ok(None) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "该公告信息不存在或者已删除", "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

/// 标记公告为已读（专用接口，PUT /notice/user/{id}/read）
pub async fn read_notice(state: web::Data<AppState>, req: HttpRequest, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    log::info!("[read_notice] 收到请求, path_info={:?}, item.id={:?}", req.match_info().get("id"), item.id);
    validate!(item.id.is_none(), t!("notice.index.id_empty", locale = "zh-CN").to_string());
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    log::info!("[read_notice] user_id={:?}", admin_token.id);
    match notice_service::mark_notice_read(&db, &item.id, &admin_token.id).await {
        Ok(result) if result > 0 => {
            log::info!("[read_notice] 标记成功, result={}", result);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success("标记已读成功", "local")))
        }
        Ok(_) => {
            log::warn!("[read_notice] 未找到公告关联记录, result=0");
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未找到公告关联记录", "local")))
        }
        Err(err) => {
            log::error!("[read_notice] 错误: {:?}", err);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}


pub async fn get_by_my_page(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut query = query.into_inner();
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    query.user_id = admin_token.id;
    notice_service::get_by_my_page(&db, query).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    notice_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册公告管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(notice_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notice")
            // POST /notice/add - 添加公告
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/add",
                web::post()
                    .to(add_notice)
                    .wrap(require_permission("system:notice:add")),
            )
            // DELETE /notice/bath_delete - 批量删除公告
            .route(
                "/bath_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("system:notice:delete")),
            )
            // PUT /notice/update/{id} - 更新公告
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("system:notice:update")),
            )
            // PUT /notice/read-all - 标记全部已读
            .route("/read-all", web::put().to(user_read_all))
            // PUT /notice/{id}/revoke - 撤销公告
            .route(
                "/{id}/revoke",
                web::put()
                    .to(revoke_notice)
                    .wrap(require_permission("system:notice:revoke")),
            )
            // PUT /notice/{id}/publish - 发布公告
            .route(
                "/{id}/publish",
                web::put()
                    .to(publish_notice)
                    .wrap(require_permission("system:notice:publish")),
            )
            // GET /notice/detail/{id} - 公告详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("system:notice:view")),
            )
            // GET /notice/user/detail-{id} - 用户公告详情
            .route("/user/detail-{id}", web::get().to(get_by_user_detail))
            // PUT /notice/user/{id}/read - 标记公告为已读（专用接口）
            .route("/user/{id}/read", web::put().to(read_notice))
            // GET /notice/my-page - 我的公告分页
            .route("/my-page", web::get().to(get_by_my_page))
            // GET /notice/list - 公告列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("system:notice:list")),
            ),
    );
}
