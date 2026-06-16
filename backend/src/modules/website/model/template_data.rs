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
use crate::modules::website::entity::{template_data, template_data::Entity as TemplateData};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateDataSaveRequest {
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

impl From<template_data::Model> for TemplateDataListVO {
    fn from(model: template_data::Model) -> Self {
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

impl From<template_data::Model> for TemplateDataDetailVO {
    fn from(model: template_data::Model) -> Self {
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

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            name,
            template_id,
            model_id,
            type_id,
            status,
        }
    }
}

pub struct TemplateDataModel;

impl TemplateDataModel {
    pub async fn insert(db: &DbConn, dto: &TemplateDataSaveDTO) -> Result<i64, DbErr> {
        let model = template_data::ActiveModel {
            id:           Set(dto.id.unwrap_or_default().to_owned()),
            template_id:  Set(dto.template_id.to_owned()),
            model_id:     Set(dto.model_id.to_owned()),
            type_id:      Set(dto.type_id.to_owned()),
            name:         Set(dto.name.to_owned()),
            temptext:     Set(dto.temptext.to_owned()),
            sort:         Set(dto.sort.to_owned()),
            status:       Set(dto.status.to_owned()),
            create_time:  Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = TemplateData::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量删除
    /// * `db` 数据库链接
    /// * `ids` id数组
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        TemplateData::delete_many()
            .filter(template_data::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &TemplateDataSaveDTO) -> Result<i64, DbErr> {
        let model = template_data::ActiveModel {
            template_id:  Set(dto.template_id.to_owned()),
            model_id:     Set(dto.model_id.to_owned()),
            type_id:      Set(dto.type_id.to_owned()),
            name:         Set(dto.name.to_owned()),
            temptext:     Set(dto.temptext.to_owned()),
            sort:         Set(dto.sort.to_owned()),
            status:       Set(dto.status.to_owned()),
            ..Default::default()
        };
        let update_result: UpdateResult = TemplateData::update_many()
            .set(model)
            .filter(template_data::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_update_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = TemplateData::update_many()
            .set(template_data::ActiveModel {
                deleted: Set(Some(1).to_owned()),
                ..Default::default()
            })
            .filter(template_data::Column::Id.is_in(ids))
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
        let count = TemplateData::find()
            .filter(template_data::Column::Name.eq(name.clone().unwrap_or_default()))
            .filter(template_data::Column::TemplateId.eq(template_id.clone().unwrap_or_default()))
            .filter(template_data::Column::TypeId.eq(type_id.clone().unwrap_or_default()))
            .filter(template_data::Column::ModelId.eq(model.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(template_data::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(count)
    }
    
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template_data::Model>, DbErr> {
        let result = template_data::Entity::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(result)
    }
    
    pub async fn select_by_template_id(
        db: &DbConn,
        template_id: &Option<i64>,
    ) -> Result<Vec<template_data::Model>, DbErr> {
        let query = template_data::Entity::find()
            .filter(template_data::Column::TemplateId.eq(template_id.clone().unwrap_or_default()))
            .filter(template_data::Column::Status.eq(1))
            .order_by_asc(template_data::Column::Sort);
            
        let sql = query.build(DbBackend::Postgres);
        log::info!("template_data SQL: {}", sql.to_string());
        let result = query.all(db)
            .await?;
        Ok(result)
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        TemplateData::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(template_data::Column::Name.eq(v))
            })
            .apply_if(wheres.template_id, |query, v| {
                query.filter(template_data::Column::TemplateId.eq(v))
            })
            .apply_if(wheres.model_id, |query, v| {
                query.filter(template_data::Column::ModelId.eq(v))
            })
            .apply_if(wheres.type_id, |query, v| {
                query.filter(template_data::Column::TypeId.eq(v))
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
    ) -> Result<(Vec<template_data::Model>, i64), DbErr> {
        let query = TemplateData::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(template_data::Column::Name.eq(v))
            })
            .apply_if(wheres.template_id, |query, v| {
                query.filter(template_data::Column::TemplateId.eq(v))
            })
            .apply_if(wheres.model_id, |query, v| {
                query.filter(template_data::Column::ModelId.eq(v))
            })
            .apply_if(wheres.type_id, |query, v| {
                query.filter(template_data::Column::TypeId.eq(v))
            })
            .order_by_desc(template_data::Column::Id);
            

        let sql = query.build(DbBackend::Postgres);
        log::info!("template_data SQL: {}", sql.to_string());
        
        let paginator = query.paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}