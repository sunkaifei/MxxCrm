use sea_orm::*;
use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::tag;
use crate::modules::system::model::tag::{TagListVO, TagDetailVO, TagSuggestVO, TagStatisticsVO, TagSaveDTO, TagModel};
use crate::modules::system::model::tag_group::{TagGroupModel};
use crate::modules::system::model::tag_merge::{TagEntityResult, TagMergeModel};

pub struct TagService;

impl TagService {
    pub async fn get_list(
        db: &DbConn,
        page: i64,
        per_page: i64,
        tag_name: Option<String>,
        group_id: Option<i64>,
        is_global: Option<bool>,
    ) -> Result<(Vec<TagListVO>, i64)> {
        let (tags, num_pages) = TagModel::select_in_page(db, page, per_page, tag_name, group_id, is_global).await.map_err(|e| Error::Database(e.to_string()))?;
        let groups = TagGroupModel::find_all(db).await.map_err(|e| Error::Database(e.to_string()))?;
        let group_map: std::collections::HashMap<i64, (String, String)> = groups.into_iter()
            .filter(|g| g.group_name.is_some())
            .map(|g| (g.id, (g.group_name.unwrap(), g.group_color.unwrap_or("#1890ff".to_string()))))
            .collect();
        let vo_list: Vec<TagListVO> = tags.into_iter()
            .map(|tag| {
                let (group_name, group_color) = tag.group_id
                    .and_then(|gid| group_map.get(&gid).cloned())
                    .unwrap_or_else(|| ("未分组".to_string(), "#d9d9d9".to_string()));
                TagListVO {
                    id: Some(tag.id),
                    group_id: tag.group_id,
                    group_name: Some(group_name),
                    group_color: Some(group_color),
                    tag_name: tag.tag_name,
                    tag_color: tag.tag_color,
                    description: tag.description,
                    is_global: tag.is_global,
                    create_time: tag.create_time,
                }
            })
            .collect();
        Ok((vo_list, num_pages))
    }

    pub async fn get_by_id(db: &DbConn, id: i64) -> Result<TagDetailVO> {
        let tag = TagModel::find_by_id(db, id).await.map_err(|e| Error::Database(e.to_string()))?;
        match tag {
            Some(t) => Ok(t.into()),
            None => Err(Error::NotFound("标签不存在".to_string())),
        }
    }

    pub async fn save(db: &DbConn, mut req: TagSaveDTO, admin_id: Option<i64>) -> Result<bool> {
        if req.tag_name.as_ref().unwrap_or(&"".to_string()).trim().is_empty() {
            return Err(Error::BadRequest("标签名称不能为空".to_string()));
        }
        let exist = TagModel::find_by_name_unique(db, &req.tag_name, &req.id).await.map_err(|e| Error::Database(e.to_string()))?;
        if exist {
            return Err(Error::BadRequest("标签名称已存在".to_string()));
        }
        req.created_by = admin_id;
        req.updated_by = admin_id;
        if req.id.is_none() {
            TagModel::insert(db, &req).await.map_err(|e| Error::Database(e.to_string()))?;
        } else {
            TagModel::update_by_id(db, &req.id, &req).await.map_err(|e| Error::Database(e.to_string()))?;
        }
        Ok(true)
    }

    pub async fn delete(db: &DbConn, id: i64) -> Result<bool> {
        let tag = TagModel::find_by_id(db, id).await.map_err(|e| Error::Database(e.to_string()))?;
        if tag.is_none() {
            return Err(Error::NotFound("标签不存在".to_string()));
        }
        TagModel::delete_by_id(db, id).await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(true)
    }

    pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<bool> {
        TagModel::batch_delete_by_ids(db, ids).await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(true)
    }

    pub async fn suggest(db: &DbConn, keyword: &str) -> Result<Vec<TagSuggestVO>> {
        let tags = TagModel::suggest(db, keyword).await.map_err(|e| Error::Database(e.to_string()))?;
        let groups = TagGroupModel::find_all(db).await.map_err(|e| Error::Database(e.to_string()))?;
        let group_map: std::collections::HashMap<i64, (String, String)> = groups.into_iter()
            .filter(|g| g.group_name.is_some())
            .map(|g| (g.id, (g.group_name.unwrap(), g.group_color.unwrap_or("#1890ff".to_string()))))
            .collect();
        let vo_list: Vec<TagSuggestVO> = tags.into_iter()
            .map(|tag| {
                let (group_name, group_color) = tag.group_id
                    .and_then(|gid| group_map.get(&gid).cloned())
                    .unwrap_or_else(|| ("未分组".to_string(), "#d9d9d9".to_string()));
                TagSuggestVO {
                    id: Some(tag.id),
                    group_id: tag.group_id,
                    group_name: Some(group_name),
                    group_color: Some(group_color),
                    tag_name: tag.tag_name,
                    tag_color: tag.tag_color,
                }
            })
            .collect();
        Ok(vo_list)
    }

    pub async fn get_statistics(db: &DbConn) -> Result<Vec<TagStatisticsVO>> {
        let stats = TagModel::get_statistics(db).await.map_err(|e| Error::Database(e.to_string()))?;
        let vo_list: Vec<TagStatisticsVO> = stats.into_iter()
            .map(|(group_id, group_name, count)| TagStatisticsVO {
                group_id: Some(group_id),
                group_name: Some(group_name),
                tag_count: Some(count),
            })
            .collect();
        Ok(vo_list)
    }

