use sea_orm::*;
use sea_orm::entity::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{tag_group, tag_group::Entity as TagGroup};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagGroupSaveRequest {
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub sort_order: Option<i32>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
}

impl From<TagGroupSaveRequest> for TagGroupSaveDTO {
    fn from(item: TagGroupSaveRequest) -> Self {
        TagGroupSaveDTO {
            id: None,
            group_name: item.group_name,
            group_color: item.group_color,
            sort_order: item.sort_order,
            description: item.description,
            is_global: item.is_global,
            created_by: None,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagGroupUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub sort_order: Option<i32>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
}

impl From<TagGroupUpdateRequest> for TagGroupSaveDTO {
    fn from(item: TagGroupUpdateRequest) -> Self {
        TagGroupSaveDTO {
            id: item.id,
            group_name: item.group_name,
            group_color: item.group_color,
            sort_order: item.sort_order,
            description: item.description,
            is_global: item.is_global,
            created_by: None,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagGroupSaveDTO {
    pub id: Option<i64>,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub sort_order: Option<i32>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagGroupListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub sort_order: Option<i32>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub created_at: Option<DateTime>,
    pub tag_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagGroupDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub sort_order: Option<i32>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
}

impl From<tag_group::Model> for TagGroupDetailVO {
    fn from(item: tag_group::Model) -> Self {
        TagGroupDetailVO {
            id: Option::from(item.id),
            group_name: item.group_name,
            group_color: item.group_color,
            sort_order: item.sort_order,
            description: item.description,
            is_global: item.is_global,
            created_by: item.created_by,
            created_at: item.created_at,
            updated_by: item.updated_by,
            updated_at: item.updated_at,
        }
    }
}

pub struct TagGroupModel;

impl TagGroupModel {
    pub async fn insert(db: &DbConn, req: &TagGroupSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = tag_group::ActiveModel {
            group_name: Set(req.group_name.clone()),
            group_color: Set(req.group_color.clone()),
            sort_order: Set(req.sort_order.clone()),
            description: Set(req.description.clone()),
            is_global: Set(req.is_global.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
            ..Default::default()
        };
        TagGroup::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &TagGroupSaveDTO) -> Result<i64, DbErr> {
        let payload = tag_group::ActiveModel {
            group_name: Set(req.group_name.clone()),
            group_color: Set(req.group_color.clone()),
            sort_order: Set(req.sort_order.clone()),
            description: Set(req.description.clone()),
            is_global: Set(req.is_global.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = TagGroup::update_many()
            .set(payload)
            .filter(tag_group::Column::Id.eq(id.clone().unwrap_or_default()))
            .filter(tag_group::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64, DbErr> {
        let payload = tag_group::ActiveModel {
            deleted: Set(Some(1)),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = TagGroup::update_many()
            .set(payload)
            .filter(tag_group::Column::Id.eq(id))
            .filter(tag_group::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let payload = tag_group::ActiveModel {
            deleted: Set(Some(1)),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = TagGroup::update_many()
            .set(payload)
            .filter(tag_group::Column::Id.is_in(ids.clone()))
            .filter(tag_group::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<tag_group::Model>, DbErr> {
        TagGroup::find_by_id(id)
            .filter(tag_group::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn find_all(db: &DbConn) -> Result<Vec<tag_group::Model>, DbErr> {
        TagGroup::find()
            .filter(tag_group::Column::Deleted.eq(0))
            .order_by_asc(tag_group::Column::SortOrder)
            .all(db)
            .await
    }

    pub async fn find_by_name_unique(db: &DbConn, group_name: &Option<String>, exclude_id: &Option<i64>) -> Result<bool, DbErr> {
        let mut query = TagGroup::find().filter(tag_group::Column::GroupName.eq(group_name.clone().unwrap_or_default()));
        query = query.filter(tag_group::Column::Deleted.eq(0));
        if let Some(id) = exclude_id {
            query = query.filter(tag_group::Column::Id.ne(id));
        }
        let count = query.count(db).await?;
        Ok(count > 0)
    }
}
