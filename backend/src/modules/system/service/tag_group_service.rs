use sea_orm::*;
use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::tag_group;
use crate::modules::system::model::tag_group::{TagGroupListVO, TagGroupDetailVO, TagGroupSaveDTO, TagGroupModel};

pub struct TagGroupService;

impl TagGroupService {
    pub async fn get_list(db: &DbConn) -> Result<Vec<TagGroupListVO>> {
        let groups = TagGroupModel::find_all(db).await?;
        let mut vo_list: Vec<TagGroupListVO> = Vec::new();
        for group in groups {
            let tag_count = TagGroupService::count_tags_in_group(db, group.id).await?;
            vo_list.push(TagGroupListVO {
                id: Some(group.id),
                group_name: group.group_name,
                group_color: group.group_color,
                sort_order: group.sort_order,
                description: group.description,
                is_global: group.is_global,
                create_time: group.create_time,
                tag_count: Some(tag_count),
            });
        }
        Ok(vo_list)
    }

    pub async fn get_by_id(db: &DbConn, id: i64) -> Result<TagGroupDetailVO> {
        let group = TagGroupModel::find_by_id(db, id).await?;
        match group {
            Some(g) => Ok(g.into()),
            None => Err(Error::from("分组不存在")),
        }
    }

    pub async fn save(db: &DbConn, mut req: TagGroupSaveDTO, admin_id: Option<i64>) -> Result<bool> {
        if req.group_name.as_ref().unwrap_or(&"".to_string()).trim().is_empty() {
            return Err(Error::from("分组名称不能为空"));
        }
        let exist = TagGroupModel::find_by_name_unique(db, &req.group_name, &req.id).await?;
        if exist {
            return Err(Error::from("分组名称已存在"));
        }
        req.created_by = admin_id;
        req.updated_by = admin_id;
        if req.id.is_none() {
            TagGroupModel::insert(db, &req).await?;
        } else {
            TagGroupModel::update_by_id(db, &req.id, &req).await?;
        }
        Ok(true)
    }

    pub async fn delete(db: &DbConn, id: i64) -> Result<bool> {
        let group = TagGroupModel::find_by_id(db, id).await?;
        if group.is_none() {
            return Err(Error::from("分组不存在"));
        }
        let tag_count = TagGroupService::count_tags_in_group(db, id).await?;
        if tag_count > 0 {
            return Err(Error::from("分组下存在标签，无法删除"));
        }
        TagGroupModel::delete_by_id(db, id).await?;
        Ok(true)
    }

    pub async fn batch_delete(db: &DbConn, ids: &Vec<i64>) -> Result<bool> {
        for &id in ids {
            let tag_count = TagGroupService::count_tags_in_group(db, id).await?;
            if tag_count > 0 {
                return Err(Error::from("部分分组下存在标签，无法删除"));
            }
        }
        TagGroupModel::batch_delete_by_ids(db, ids).await?;
        Ok(true)
    }

    pub async fn count_tags_in_group(db: &DbConn, group_id: i64) -> Result<i64> {
        use crate::modules::system::entity::{tag, tag::Entity as Tag};
        let count = Tag::find()
            .filter(tag::Column::GroupId.eq(group_id))
            .filter(tag::Column::Deleted.eq(0))
            .count(db)
            .await?;
        Ok(count as i64)
    }
}
