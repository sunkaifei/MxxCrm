//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::website::entity::{template_category, template_category::Entity as TemplateCategory};
use sea_orm::prelude::DateTime;
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CategorySaveRequest {
    // 父分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
}

impl From<CategorySaveRequest> for CategorySaveDTO {
    fn from(form_data: CategorySaveRequest) -> Self {
        CategorySaveDTO {
            id: None,
            parent_id:      form_data.parent_id,
            name:           form_data.name,
            sort:           form_data.sort,
            is_show:        form_data.is_show,
            status:         form_data.status,
            create_time:    None,
            update_time:    None,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryUpdateRequest {
    // 分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 父分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
}

impl From<CategoryUpdateRequest> for CategorySaveDTO {
    fn from(form_data: CategoryUpdateRequest) -> Self {
        CategorySaveDTO {
            id:             form_data.id,
            parent_id:      form_data.parent_id,
            name:           form_data.name,
            sort:           form_data.sort,
            is_show:        form_data.is_show,
            status:         form_data.status,
            create_time:    None,
            update_time:    None,
        }
    }
}

pub struct CategorySaveDTO {
    // 分类ID
    pub id: Option<i64>,
    // 父分类ID
    pub parent_id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 添加时间
    pub create_time: Option<DateTime>,
    // 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryListVO {
    // 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 添加时间
    pub create_time: Option<String>,
    // 更新时间
    pub update_time: Option<String>,
    /// 子菜单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<CategoryListVO>>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectListVO {
    // 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
}

impl From<template_category::Model> for SelectListVO {
    fn from(model: template_category::Model) -> Self {
        SelectListVO {
            id:             Option::from(model.id),
            name:           model.name,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDetailVO {
    // 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    // 分类名称
    pub name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 添加时间
    pub create_time: Option<String>,
    // 更新时间
    pub update_time: Option<String>,
}

impl From<template_category::Model> for CategoryDetailVO {
    fn from(model: template_category::Model) -> Self {
        CategoryDetailVO {
            id:             Option::from(model.id),
            parent_id:      model.parent_id,
            name:           model.name,
            sort:           model.sort,
            is_show:        model.is_show,
            status:         model.status,
            create_time:    model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time:    model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub name: Option<String>,
    pub is_show: Option<i32>,
    pub status: Option<i32>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    pub is_show: Option<i32>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        let mut is_show = None;
        if self.is_show > Some(0) {
            is_show = self.is_show.clone();
        }

        let mut status = None;
        if self.status == Some(0) {
            status = self.status;
        }

        Self {
            name,
            is_show,
            status,
        }
    }
}

pub struct TemplateCategoryModel;

impl TemplateCategoryModel {
    pub async fn insert(db: &DbConn, form_data: &CategorySaveDTO) -> Result<i64, DbErr> {
        let template_category = template_category::ActiveModel {
            parent_id:      Set(form_data.parent_id.to_owned()),
            name:           Set(form_data.name.to_owned()),
            sort:           Set(form_data.sort.to_owned()),
            is_show:        Set(form_data.is_show.to_owned()),
            status:         Set(form_data.status.to_owned()),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = TemplateCategory::insert(template_category)
            .exec(db)
            .await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量删除
    /// * `db` 数据库链接
    /// * `ids` id数组
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        TemplateCategory::delete_many()
            .filter(template_category::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &CategorySaveDTO) -> Result<i64, DbErr> {
        let payload = template_category::ActiveModel {
            parent_id:      Set(form_data.parent_id.to_owned()),
            name:           Set(form_data.name.to_owned()),
            sort:           Set(form_data.sort.to_owned()),
            is_show:        Set(form_data.is_show.to_owned()),
            status:         Set(form_data.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = TemplateCategory::update_many()
            .set(payload)
            .filter(template_category::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 按name查询是否有重复
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        TemplateCategory::find()
            .filter(template_category::Column::Name.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(template_category::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 按id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template_category::Model>, DbErr> {
        let res = TemplateCategory::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(res)
    }

    pub async fn find_all(db: &DbConn) -> Result<Vec<template_category::Model>, DbErr> {
        let res = TemplateCategory::find()
            .all(db)
            .await?;
        Ok(res)
    }
    
    pub async fn select_by_parent_id(db: &DbConn, parent_id: &Option<i64>) -> Result<Vec<template_category::Model>, DbErr> {
        let res = TemplateCategory::find()
            .filter(template_category::Column::ParentId.eq(parent_id.clone().unwrap_or_default()))
            .filter(template_category::Column::Status.eq(1))
            .order_by_asc(template_category::Column::Sort)
            .all(db)
            .await?;
        Ok(res)
    }

    pub async fn select_all(db: &DbConn, select_where: &PageWhere) -> Result<Vec<template_category::Model>, DbErr> {
        let res = TemplateCategory::find()
            .apply_if(select_where.name.clone(), |query, v| {
                query.filter(template_category::Column::Name.contains(v))
            })
            .apply_if(select_where.is_show.clone(), |query, v| {
                query.filter(template_category::Column::IsShow.eq(v))
            })
            .apply_if(select_where.status.clone(), |query, v| {
                query.filter(template_category::Column::Status.eq(v))
            })
            .all(db)
            .await?;
        Ok(res)
    }


}