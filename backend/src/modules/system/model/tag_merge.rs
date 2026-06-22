use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{tag_merge, tag_merge::Entity as TagMerge};
use crate::modules::system::entity::{tag, tag::Entity as Tag};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityRequest {
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityRemoveRequest {
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityBatchRequest {
    pub entity_type: Option<String>,
    pub entity_ids: Option<Vec<i64>>,
    pub tag_ids: Option<Vec<i64>>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityResult {
    pub added_count: Option<i64>,
    pub existing_count: Option<i64>,
}

pub struct TagMergeModel;

impl TagMergeModel {
    pub async fn add_tags_to_entity(
        db: &DbConn,
        entity_type: &str,
        entity_id: i64,
        tag_ids: &Vec<i64>,
    ) -> Result<(i64, i64), DbErr> {
        let mut added_count = 0i64;
        let mut existing_count = 0i64;

        for &tag_id in tag_ids {
            let exists = TagMerge::find()
                .filter(tag_merge::Column::TagId.eq(tag_id))
                .filter(tag_merge::Column::EntityType.eq(entity_type))
                .filter(tag_merge::Column::EntityId.eq(entity_id))
                .count(db)
                .await?;

            if exists > 0 {
                existing_count += 1;
            } else {
                let payload = tag_merge::ActiveModel {
                    tag_id: Set(Some(tag_id)),
                    entity_type: Set(Some(entity_type.to_string())),
                    entity_id: Set(Some(entity_id)),
                    created_at: Set(Option::from(chrono::Utc::now())),
                    ..Default::default()
                };
                TagMerge::insert(payload).exec(db).await?;
                added_count += 1;
            }
        }

        Ok((added_count, existing_count))
    }

    pub async fn remove_tags_from_entity(
        db: &DbConn,
        entity_type: &str,
        entity_id: i64,
        tag_ids: &Vec<i64>,
    ) -> Result<i64, DbErr> {
        let delete_result = TagMerge::delete_many()
            .filter(tag_merge::Column::EntityType.eq(entity_type))
            .filter(tag_merge::Column::EntityId.eq(entity_id))
            .filter(tag_merge::Column::TagId.is_in(tag_ids.clone()))
            .exec(db)
            .await?;
        Ok(delete_result.rows_affected as i64)
    }

    pub async fn get_tags_by_entity(
        db: &DbConn,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<tag::Model>, DbErr> {
        Tag::find()
            .join(
                JoinType::InnerJoin,
                tag_merge::Relation::Tag.def().rev(),
            )
            .filter(tag_merge::Column::EntityType.eq(entity_type))
            .filter(tag_merge::Column::EntityId.eq(entity_id))
            .filter(tag::Column::Deleted.eq(0))
            .order_by_desc(tag_merge::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn batch_add_tags_to_entities(
        db: &DbConn,
        entity_type: &str,
        entity_ids: &Vec<i64>,
        tag_ids: &Vec<i64>,
    ) -> Result<i64, DbErr> {
        let mut total_added = 0i64;
        for &entity_id in entity_ids {
            let (added, _) = Self::add_tags_to_entity(db, entity_type, entity_id, tag_ids).await?;
            total_added += added;
        }
        Ok(total_added)
    }

    pub async fn batch_remove_tags_from_entities(
        db: &DbConn,
        entity_type: &str,
        entity_ids: &Vec<i64>,
        tag_ids: &Vec<i64>,
    ) -> Result<i64, DbErr> {
        let mut total_removed = 0i64;
        for &entity_id in entity_ids {
            let removed = Self::remove_tags_from_entity(db, entity_type, entity_id, tag_ids).await?;
            total_removed += removed;
        }
        Ok(total_removed)
    }

    pub async fn count_tag_usage(db: &DbConn, tag_id: i64) -> Result<i64, DbErr> {
        TagMerge::find()
            .filter(tag_merge::Column::TagId.eq(tag_id))
            .count(db)
            .await
            .map(|c| c as i64)
    }
}
