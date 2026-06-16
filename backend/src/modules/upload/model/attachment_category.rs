//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::upload::entity::attachment_category::ActiveModel;
use crate::modules::upload::entity::{attachment_category, attachment_category::Entity as AttachmentCategory};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64, u64_to_string};
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachCategorySaveRequest {
    /// 父级分类
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 文件管理类型（1image,2video）
    pub type_id: Option<i32>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类目录
    pub en_name: Option<String>,
    /// 排序（升序）
    pub sort: Option<i32>,
}

impl From<AttachCategorySaveRequest> for AttachCategorySaveDTO {
    fn from(request: AttachCategorySaveRequest) -> Self {
        Self {
            id: Some(0),
            parent_id: Some(0),
            type_id: request.type_id,
            name: request.name,
            en_name: request.en_name,
            sort: request.sort,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachCategoryUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 父级分类
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 文件管理类型（image,video）
    pub type_id: Option<i32>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类目录
    pub en_name: Option<String>,
    /// 排序（升序）
    pub sort: Option<i32>,
}

impl From<AttachCategoryUpdateRequest> for AttachCategorySaveDTO {
    fn from(request: AttachCategoryUpdateRequest) -> Self {
        Self {
            id: request.id,
            parent_id: request.parent_id,
            type_id: request.type_id,
            name: request.name,
            en_name: request.en_name,
            sort: request.sort,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct AttachCategorySaveDTO {
    pub id: Option<i64>,
    /// 父级分类
    pub parent_id: Option<i64>,
    /// 文件管理类型（image,video）
    pub type_id: Option<i32>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类目录
    pub en_name: Option<String>,
    /// 排序（升序）
    pub sort: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachCategoryVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 父级分类
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 文件管理类型（image,video）
    pub type_id: Option<i32>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类目录
    pub en_name: Option<String>,
    /// 排序（升序）
    pub sort: Option<i32>,
}

impl From<attachment_category::Model> for AttachCategoryVO {
    fn from(model: attachment_category::Model) -> Self {
        Self {
            id: Option::from(model.id),
            parent_id: model.parent_id,
            type_id: model.type_id,
            name: model.name,
            en_name: model.en_name,
            sort: model.sort,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachCategoryListVO {
    /// 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 父级分类
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 文件管理类型（image,video）
    pub type_id: Option<i32>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类目录
    pub en_name: Option<String>,
    /// 排序（升序）
    pub sort: Option<i32>,
}

impl From<attachment_category::Model> for AttachCategoryListVO {
    fn from(model: attachment_category::Model) -> Self {
        Self {
            id: Option::from(model.id),
            parent_id: model.parent_id,
            type_id: model.type_id,
            name: model.name,
            en_name: model.en_name,
            sort: model.sort,
        }
    }
}


/// 菜单下拉树形结构
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTreeVO {
    /// 菜单ID
    #[serde(serialize_with = "u64_to_string")]
    pub value: i64,
    /// 父菜单ID
    pub label: Option<String>,
    /// 子菜单
    #[serde(skip_serializing_if = "Option::is_none")]
        pub children: Option<Vec<CategoryTreeVO>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub name: Option<String>,
    pub type_id: Option<i64>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        Self {
            name,
        }
    }
}

pub struct AttachmentCategoryModel;

impl AttachmentCategoryModel {
    /// 插入分类
    pub async fn insert(db: &DbConn, form_data: &AttachCategorySaveDTO) -> Result<i64, DbErr> {
        let payload = ActiveModel {
            parent_id:          Set(form_data.parent_id.to_owned()),
            type_id:            Set(form_data.type_id.to_owned()),
            name:               Set(form_data.name.to_owned()),
            en_name:            Set(form_data.en_name.to_owned()),
            sort:               Set(form_data.sort.to_owned()),
            ..Default::default()
        };

        attachment_category::Entity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除分类
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let delete_result = AttachmentCategory::delete_many()
            .filter(attachment_category::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(delete_result.rows_affected as i64)
    }

    /// # 更新分类
    /// * 'db'
    /// * 'form_data' 需要更新的分类数据
    pub async fn update(db: &DbConn, id: &Option<i64>, form_data: &AttachCategorySaveDTO) -> Result<i64, DbErr> {
        let update_result = AttachmentCategory::update_many()
            .set(ActiveModel {
                parent_id:         Set(form_data.parent_id.to_owned()),
                type_id:           Set(form_data.type_id.to_owned()),
                name:              Set(form_data.name.to_owned()),
                en_name:           Set(form_data.en_name.to_owned()),
                sort:              Set(form_data.sort.to_owned()),
                ..Default::default()
            })
            .filter(attachment_category::Column::Id.eq(id.clone().unwrap_or_default()));

        Ok(update_result.exec(db).await?.rows_affected as i64)
    }
    
    /// 根据id查询分类
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<attachment_category::Model>, DbErr> {
        let find_result = AttachmentCategory::find_by_id(id.clone().unwrap_or_default()).one(db).await?;
        Ok(find_result)
    }

    /// 查询所有分类
    pub async fn find_all(db: &DbConn, wheres: PageWhere,) -> Result<Vec<attachment_category::Model>, DbErr> {
        let find_result = AttachmentCategory::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(attachment_category::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .all(db).await?;
        Ok(find_result)
    }
}