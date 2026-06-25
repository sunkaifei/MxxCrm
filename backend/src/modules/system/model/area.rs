use chrono::NaiveDateTime;
use sea_orm::*;
use sea_orm::prelude::Decimal;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{area, area::Entity as Area};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaSaveRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub sort: Option<i32>,
    pub country_code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

impl From<AreaSaveRequest> for AreaSaveDTO {
    fn from(request: AreaSaveRequest) -> Self {
        Self {
            id: None,
            parent_id: request.parent_id,
            name: request.name,
            name_en: request.name_en,
            code: request.code,
            level: request.level,
            sort: request.sort,
            country_code: request.country_code,
            latitude: request.latitude,
            longitude: request.longitude,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub sort: Option<i32>,
    pub country_code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

impl From<AreaUpdateRequest> for AreaSaveDTO {
    fn from(request: AreaUpdateRequest) -> Self {
        Self {
            id: request.id,
            parent_id: request.parent_id,
            name: request.name,
            name_en: request.name_en,
            code: request.code,
            level: request.level,
            sort: request.sort,
            country_code: request.country_code,
            latitude: request.latitude,
            longitude: request.longitude,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AreaSaveDTO {
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub sort: Option<i32>,
    pub country_code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub sort: Option<i32>,
    pub country_code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

impl From<area::Model> for AreaDetailVO {
    fn from(model: area::Model) -> Self {
        Self {
            id: Option::from(model.id),
            parent_id: model.parent_id,
            name: model.name,
            name_en: model.name_en,
            code: model.code,
            level: model.level,
            sort: model.sort,
            country_code: model.country_code,
            latitude: model.latitude,
            longitude: model.longitude,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaTreeVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<AreaTreeVO>>,
}

impl AreaTreeVO {
    pub fn from_model(model: &crate::modules::system::entity::area::Model, _all: &[crate::modules::system::entity::area::Model]) -> Self {
        Self {
            id: Option::from(model.id),
            parent_id: model.parent_id,
            name: model.name.clone(),
            name_en: model.name_en.clone(),
            code: model.code.clone(),
            level: model.level,
            country_code: model.country_code.clone(),
            children: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaCascaderVO {
    pub value: Option<String>,
    pub label: Option<String>,
    pub label_en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<AreaCascaderVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub name: Option<String>,
    pub level: Option<i32>,
    pub country_code: Option<String>,
}

pub struct AreaModel;

impl AreaModel {
    pub async fn insert(db: &DbConn, form_data: &AreaSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = area::ActiveModel {
            parent_id: Set(form_data.parent_id.to_owned()),
            name: Set(form_data.name.to_owned()),
            name_en: Set(form_data.name_en.to_owned()),
            code: Set(form_data.code.to_owned()),
            level: Set(form_data.level.to_owned()),
            sort: Set(form_data.sort.to_owned()),
            country_code: Set(form_data.country_code.to_owned()),
            latitude: Set(form_data.latitude.to_owned()),
            longitude: Set(form_data.longitude.to_owned()),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };
        Area::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        Area::delete_many()
            .filter(area::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: i64, form_data: AreaSaveDTO) -> Result<i64, DbErr> {
        let payload = area::ActiveModel {
            parent_id: Set(form_data.parent_id.to_owned()),
            name: Set(form_data.name.to_owned()),
            name_en: Set(form_data.name_en.to_owned()),
            code: Set(form_data.code.to_owned()),
            level: Set(form_data.level.to_owned()),
            sort: Set(form_data.sort.to_owned()),
            country_code: Set(form_data.country_code.to_owned()),
            latitude: Set(form_data.latitude.to_owned()),
            longitude: Set(form_data.longitude.to_owned()),
            update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let update_result: UpdateResult = Area::update_many()
            .set(payload)
            .filter(area::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<area::Model>, DbErr> {
        Area::find_by_id(id)
            .one(db)
            .await
    }

    pub async fn find_by_parent_id(db: &DbConn, parent_id: i64) -> Result<Vec<area::Model>, DbErr> {
        Area::find()
            .filter(area::Column::ParentId.eq(parent_id))
            .order_by_asc(area::Column::Sort)
            .order_by_asc(area::Column::Id)
            .all(db)
            .await
    }

    pub async fn find_by_country_code(db: &DbConn, country_code: &str) -> Result<Vec<area::Model>, DbErr> {
        Area::find()
            .filter(area::Column::CountryCode.eq(country_code))
            .order_by_asc(area::Column::Level)
            .order_by_asc(area::Column::Sort)
            .all(db)
            .await
    }

    pub async fn find_countries(db: &DbConn) -> Result<Vec<area::Model>, DbErr> {
        Area::find()
            .filter(area::Column::Level.eq(1))
            .order_by_asc(area::Column::Sort)
            .order_by_asc(area::Column::Id)
            .all(db)
            .await
    }

    pub async fn find_all(db: &DbConn) -> Result<Vec<area::Model>, DbErr> {
        Area::find()
            .order_by_asc(area::Column::Level)
            .order_by_asc(area::Column::Sort)
            .all(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        name: Option<String>,
        level: Option<i32>,
        country_code: Option<String>,
    ) -> Result<(Vec<area::Model>, i64), DbErr> {
        let mut query = Area::find();

        if let Some(n) = name {
            query = query.filter(area::Column::Name.contains(n));
        }
        if let Some(l) = level {
            query = query.filter(area::Column::Level.eq(l));
        }
        if let Some(c) = country_code {
            query = query.filter(area::Column::CountryCode.eq(c));
        }

        let paginator = query
            .order_by_asc(area::Column::Level)
            .order_by_asc(area::Column::Sort)
            .paginate(db, per_page as u64);
        let total_items = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total_items))
    }
}
