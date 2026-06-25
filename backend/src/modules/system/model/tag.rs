use sea_orm::*;
use chrono::NaiveDateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{tag, tag::Entity as Tag};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagSaveRequest {
    pub group_id: Option<i64>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
}

impl From<TagSaveRequest> for TagSaveDTO {
    fn from(item: TagSaveRequest) -> Self {
        TagSaveDTO {
            id: None,
            group_id: item.group_id,
            tag_name: item.tag_name,
            tag_color: item.tag_color,
            description: item.description,
            is_global: item.is_global,
            created_by: None,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub group_id: Option<i64>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
}

impl From<TagUpdateRequest> for TagSaveDTO {
    fn from(item: TagUpdateRequest) -> Self {
        TagSaveDTO {
            id: item.id,
            group_id: item.group_id,
            tag_name: item.tag_name,
            tag_color: item.tag_color,
            description: item.description,
            is_global: item.is_global,
            created_by: None,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagMoveToGroupRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub group_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagSaveDTO {
    pub id: Option<i64>,
    pub group_id: Option<i64>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub group_id: Option<i64>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub created_by: Option<i64>,
    pub create_time: Option<NaiveDateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<NaiveDateTime>,
}

impl From<tag::Model> for TagDetailVO {
    fn from(item: tag::Model) -> Self {
        TagDetailVO {
            id: Option::from(item.id),
            group_id: item.group_id,
            tag_name: item.tag_name,
            tag_color: item.tag_color,
            description: item.description,
            is_global: item.is_global,
            created_by: item.created_by,
            create_time: item.create_time,
            updated_by: item.updated_by,
            update_time: item.update_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub tag_name: Option<String>,
    pub group_id: Option<i64>,
    pub is_global: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagSuggestVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagStatisticsVO {
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub tag_count: Option<i64>,
}

pub struct TagModel;

impl TagModel {
    pub async fn insert(db: &DbConn, req: &TagSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = tag::ActiveModel {
            group_id: Set(req.group_id.clone()),
            tag_name: Set(req.tag_name.clone()),
            tag_color: Set(req.tag_color.clone()),
            description: Set(req.description.clone()),
            is_global: Set(req.is_global.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };
        Tag::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &TagSaveDTO) -> Result<i64, DbErr> {
        let payload = tag::ActiveModel {
            group_id: Set(req.group_id.clone()),
            tag_name: Set(req.tag_name.clone()),
            tag_color: Set(req.tag_color.clone()),
            description: Set(req.description.clone()),
            is_global: Set(req.is_global.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        let update_result: UpdateResult = Tag::update_many()
            .set(payload)
            .filter(tag::Column::Id.eq(id.clone().unwrap_or_default()))
            .filter(tag::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64, DbErr> {
        let payload = tag::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        let update_result: UpdateResult = Tag::update_many()
            .set(payload)
            .filter(tag::Column::Id.eq(id))
            .filter(tag::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let payload = tag::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        let update_result: UpdateResult = Tag::update_many()
            .set(payload)
            .filter(tag::Column::Id.is_in(ids.clone()))
            .filter(tag::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<tag::Model>, DbErr> {
        Tag::find_by_id(id)
            .filter(tag::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        tag_name: Option<String>,
        group_id: Option<i64>,
        is_global: Option<bool>,
    ) -> Result<(Vec<tag::Model>, i64), DbErr> {
        let mut query = Tag::find().filter(tag::Column::Deleted.eq(0));
        if let Some(n) = tag_name {
            query = query.filter(tag::Column::TagName.contains(n));
        }
        if let Some(g) = group_id {
            query = query.filter(tag::Column::GroupId.eq(g));
        }
        if let Some(global) = is_global {
            query = query.filter(tag::Column::IsGlobal.eq(global));
        }
        let paginator = query.order_by_desc(tag::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn find_by_name_unique(db: &DbConn, tag_name: &Option<String>, exclude_id: &Option<i64>) -> Result<bool, DbErr> {
        let mut query = Tag::find().filter(tag::Column::TagName.eq(tag_name.clone().unwrap_or_default()));
        query = query.filter(tag::Column::Deleted.eq(0));
        if let Some(id) = exclude_id {
            query = query.filter(tag::Column::Id.ne(id));
        }
        let count = query.count(db).await?;
        Ok(count > 0)
    }

    pub async fn find_by_group(db: &DbConn, group_id: i64) -> Result<Vec<tag::Model>, DbErr> {
        Tag::find()
            .filter(tag::Column::GroupId.eq(group_id))
            .filter(tag::Column::Deleted.eq(0))
            .order_by_desc(tag::Column::CreateTime)
            .all(db)
            .await
    }

    pub async fn suggest(db: &DbConn, keyword: &str) -> Result<Vec<tag::Model>, DbErr> {
        Tag::find()
            .filter(tag::Column::TagName.contains(keyword))
            .filter(tag::Column::Deleted.eq(0))
            .limit(20)
            .all(db)
            .await
    }

    pub async fn get_statistics(db: &DbConn) -> Result<Vec<(i64, String, i64)>, DbErr> {
        use crate::modules::system::entity::{tag_group, tag_group::Entity as TagGroup};
        Ok(Vec::new())
    }

    pub async fn find_all(db: &DbConn) -> Result<Vec<tag::Model>, DbErr> {
        Tag::find()
            .filter(tag::Column::Deleted.eq(0))
            .order_by_desc(tag::Column::CreateTime)
            .all(db)
            .await
    }

    pub async fn move_to_group(db: &DbConn, group_id: i64, tag_ids: &Vec<i64>) -> Result<i64, DbErr> {
        let payload = tag::ActiveModel {
            group_id: Set(Some(group_id)),
            update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        let update_result: UpdateResult = Tag::update_many()
            .set(payload)
            .filter(tag::Column::Id.is_in(tag_ids.clone()))
            .filter(tag::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }
}
