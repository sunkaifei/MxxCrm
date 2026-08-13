//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 单据修改日志实体
//! 表 mxx_inventory_doc_change_log：记录入库/出库单完成后的每次修改（审核开关关闭场景）

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_inventory_doc_change_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 单据类型：inbound / outbound
    pub doc_type: Option<String>,
    /// 单据ID
    pub doc_id: Option<i64>,
    /// 单据编号
    pub doc_no: Option<String>,
    /// 操作类型：update / delete 等
    pub action: Option<String>,
    /// 修改原因
    pub change_reason: Option<String>,
    /// 修改前快照
    pub before_snapshot: Option<Json>,
    /// 修改后快照
    pub after_snapshot: Option<Json>,
    /// 操作人ID
    pub operator_id: Option<i64>,
    /// 操作人姓名
    pub operator_name: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
