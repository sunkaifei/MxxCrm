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

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_inventory_inbound")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 主键
    pub id: i64,
    /// 入库单号
    pub inbound_no: Option<String>,
    /// 入库类型：purchase/return/surplus/initial/other
    pub inbound_type: Option<String>,
    /// 关联源单ID
    pub source_order_id: Option<i64>,
    /// 关联源单单号
    pub source_order_no: Option<String>,
    /// 入库仓库
    pub warehouse_id: Option<i64>,
    /// 状态：0=草稿 1=审核中 2=已审核 3=已完成 4=已驳回
    pub status: Option<i32>,
    /// 审批实例ID（关联 mxx_system_approval_instance）
    pub instance_id: Option<i64>,
    /// 总入库数量
    pub total_quantity: Option<Decimal>,
    /// 总入库金额
    pub total_amount: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
    /// 审核人
    pub audit_by: Option<i64>,
    /// 审核时间
    pub audit_time: Option<DateTime>,
    /// 删除标识（0未删除 1已删除）
    pub deleted: Option<i32>,
    /// 创建人（制单人）
    pub created_by: Option<i64>,
    /// 提交人（谁提交审核）
    pub submitted_by: Option<i64>,
    /// 更新人
    pub updated_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 最近修改原因（已完成单据被修改时记录）
    pub last_change_reason: Option<String>,
    /// 最近修改人ID
    pub last_change_by: Option<i64>,
    /// 最近修改时间
    pub last_change_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::inbound_item::Entity")]
    InboundItem,
}

impl Related<super::inbound_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InboundItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}