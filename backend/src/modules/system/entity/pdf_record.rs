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
#[sea_orm(table_name = "mxx_system_pdf_record")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 单据类型：quotation/order/contract
    pub doc_type: Option<String>,
    /// 单据ID
    pub doc_id: Option<i64>,
    /// 单据编号
    pub doc_no: Option<String>,
    /// 使用的模板ID
    pub template_id: Option<i64>,
    /// 模板名称（冗余）
    pub template_name: Option<String>,
    /// 生成的文件名
    pub file_name: Option<String>,
    /// 服务器存储路径
    pub file_path: Option<String>,
    /// 访问URL
    pub file_url: Option<String>,
    /// 文件大小(字节)
    pub file_size: Option<i64>,
    /// 页数
    pub page_count: Option<i32>,
    /// 触发方式：auto=审批自动 / manual=手动
    pub trigger_type: Option<String>,
    /// 状态（1成功 0失败）
    pub status: Option<i32>,
    /// 失败原因
    pub error_msg: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
