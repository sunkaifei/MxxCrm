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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_member_fee")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 会员类型: 1=月度会员, 2=季度会员, 3=年度会员
    pub member_type: Option<i32>,

    /// 支付金额
    pub amount: Decimal,

    /// 会员有效期开始时间
    pub valid_start_time: Option<DateTime>,

    /// 会员有效期结束时间
    pub valid_end_time: Option<DateTime>,

    /// 支付状态: 0=待支付, 1=已支付, 2=已过期
    pub status: Option<i32>,

    /// 关联支付记录ID
    pub payment_record_id: Option<i64>,

    /// 备注
    pub remark: Option<String>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
