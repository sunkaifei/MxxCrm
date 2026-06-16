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
use crate::modules::shop::entity::shop_supplier_apply::{self, Entity as ApplyEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::DateTime;
use sea_orm::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRequest {
    pub shop_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub shop_logo: Option<String>,
    pub shop_desc: Option<String>,
    pub business_license: Option<String>,
}

pub struct ApplyDTO {
    pub user_id: i64,
    pub shop_name: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub shop_logo: Option<String>,
    pub shop_desc: Option<String>,
    pub business_license: Option<String>,
    pub status: i16,
}

impl From<ApplyRequest> for ApplyDTO {
    fn from(req: ApplyRequest) -> Self {
        ApplyDTO {
            user_id: 0,
            shop_name: req.shop_name.unwrap_or_default(),
            contact_name: req.contact_name.unwrap_or_default(),
            contact_phone: req.contact_phone.unwrap_or_default(),
            shop_logo: req.shop_logo,
            shop_desc: req.shop_desc,
            business_license: req.business_license,
            status: 0,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApplyVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    pub shop_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub shop_logo: Option<String>,
    pub shop_desc: Option<String>,
    pub business_license: Option<String>,
    pub status: Option<i16>,
    pub audit_remark: Option<String>,
    pub audit_time: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<shop_supplier_apply::Model> for ApplyVO {
    fn from(model: shop_supplier_apply::Model) -> Self {
        Self {
            id: Some(model.id),
            user_id: Some(model.user_id),
            shop_name: Some(model.shop_name),
            contact_name: Some(model.contact_name),
            contact_phone: Some(model.contact_phone),
            shop_logo: model.shop_logo,
            shop_desc: model.shop_desc,
            business_license: model.business_license,
            status: Some(model.status),
            audit_remark: model.audit_remark,
            audit_time: model.audit_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPageQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<i16>,
}

pub struct SupplierApplyModel;

impl SupplierApplyModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &ApplyDTO) -> Result<i64, DbErr> {
        let active = shop_supplier_apply::ActiveModel {
            user_id: Set(form.user_id),
            shop_name: Set(form.shop_name.clone()),
            contact_name: Set(form.contact_name.clone()),
            contact_phone: Set(form.contact_phone.clone()),
            shop_logo: Set(form.shop_logo.clone()),
            shop_desc: Set(form.shop_desc.clone()),
            business_license: Set(form.business_license.clone()),
            status: Set(form.status),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        ApplyEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn find_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Option<shop_supplier_apply::Model>, DbErr> {
        ApplyEntity::find()
            .filter(shop_supplier_apply::Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_supplier_apply::Model>, DbErr> {
        ApplyEntity::find_by_id(id).one(db).await
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i16, audit_remark: Option<String>) -> Result<i64, DbErr> {
        let active = shop_supplier_apply::ActiveModel {
            id: Set(id),
            status: Set(status),
            audit_remark: Set(audit_remark),
            audit_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        ApplyEntity::update_many()
            .set(active)
            .filter(shop_supplier_apply::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_page<C: ConnectionTrait>(db: &C, page: i64, per_page: i64, status: Option<i16>) -> Result<(Vec<shop_supplier_apply::Model>, i64), DbErr> {
        let paginator = ApplyEntity::find()
            .apply_if(status, |query, v| {
                query.filter(shop_supplier_apply::Column::Status.eq(v))
            })
            .order_by_desc(shop_supplier_apply::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;
        Ok((items, total))
    }
}
