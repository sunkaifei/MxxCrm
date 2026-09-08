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

/// 网站媒体资源数据模型
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_media")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 原始文件名
    pub original_name: Option<String>,
    /// 存储文件名
    pub storage_name: Option<String>,
    /// 文件存储路径
    pub file_path: Option<String>,
    /// 文件访问URL
    pub file_url: Option<String>,
    /// 文件扩展名
    pub file_ext: Option<String>,
    /// 文件大小（字节）
    pub file_size: Option<i64>,
    /// 文件类型：1=图片, 2=视频, 3=文档, 4=音频, 5=其他
    pub file_type: Option<i32>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// 图片宽度
    pub width: Option<i32>,
    /// 图片高度
    pub height: Option<i32>,
    /// 小缩略图URL
    pub thumb_small: Option<String>,
    /// 中缩略图URL
    pub thumb_medium: Option<String>,
    /// 大缩略图URL
    pub thumb_large: Option<String>,
    /// 替代文本
    pub alt_text: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 说明文字
    pub caption: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 分类ID
    pub category_id: Option<i64>,
    /// 标签数组
    pub tags: Option<Vec<String>>,
    /// 引用计数
    pub ref_count: Option<i32>,
    /// 是否有水印：0无，1有
    pub has_watermark: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 是否删除：0未删除，1已删除
    pub deleted: Option<i32>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
