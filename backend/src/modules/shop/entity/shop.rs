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
#[sea_orm(table_name = "mxx_shop")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    /// 店铺LOGO
    pub store_logo: Option<String>,
    /// 店铺名称
    pub store_name: Option<String>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: i16,
    /// 店铺详细地址
    pub store_address_detail: Option<String>,
    /// 店铺地址ID路径
    pub store_address_id_path: Option<String>,
    /// 店铺地址路径
    pub store_address_path: Option<String>,
    /// 店铺业务结束时间
    pub store_end_time: Option<DateTime>,
    /// 店铺状态: 0=正常, 1=关闭
    pub store_disable: i16,
    /// 店铺中心位置
    pub store_center: Option<String>,
    /// 店铺简介
    pub store_desc: Option<String>,
    /// 配送评分
    pub delivery_score: Option<Decimal>,
    /// 描述评分
    pub description_score: Option<Decimal>,
    /// 服务评分
    pub service_score: Option<Decimal>,
    /// 商品数量
    pub goods_num: Option<i32>,
    /// 收藏数量
    pub collection_num: Option<i32>,
    /// 易支付小程序签名
    pub yzf_mp_sign: Option<String>,
    /// 易支付签名
    pub yzf_sign: Option<String>,
    /// 商户EUID
    pub merchant_euid: Option<String>,
    /// 是否页面展示: 0=否, 1=是
    pub page_show: Option<i16>,
    /// 是否支持自提: 0=否, 1=是
    pub self_pick_flag: Option<i16>,
    /// 用户ID
    pub user_id: Option<i64>,
    /// 店铺状态: 0=待审核, 1=已开通, 2=已下架, 3=已注销
    pub status: i16,
    /// 佣金比例
    pub commission_rate: Decimal,
    /// 余额(分)
    pub balance: Decimal,
    /// 店铺电话
    pub store_phone: Option<String>,
    /// 结算周期: 1=月结
    pub settlement_period: i16,
    /// 删除标志(0=未删除,1=已删除)
    pub deleted: Option<i32>,
    /// 创建人
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志: 0=未删除, 1=已删除
    pub delete_flag: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}