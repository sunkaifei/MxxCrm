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

use crate::modules::system::entity::workspace;

/// 工作台保存请求（新增/更新二合一，id 空即新增）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WorkspaceSaveRequest {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 工作台编码（全局唯一，同时作为 page_key）
    pub workspace_code: Option<String>,
    /// 工作台名称
    pub workspace_name: Option<String>,
    /// 图标（lucide: 前缀）
    pub icon: Option<String>,
    /// 是否默认工作台（1是 0否）
    pub is_default: Option<i32>,
    /// 状态（1启用 0停用）
    pub status: Option<i32>,
    /// 显示顺序
    pub sort: Option<i32>,
}

/// 工作台视图对象
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceVO {
    pub id: i64,
    pub workspace_code: Option<String>,
    pub workspace_name: Option<String>,
    pub icon: Option<String>,
    pub is_default: Option<i32>,
    pub status: Option<i32>,
    pub sort: Option<i32>,
}

pub struct WorkspaceModel;

impl WorkspaceModel {
    /// 新增工作台
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &WorkspaceSaveRequest,
        create_by: &Option<String>,
    ) -> Result<i64, DbErr> {
        let payload = workspace::ActiveModel {
            workspace_code: Set(req.workspace_code.clone()),
            workspace_name: Set(req.workspace_name.clone()),
            icon: Set(req.icon.clone()),
            is_default: Set(req.is_default.or(Some(0))),
            status: Set(req.status.or(Some(1))),
            sort: Set(req.sort.or(Some(0))),
            deleted: Set(Some(0)),
            create_by: Set(create_by.clone()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        workspace::Entity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新工作台
    pub async fn update_by_id<C: ConnectionTrait>(
        db: &C,
        req: &WorkspaceSaveRequest,
    ) -> Result<u64, DbErr> {
        let mut payload = workspace::ActiveModel {
            ..Default::default()
        };
        if let Some(v) = req.workspace_code.clone() {
            payload.workspace_code = Set(Some(v));
        }
        if let Some(v) = req.workspace_name.clone() {
            payload.workspace_name = Set(Some(v));
        }
        if let Some(v) = req.icon.clone() {
            payload.icon = Set(Some(v));
        }
        if let Some(v) = req.is_default {
            payload.is_default = Set(Some(v));
        }
        if let Some(v) = req.status {
            payload.status = Set(Some(v));
        }
        if let Some(v) = req.sort {
            payload.sort = Set(Some(v));
        }
        let res = workspace::Entity::update_many()
            .set(payload)
            .filter(workspace::Column::Id.eq(req.id.unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 软删除工作台
    pub async fn soft_delete_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<u64, DbErr> {
        let payload = workspace::ActiveModel {
            deleted: Set(Some(1)),
            ..Default::default()
        };
        let res = workspace::Entity::update_many()
            .set(payload)
            .filter(workspace::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 按 ID 查询
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<workspace::Model>, DbErr> {
        workspace::Entity::find_by_id(id).one(db).await
    }

    /// 按编码统计（校验唯一性，exclude_id 用于编辑时排除自身）
    pub async fn find_by_code(db: &DbConn, code: &str, exclude_id: Option<i64>) -> Result<i64, DbErr> {
        workspace::Entity::find()
            .filter(workspace::Column::WorkspaceCode.eq(code))
            .filter(workspace::Column::Deleted.eq(0))
            .apply_if(exclude_id, |q, v| q.filter(workspace::Column::Id.ne(v)))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 查询全部启用工作台（list 接口用，仅需登录）
    pub async fn find_all_enabled(db: &DbConn) -> Result<Vec<workspace::Model>, DbErr> {
        workspace::Entity::find()
            .filter(workspace::Column::Deleted.eq(0))
            .filter(workspace::Column::Status.eq(1))
            .order_by_asc(workspace::Column::Sort)
            .order_by_asc(workspace::Column::Id)
            .all(db)
            .await
    }

    /// 将其它工作台的 is_default 置 0（保证唯一默认）
    pub async fn clear_default_except<C: ConnectionTrait>(db: &C, keep_id: i64) -> Result<u64, DbErr> {
        let payload = workspace::ActiveModel {
            is_default: Set(Some(0)),
            ..Default::default()
        };
        let res = workspace::Entity::update_many()
            .set(payload)
            .filter(workspace::Column::Deleted.eq(0))
            .filter(workspace::Column::Id.ne(keep_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }
}
