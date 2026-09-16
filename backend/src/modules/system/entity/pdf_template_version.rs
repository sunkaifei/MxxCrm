//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 模板版本历史（设计文档 §7.1 versions / rollback）。

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_pdf_template_version")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 模板 id
    pub template_id: i64,
    /// 业务版本号
    pub version: i32,
    /// 模板名称快照
    pub name: Option<String>,
    /// 单据类型快照
    pub doc_type: Option<String>,
    /// 引擎快照
    pub engine: Option<String>,
    /// 版式快照
    pub layout_json: Option<Json>,
    /// HTML 模板内容快照
    pub content: Option<String>,
    /// 变更说明
    pub change_note: Option<String>,
    /// 操作人
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
