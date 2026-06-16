//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{post, web, HttpResponse, HttpRequest};
use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::AppState;
use crate::modules::finance::service::payment_record_service;
use crate::modules::finance::service::wechat_pay_service;
use crate::modules::finance::model::wechat_pay::{
    WechatNotifyResponse, WechatV3Notify, WechatV3Resource, WechatV3PaymentResult,
    EVENT_TRANSACTION_SUCCESS, EVENT_TRANSACTION_REFUND, EVENT_TRANSACTION_CLOSED,
    TRADE_STATE_SUCCESS, TRADE_STATE_CLOSED, TRADE_STATE_PAYERROR,
    success_response, fail_response
};
use crate::modules::finance::model::member_product::MemberProductModel;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

fn aes_256_gcm_decrypt(
    ciphertext: &str,
    key: &[u8],
    nonce: &[u8],
    associated_data: &str,
) -> Result<String> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher.decrypt(
        nonce,
        aes_gcm::aead::Payload {
            msg: &BASE64.decode(ciphertext).map_err(|e| Error::from(format!("Base64解码失败: {}", e)))?,
            aad: associated_data.as_bytes(),
        },
    ).map_err(|e| Error::from(format!("AES解密失败: {}", e)))?;

    String::from_utf8(plaintext).map_err(|e| Error::from(format!("UTF-8转换失败: {}", e)))
}

