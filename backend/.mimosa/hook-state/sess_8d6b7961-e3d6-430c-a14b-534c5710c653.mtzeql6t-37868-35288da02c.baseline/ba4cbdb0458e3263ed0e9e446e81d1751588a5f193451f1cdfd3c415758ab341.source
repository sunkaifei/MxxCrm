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
use crate::modules::website::entity::{leave_msg, leave_msg::Entity as LeaveMsg};

/// 留言提交请求DTO（前台访客提交）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeaveMsgSubmitRequest {
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 联系邮箱
    pub contact_email: Option<String>,
    /// 留言内容
    pub content: Option<String>,
    /// 关联产品ID（来自产品页咨询）
    pub product_id: Option<i64>,
    /// 栏目ID
    pub category_id: Option<i64>,
    /// 来源页URL
    pub source_url: Option<String>,
}

/// 留言列表查询参数（后台管理）
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeaveMsgListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub website_id: Option<i64>,
    pub status: Option<i32>,
}

/// 转线索请求（后台管理员手动转线索）
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ConvertLeadRequest {
    /// 分配给哪个负责人（admin id）
    pub assigned_to: i64,
}

/// 留言保存DTO（内部使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeaveMsgSaveDTO {
    pub id: Option<i64>,
    pub website_id: Option<i64>,
    pub category_id: Option<i64>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub content: Option<String>,
    pub status: Option<i32>,
    pub product_id: Option<i64>,
    pub source_url: Option<String>,
    pub source: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub lead_id: Option<i64>,
    pub converted_to_lead: Option<i32>,
    pub convert_lead_id: Option<i64>,
    pub remark: Option<String>,
}

impl From<LeaveMsgSubmitRequest> for LeaveMsgSaveDTO {
    fn from(item: LeaveMsgSubmitRequest) -> Self {
        LeaveMsgSaveDTO {
            id: None,
            website_id: None,
            category_id: item.category_id,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            content: item.content,
            status: Some(0),
            product_id: item.product_id,
            source_url: item.source_url,
            source: Some("website".to_string()),
            ip_address: None,
            user_agent: None,
            lead_id: None,
            converted_to_lead: Some(0),
            convert_lead_id: None,
            remark: None,
        }
    }
}

/// 留言详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeaveMsgDetailVO {
    pub id: Option<i64>,
    pub website_id: Option<i64>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub content: Option<String>,
    pub status: Option<i32>,
    pub product_id: Option<i64>,
    pub source_url: Option<String>,
    pub source: Option<String>,
    pub lead_id: Option<i64>,
    pub converted_to_lead: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
}

impl From<leave_msg::Model> for LeaveMsgDetailVO {
    fn from(item: leave_msg::Model) -> Self {
        LeaveMsgDetailVO {
            id: Option::from(item.id),
            website_id: item.website_id,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            content: item.content,
            status: item.status,
            product_id: item.product_id,
            source_url: item.source_url,
            source: item.source,
            lead_id: item.lead_id,
            converted_to_lead: item.converted_to_lead,
            remark: item.remark,
            create_time: item.create_time,
        }
    }
}

/// 留言数据模型操作类
pub struct LeaveMsgModel;

impl LeaveMsgModel {
    /// 新增留言
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &LeaveMsgSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = leave_msg::ActiveModel {
            id: Set(req.id.unwrap_or_default()),
            website_id: Set(req.website_id),
            category_id: Set(req.category_id),
            contact_name: Set(req.contact_name.clone()),
            contact_phone: Set(req.contact_phone.clone()),
            contact_email: Set(req.contact_email.clone()),
            content: Set(req.content.clone()),
            status: Set(Some(req.status.unwrap_or(0))),
            convert_lead_id: Set(req.convert_lead_id),
            product_id: Set(req.product_id),
            source_url: Set(req.source_url.clone()),
            source: Set(req.source.clone().or_else(|| Some("website".to_string()))),
            ip_address: Set(req.ip_address.clone()),
            user_agent: Set(req.user_agent.clone()),
            lead_id: Set(req.lead_id),
            converted_to_lead: Set(Some(req.converted_to_lead.unwrap_or(0))),
            remark: Set(req.remark.clone()),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            deleted: Set(Some(0)),
        };

        LeaveMsg::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 根据ID查询留言
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<leave_msg::Model>, DbErr> {
        LeaveMsg::find_by_id(id)
            .filter(leave_msg::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 更新留言的线索关联信息（转线索后调用）
    pub async fn update_lead_info<C: ConnectionTrait>(
        db: &C,
        id: i64,
        lead_id: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = leave_msg::ActiveModel {
            lead_id: Set(Some(lead_id)),
            convert_lead_id: Set(Some(lead_id)),
            converted_to_lead: Set(Some(1)),
            status: Set(Some(1)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        let result: UpdateResult = LeaveMsg::update_many()
            .set(payload)
            .filter(leave_msg::Column::Id.eq(id))
            .filter(leave_msg::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(result.rows_affected as i64)
    }

    /// 分页查询留言列表
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        website_id: Option<i64>,
        status: Option<i32>,
    ) -> Result<(Vec<leave_msg::Model>, i64), DbErr> {
        let mut query = LeaveMsg::find()
            .filter(leave_msg::Column::Deleted.eq(0));

        if let Some(wid) = website_id {
            query = query.filter(leave_msg::Column::WebsiteId.eq(wid));
        }
        if let Some(s) = status {
            query = query.filter(leave_msg::Column::Status.eq(s));
        }

        let paginator = query
            .order_by_desc(leave_msg::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 软删除（批量）
    pub async fn batch_soft_delete<C: ConnectionTrait>(
        db: &C,
        ids: Vec<i64>,
    ) -> Result<i64, DbErr> {
        if ids.is_empty() {
            return Ok(0);
        }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = LeaveMsg::update_many()
            .col_expr(leave_msg::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(leave_msg::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(leave_msg::Column::Id.is_in(ids))
            .filter(leave_msg::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 更新留言状态（标记为已处理/已忽略）
    pub async fn update_status<C: ConnectionTrait>(
        db: &C,
        id: i64,
        status: i32,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = LeaveMsg::update_many()
            .col_expr(leave_msg::Column::Status, sea_orm::sea_query::Expr::value(status))
            .col_expr(leave_msg::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(leave_msg::Column::Id.eq(id))
            .filter(leave_msg::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
