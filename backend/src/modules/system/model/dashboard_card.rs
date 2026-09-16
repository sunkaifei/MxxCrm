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
    /// 模板布局：栅格列偏移（方案 5.1 变更 1）
    pub default_x: Option<i32>,
    /// 模板布局：栅格行偏移
    pub default_y: Option<i32>,
    /// 模板布局：宽（列数 1-12）
    pub default_w: Option<i32>,
    /// 模板布局：高（行数）
    pub default_h: Option<i32>,
    /// 状态（1启用 0停用）
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 卡片配置（JSON：显示形态 displayForm / 时间范围 timeRange / 数据口径 dataScope / 图例等）
    pub card_config: Option<String>,
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

/// 角色视角预览查询参数（管理页增强：选角色预览其工作台组合）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CardRolePreviewQuery {
    pub role_id: Option<i64>,
}

/// 设计器模板布局单项（方案 5.2：POST /dashboard/card/layout/save）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CardLayoutItem {
    /// 卡片编码
    pub card_code: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub w: Option<i32>,
    pub h: Option<i32>,
}

/// 设计器模板布局保存请求
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CardLayoutSaveRequest {
    /// 页面/工作台标识
    pub page_key: Option<String>,
    /// 布局条目（该页全量提交）
    pub items: Option<Vec<CardLayoutItem>>,
}

/// 设计器模板布局视图对象
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CardLayoutVO {
    pub id: i64,
    pub card_code: Option<String>,
    pub card_name: Option<String>,
    pub page_key: Option<String>,
    /// 状态（1启用 0停用，设计器置灰展示）
    pub status: Option<i32>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// 卡片视图对象（含已分配角色ID）
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCardVO {
    pub id: i64,
    pub card_code: Option<String>,
    pub card_name: Option<String>,
    pub page_key: Option<String>,
    pub sort_order: Option<i32>,
    /// 模板布局：栅格列偏移
    pub default_x: Option<i32>,
    /// 模板布局：栅格行偏移
    pub default_y: Option<i32>,
    /// 模板布局：宽（列数）
    pub default_w: Option<i32>,
    /// 模板布局：高（行数）
    pub default_h: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    /// 卡片配置（JSON）
    pub card_config: Option<String>,
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
            default_x: Set(req.default_x),
            default_y: Set(req.default_y),
            default_w: Set(req.default_w.or(Some(12))),
            default_h: Set(req.default_h.or(Some(6))),
            status: Set(req.status.or(Some(1))),
            remark: Set(req.remark.clone()),
            card_config: Set(req.card_config.clone()),
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
        if let Some(v) = req.default_x {
            payload.default_x = Set(Some(v));
        }
        if let Some(v) = req.default_y {
            payload.default_y = Set(Some(v));
        }
        if let Some(v) = req.default_w {
            payload.default_w = Set(Some(v));
        }
        if let Some(v) = req.default_h {
            payload.default_h = Set(Some(v));
        }
        if let Some(v) = req.status {
            payload.status = Set(Some(v));
        }
        if let Some(v) = req.remark.clone() {
            payload.remark = Set(Some(v));
        }
        if let Some(v) = req.card_config.clone() {
            payload.card_config = Set(Some(v));
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

    /// 查询某页可布局卡片（含停用，设计器模板布局用）
    ///
    /// 内容模型（方案 v2.2 修订）：专属工作台 = 通用卡(page_key='default') ∪ 本工作台专属卡。
    /// 过滤语义必须与运行端 `get_user_layout` 一致，否则设计器选中专属工作台时只剩专属卡
    /// （sales/hr/finance 各 1 张、warehouse 3 张），通用卡无法编排位置。
    pub async fn find_by_page_key_with_default(
        db: &DbConn,
        page_key: &str,
    ) -> Result<Vec<dashboard_card::Model>, DbErr> {
        DashboardCard::find()
            .filter(dashboard_card::Column::Deleted.eq(0))
            .filter(
                dashboard_card::Column::PageKey
                    .eq(page_key)
                    .or(dashboard_card::Column::PageKey.eq("default")),
            )
            .order_by_asc(dashboard_card::Column::SortOrder)
            .order_by_asc(dashboard_card::Column::Id)
            .all(db)
            .await
    }

    /// 按编码更新模板布局（设计器保存，方案 5.2）
    pub async fn update_layout_by_code<C: ConnectionTrait>(
        db: &C,
        card_code: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> Result<u64, DbErr> {
        let payload = dashboard_card::ActiveModel {
            default_x: Set(Some(x)),
            default_y: Set(Some(y)),
            default_w: Set(Some(w)),
            default_h: Set(Some(h)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = DashboardCard::update_many()
            .set(payload)
            .filter(dashboard_card::Column::CardCode.eq(card_code))
            .filter(dashboard_card::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
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
