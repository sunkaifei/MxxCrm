use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{tag_merge, tag_merge::Entity as TagMerge};
use crate::modules::system::entity::{tag, tag::Entity as Tag};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityRequest {
    pub entity_type: Option<String>,
    #[serde(deserialize_with = "flex_i64_opt")]
    pub entity_id: Option<i64>,
    #[serde(deserialize_with = "flex_i64_vec_opt")]
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityRemoveRequest {
    pub entity_type: Option<String>,
    #[serde(deserialize_with = "flex_i64_opt")]
    pub entity_id: Option<i64>,
    #[serde(deserialize_with = "flex_i64_vec_opt")]
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagEntityBatchRequest {
    pub entity_type: Option<String>,
    #[serde(deserialize_with = "flex_i64_vec_opt")]
    pub entity_ids: Option<Vec<i64>>,
    #[serde(deserialize_with = "flex_i64_vec_opt")]
    pub tag_ids: Option<Vec<i64>>,
    pub action: Option<String>,
}

/// 灵活反序列化 Option<i64>：接受 JSON 数字或字符串（如 "4" 或 4），null/missing → None
fn flex_i64_opt<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    use serde::de;
    struct V;
    impl<'de> de::Visitor<'de> for V {
        type Value = Option<i64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a number, a string number, or null")
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<i64>, E> { Ok(Some(v)) }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<i64>, E> {
            v.parse::<i64>().map(Some).map_err(de::Error::custom)
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<i64>, E> { Ok(None) }
        fn visit_unit<E: de::Error>(self) -> Result<Option<i64>, E> { Ok(None) }
    }
    d.deserialize_any(V)
}

/// 灵活反序列化 Option<Vec<i64>>：接受 JSON 数组元素为数字或字符串
fn flex_i64_vec_opt<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Vec<i64>>, D::Error> {
    use serde::de;
    struct V;
    impl<'de> de::Visitor<'de> for V {
        type Value = Option<Vec<i64>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an array of numbers or string numbers, or null")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Option<Vec<i64>>, A::Error> {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<serde_json::Value>()? {
                match elem {
                    serde_json::Value::Number(n) => {
                        if let Some(v) = n.as_i64() { vec.push(v); }
                        else if let Some(v) = n.as_u64() { vec.push(v as i64); }
                        else if let Some(v) = n.as_f64() { vec.push(v as i64); }
                        else { return Err(de::Error::custom("invalid number in array")); }
                    }
                    serde_json::Value::String(s) => {
                        vec.push(s.parse::<i64>().map_err(de::Error::custom)?);
                    }
                    _ => return Err(de::Error::custom("expected number or string in array")),
                }
            }
            Ok(Some(vec))
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<Vec<i64>>, E> { Ok(None) }
        fn visit_unit<E: de::Error>(self) -> Result<Option<Vec<i64>>, E> { Ok(None) }
    }
    d.deserialize_any(V)
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
                    create_time: Set(Option::from(chrono::Utc::now().naive_utc())),
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
            .order_by_desc(tag_merge::Column::CreateTime)
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
