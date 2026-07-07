//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::crm::entity::customer_assign_history;
use crate::utils::string_utils::serialize_option_u64_to_string;

/// 客户分配历史VO（时间轴展示）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AssignHistoryVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub customer_id: Option<i64>,
    pub admin_id: Option<i64>,
    /// 负责人名称
    pub admin_name: Option<String>,
    /// 操作类型：1=领取，2=退回公海，3=管理员分配
    pub action_type: Option<i16>,
    pub start_time: Option<DateTime>,
    pub end_time: Option<DateTime>,
    pub remark: Option<String>,
    pub operated_by: Option<i64>,
    /// 操作人名称
    pub operated_by_name: Option<String>,
    pub create_time: Option<DateTime>,
}

impl From<customer_assign_history::Model> for AssignHistoryVO {
    fn from(item: customer_assign_history::Model) -> Self {
        AssignHistoryVO {
            id: Option::from(item.id),
            customer_id: item.customer_id,
            admin_id: item.admin_id,
            admin_name: None,
            action_type: item.action_type,
            start_time: item.start_time,
            end_time: item.end_time,
            remark: item.remark,
            operated_by: item.operated_by,
            operated_by_name: None,
            create_time: item.create_time,
        }
    }
}
