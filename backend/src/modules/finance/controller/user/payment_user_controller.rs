//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpResponse, Result, HttpRequest, http::StatusCode};
use crate::core::kit::global::AppState;
use crate::core::web::response::{MetaResp, ResultPage};
use crate::modules::finance::model::{
    payment_record::{PaymentRecordSaveRequest, PaymentRecordQuery, PaymentRecordModel},
    member_product::MemberProductModel,
    member_order::{MemberOrderRequest, MemberOrderResponse},
};
use crate::modules::finance::service::payment_record_service;
use crate::modules::finance::service::wechat_pay_service;
use crate::modules::user::model::user_platform::UserPlatformModel;
use crate::core::kit::jwt_util::JWTToken;
use crate::config;

#[get("/payment/list")]
pub async fn list(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<PaymentRecordQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[支付记录列表] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[支付记录列表] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[支付记录列表] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[支付记录列表] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[支付记录列表] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let mut query_inner = query.into_inner();
    query_inner.user_id = Some(user_id);

    let result = payment_record_service::get_list(db, query_inner).await;

    match result {
        Ok((list, total)) => {
            let page_data = ResultPage::new(list, total, 1, 20);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local")))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/payment/detail/{id}")]
pub async fn detail(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[支付记录详情] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[支付记录详情] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[支付记录详情] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[支付记录详情] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[支付记录详情] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let result = payment_record_service::get_by_id(db, id).await;

    match result {
        Ok(Some(data)) => {
            if data.user_id == user_id {
                Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")))
            } else {
                Ok(HttpResponse::Forbidden().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "无权访问该订单", "local")))
            }
        },
        Ok(None) => Ok(HttpResponse::NotFound().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "订单不存在", "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/payment/create")]
pub async fn create_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    item: web::Json<PaymentRecordSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[创建支付] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[创建支付] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[创建支付] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[创建支付] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[创建支付] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    let mut req = item.into_inner();
    req.user_id = user_id;

    let result = payment_record_service::insert(db, req).await;

    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/payment/member-experience")]
pub async fn create_member_experience_order(
    req: HttpRequest,
    state: web::Data<AppState>,
    item: web::Json<MemberOrderRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let order_req = item.into_inner();

    log::info!("[会员订单] 开始处理会员购买请求, product_id: {}", order_req.product_id);

    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token_str = match req.headers().get("Authorization") {
        Some(header) => match header.to_str() {
            Ok(s) => s.trim_start_matches("Bearer ").to_string(),
            Err(e) => {
                log::error!("[会员订单] Authorization header解析失败: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
            }
        },
        None => {
            log::error!("[会员订单] Authorization header不存在");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token不存在，请重新登录", "local")));
        }
    };

    let decoded_token = match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(t) => t,
        Err(e) => {
            log::error!("[会员订单] token验证失败: {:?}", e);
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效或已过期，请重新登录", "local")));
        }
    };

    if decoded_token.iss != "mxx_B2B_user" {
        log::error!("[会员订单] token签发者不匹配");
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token类型错误，请重新登录", "local")));
    }

    let user_id = match decoded_token.id {
        Some(id) => id,
        None => {
            log::error!("[会员订单] token中未包含用户ID");
            return Ok(HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/msgpack").body(MetaResp::<String>::fail(400, "token无效，请重新登录", "local")));
        }
    };

    log::info!("[会员订单] 用户ID: {}", user_id);

    let openid = match UserPlatformModel::find_openid_by_user_id(db, user_id).await {
        Ok(Some(id)) => id,
        Ok(None) => return Ok(HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户未绑定微信", "local"))),
        Err(e) => return Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("查询用户微信信息失败: {}", e), "local"))),
    };

    log::info!("[会员订单] 用户OpenID: {}", openid);

    let client_ip = req.connection_info().peer_addr().unwrap_or("127.0.0.1").to_string();

    log::info!("[会员订单] 客户端IP: {}", client_ip);

    log::info!("[会员订单] 开始查询商品, product_id={}", order_req.product_id);
    
    let product = match MemberProductModel::find_by_product_id(db, &order_req.product_id).await {
        Ok(Some(p)) => {
            log::info!("[会员订单] 商品查询成功: id={}, name={}, price={}", p.id, p.product_name, p.price);
            p
        },
        Ok(None) => {
            log::warn!("[会员订单] 商品不存在或已下架, product_id={}", order_req.product_id);
            return Ok(HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "商品不存在或已下架", "local")));
        },
        Err(e) => {
            log::error!("[会员订单] 查询商品失败: product_id={}, error={}", order_req.product_id, e);
            return Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("查询商品失败: {}", e), "local")));
        },
    };

    log::info!("[会员订单] 商品信息: id={}, name={}, price={}", product.product_id, product.product_name, product.price);

    log::info!("[会员订单] 开始检查购买限制, user_id={}, product_id={}, limit_type={:?}, limit_count={:?}", 
        user_id, product.id, product.purchase_limit_type, product.purchase_limit_count);
    
    let purchase_result = PaymentRecordModel::check_purchase_limit(
        db,
        user_id,
        product.id,
        product.purchase_limit_type,
        product.purchase_limit_count,
    ).await;
    
    log::info!("[会员订单] 购买限制检查结果: {:?}", purchase_result);
    
    match purchase_result {
        Ok(false) => {
            log::warn!("[会员订单] 购买次数已达上限: user_id={}, product_id={}", user_id, product.id);
            return Ok(HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "购买次数已达上限", "local")));
        }
        Ok(true) => {
            log::info!("[会员订单] 购买限制检查通过: user_id={}, product_id={}", user_id, product.id);
        }
        Err(e) => {
            log::error!("[会员订单] 购买限制检查失败, user_id={}, product_id={}, error={}", user_id, product.id, e);
            return Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("检查购买限制失败: {}", e), "local")));
        }
    }

    let order_id = wechat_pay_service::generate_order_id();

    log::info!("[会员订单] 生成订单号: {}", order_id);

    let payment_record = PaymentRecordSaveRequest {
        user_id,
        member_product_id: Some(product.id),
        order_id: Some(order_id.clone()),
        payment_type: Some(2),
        amount: product.price,
        pay_method: Some(1),
        status: Some(0),
        transaction_id: None,
        pay_time: None,
        remark: Some(format!("{} - {}", product.product_name, product.product_id)),
    };

    log::info!("[会员订单] 准备保存支付记录");

    if let Err(e) = payment_record_service::insert(db, payment_record).await {
        log::error!("保存支付记录失败: {}", e);
        return Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "保存订单失败", "local")));
    }

    log::info!("[会员订单] 支付记录保存成功");

    log::info!("[会员订单] 准备调用微信支付统一下单, 金额: {} 分", (product.price * 100.0) as i32);

    let pay_response = match wechat_pay_service::create_member_experience_order(user_id, &order_id, &openid, &client_ip, (product.price * 100.0) as i32).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("[会员订单] 创建支付订单失败: {}", e);
            return Ok(HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("创建支付订单失败: {}", e), "local")));
        }
    };

    log::info!("[会员订单] 微信支付统一下单成功，返回预支付信息");

    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(pay_response, "local")))
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list);
    cfg.service(detail);
    cfg.service(create_payment);
    cfg.service(create_member_experience_order);
}
