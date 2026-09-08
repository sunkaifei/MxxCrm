//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//

use crate::core::kit::global::{Deserialize, Serialize};

/// 手动重算请求
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AggRefreshRequest {
    /// topic：contract / payment / employee / customer / all
    pub topic: String,
    /// YYYY-MM-DD
    pub start_date: String,
    /// YYYY-MM-DD
    pub end_date: String,
}

/// 批次查询
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AggBatchQuery {
    pub topic: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
