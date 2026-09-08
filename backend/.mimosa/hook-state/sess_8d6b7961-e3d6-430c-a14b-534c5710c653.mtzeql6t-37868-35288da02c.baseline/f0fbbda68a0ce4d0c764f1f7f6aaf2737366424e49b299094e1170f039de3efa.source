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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberOrderRequest {
    pub product_id: String,
}

#[derive(Debug, Serialize)]
pub struct MemberOrderResponse {
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub amount: f64,
    pub status: i32,
    pub pay_method: i32,
}
