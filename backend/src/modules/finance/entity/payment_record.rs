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
#[sea_orm(table_name = "mxx_payment_record")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 用户ID
    pub user_id: i64,

    /// 会员产品ID（关联会员产品）
    pub member_product_id: Option<i64>,

    /// 订单ID（关联业务订单）
    pub order_id: Option<String>,

    /// 支付类型: 1=会员费用, 2=商品购买, 3=充值, 4=其他
    pub payment_type: Option<i32>,

    /// 支付金额
    pub amount: Decimal,

    /// 支付方式: 1=微信支付, 2=支付宝, 3=银行卡
    pub pay_method: Option<i32>,

    /// 支付状态: 0=待支付, 1=支付成功, 2=支付失败, 3=已退款
    pub status: Option<i32>,

    /// 第三方支付交易号
    pub transaction_id: Option<String>,

    /// 支付时间
    pub pay_time: Option<DateTime>,

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
