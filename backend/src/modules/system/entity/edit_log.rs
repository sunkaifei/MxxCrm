//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 通用编辑日志实体
/// 表 mxx_system_edit_log：记录报价单、订单、合同等模块的字段级修改
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_edit_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 业务类型：1=报价单, 2=订单, 3=合同
    pub business_type: Option<i32>,
    /// 业务记录ID
    pub business_id: Option<i64>,
    /// 业务编号（报价单号/订单号/合同号，冗余便于查询）
    pub business_no: Option<String>,
    /// 业务标题（冗余便于查询）
    pub business_title: Option<String>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 编辑人姓名（冗余，避免联查）
    pub editor_name: Option<String>,
    /// 变更内容 JSON 数组
    pub content: Option<Json>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
    /// 软删除标记
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