    pub async fn get_tags_by_group(db: &DbConn, group_id: i64) -> Result<Vec<TagListVO>> {
        let tags = TagModel::find_by_group(db, group_id).await.map_err(|e| Error::Database(e.to_string()))?;
        let group = TagGroupModel::find_by_id(db, group_id).await.map_err(|e| Error::Database(e.to_string()))?;
        let (group_name, group_color) = group
            .map(|g| (g.group_name.unwrap_or_default(), g.group_color.unwrap_or("#1890ff".to_string())))
            .unwrap_or_else(|| ("未分组".to_string(), "#d9d9d9".to_string()));
        let vo_list: Vec<TagListVO> = tags.into_iter()
            .map(|tag| TagListVO {
                id: Some(tag.id),
                group_id: Some(group_id),
                group_name: Some(group_name.clone()),
                group_color: Some(group_color.clone()),
                tag_name: tag.tag_name,
                tag_color: tag.tag_color,
                description: tag.description,
                is_global: tag.is_global,
                create_time: tag.create_time,
            })
            .collect();
        Ok(vo_list)
    }

    pub async fn get_all_tags(db: &DbConn) -> Result<Vec<TagListVO>> {
        let tags = TagModel::find_all(db).await.map_err(|e| Error::Database(e.to_string()))?;
        let groups = TagGroupModel::find_all(db).await.map_err(|e| Error::Database(e.to_string()))?;
        let group_map: std::collections::HashMap<i64, (String, String)> = groups.into_iter()
            .filter(|g| g.group_name.is_some())
            .map(|g| (g.id, (g.group_name.unwrap(), g.group_color.unwrap_or("#1890ff".to_string()))))
            .collect();
        let vo_list: Vec<TagListVO> = tags.into_iter()
            .map(|tag| {
                let (group_name, group_color) = tag.group_id
                    .and_then(|gid| group_map.get(&gid).cloned())
                    .unwrap_or_else(|| ("未分组".to_string(), "#d9d9d9".to_string()));
                TagListVO {
                    id: Some(tag.id),
                    group_id: tag.group_id,
                    group_name: Some(group_name),
                    group_color: Some(group_color),
                    tag_name: tag.tag_name,
                    tag_color: tag.tag_color,
                    description: tag.description,
                    is_global: tag.is_global,
                    create_time: tag.create_time,
                }
            })
            .collect();
        Ok(vo_list)
    }

    pub async fn move_to_group(db: &DbConn, group_id: i64, tag_ids: &Vec<i64>) -> Result<bool> {
        let group = TagGroupModel::find_by_id(db, group_id).await.map_err(|e| Error::Database(e.to_string()))?;
        if group.is_none() {
            return Err(Error::NotFound("分组不存在".to_string()));
        }
        TagModel::move_to_group(db, group_id, tag_ids).await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(true)
    }

    pub async fn add_tags_to_entity(db: &DbConn, entity_type: &str, entity_id: i64, tag_ids: &Vec<i64>) -> Result<TagEntityResult> {
        let (added_count, existing_count) = TagMergeModel::add_tags_to_entity(db, entity_type, entity_id, tag_ids).await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(TagEntityResult {
            added_count: Some(added_count),
            existing_count: Some(existing_count),
        })
    }

    pub async fn remove_tags_from_entity(db: &DbConn, entity_type: &str, entity_id: i64, tag_ids: &Vec<i64>) -> Result<i64> {
        TagMergeModel::remove_tags_from_entity(db, entity_type, entity_id, tag_ids).await.map_err(|e| Error::Database(e.to_string()))
    }

    pub async fn get_tags_by_entity(db: &DbConn, entity_type: &str, entity_id: i64) -> Result<Vec<TagListVO>> {
        let tags = TagMergeModel::get_tags_by_entity(db, entity_type, entity_id).await.map_err(|e| Error::Database(e.to_string()))?;
        let groups = TagGroupModel::find_all(db).await.map_err(|e| Error::Database(e.to_string()))?;
        let group_map: std::collections::HashMap<i64, (String, String)> = groups.into_iter()
            .filter(|g| g.group_name.is_some())
            .map(|g| (g.id, (g.group_name.unwrap(), g.group_color.unwrap_or("#1890ff".to_string()))))
            .collect();
        let vo_list: Vec<TagListVO> = tags.into_iter()
            .map(|tag| {
                let (group_name, group_color) = tag.group_id
                    .and_then(|gid| group_map.get(&gid).cloned())
                    .unwrap_or_else(|| ("未分组".to_string(), "#d9d9d9".to_string()));
                TagListVO {
                    id: Some(tag.id),
                    group_id: tag.group_id,
                    group_name: Some(group_name),
                    group_color: Some(group_color),
                    tag_name: tag.tag_name,
                    tag_color: tag.tag_color,
                    description: tag.description,
                    is_global: tag.is_global,
                    create_time: tag.create_time,
                }
            })
            .collect();
        Ok(vo_list)
    }

    pub async fn batch_add_tags_to_entities(db: &DbConn, entity_type: &str, entity_ids: &Vec<i64>, tag_ids: &Vec<i64>) -> Result<i64> {
        TagMergeModel::batch_add_tags_to_entities(db, entity_type, entity_ids, tag_ids).await.map_err(|e| Error::Database(e.to_string()))
    }

    pub async fn batch_remove_tags_from_entities(db: &DbConn, entity_type: &str, entity_ids: &Vec<i64>, tag_ids: &Vec<i64>) -> Result<i64> {
        TagMergeModel::batch_remove_tags_from_entities(db, entity_type, entity_ids, tag_ids).await.map_err(|e| Error::Database(e.to_string()))
    }
}
