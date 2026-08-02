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
use crate::modules::website::entity::{template_var, template_var::Entity as TemplateVar};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateVarSaveRequest {
    /// 变量key
    pub var_key: Option<String>,
    /// 变量标签
    pub var_label: Option<String>,
    /// 变量值
    pub var_value: Option<String>,
    /// 变量类型：1=文本, 2=数字, 3=布尔, 4=HTML, 5=图片
    pub var_type: Option<i32>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<TemplateVarSaveRequest> for TemplateVarSaveDTO {
    fn from(req: TemplateVarSaveRequest) -> Self {
        TemplateVarSaveDTO {
            id: None,
            var_key: req.var_key,
            var_label: req.var_label,
            var_value: req.var_value,
            var_type: req.var_type,
            var_group: req.var_group,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateVarUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 变量key
    pub var_key: Option<String>,
    /// 变量标签
    pub var_label: Option<String>,
    /// 变量值
    pub var_value: Option<String>,
    /// 变量类型：1=文本, 2=数字, 3=布尔, 4=HTML, 5=图片
    pub var_type: Option<i32>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<TemplateVarUpdateRequest> for TemplateVarSaveDTO {
    fn from(req: TemplateVarUpdateRequest) -> Self {
        TemplateVarSaveDTO {
            id: req.id,
            var_key: req.var_key,
            var_label: req.var_label,
            var_value: req.var_value,
            var_type: req.var_type,
            var_group: req.var_group,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVarSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 变量key
    pub var_key: Option<String>,
    /// 变量标签
    pub var_label: Option<String>,
    /// 变量值
    pub var_value: Option<String>,
    /// 变量类型：1=文本, 2=数字, 3=布尔, 4=HTML, 5=图片
    pub var_type: Option<i32>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVarListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 变量key
    pub var_key: Option<String>,
    /// 变量标签
    pub var_label: Option<String>,
    /// 变量值
    pub var_value: Option<String>,
    /// 变量类型：1=文本, 2=数字, 3=布尔, 4=HTML, 5=图片
    pub var_type: Option<i32>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<template_var::Model> for TemplateVarListVO {
    fn from(model: template_var::Model) -> Self {
        TemplateVarListVO {
            id: Option::from(model.id),
            var_key: model.var_key,
            var_label: model.var_label,
            var_value: model.var_value,
            var_type: model.var_type,
            var_group: model.var_group,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVarDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 变量key
    pub var_key: Option<String>,
    /// 变量标签
    pub var_label: Option<String>,
    /// 变量值
    pub var_value: Option<String>,
    /// 变量类型：1=文本, 2=数字, 3=布尔, 4=HTML, 5=图片
    pub var_type: Option<i32>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<template_var::Model> for TemplateVarDetailVO {
    fn from(model: template_var::Model) -> Self {
        TemplateVarDetailVO {
            id: Option::from(model.id),
            var_key: model.var_key,
            var_label: model.var_label,
            var_value: model.var_value,
            var_type: model.var_type,
            var_group: model.var_group,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub keywords: Option<String>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub keywords: Option<String>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut keywords = None;
        if self.keywords != Some("".to_string()) {
            keywords = self.keywords.clone();
        }

        let mut var_group = None;
        if self.var_group != Some("".to_string()) {
            var_group = self.var_group.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            keywords,
            var_group,
            status,
        }
    }
}

pub struct TemplateVarModel;

impl TemplateVarModel {
    pub async fn insert(db: &DbConn, dto: &TemplateVarSaveDTO) -> Result<i64, DbErr> {
        let model = template_var::ActiveModel {
            var_key: Set(dto.var_key.to_owned()),
            var_label: Set(dto.var_label.to_owned()),
            var_value: Set(dto.var_value.to_owned()),
            var_type: Set(dto.var_type.to_owned()),
            var_group: Set(dto.var_group.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = TemplateVar::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result: UpdateResult = TemplateVar::update_many()
            .set(template_var::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(template_var::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &TemplateVarSaveDTO) -> Result<i64, DbErr> {
        let model = template_var::ActiveModel {
            var_key: Set(dto.var_key.to_owned()),
            var_label: Set(dto.var_label.to_owned()),
            var_value: Set(dto.var_value.to_owned()),
            var_type: Set(dto.var_type.to_owned()),
            var_group: Set(dto.var_group.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = TemplateVar::update_many()
            .set(model)
            .filter(template_var::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template_var::Model>, DbErr> {
        let result = template_var::Entity::find_by_id(id.clone().unwrap_or_default())
            .filter(template_var::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_key(db: &DbConn, key: &Option<String>) -> Result<Option<template_var::Model>, DbErr> {
        let result = template_var::Entity::find()
            .filter(template_var::Column::VarKey.eq(key.clone().unwrap_or_default()))
            .filter(template_var::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page_num: i64,
        page_size: i64,
        where_case: PageWhere,
    ) -> Result<(Vec<template_var::Model>, u64), DbErr> {
        let paginator = TemplateVar::find()
            .filter(template_var::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(template_var::Column::VarLabel.contains(v))
            })
            .apply_if(where_case.var_group.clone(), |query, v| {
                query.filter(template_var::Column::VarGroup.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(template_var::Column::Status.eq(v))
            })
            .order_by_asc(template_var::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = TemplateVar::find()
            .filter(template_var::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(template_var::Column::VarLabel.contains(v))
            })
            .apply_if(where_case.var_group.clone(), |query, v| {
                query.filter(template_var::Column::VarGroup.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(template_var::Column::Status.eq(v))
            })
            .count(db)
            .await? as i64;
        Ok(count)
    }

    pub async fn select_all(db: &DbConn) -> Result<Vec<template_var::Model>, DbErr> {
        let list = TemplateVar::find()
            .filter(template_var::Column::Deleted.eq(0))
            .filter(template_var::Column::Status.eq(1))
            .order_by_asc(template_var::Column::Sort)
            .all(db)
            .await?;
        Ok(list)
    }
}
