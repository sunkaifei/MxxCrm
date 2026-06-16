//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::AppState;
use crate::core::kit::template::get_template;
use crate::core::web::response::MetaResp;
use crate::modules::system::model::config;
use actix_web::http::header::ContentType;
use actix_web::middleware::Next;
use actix_web::{body::BoxBody, dev, dev::ServiceRequest, http::Method, web, Error, HttpResponse};
use minijinja::context;

//  妫€娴嬬綉绔欐槸鍚﹀紑鍚?
pub async fn check(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<dev::ServiceResponse<BoxBody>, Error> {
    let state = req.app_data::<web::Data<AppState>>().unwrap();
    let db = &state.db;
    let setting_data = config::ConfigModel::find_by_key(&db, "website_status").await.unwrap_or_default();

    let mut status: String = "0".to_string();
    if let Some(s) = setting_data {
        status = s.config_value.unwrap_or_default();
    }

    if status.as_str() == "1" {
        return next.call(req).await;
    }

    let error = "缃戠珯鍏抽棴缁存姢涓?..";

    let method = req.method();
    if method == Method::POST {
        return Ok(req.into_response(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, error, "local"))));
    }
    let ctx = context!(
            error => error,
        );
    let out = get_template("default/error.html", ctx)?;
    let res_body_data = HttpResponse::Ok().content_type(ContentType::html()).body(out);
    Ok(req.into_response(res_body_data))
}
