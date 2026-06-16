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
#[sea_orm(table_name = "mxx_attachment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 用户ID
    pub user_id: Option<i64>,
    /// 分类id
    pub type_id: Option<i64>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub upload_url: Option<String>,
    pub ext: Option<String>,
    /// 文件大小
    pub size: Option<i64>,
    /// 文件md5
    pub md5: Option<String>,
    /// 图片上传类型 1本地 2七牛云 3阿里云 4腾讯云
    pub storage_type: Option<i32>,
    /// 附件类型。1-附件,2-图片
    pub r#type: Option<i32>,
    pub status: Option<i32>,
    /// 被引用次数
    pub count_quote: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}