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

use crate::modules::system::entity::dashboard_user_layout;

/// 个人布局单项（拖拽/缩放/隐藏结果）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserLayoutItem {
    /// 卡片编码
    pub card_code: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub w: Option<i32>,
    pub h: Option<i32>,
    /// 是否隐藏（1隐藏 0显示）
    pub hidden: Option<i32>,
}

/// 个人布局保存请求（方案 9.4 契约：{ pageKey, cards, reset }；reset=true 时清空该页个人覆盖，回模板布局）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserLayoutSaveRequest {
    /// 页面/工作台标识
    pub page_key: Option<String>,
    /// 重置为模板布局
    pub reset: Option<bool>,
    /// 布局条目（整页全量提交）
    pub cards: Option<Vec<UserLayoutItem>>,
}

/// 个人布局视图对象（模板布局 + 个人覆盖合并结果）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserLayoutVO {
    pub card_code: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// 是否隐藏（1隐藏 0显示）
    pub hidden: i32,
}

pub struct DashboardUserLayoutModel;

impl DashboardUserLayoutModel {
    /// 查询用户某页的全部个人覆盖
    pub async fn find_by_user_page(
        db: &DbConn,
        admin_id: i64,
        page_key: &str,
    ) -> Result<Vec<dashboard_user_layout::Model>, DbErr> {
        dashboard_user_layout::Entity::find()
            .filter(dashboard_user_layout::Column::AdminId.eq(admin_id))
            .filter(dashboard_user_layout::Column::PageKey.eq(page_key))
            .all(db)
            .await
    }

    /// 删除用户某页的全部个人覆盖（reset 与整页重存前置步骤共用）
    pub async fn delete_by_user_page<C: ConnectionTrait>(
        db: &C,
        admin_id: i64,
        page_key: &str,
    ) -> Result<u64, DbErr> {
        let res = dashboard_user_layout::Entity::delete_many()
            .filter(dashboard_user_layout::Column::AdminId.eq(admin_id))
            .filter(dashboard_user_layout::Column::PageKey.eq(page_key))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 按卡片编码集合删除个人覆盖（删除卡片时级联清理，防止残留记录阻塞后续保存）
    pub async fn delete_by_card_codes<C: ConnectionTrait>(
        db: &C,
        card_codes: &[String],
    ) -> Result<u64, DbErr> {
        if card_codes.is_empty() {
            return Ok(0);
        }
        let res = dashboard_user_layout::Entity::delete_many()
            .filter(dashboard_user_layout::Column::CardCode.is_in(card_codes.to_vec()))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 批量插入个人覆盖
    pub async fn insert_batch<C: ConnectionTrait>(
        db: &C,
        admin_id: i64,
        page_key: &str,
        items: &[UserLayoutItem],
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local();
        let rows: Vec<dashboard_user_layout::ActiveModel> = items
            .iter()
            .map(|item| dashboard_user_layout::ActiveModel {
                admin_id: Set(Some(admin_id)),
                page_key: Set(Some(page_key.to_string())),
                card_code: Set(item.card_code.clone()),
                x: Set(Some(item.x.unwrap_or(0))),
                y: Set(Some(item.y.unwrap_or(0))),
                w: Set(Some(item.w.unwrap_or(12))),
                h: Set(Some(item.h.unwrap_or(6))),
                hidden: Set(Some(item.hidden.unwrap_or(0) as i16)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .collect();
        if rows.is_empty() {
            return Ok(0);
        }
        let res = dashboard_user_layout::Entity::insert_many(rows).exec(db).await?;
        Ok(res.last_insert_id.unwrap_or_default())
    }
}
