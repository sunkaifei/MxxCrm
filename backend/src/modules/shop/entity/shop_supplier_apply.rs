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
#[sea_orm(table_name = "mxx_shop_supplier_apply")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 申请用户ID
    pub user_id: i64,
    /// 店铺名称
    pub shop_name: String,
    /// 联系人
    pub contact_name: String,
    /// 联系电话
    pub contact_phone: String,
    /// 店铺LOGO
    pub shop_logo: Option<String>,
    /// 店铺简介
    pub shop_desc: Option<String>,
    /// 营业执照图片
    pub business_license: Option<String>,
    /// 状态: 0=待审核, 1=通过, 2=退回
    pub status: i16,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 审核时间
    pub audit_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
