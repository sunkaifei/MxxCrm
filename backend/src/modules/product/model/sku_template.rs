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
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::product::entity::sku_template;
use crate::modules::product::entity::sku_template::Entity as SkuTemplate;
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 规格值保存项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SpecValueSaveItem {
    pub value: String,
    pub sort_order: Option<i32>,
}

/// 规格保存项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SpecSaveItem {
    pub name: String,
    pub sort_order: Option<i32>,
    pub values: Vec<SpecValueSaveItem>,
}

/// SKU模板保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SkuTemplateSaveRequest {
    pub template_name: String,
    pub product_type: Option<String>,
    pub description: Option<String>,
}

/// SKU模板更新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SkuTemplateUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub template_name: Option<String>,
    pub product_type: Option<String>,
    pub description: Option<String>,
}

/// 规格值VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SpecValueVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub value: Option<String>,
    pub sort_order: Option<i32>,
}

/// 规格VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SpecVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub values: Vec<SpecValueVO>,
}

/// SKU模板详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SkuTemplateDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub template_name: Option<String>,
    pub product_type: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub specs: Vec<SpecVO>,
}

/// SKU模板列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SkuTemplateListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub template_name: Option<String>,
    pub product_type: Option<String>,
    pub description: Option<String>,
    pub spec_count: i64,
    pub create_time: Option<DateTime>,
}

impl From<sku_template::Model> for SkuTemplateListVO {
    fn from(item: sku_template::Model) -> Self {
        SkuTemplateListVO {
            id: Some(item.id),
            template_name: Some(item.template_name),
            product_type: item.product_type,
            description: item.description,
            spec_count: 0,
            create_time: item.create_time,
        }
    }
}

/// SKU模板列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkuTemplateListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub product_type: Option<String>,
}

/// SKU模板规格批量保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TemplateSpecBatchSaveRequest {
    pub template_id: i64,
    pub specs: Vec<SpecSaveItem>,
}

pub struct SkuTemplateModel;

impl SkuTemplateModel {
    pub async fn insert(db: &DbConn, req: &SkuTemplateSaveRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = sku_template::ActiveModel {
            template_name: Set(req.template_name.clone()),
            product_type: Set(req.product_type.clone()),
            description: Set(req.description.clone()),
            created_by: Set(None),
            create_time: Set(Some(now)),
            ..Default::default()
        };

        SkuTemplate::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update_by_id(db: &DbConn, id: i64, req: &SkuTemplateUpdateRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = sku_template::ActiveModel {
            template_name: Set(req.template_name.clone().unwrap_or_default()),
            product_type: Set(req.product_type.clone()),
            description: Set(req.description.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result = SkuTemplate::update_many()
            .set(payload)
            .filter(sku_template::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64, DbErr> {
        SkuTemplate::update_many()
            .set(sku_template::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(sku_template::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<sku_template::Model>, DbErr> {
        SkuTemplate::find_by_id(id)
            .filter(sku_template::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        product_type: Option<String>,
    ) -> Result<(Vec<sku_template::Model>, i64), DbErr> {
        let mut query = SkuTemplate::find()
            .filter(sku_template::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(sku_template::Column::TemplateName.contains(k));
        }
        if let Some(pt) = product_type {
            query = query.filter(sku_template::Column::ProductType.eq(pt));
        }

        let paginator = query.order_by_desc(sku_template::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}
