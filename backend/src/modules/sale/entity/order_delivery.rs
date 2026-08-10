//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 虚拟商品交付记录实体（mxx_sale_order_delivery）
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_order_delivery")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub delivery_no: Option<String>,
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    /// 交付方式：1=卡密, 2=下载链接, 3=账号密码, 4=激活码, 5=服务开通
    pub delivery_method: Option<i32>,
    /// 卡密/激活码（加密存储）
    pub card_key: Option<String>,
    pub download_url: Option<String>,
    pub account_name: Option<String>,
    /// 密码（加密存储）
    pub account_password: Option<String>,
    pub extra_content: Option<String>,
    /// 状态：1=待发送, 2=已发送, 3=已签收, 4=已撤销, 5=已失效
    pub status: Option<i32>,
    /// 交付类型：1=自动交付, 2=手动交付
    pub deliver_type: Option<i32>,
    pub sent_time: Option<DateTime>,
    pub received_time: Option<DateTime>,
    pub expire_time: Option<DateTime>,
    pub card_pool_id: Option<i64>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
