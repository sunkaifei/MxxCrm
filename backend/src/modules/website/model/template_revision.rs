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
use crate::modules::website::entity::{template_revision, template_revision::Entity as TemplateRevision};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateRevisionSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板数据id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub template_data_id: Option<i64>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 版本备注
    pub revision_note: Option<String>,
    /// 创建人
    pub create_by: Option<i64>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateRevisionListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板数据id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_data_id: Option<i64>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 版本备注
    pub revision_note: Option<String>,
    /// 创建人
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<template_revision::Model> for TemplateRevisionListVO {
    fn from(model: template_revision::Model) -> Self {
        TemplateRevisionListVO {
            id: Option::from(model.id),
            template_data_id: model.template_data_id,
            temptext: model.temptext,
            revision_note: model.revision_note,
            create_by: model.create_by,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

pub struct TemplateRevisionModel;

impl TemplateRevisionModel {
    pub async fn insert(db: &DbConn, dto: &TemplateRevisionSaveDTO) -> Result<i64, DbErr> {
        let model = template_revision::ActiveModel {
            template_data_id: Set(dto.template_data_id.to_owned()),
            temptext: Set(dto.temptext.to_owned()),
            revision_note: Set(dto.revision_note.to_owned()),
            create_by: Set(dto.create_by.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = TemplateRevision::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    pub async fn find_by_template_data_id(
        db: &DbConn,
        template_data_id: &Option<i64>,
    ) -> Result<Option<template_revision::Model>, DbErr> {
        let result = template_revision::Entity::find()
            .filter(template_revision::Column::TemplateDataId.eq(template_data_id.clone().unwrap_or_default()))
            .order_by_desc(template_revision::Column::Id)
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn select_by_template_data_id(
        db: &DbConn,
        template_data_id: &Option<i64>,
    ) -> Result<Vec<template_revision::Model>, DbErr> {
        let list = template_revision::Entity::find()
            .filter(template_revision::Column::TemplateDataId.eq(template_data_id.clone().unwrap_or_default()))
            .order_by_desc(template_revision::Column::Id)
            .all(db)
            .await?;
        Ok(list)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template_revision::Model>, DbErr> {
        let result = template_revision::Entity::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(result)
    }
}