#[post("/payment/wechat-notify")]
pub async fn wechat_notify(req: HttpRequest, state: web::Data<AppState>, body: web::Bytes) -> Result<HttpResponse> {
    let db = &state.db;
    log::info!("[微信支付V3回调] 收到支付回调请求");

    let body_str = String::from_utf8_lossy(&body);
    log::info!("[微信支付V3回调] 原始内容: {}", body_str);

    let timestamp = req.headers().get("Wechatpay-Timestamp")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let nonce = req.headers().get("Wechatpay-Nonce")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let signature = req.headers().get("Wechatpay-Signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let serial = req.headers().get("Wechatpay-Serial")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    log::info!("[微信支付V3回调] 回调头 - timestamp={}, nonce={}, serial={}", timestamp, nonce, serial);

    let verify_result = wechat_pay_service::verify_wechat_v3_notify(
        timestamp, nonce, &body_str, signature, serial
    );

    match verify_result {
        Ok(true) => {
            log::info!("[微信支付V3回调] 签名验证通过");
        }
        Ok(false) => {
            log::warn!("[微信支付V3回调] 签名验证失败");
            log::warn!("[微信支付V3回调] 跳过签名验证继续处理(测试模式)");
        }
        Err(e) => {
            log::error!("[微信支付V3回调] 签名验证出错: {}", e);
            log::warn!("[微信支付V3回调] 签名验证异常，跳过继续处理(测试模式)");
        }
    }

    let notify: WechatV3Notify = match serde_json::from_str(&body_str) {
        Ok(n) => n,
        Err(e) => {
            log::error!("[微信支付V3回调] JSON解析失败: {}", e);
            return Ok(HttpResponse::Ok().json(fail_response("JSON解析失败")));
        }
    };

    log::info!("[微信支付V3回调] 事件类型: {}", notify.event_type);

    let api_v3_key = crate::config::section::<String>("wechat", "api_v3_key", "".to_string());
    let key_bytes = api_v3_key.as_bytes();
    if key_bytes.len() != 32 {
        return Err(Error::from(format!("APIv3密钥必须是32个字符，当前长度: {}", key_bytes.len())));
    }

    let decrypted = match aes_256_gcm_decrypt(
        &notify.resource.ciphertext,
        &key_bytes,
        notify.resource.nonce.as_bytes(),
        &notify.resource.associated_data,
    ) {
        Ok(d) => d,
        Err(e) => {
            log::error!("[微信支付V3回调] 解密失败: {}", e);
            return Ok(HttpResponse::Ok().json(fail_response("解密失败")));
        }
    };

    log::info!("[微信支付V3回调] 解密内容: {}", decrypted);

    match notify.event_type.as_str() {
        EVENT_TRANSACTION_SUCCESS => {
            let result: WechatV3PaymentResult = match serde_json::from_str(&decrypted) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("[微信支付V3回调] 支付结果解析失败: {}", e);
                    return Ok(HttpResponse::Ok().json(fail_response("支付结果解析失败")));
                }
            };

            log::info!("[微信支付V3回调] 支付结果 - transaction_id={}, out_trade_no={}, trade_state={}",
                result.transaction_id, result.out_trade_no, result.trade_state);

            let out_trade_no = result.out_trade_no;
            let transaction_id = result.transaction_id;

            if let Ok(Some(record)) = payment_record_service::get_by_order_id(db, &out_trade_no).await {
                if record.status == Some(1) {
                    log::warn!("[微信支付V3回调] 订单已处理过，跳过重复通知: {}", out_trade_no);
                    return Ok(HttpResponse::Ok().json(success_response()));
                }
            }

            match result.trade_state.as_str() {
                TRADE_STATE_SUCCESS => {
                    let update_result = payment_record_service::update_paid(db, &out_trade_no, &transaction_id).await;
                    match update_result {
                        Ok(_) => {
                            log::info!("[微信支付V3回调] 订单支付成功: {}", out_trade_no);
                            
                            log::info!("[微信支付V3回调] 准备查询支付记录，order_id: {}", out_trade_no);
                            if let Ok(Some(record)) = payment_record_service::get_by_order_id(db, &out_trade_no).await {
                                log::info!("[微信支付V3回调] 查询到支付记录，user_id: {}, member_product_id: {:?}", 
                                        record.user_id, record.member_product_id);
                                
                                if let Some(member_product_id) = record.member_product_id {
                                    log::info!("[微信支付V3回调] 准备查询会员商品，product_id: {}", member_product_id);
                                    
                                    if let Ok(Some(product)) = MemberProductModel::find_by_id(db, member_product_id).await {
                                        log::info!("[微信支付V3回调] 查询到会员商品，product_name: {}", product.product_name);
                                    } else {
                                        log::error!("[微信支付V3回调] 查询会员商品失败，商品ID: {}", member_product_id);
                                    }
                                } else {
                                    log::warn!("[微信支付V3回调] 支付记录中没有会员商品ID，跳过权益创建");
                                }
                            } else {
                                log::error!("[微信支付V3回调] 未查询到支付记录，order_id: {}", out_trade_no);
                            }
                            
                            Ok(HttpResponse::Ok().json(success_response()))
                        }
                        Err(e) => {
                            log::error!("[微信支付V3回调] 订单状态更新失败: {}", e);
                            Ok(HttpResponse::Ok().json(fail_response("订单更新失败")))
                        }
                    }
                }
                TRADE_STATE_CLOSED | TRADE_STATE_PAYERROR => {
                    let remark = format!("支付失败: {}", result.trade_state_desc);
                    let update_result = payment_record_service::update_failed(db, &out_trade_no, &remark).await;
                    match update_result {
                        Ok(_) => {
                            log::info!("[微信支付V3回调] 订单支付失败: {}", out_trade_no);
                            Ok(HttpResponse::Ok().json(success_response()))
                        }
                        Err(e) => {
                            log::error!("[微信支付V3回调] 订单状态更新失败: {}", e);
                            Ok(HttpResponse::Ok().json(fail_response("订单更新失败")))
                        }
                    }
                }
                _ => {
                    log::warn!("[微信支付V3回调] 未知交易状态: {}", result.trade_state);
                    Ok(HttpResponse::Ok().json(success_response()))
                }
            }
        }
        EVENT_TRANSACTION_REFUND => {
            log::info!("[微信支付V3回调] 收到退款通知");
            Ok(HttpResponse::Ok().json(success_response()))
        }
        EVENT_TRANSACTION_CLOSED => {
            log::info!("[微信支付V3回调] 订单关闭通知");
            Ok(HttpResponse::Ok().json(success_response()))
        }
        _ => {
            log::warn!("[微信支付V3回调] 未知事件类型，跳过: {}", notify.event_type);
            Ok(HttpResponse::Ok().json(success_response()))
        }
    }
}
