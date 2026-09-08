//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_approval_record::{self, Entity as ApprovalRecord};
use sea_orm::prelude::DateTime;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRecordDTO {
    pub biz_type: Option<String>,
    pub biz_id: Option<i64>,
    pub approval_level: Option<i32>,
    pub approver_id: Option<i64>,
    pub action: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRecordVO {
    pub id: Option<i64>,
    pub biz_type: Option<String>,
    pub biz_id: Option<i64>,
    pub approval_level: Option<i32>,
    pub approver_id: Option<i64>,
    pub action: Option<String>,
    pub comment: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<purchase_approval_record::Model> for ApprovalRecordVO {
    fn from(model: purchase_approval_record::Model) -> Self {
        ApprovalRecordVO {
            id: Some(model.id),
            biz_type: model.biz_type,
            biz_id: model.biz_id,
            approval_level: model.approval_level,
            approver_id: model.approver_id,
            action: model.action,
            comment: model.comment,
            created_by: model.created_by,
            create_time: model.create_time,
        }
    }
}

pub struct ApprovalRecordModel;

impl ApprovalRecordModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, dto: &ApprovalRecordDTO, operator: i64) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local();
        let active = purchase_approval_record::ActiveModel {
            biz_type: Set(dto.biz_type.clone()),
            biz_id: Set(dto.biz_id),
            approval_level: Set(dto.approval_level),
            approver_id: Set(dto.approver_id),
            action: Set(dto.action.clone()),
            comment: Set(dto.comment.clone()),
            deleted: Set(Some(0)),
            created_by: Set(Some(operator)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        ApprovalRecord::insert(active)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn find_by_biz<C: ConnectionTrait>(
        db: &C,
        biz_type: &str,
        biz_id: i64,
    ) -> Result<Vec<purchase_approval_record::Model>, DbErr> {
        ApprovalRecord::find()
            .filter(purchase_approval_record::Column::BizType.eq(biz_type))
            .filter(purchase_approval_record::Column::BizId.eq(biz_id))
            .filter(purchase_approval_record::Column::Deleted.eq(0))
            .order_by_asc(purchase_approval_record::Column::ApprovalLevel)
            .all(db)
            .await
    }

    /// 查询当前待审批记录
    pub async fn find_pending_by_biz<C: ConnectionTrait>(
        db: &C,
        biz_type: &str,
        biz_id: i64,
    ) -> Result<Option<purchase_approval_record::Model>, DbErr> {
        ApprovalRecord::find()
            .filter(purchase_approval_record::Column::BizType.eq(biz_type))
            .filter(purchase_approval_record::Column::BizId.eq(biz_id))
            .filter(purchase_approval_record::Column::Action.eq("pending"))
            .filter(purchase_approval_record::Column::Deleted.eq(0))
            .order_by_asc(purchase_approval_record::Column::ApprovalLevel)
            .one(db)
            .await
    }

    /// 查询我的审批列表（分页）
    pub async fn find_my_approval_list<C: ConnectionTrait>(
        db: &C,
        user_id: i64,
        page_num: u64,
        page_size: u64,
    ) -> Result<(Vec<purchase_approval_record::Model>, u64), DbErr> {
        let q = ApprovalRecord::find()
            .filter(purchase_approval_record::Column::ApproverId.eq(user_id))
            .filter(purchase_approval_record::Column::Deleted.eq(0))
            .order_by_desc(purchase_approval_record::Column::CreateTime);

        let paginator = q.paginate(db, page_size);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num).await?;
        Ok((list, total))
    }
}