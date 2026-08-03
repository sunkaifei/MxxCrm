//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{HttpRequest, HttpResponse, web};
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::leave_msg::LeaveMsgSubmitRequest;
use crate::modules::website::service::{leave_msg_service, website_service};

/// 提交留言（前台访客公开接口）
///
/// 接收访客从网站咨询表单提交的留言，自动转线索（若站点配置了 lead_owner_id）。
/// POST /api/open/leave_msg/submit
pub async fn submit(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<LeaveMsgSubmitRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    // 参数校验：联系人和联系方式至少有一个，且内容不能为空
    let has_contact = form_data.contact_name.as_ref().map_or(false, |s| !s.trim().is_empty())
        || form_data.contact_phone.as_ref().map_or(false, |s| !s.trim().is_empty());
    if !has_contact {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "请填写联系人姓名或电话", "local")));
    }
    if form_data.content.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "留言内容不能为空", "local")));
    }

    // 提取访客 IP 和 User-Agent
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // 获取默认站点
    let site = website_service::find_default(db).await?;

    let result = leave_msg_service::submit(db, &site, form_data, ip_address, user_agent).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}
