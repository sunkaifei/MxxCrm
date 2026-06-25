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
#[sea_orm(table_name = "mxx_crm_contact")]
pub struct Model {
    /// 联系人ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 关联客户ID
    pub customer_id: Option<i64>,

    /// 联系人姓名
    pub name: Option<String>,

    /// 职位/头衔
    pub title: Option<String>,

    /// 邮箱地址
    pub email: Option<String>,

    /// 固定电话
    pub phone: Option<String>,

    /// 手机号码
    pub mobile: Option<String>,

    /// WhatsApp
    pub whatsapp: Option<String>,

    /// 微信号
    pub wechat: Option<String>,

    /// 是否主要联系人
    pub is_primary: Option<bool>,

    /// 是否财务联系人
    pub is_billing: Option<bool>,

    /// 是否收货联系人
    pub is_shipping: Option<bool>,

    /// 生日
    pub birthday: Option<Date>,

    /// 备注
    pub notes: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 软删除标识（0-未删除，1-已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
