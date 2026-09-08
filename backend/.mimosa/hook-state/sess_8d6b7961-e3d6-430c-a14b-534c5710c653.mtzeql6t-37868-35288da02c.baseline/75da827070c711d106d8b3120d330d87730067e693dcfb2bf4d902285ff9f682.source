//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};

pub const EVENT_TRANSACTION_SUCCESS: &str = "TRANSACTION.SUCCESS";
pub const EVENT_TRANSACTION_REFUND: &str = "REFUND.SUCCESS";
pub const EVENT_TRANSACTION_CLOSED: &str = "TRANSACTION.CLOSED";

pub const TRADE_STATE_SUCCESS: &str = "SUCCESS";
pub const TRADE_STATE_REFUND: &str = "REFUND";
pub const TRADE_STATE_NOTPAY: &str = "NOTPAY";
pub const TRADE_STATE_CLOSED: &str = "CLOSED";
pub const TRADE_STATE_REVOKED: &str = "REVOKED";
pub const TRADE_STATE_USERPAYING: &str = "USERPAYING";
pub const TRADE_STATE_PAYERROR: &str = "PAYERROR";

/// 寰俊鏀粯閫氱煡鍝嶅簲
pub fn success_response() -> WechatNotifyResponse {
    WechatNotifyResponse {
        code: "SUCCESS".to_string(),
        message: "鎴愬姛".to_string(),
    }
}

pub fn fail_response(message: &str) -> WechatNotifyResponse {
    WechatNotifyResponse {
        code: "FAIL".to_string(),
        message: message.to_string(),
    }
}

#[derive(Debug, Serialize)]
pub struct WechatNotifyResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct WechatV3Notify {
    pub id: String,
    pub create_time: String,
    pub event_type: String,
    pub resource_type: String,
    pub resource: WechatV3Resource,
}

#[derive(Debug, Deserialize)]
pub struct WechatV3Resource {
    pub original_type: String,
    pub algorithm: String,
    pub ciphertext: String,
    pub associated_data: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct WechatV3PaymentResult {
    pub transaction_id: String,
    pub out_trade_no: String,
    pub trade_state: String,
    pub trade_state_desc: String,
    pub amount: WechatV3Amount,
}

#[derive(Debug, Deserialize)]
pub struct WechatV3Amount {
    pub total: i32,
    pub payer_total: i32,
    pub currency: String,
    pub payer_currency: String,
}
