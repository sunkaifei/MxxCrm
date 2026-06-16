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
use crate::modules::system::entity::{role, role::Entity as Role};
use crate::utils::string_utils::{deserialize_string_to_u64,serialize_option_u64_to_string,deserialize_string_vec_to_u64_vec};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RoleSaveRequest {
    /// 角色名称
    pub role_name: Option<String>,
    pub role_key: Option<String>,
    pub data_scope: Option<i32>,
    /// 显示顺序
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

impl From<RoleSaveRequest> for RoleSaveDTO {
    fn from(item: RoleSaveRequest) -> Self {
        RoleSaveDTO {
            id: None,
            role_name: item.role_name,
            role_key: item.role_key,
            data_scope: item.data_scope,
            sort: item.sort,
            status: item.status,
            remark: item.remark,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RoleUpdateRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub role_name: Option<String>,
    /// 角色权限字符串
    pub role_key: Option<String>,
    pub data_scope: Option<i32>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

impl From<RoleUpdateRequest> for RoleSaveDTO {
    fn from(item: RoleUpdateRequest) -> Self {
        RoleSaveDTO {
            id: item.id,
            role_name: item.role_name,
            role_key: item.role_key,
            data_scope: item.data_scope,
            sort: item.sort,
            status: item.status,
            remark: item.remark,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UpdateRoleMenuRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub role_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_string_vec_to_u64_vec")]
    pub menu_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RoleSaveDTO {
    pub id: Option<i64>,
    pub role_name: Option<String>,
    /// 角色权限字符串
    pub role_key: Option<String>,
    pub data_scope: Option<i32>,
    pub sort: Option<i32>,
    /// 角色状态（0停用 1正常）
    pub status: Option<i32>,
    /// 删除标志（0代表存在 2代表删除）
    pub deleted: Option<i32>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminRoleByName {
    pub admin_id: Option<i64>,
    pub role_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RoleDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub role_name: Option<String>,
    /// 角色权限字符串
    pub role_key: Option<String>,
    /// 数据范围（1：全部数据权限 2：自定数据权限 3：本部门数据权限 4：本部门及以下数据权限）
    pub data_scope: Option<i32>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

impl From<role::Model> for RoleDetailVO {
    fn from(item: role::Model) -> Self {
        RoleDetailVO {
            id: Option::from(item.id),
            role_name: item.role_name,
            role_key: item.role_key,
            data_scope: item.data_scope,
            remark: item.remark,
            sort: item.sort,
            status: item.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RoleListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub role_name: Option<String>,
    /// 角色权限字符串
    pub role_key: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

impl From<role::Model> for RoleListVO {
    fn from(item: role::Model) -> Self {
        RoleListVO {
            id: Option::from(item.id),
            role_name: item.role_name,
            role_key: item.role_key,
            remark: item.remark,
            sort: item.sort,
            status: item.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RoleOptionVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub value: Option<i64>,
    pub label: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(rename = "name")]
    pub keywords: Option<String>,
    pub status: Option<i32>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub role_name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut role_name = None;
        if self.role_name != Some("".to_string()) {
            role_name = self.role_name.clone();
        }
        
        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            role_name,
            status,
        }
    }
}

pub struct RoleModel;

impl RoleModel {
    /// 新增角色
    pub async fn insert(db: &DbConn, req: &RoleSaveDTO) -> Result<i64, DbErr> {
        let payload = role::ActiveModel {
            role_name:   Set(req.role_name.clone()),
            role_key:    Set(req.role_key.clone()),
            data_scope:  Set(req.data_scope.clone()),
            sort:        Set(req.sort.clone()),
            status:      Set(req.status.clone()),
            remark:      Set(req.remark.clone()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        Role::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Role::delete_many()
            .filter(role::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    /// # 更新
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &RoleSaveDTO) -> Result<i64, DbErr> {
        let payload = role::ActiveModel {
            role_name:      Set(req.role_name.clone()),
            role_key:       Set(req.role_key.clone()),
            data_scope:     Set(req.data_scope.clone()),
            sort:           Set(req.sort.clone()),
            status:         Set(req.status.clone()),
            remark:         Set(req.remark.clone()),
            update_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        let update_result: UpdateResult = Role::update_many()
            .set(payload)
            .filter(role::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    /// # 根据角色名称查询角色数量
    /// * `db` 数据库连接
    /// * `name` 角色名称
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let result = Role::find()
            .filter(role::Column::RoleName.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(role::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)?;
        Ok(result)
    }
    
    /// # 查询单个
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<role::Model>, DbErr> {
        let result = Role::find_by_id(id)
            .one(db)
            .await?;
        Ok(result)
    }
    
    /// 批量查询
    pub async fn find_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<Vec<role::Model>, DbErr> {
        let result = Role::find()
            .filter(role::Column::Id.is_in(ids))
            .all(db)
            .await;
        Ok(result?)
    }

    /// 查询所有
    pub async fn find_all(db: &DbConn) -> Result<Vec<role::Model>, DbErr> {
        let result = Role::find()
            .filter(role::Column::Status.eq(1))
            .order_by_asc(role::Column::Id)
            .all(db)
            .await;
        Ok(result?)
    }


    /// 根据角色名称查询
    pub async fn find_by_role_name(db: &DbConn, role_name: &str) -> Result<Option<role::Model>, DbErr> {
        Role::find()
            .filter(role::Column::RoleName.eq(role_name))
            .one(db)
            .await
    }
    
    pub async fn find_by_role_key(db: &DbConn, role_key: &str) -> Result<Option<role::Model>, DbErr> {
        Role::find()
            .filter(role::Column::RoleKey.eq(role_key))
            .one(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Role::find()
            .apply_if(wheres.role_name, |query, v| {
                query.filter(role::Column::RoleName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(role::Column::Status.eq(v))
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
    ) -> Result<(Vec<role::Model>, i64), DbErr> {
        let paginator = Role::find()
            .apply_if(wheres.role_name, |query, v| {
                query.filter(role::Column::RoleName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(role::Column::Status.eq(v))
            })
            .order_by_asc(role::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
    
}