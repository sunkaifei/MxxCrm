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

/// 离职申请请求（admin 端 HR/管理员代发起 与 个人中心本人发起 共用）
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResignApplyRequest {
    /// 被离职员工ID：admin 端必传；个人中心不传（取 JWT 本人）
    pub admin_id: Option<i64>,
    /// 离职类型：1主动辞职 2协商解除 3辞退
    pub resign_type: i32,
    /// 期望离职日期（YYYY-MM-DD）
    pub resign_date: Option<String>,
    /// 离职原因（敏感：仅审批链与人事可见）
    pub reason: Option<String>,
    /// 交接人（接手员工ID）
    pub transfer_to_admin_id: Option<i64>,
}
