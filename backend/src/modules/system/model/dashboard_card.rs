//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::DateTime;
use sea_orm::*;

use crate::modules::system::entity::{
    dashboard_card, dashboard_card::Entity as DashboardCard, dashboard_card_role_merge,
    dashboard_card_role_merge::Entity as DashboardCardRoleMerge,
};

/// 卡片新增/更新请求
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DashboardCardSaveRequest {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 卡片编码（全局唯一）
    pub card_code: Option<String>,
    /// 卡片名称
    pub card_name: Option<String>,
    /// 所属页面标识（如 finance/payslip）
    pub page_key: Option<String>,
    /// 显示顺序
    pub sort_order: Option<i32>,
    /// 状态（1启用 0停用）
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

/// 卡片-角色分配请求
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DashboardCardAssignRolesRequest {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub card_id: Option<i64>,
    /// 角色ID集合（空数组=清空所有可见角色）
    pub role_ids: Option<Vec<i64>>,
}

/// 卡片列表查询参数
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCardListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(rename = "name")]
    pub keywords: Option<String>,
    pub page_key: Option<String>,
    pub status: Option<i32>,
}

/// 卡片视图对象（含已分配角色ID）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCardVO {
    pub id: i64,
    pub card_code: Option<String>,
    pub card_name: Option<String>,
    pub page_key: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    /// 已分配可见角色ID集合
    pub role_ids: Vec<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

/// 卡片-角色关联批量保存 DTO
#[derive(PartialEq, Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardCardRoleMergeSaveDTO {
    pub id: Option<i64>,
    pub card_id: Option<i64>,
    pub role_id: Option<i64>,
    pub create_time: Option<DateTime>,
}

pub struct DashboardCardModel;

impl DashboardCardModel {
    /// 新增卡片
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &DashboardCardSaveRequest,
        create_by: &Option<String>,
    ) -> Result<i64, DbErr> {
        let payload = dashboard_card::ActiveModel {
            card_code: Set(req.card_code.clone()),
            card_name: Set(req.card_name.clone()),
            page_key: Set(req.page_key.clone()),
            sort_order: Set(req.sort_order),
            status: Set(req.status.or(Some(1))),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            create_by: Set(create_by.clone()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        DashboardCard::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新卡片
    pub async fn update_by_id<C: ConnectionTrait>(
        db: &C,
        req: &DashboardCardSaveRequest,
        update_by: &Option<String>,
    ) -> Result<i64, DbErr> {
        let mut payload = dashboard_card::ActiveModel {
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        if let Some(v) = req.card_code.clone() {
            payload.card_code = Set(Some(v));
        }
        if let Some(v) = req.card_name.clone() {
            payload.card_name = Set(Some(v));
        }
        if let Some(v) = req.page_key.clone() {
            payload.page_key = Set(Some(v));
        }
        if let Some(v) = req.sort_order {
            payload.sort_order = Set(Some(v));
        }
        if let Some(v) = req.status {
            payload.status = Set(Some(v));
        }
        if let Some(v) = req.remark.clone() {
            payload.remark = Set(Some(v));
        }
        if let Some(v) = update_by.clone() {
            payload.update_by = Set(Some(v));
        }

        let res = DashboardCard::update_many()
            .set(payload)
            .filter(dashboard_card::Column::Id.eq(req.id.unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }

    /// 软删除卡片
    pub async fn soft_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let payload = dashboard_card::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = DashboardCard::update_many()
            .set(payload)
            .filter(dashboard_card::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }

    /// 查询单个卡片
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<dashboard_card::Model>, DbErr> {
        DashboardCard::find_by_id(id).one(db).await
    }

    /// 按编码统计（校验唯一性，exclude_id 用于编辑时排除自身）
    pub async fn find_by_code(db: &DbConn, code: &str, exclude_id: Option<i64>) -> Result<i64, DbErr> {
        DashboardCard::find()
            .filter(dashboard_card::Column::CardCode.eq(code))
            .filter(dashboard_card::Column::Deleted.eq(0))
            .apply_if(exclude_id, |q, v| q.filter(dashboard_card::Column::Id.ne(v)))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 查询所有启用卡片（visible 接口用）
    pub async fn find_all_enabled(db: &DbConn) -> Result<Vec<dashboard_card::Model>, DbErr> {
        DashboardCard::find()
            .filter(dashboard_card::Column::Deleted.eq(0))
            .filter(dashboard_card::Column::Status.eq(1))
            .order_by_asc(dashboard_card::Column::SortOrder)
            .all(db)
            .await
    }
}

pub struct DashboardCardRoleMergeModel;

impl DashboardCardRoleMergeModel {
    /// 批量插入卡片-角色关联
    pub async fn insert_batch<C: ConnectionTrait>(
        db: &C,
        list: &Vec<DashboardCardRoleMergeSaveDTO>,
    ) -> Result<i64, DbErr> {
        let result: Vec<dashboard_card_role_merge::ActiveModel> = list
            .iter()
            .map(|item| dashboard_card_role_merge::ActiveModel {
                card_id: Set(item.card_id),
                role_id: Set(item.role_id),
                create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .collect();
        let insert_result = DashboardCardRoleMerge::insert_many(result).exec(db).await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }

    /// 按卡片ID删除关联
    pub async fn delete_by_card_id<C: ConnectionTrait>(db: &C, card_id: &Option<i64>) -> Result<i64, DbErr> {
        let res = DashboardCardRoleMerge::delete_many()
            .filter(dashboard_card_role_merge::Column::CardId.eq(card_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }

    /// 按卡片ID集合删除关联（删除卡片时清理）
    pub async fn delete_by_card_ids<C: ConnectionTrait>(db: &C, card_ids: &Vec<i64>) -> Result<i64, DbErr> {
        let res = DashboardCardRoleMerge::delete_many()
            .filter(dashboard_card_role_merge::Column::CardId.is_in(card_ids.clone()))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }

    /// 按卡片ID集合查询全部关联（组装 VO 用）
    pub async fn find_by_card_ids(db: &DbConn, card_ids: &Vec<i64>) -> Result<Vec<dashboard_card_role_merge::Model>, DbErr> {
        DashboardCardRoleMerge::find()
            .filter(dashboard_card_role_merge::Column::CardId.is_in(card_ids.clone()))
            .all(db)
            .await
    }

    /// 按角色ID集合查询可见卡片ID
    pub async fn find_card_ids_by_role_ids(db: &DbConn, role_ids: &Vec<i64>) -> Result<Vec<i64>, DbErr> {
        let list = DashboardCardRoleMerge::find()
            .filter(dashboard_card_role_merge::Column::RoleId.is_in(role_ids.clone()))
            .all(db)
            .await?;
        Ok(list.into_iter().filter_map(|m| m.card_id).collect())
    }
}
