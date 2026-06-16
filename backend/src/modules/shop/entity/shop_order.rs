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
#[sea_orm(table_name = "mxx_shop_order")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 订单编号
    pub order_no: String,
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 订单总金额(元)
    pub total_amount: Decimal,
    /// 运费(元)
    pub freight_amount: Decimal,
    /// 平台佣金(元)
    pub commission_amount: Decimal,
    /// 佣金比例
    pub commission_rate: Decimal,
    /// 供货商结算金额(元)
    pub settlement_amount: Decimal,
    /// 退款金额(元)
    pub refund_amount: Decimal,
    /// 商品数量
    pub goods_count: i32,
    /// 状态: 0=待支付, 1=待发货, 2=已发货, 3=已签收, 4=已完成, 5=已取消, 6=退款中, 7=已退款
    pub status: i16,
    /// 收货人
    pub receiver_name: String,
    /// 收货电话
    pub receiver_phone: String,
    /// 收货地址
    pub receiver_address: String,
    /// 买家备注
    pub buyer_remark: Option<String>,
    /// 取消原因
    pub cancel_reason: Option<String>,
    /// 支付方式
    pub pay_method: Option<String>,
    /// 支付时间
    pub pay_time: Option<DateTime>,
    /// 发货时间
    pub delivery_time: Option<DateTime>,
    /// 签收/确认收货时间
    pub receive_time: Option<DateTime>,
    /// 完成(可评价)时间
    pub finish_time: Option<DateTime>,
    /// 取消时间
    pub cancel_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
