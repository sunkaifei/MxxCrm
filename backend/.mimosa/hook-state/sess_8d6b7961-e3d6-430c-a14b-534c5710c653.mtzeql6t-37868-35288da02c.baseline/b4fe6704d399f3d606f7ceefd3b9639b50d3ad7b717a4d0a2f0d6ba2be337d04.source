//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::Deserialize;
use serde::Serialize;

/// 审计事件查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEventQuery {
    pub user_id: Option<i64>,
    pub module: Option<String>,
    pub action: Option<String>,
    /// YYYY-MM-DD
    pub start_date: Option<String>,
    /// YYYY-MM-DD
    pub end_date: Option<String>,
    /// 摘要关键字（模糊匹配）
    pub keyword: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
