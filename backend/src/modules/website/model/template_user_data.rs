//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{template_user_data, template_user_data::Entity as TemplateUserData, website_template_merge};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateDataSaveRequest {
    /// 模板id
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<TemplateDataSaveRequest> for TemplateDataSaveDTO {
    fn from(req: TemplateDataSaveRequest) -> Self {
        TemplateDataSaveDTO {
            id: None,
            template_id: req.template_id,
            model_id: req.model_id,
            type_id: req.type_id,
            name: req.name,
            temptext: req.temptext,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateDataUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 模板id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<TemplateDataUpdateRequest> for TemplateDataSaveDTO {
    fn from(req: TemplateDataUpdateRequest) -> Self {
        TemplateDataSaveDTO {
            id: req.id,
            template_id: req.template_id,
            model_id: req.model_id,
            type_id: req.type_id,
            name: req.name,
            temptext: req.temptext,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDataSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDataListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<template_user_data::Model> for TemplateDataListVO {
    fn from(model: template_user_data::Model) -> Self {
        TemplateDataListVO {
            id: Option::from(model.id),
            template_id: model.template_id,
            model_id: model.model_id,
            type_id: model.type_id,
            name: model.name,
            temptext: model.temptext,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDataDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<template_user_data::Model> for TemplateDataDetailVO {
    fn from(model: template_user_data::Model) -> Self {
        TemplateDataDetailVO {
            id: Option::from(model.id),
            template_id: model.template_id,
            model_id: model.model_id,
            type_id: model.type_id,
            name: model.name,
            temptext: model.temptext,
            sort: model.sort,
            status: model.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    pub keywords: Option<String>,
    /// 模板id
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
    /// 网站id
    pub website_id: Option<i64>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}
#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    /// 模板id
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 网站id
    pub website_id: Option<i64>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        let mut template_id = None;
        if self.template_id > Some(0) {
            template_id = self.template_id;
        }

        let mut model_id = None;
        if self.model_id > Some(0) {
            model_id = self.model_id;
        }

        let mut type_id = None;
        if self.type_id > Some(0) {
            type_id = self.type_id;
        }

        let mut website_id = None;
        if let Some(id) = self.website_id {
            if id > 0 {
                website_id = Some(id);
            }
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            name,
            template_id,
            model_id,
            type_id,
            website_id,
            status
        }
    }
}

pub struct TemplateUserDataModel;

impl TemplateUserDataModel {
    pub async fn insert(db: &DbConn, dto: &TemplateDataSaveDTO) -> Result<i64, DbErr> {
        let model = template_user_data::ActiveModel {
            id:           Set(dto.id.unwrap_or_default().to_owned()),
            template_id:  Set(dto.template_id.to_owned()),
            model_id:     Set(dto.model_id.to_owned()),
            type_id:      Set(dto.type_id.to_owned()),
            name:         Set(dto.name.to_owned()),
            temptext:     Set(dto.temptext.to_owned()),
            sort:         Set(dto.sort.to_owned()),
            status:       Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = TemplateUserData::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量删除
    /// * `db` 数据库链接
    /// * `ids` id数组
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        TemplateUserData::delete_many()
            .filter(template_user_data::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &TemplateDataSaveDTO) -> Result<i64, DbErr> {
        let model = template_user_data::ActiveModel {
            id:           Set(dto.id.unwrap_or_default().to_owned()),
            template_id:  Set(dto.template_id.to_owned()),
            model_id:     Set(dto.model_id.to_owned()),
            type_id:      Set(dto.type_id.to_owned()),
            name:         Set(dto.name.to_owned()),
            temptext:     Set(dto.temptext.to_owned()),
            sort:         Set(dto.sort.to_owned()),
            status:       Set(dto.status.to_owned()),
            ..Default::default()
        };
        let update_result: UpdateResult = TemplateUserData::update_many()
            .set(model)
            .filter(template_user_data::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_update_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = TemplateUserData::update_many()
            .set(template_user_data::ActiveModel {
                deleted: Set(Some(1).to_owned()),
                ..Default::default()
            })
            .filter(template_user_data::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_name_unique(
        db: &DbConn,
        name: &Option<String>,
        template_id: &Option<i64>,
        type_id: &Option<i32>,
        model: &Option<i32>,
        id: &Option<i64>
    ) -> Result<i64, DbErr> {
        let count = TemplateUserData::find()
            .filter(template_user_data::Column::Name.eq(name.clone().unwrap_or_default()))
            .filter(template_user_data::Column::TemplateId.eq(template_id.clone().unwrap_or_default()))
            .filter(template_user_data::Column::TypeId.eq(type_id.clone().unwrap_or_default()))
            .filter(template_user_data::Column::ModelId.eq(model.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(template_user_data::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(count)
    }
    
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template_user_data::Model>, DbErr> {
        let result = TemplateUserData::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(result)
    }

    /// 查询最新一条数据
    /// * `db` 数据库链接
    /// * `template_id` 模板id
    /// * `type_id` 类型id,1首页，2列表，3内容，4标签，5专题
    pub async fn find_latest_by_template_and_type(
        db: &DbConn,
        template_id: &Option<i64>,
        type_id: &Option<i32>,
    ) -> Result<Option<template_user_data::Model>, DbErr> {
        let data = TemplateUserData::find()
            .filter(template_user_data::Column::TemplateId.eq(template_id.clone()))
            .filter(template_user_data::Column::TypeId.eq(type_id.clone()))
            .order_by_desc(template_user_data::Column::Id)
            .one(db)
            .await?;
        Ok(data)
    }
    
    pub async fn find_by_template_id(
        db: &DbConn,
        template_id: &Option<i64>,
    ) -> Result<Vec<template_user_data::Model>, DbErr> {
        let data = TemplateUserData::find()
            .filter(template_user_data::Column::TemplateId.eq(template_id.clone()))
            .all(db)
            .await?;
        Ok(data)
    }
    

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        TemplateUserData::find()
            .distinct()
            .join_rev(
                JoinType::LeftJoin,
                website_template_merge::Relation::UserTemplate.def(),
            )
            .apply_if(wheres.name, |query, v| {
                query.filter(template_user_data::Column::Name.eq(v))
            })
            .apply_if(wheres.template_id, |query, v| {
                query.filter(template_user_data::Column::TemplateId.eq(v))
            })
            .apply_if(wheres.model_id, |query, v| {
                query.filter(template_user_data::Column::ModelId.eq(v))
            })
            .apply_if(wheres.type_id, |query, v| {
                query.filter(template_user_data::Column::TypeId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template_user_data::Column::Status.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template_user_data::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<template_user_data::Model>, i64), DbErr> {
        let paginator = TemplateUserData::find()
            .distinct()
            .join_rev(
                JoinType::LeftJoin,
                website_template_merge::Relation::UserTemplate.def(),
            )
            .apply_if(wheres.name, |query, v| {
                query.filter(template_user_data::Column::Name.eq(v))
            })
            .apply_if(wheres.template_id, |query, v| {
                query.filter(template_user_data::Column::TemplateId.eq(v))
            })
            .apply_if(wheres.model_id, |query, v| {
                query.filter(template_user_data::Column::ModelId.eq(v))
            })
            .apply_if(wheres.type_id, |query, v| {
                query.filter(template_user_data::Column::TypeId.eq(v))
            })
            .apply_if(wheres.website_id, |query, v| {
                query.filter(website_template_merge::Column::WebsiteId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template_user_data::Column::Status.eq(v))
            })
            .order_by_asc(template_user_data::Column::Id);

        let sql = paginator.build(DbBackend::Postgres);
        log::info!("===============Generated SQL: {}", sql.to_string());

        let paginator = paginator.paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}