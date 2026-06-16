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
use actix_web::{get, HttpResponse, web, HttpRequest};
use sea_orm::prelude::Decimal;
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::statistics::model::access_record::{VisitStatsVO, VisitTrendVO};

#[get("/visit-stats/visit-count")]
pub async fn get_visit_count(_state: web::Data<AppState>, _req: HttpRequest) -> Result<HttpResponse> {
    let result = VisitStatsVO{
        today_uv_count: Some(233),
        total_uv_count: Some(233),
        uv_growth_rate: Some(Decimal::from(233)),
        today_pv_count: Some(233),
        total_pv_count: Some(233),
        pv_growth_rate: Some(Decimal::from(233)),
    };

    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}


#[get("/visit-stats/visit-trend")]
pub async fn get_visit_trend(_state: web::Data<AppState>, _req: HttpRequest) -> Result<HttpResponse> {
    let result = VisitTrendVO {
        dates: Some(vec!["2023-01-01".to_string(), "2023-01-02".to_string(), "2023-01-03".to_string()]),
        pv_list: Some(vec![1, 2, 3]),
        ip_list: Some(vec![1, 2, 3]),
    };

    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}