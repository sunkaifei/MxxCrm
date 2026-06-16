//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, HttpRequest, HttpResponse, web};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::statistics::model::access_record::{AccessRecordSaveDTO, RecordSaveRequest};
use crate::modules::statistics::service::access_record_service;
use crate::utils::string_utils::{user_agent_browser, user_agent_os};

#[get("/statistics/record")]
pub async fn save_statistics_record(state: web::Data<AppState>, request: HttpRequest, payload: web::Query<RecordSaveRequest>) -> HttpResponse {
    let db = &state.db;
    let record_request = payload.0;
    
    // if let Some(url) = record_request.access_url {
    //     if !url.starts_with("https://www.97560.com") {
    //         return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "统计网址不正确!", "local")));
    //     }
    // } else {
    //     return HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到统计网址地址!", "local")));
    // }
    let mut record_dto = AccessRecordSaveDTO::default();
    if let Some(ip) = request.connection_info().realip_remote_addr() {
        println!("Client IP: {}", ip);
        record_dto.access_ip = Option::from(ip.to_string());
    }
    if let Some(user_agent) = request.headers().get("user-agent") {
        println!("===========user_agent============: {:?}", user_agent);
        if let Some(found_element) = user_agent_os(user_agent.to_str().unwrap_or_default()) {
            println!("=======================: {}", found_element);
            record_dto.access_device = Option::from(found_element.to_string());
        } else {
            println!("No matching element found.");
        }

        if let Some(found_browser) = user_agent_browser(user_agent.to_str().unwrap_or_default()) {
            println!("=======================: {}", found_browser);
            record_dto.access_browser = Option::from(found_browser.to_string());
        }
    }
    if record_request.source_url.is_some() {
        record_dto.source_url = record_request.source_url.clone();
    }
    if record_request.access_url.is_some() {
        record_dto.access_url = record_request.access_url.clone();
    }
    let result = access_record_service::insert(&db, &record_request).await;
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result))
}


