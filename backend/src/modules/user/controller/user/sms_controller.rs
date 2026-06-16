//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{post, web, HttpRequest, HttpResponse};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::MetaResp;
use crate::core::kit::global::AppState;
use crate::core::kit::user_auth::get_user_id_from_request;
use crate::modules::user::model::sms_verification::{SmsVerificationRequest, SmsLoginRequest};
use crate::modules::user::service::sms_service::SmsService;
use crate::modules::user::service::tencent_sms_service::TencentSmsService;
use serde_json;

fn get_client_ip(req: &HttpRequest) -> String {
    if let Some(ip) = req.headers().get("X-Forwarded-For") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.split(',').next().unwrap_or("").trim().to_string();
        }
    }
    if let Some(ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }
    req.connection_info().peer_addr().unwrap_or("unknown").to_string()
}

#[post("/sms/send")]
pub async fn send_sms(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<SmsVerificationRequest>,
) -> Result<HttpResponse> {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let db = &state.db;
    let ip = get_client_ip(&req);

    let sms_service = TencentSmsService::new();

    match SmsService::send_code(db, &body.phone, &body.scene, &ip, &sms_service).await {
        Ok(result) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/sms/login")]
pub async fn sms_login(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<SmsLoginRequest>,
) -> Result<HttpResponse> {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };
    let db = &state.db;
    let ip = get_client_ip(&req);

    match SmsService::sms_login(db, &body.phone, &body.code, &ip).await {
        Ok(result) => {
            let mut data = serde_json::Map::new();
            data.insert("token".to_string(), serde_json::Value::String(result.token));
            data.insert("expireTime".to_string(), serde_json::Value::Number(result.expire_time.into()));
            data.insert("userInfo".to_string(), serde_json::to_value(result.user_info)?);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")))
        }
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &e.to_string(), "local"))),
    }
}
