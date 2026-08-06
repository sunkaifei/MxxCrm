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
#[sea_orm(table_name = "mxx_system_pdf_download_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// PDF记录ID
    pub record_id: i64,
    /// 单据类型
    pub doc_type: Option<String>,
    /// 单据ID
    pub doc_id: Option<i64>,
    /// 单据编号
    pub doc_no: Option<String>,
    /// 文件名
    pub file_name: Option<String>,
    /// 下载人ID
    pub operator_id: Option<i64>,
    /// 下载人名称
    pub operator_name: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 下载时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
