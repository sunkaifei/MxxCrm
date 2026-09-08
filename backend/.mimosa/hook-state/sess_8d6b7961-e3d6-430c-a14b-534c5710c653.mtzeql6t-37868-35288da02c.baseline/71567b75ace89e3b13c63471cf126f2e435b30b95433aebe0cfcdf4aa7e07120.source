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
use serde::{Deserialize, Serialize};
use crate::modules::system::entity::edit_log;

/// 单个字段的变更记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditLogItem {
    /// 字段名
    pub field: String,
    /// 字段中文标签
    pub field_label: String,
    /// 修改前值
    pub old: Option<String>,
    /// 修改后值
    pub new: Option<String>,
}

/// 编辑日志 VO（前端展示用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditLogVO {
    pub id: Option<i64>,
    /// 业务类型：1=报价单, 2=订单, 3=合同
    pub business_type: Option<i32>,
    /// 业务记录ID
    pub business_id: Option<i64>,
    /// 业务编号
    pub business_no: Option<String>,
    /// 业务标题
    pub business_title: Option<String>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 编辑人姓名
    pub editor_name: Option<String>,
    /// 变更内容
    pub content: Option<Vec<EditLogItem>>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
}

impl From<edit_log::Model> for EditLogVO {
    fn from(item: edit_log::Model) -> Self {
        let content = item.content.as_ref().and_then(|j| {
            serde_json::from_value::<Vec<EditLogItem>>(j.clone()).ok()
        });
        EditLogVO {
            id: Option::from(item.id),
            business_type: item.business_type,
            business_id: item.business_id,
            business_no: item.business_no,
            business_title: item.business_title,
            editor_id: item.editor_id,
            editor_name: item.editor_name,
            content,
            edit_time: item.edit_time,
        }
    }
}

/// 编辑日志查询参数
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditLogQuery {
    /// 业务类型：1=报价单, 2=订单, 3=合同，None=全部
    pub business_type: Option<i32>,
    /// 业务记录ID
    pub business_id: Option<i64>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 关键词搜索（编号/标题）
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
