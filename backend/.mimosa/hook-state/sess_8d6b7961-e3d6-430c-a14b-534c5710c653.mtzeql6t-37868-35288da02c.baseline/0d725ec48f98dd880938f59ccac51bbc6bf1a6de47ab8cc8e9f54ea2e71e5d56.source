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
use crate::modules::system::entity::{perm_set, perm_set::Entity as PermSet};
use crate::utils::string_utils::{deserialize_string_to_u64,serialize_option_u64_to_string,deserialize_string_vec_to_u64_vec};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PermSetSaveRequest {
    /// 权限集名称
    pub perm_set_name: Option<String>,
    /// 权限集权限字符串
    pub perm_set_key: Option<String>,
    /// 显示顺序
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

impl From<PermSetSaveRequest> for PermSetSaveDTO {
    fn from(item: PermSetSaveRequest) -> Self {
        PermSetSaveDTO {
            id: None,
            perm_set_name: item.perm_set_name,
            perm_set_key: item.perm_set_key,
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
pub struct PermSetUpdateRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 权限集名称
    pub perm_set_name: Option<String>,
    /// 权限集权限字符串
    pub perm_set_key: Option<String>,
    /// 显示顺序
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

impl From<PermSetUpdateRequest> for PermSetSaveDTO {
    fn from(item: PermSetUpdateRequest) -> Self {
        PermSetSaveDTO {
            id: item.id,
            perm_set_name: item.perm_set_name,
            perm_set_key: item.perm_set_key,
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
pub struct UpdatePermSetMenuRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub perm_set_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_string_vec_to_u64_vec")]
    pub menu_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PermSetSaveDTO {
    pub id: Option<i64>,
    /// 权限集名称
    pub perm_set_name: Option<String>,
    /// 权限集权限字符串
    pub perm_set_key: Option<String>,
    pub sort: Option<i32>,
    /// 权限集状态（0停用 1正常）
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
#[serde(rename_all(serialize = "camelCase"))]
pub struct PermSetDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub perm_set_name: Option<String>,
    pub perm_set_key: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

impl From<perm_set::Model> for PermSetDetailVO {
    fn from(item: perm_set::Model) -> Self {
        PermSetDetailVO {
            id: Option::from(item.id),
            perm_set_name: item.perm_set_name,
            perm_set_key: item.perm_set_key,
            remark: item.remark,
            sort: item.sort,
            status: item.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PermSetListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub perm_set_name: Option<String>,
    /// 权限集权限字符串
    pub perm_set_key: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub create_time: Option<String>,
}

impl From<perm_set::Model> for PermSetListVO {
    fn from(item: perm_set::Model) -> Self {
        PermSetListVO {
            id: Option::from(item.id),
            perm_set_name: item.perm_set_name,
            perm_set_key: item.perm_set_key,
            remark: item.remark,
            sort: item.sort,
            status: item.status,
            create_time: item.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PermSetOptionVO {
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
    pub perm_set_name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut perm_set_name = None;
        if self.perm_set_name != Some("".to_string()) {
            perm_set_name = self.perm_set_name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            perm_set_name,
            status,
        }
    }
}

pub struct PermSetModel;

impl PermSetModel {
    /// 新增权限集
    pub async fn insert(db: &DbConn, req: &PermSetSaveDTO) -> Result<i64, DbErr> {
        let payload = perm_set::ActiveModel {
            perm_set_name: Set(req.perm_set_name.clone()),
            perm_set_key:  Set(req.perm_set_key.clone()),
            sort:          Set(req.sort.clone()),
            status:        Set(req.status.clone()),
            remark:        Set(req.remark.clone()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        PermSet::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        PermSet::delete_many()
            .filter(perm_set::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// # 更新
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &PermSetSaveDTO) -> Result<i64, DbErr> {
        let mut payload = perm_set::ActiveModel {
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        if let Some(v) = req.perm_set_name.clone() { payload.perm_set_name = Set(Some(v)); }
        if let Some(v) = req.perm_set_key.clone() { payload.perm_set_key = Set(Some(v)); }
        if let Some(v) = req.sort { payload.sort = Set(Some(v)); }
        if let Some(v) = req.status { payload.status = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.update_by.clone() { payload.update_by = Set(Some(v)); }

        let update_result: UpdateResult = PermSet::update_many()
            .set(payload)
            .filter(perm_set::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// # 根据权限集名称查询权限集数量
    /// * `db` 数据库连接
    /// * `name` 权限集名称
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let result = PermSet::find()
            .filter(perm_set::Column::PermSetName.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(perm_set::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)?;
        Ok(result)
    }

    /// # 查询单个
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<perm_set::Model>, DbErr> {
        let result = PermSet::find_by_id(id)
            .one(db)
            .await?;
        Ok(result)
    }

    /// 根据权限集key查询（P3-1 复制时用于检测 key 冲突）
    pub async fn find_by_perm_set_key(db: &DbConn, perm_set_key: &str) -> Result<Option<perm_set::Model>, DbErr> {
        PermSet::find()
            .filter(perm_set::Column::PermSetKey.eq(perm_set_key))
            .one(db)
            .await
    }

    /// 批量查询
    pub async fn find_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<Vec<perm_set::Model>, DbErr> {
        let result = PermSet::find()
            .filter(perm_set::Column::Id.is_in(ids))
            .all(db)
            .await;
        Ok(result?)
    }

    /// 查询所有
    pub async fn find_all(db: &DbConn) -> Result<Vec<perm_set::Model>, DbErr> {
        let result = PermSet::find()
            .filter(perm_set::Column::Status.eq(1))
            .order_by_asc(perm_set::Column::Id)
            .all(db)
            .await;
        Ok(result?)
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        PermSet::find()
            .apply_if(wheres.perm_set_name, |query, v| {
                query.filter(perm_set::Column::PermSetName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(perm_set::Column::Status.eq(v))
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
    ) -> Result<(Vec<perm_set::Model>, i64), DbErr> {
        let paginator = PermSet::find()
            .apply_if(wheres.perm_set_name, |query, v| {
                query.filter(perm_set::Column::PermSetName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(perm_set::Column::Status.eq(v))
            })
            .order_by_asc(perm_set::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

}
