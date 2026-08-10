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
#[sea_orm(table_name = "mxx_crm_electronic_signature")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub contract_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub sign_no: Option<String>,
    pub platform: Option<i32>,
    pub platform_flow_id: Option<String>,
    pub sign_url: Option<String>,
    pub status: Option<i32>,
    pub signed_pdf_url: Option<String>,
    pub signer_name: Option<String>,
    pub signer_phone: Option<String>,
    pub signer_email: Option<String>,
    pub expire_time: Option<DateTime>,
    pub signed_time: Option<DateTime>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
