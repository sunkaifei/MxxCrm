//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 设计器素材表（设计文档 §4.2）：Logo / 签章 / 底图 / 条码字体 / 自定义字体。

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_pdf_asset")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 素材名称
    pub name: String,
    /// 分类：logo | seal | background | font | barcode
    pub category: String,
    /// 访问 URL
    pub file_url: String,
    /// 服务器存储路径（虚拟 FS 读取用）
    pub file_path: Option<String>,
    /// 文件大小（字节）
    pub file_size: Option<i64>,
    /// 内容 md5（导入去重、素材包命名）
    pub md5: Option<String>,
    /// 图片原始宽（px）
    pub width_px: Option<i32>,
    /// 图片原始高（px）
    pub height_px: Option<i32>,
    /// 排序
    pub sort: i32,
    /// 状态（1启用 0禁用）
    pub status: i32,
    /// 创建人
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志（0未删除 1已删除）
    pub deleted: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
