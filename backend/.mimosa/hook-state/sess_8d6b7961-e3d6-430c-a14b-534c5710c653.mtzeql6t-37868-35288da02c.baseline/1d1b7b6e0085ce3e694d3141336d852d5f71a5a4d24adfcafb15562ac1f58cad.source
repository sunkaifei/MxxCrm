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
#[sea_orm(table_name = "mxx_system_pdf_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 模板名称
    pub name: Option<String>,
    /// 模板编码
    pub template_code: Option<String>,
    /// 单据类型：quotation/order/contract
    pub doc_type: Option<String>,
    /// 模板内容（typst 语法）
    pub content: Option<String>,
    /// 页眉 typst 片段
    pub header_content: Option<String>,
    /// 页脚 typst 片段
    pub footer_content: Option<String>,
    /// 纸张大小：a4/a3/letter
    pub paper_size: Option<String>,
    /// 方向：portrait/landscape
    pub orientation: Option<String>,
    /// 上边距(pt)
    pub margin_top: Option<i32>,
    /// 下边距(pt)
    pub margin_bottom: Option<i32>,
    /// 左边距(pt)
    pub margin_left: Option<i32>,
    /// 右边距(pt)
    pub margin_right: Option<i32>,
    /// 主字体
    pub font_family: Option<String>,
    /// 是否默认模板（0否 1是）
    pub is_default: Option<i32>,
    /// 状态（1启用 0禁用）
    pub status: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人ID
    pub update_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
