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

/// 内容模型
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_content_model")]
pub struct Model {
    // 模型ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    // 模型编码
    pub model_code: Option<String>,
    // 模型名称
    pub model_name: Option<String>,
    // 模型图标
    pub model_icon: Option<String>,
    // 描述
    pub description: Option<String>,
    // 是否有标题
    pub has_title: Option<i32>,
    // 是否有内容
    pub has_content: Option<i32>,
    // 是否有封面
    pub has_cover: Option<i32>,
    // 是否有作者
    pub has_author: Option<i32>,
    // 是否有摘要
    pub has_summary: Option<i32>,
    // 是否有SEO
    pub has_seo: Option<i32>,
    // 是否有图集
    pub has_images: Option<i32>,
    // 是否有附件
    pub has_attachment: Option<i32>,
    // 列表模板ID
    pub list_template_id: Option<i64>,
    // 详情模板ID
    pub detail_template_id: Option<i64>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 是否系统内置：0否，1是
    pub is_system: Option<i32>,
    // 删除标记：0未删除，1已删除
    pub deleted: Option<i32>,
    // 创建时间
    pub create_time: Option<DateTime>,
    // 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
