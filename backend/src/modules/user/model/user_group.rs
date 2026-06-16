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
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::user::entity::{user_group, user_group::Entity as UserGroupEntity};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupSaveRequest {
    /// 组名
    pub name: Option<String>,
    /// 权限规则ID
    pub rules: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupUpdateRequest {
    /// ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 组名
    pub name: Option<String>,
    /// 权限规则ID
    pub rules: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}


pub struct GroupSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 组名
    pub name: Option<String>,
    /// 权限规则ID
    pub rules: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GroupListVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 组名
    pub name: Option<String>,
    /// 权限规则ID
    pub rules: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<user_group::Model> for GroupListVO {
    fn from(model: user_group::Model) -> Self {
        Self {
            id: Option::from(model.id),
            name: model.name,
            rules: model.rules,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GroupDetailVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 组名
    pub name: Option<String>,
    /// 权限规则ID
    pub rules: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

impl From<user_group::Model> for GroupDetailVO {
    fn from(model: user_group::Model) -> Self {
        Self {
            id: Option::from(model.id),
            name: model.name,
            rules: model.rules,
            status: model.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            name,
            status,
        }
    }
}




pub struct UserGroupModel;

impl UserGroupModel {
    pub async fn insert(db: &DbConn, form_data: &GroupSaveDTO) -> Result<i64, DbErr> {
        let payload = user_group::ActiveModel {
            name:             Set(form_data.name.to_owned()),
            rules:            Set(form_data.rules.to_owned()),
            status:           Set(form_data.status.to_owned()),
            create_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        UserGroupEntity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
     }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        UserGroupEntity::delete_many()
            .filter(user_group::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &GroupSaveDTO) -> Result<i64, DbErr> {
        let payload = user_group::ActiveModel {
            name:            Set(form_data.name.to_owned()),
            rules:           Set(form_data.rules.to_owned()),
            status:          Set(form_data.status.to_owned()),
            update_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = UserGroupEntity::update_many()
            .set(payload)
            .filter(user_group::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let result = UserGroupEntity::find()
            .filter(user_group::Column::Name.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(user_group::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)?;
        Ok(result)
    }
    
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<user_group::Model>, DbErr> {
        UserGroupEntity::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        UserGroupEntity::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(user_group::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user_group::Column::Status.eq(v))
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
    ) -> Result<(Vec<user_group::Model>, i64), DbErr> {
        let paginator = UserGroupEntity::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(user_group::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user_group::Column::Status.eq(v))
            })
            .order_by_desc(user_group::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}