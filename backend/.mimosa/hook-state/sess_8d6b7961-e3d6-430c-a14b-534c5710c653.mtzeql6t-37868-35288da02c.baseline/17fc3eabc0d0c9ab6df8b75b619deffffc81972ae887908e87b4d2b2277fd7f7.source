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
#[sea_orm(table_name = "mxx_refund_record")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 关联支付记录ID
    pub payment_record_id: i64,

    /// 退款金额
    pub amount: Decimal,

    /// 退款状态: 0=待退款, 1=退款处理中, 2=退款成功, 3=退款失败, 4=退款关闭
    pub status: Option<i32>,

    /// 第三方退款交易号
    pub transaction_id: Option<String>,

    /// 退款时间
    pub refund_time: Option<DateTime>,

    /// 退款原因
    pub reason: Option<String>,

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
