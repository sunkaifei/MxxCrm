//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_requisition::{self, Entity as PurchaseRequisition};
use crate::modules::purchase::entity::purchase_requisition_item::{self, Entity as PurchaseRequisitionItem};
use sea_orm::prelude::{Decimal, DateTime, Date};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, UpdateResult,
};
use serde::{Deserialize, Serialize};

/// 采购申请单状态常量
pub mod requisition_status {
    /// 草稿
    pub const DRAFT: i32 = 0;
    /// 待审批
    pub const PENDING: i32 = 1;
    /// 审批中
    pub const APPROVING: i32 = 2;
    /// 已通过
    pub const APPROVED: i32 = 3;
    /// 已驳回
    pub const REJECTED: i32 = 4;
    /// 已转采购单
    pub const CONVERTED: i32 = 5;
}

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionItemDTO {
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<Decimal>,
    pub estimated_price: Option<Decimal>,
    pub estimated_amount: Option<Decimal>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionSaveRequest {
    pub id: Option<i64>,
    pub pr_type: Option<String>,
    pub title: Option<String>,
    pub department_id: Option<i64>,
    pub requester_id: Option<i64>,
    pub expected_date: Option<Date>,
    pub urgency: Option<String>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<String>,
    pub reason: Option<String>,
    pub remark: Option<String>,
    pub items: Vec<RequisitionItemDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionSaveDTO {
    pub id: Option<i64>,
    pub pr_no: Option<String>,
    pub pr_type: Option<String>,
    pub title: Option<String>,
    pub department_id: Option<i64>,
    pub requester_id: Option<i64>,
    pub expected_date: Option<Date>,
    pub urgency: Option<String>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<String>,
    pub status: Option<i32>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub source_no: Option<String>,
    pub reason: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionUpdateDTO {
    pub id: i64,
    pub pr_type: Option<String>,
    pub title: Option<String>,
    pub department_id: Option<i64>,
    pub requester_id: Option<i64>,
    pub expected_date: Option<Date>,
    pub urgency: Option<String>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<String>,
    pub reason: Option<String>,
    pub remark: Option<String>,
    pub items: Vec<RequisitionItemDTO>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
    pub department_id: Option<i64>,
    pub requester_id: Option<i64>,
    pub pr_type: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDetailVO {
    pub id: Option<i64>,
    pub pr_no: Option<String>,
    pub pr_type: Option<String>,
    pub title: Option<String>,
    pub department_id: Option<i64>,
    pub requester_id: Option<i64>,
    pub expected_date: Option<Date>,
    pub urgency: Option<String>,
    pub total_amount: Option<Decimal>,
    pub currency: Option<String>,
    pub status: Option<i32>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub source_no: Option<String>,
    pub reason: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub items: Vec<RequisitionItemVO>,
    pub approval_records: Vec<crate::modules::purchase::model::purchase_approval_record::ApprovalRecordVO>,
}

impl From<purchase_requisition::Model> for RequisitionDetailVO {
    fn from(model: purchase_requisition::Model) -> Self {
        RequisitionDetailVO {
            id: Some(model.id),
            pr_no: model.pr_no,
            pr_type: model.pr_type,
            title: model.title,
            department_id: model.department_id,
            requester_id: model.requester_id,
            expected_date: model.expected_date,
            urgency: model.urgency,
            total_amount: model.total_amount,
            currency: model.currency,
            status: model.status,
            source_type: model.source_type,
            source_id: model.source_id,
            source_no: model.source_no,
            reason: model.reason,
            remark: model.remark,
            created_by: model.created_by,
            create_time: model.create_time,
            updated_by: model.updated_by,
            update_time: model.update_time,
            items: vec![],
            approval_records: vec![],
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionItemVO {
    pub id: Option<i64>,
    pub pr_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub spec: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<Decimal>,
    pub estimated_price: Option<Decimal>,
    pub estimated_amount: Option<Decimal>,
    pub remark: Option<String>,
}

impl From<purchase_requisition_item::Model> for RequisitionItemVO {
    fn from(model: purchase_requisition_item::Model) -> Self {
        RequisitionItemVO {
            id: Some(model.id),
            pr_id: model.pr_id,
            product_id: model.product_id,
            product_name: model.product_name,
            product_sku: model.product_sku,
            spec: model.spec,
            unit: model.unit,
            quantity: model.quantity,
            estimated_price: model.estimated_price,
            estimated_amount: model.estimated_amount,
            remark: model.remark,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionListVO {
    pub id: Option<i64>,
    pub pr_no: Option<String>,
    pub pr_type: Option<String>,
    pub title: Option<String>,
    pub department_id: Option<i64>,
    pub requester_id: Option<i64>,
    pub status: Option<i32>,
    pub total_amount: Option<Decimal>,
    pub urgency: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

impl From<purchase_requisition::Model> for RequisitionListVO {
    fn from(model: purchase_requisition::Model) -> Self {
        RequisitionListVO {
            id: Some(model.id),
            pr_no: model.pr_no,
            pr_type: model.pr_type,
            title: model.title,
            department_id: model.department_id,
            requester_id: model.requester_id,
            status: model.status,
            total_amount: model.total_amount,
            urgency: model.urgency,
            created_by: model.created_by,
            create_time: model.create_time,
        }
    }
}

// ==================== 数据库操作 ====================

pub struct RequisitionModel;

impl RequisitionModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &RequisitionSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local();
        let payload = purchase_requisition::ActiveModel {
            pr_no: Set(req.pr_no.clone()),
            pr_type: Set(req.pr_type.clone()),
            title: Set(req.title.clone()),
            department_id: Set(req.department_id),
            requester_id: Set(req.requester_id),
            expected_date: Set(req.expected_date),
            urgency: Set(req.urgency.clone()),
            total_amount: Set(req.total_amount),
            currency: Set(req.currency.clone()),
            status: Set(req.status),
            source_type: Set(req.source_type.clone()),
            source_id: Set(req.source_id),
            source_no: Set(req.source_no.clone()),
            reason: Set(req.reason.clone()),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            created_by: Set(req.created_by),
            updated_by: Set(req.updated_by),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        PurchaseRequisition::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update<C: ConnectionTrait>(db: &C, req: &RequisitionUpdateDTO) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        let payload = purchase_requisition::ActiveModel {
            id: Set(req.id),
            pr_type: Set(req.pr_type.clone()),
            title: Set(req.title.clone()),
            department_id: Set(req.department_id),
            requester_id: Set(req.requester_id),
            expected_date: Set(req.expected_date),
            urgency: Set(req.urgency.clone()),
            total_amount: Set(req.total_amount),
            currency: Set(req.currency.clone()),
            reason: Set(req.reason.clone()),
            remark: Set(req.remark.clone()),
            updated_by: Set(req.updated_by),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        PurchaseRequisition::update(payload)
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32, operator: i64) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        PurchaseRequisition::update_many()
            .set(purchase_requisition::ActiveModel {
                status: Set(Some(status)),
                updated_by: Set(Some(operator)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(purchase_requisition::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<purchase_requisition::Model>, DbErr> {
        PurchaseRequisition::find_by_id(id)
            .filter(purchase_requisition::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn find_by_pr_no<C: ConnectionTrait>(db: &C, pr_no: &str) -> Result<Option<purchase_requisition::Model>, DbErr> {
        PurchaseRequisition::find()
            .filter(purchase_requisition::Column::PrNo.eq(pr_no))
            .filter(purchase_requisition::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询当天指定前缀的最大采购申请单号（用于生成流水号）
    pub async fn find_max_pr_no_today<C: ConnectionTrait>(db: &C, prefix: &str) -> Result<Option<String>, DbErr> {
        let result = PurchaseRequisition::find()
            .filter(purchase_requisition::Column::PrNo.starts_with(prefix))
            .filter(purchase_requisition::Column::Deleted.eq(0))
            .order_by_desc(purchase_requisition::Column::PrNo)
            .one(db)
            .await?;
        Ok(result.and_then(|m| m.pr_no))
    }

    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<u64, DbErr> {
        let result = PurchaseRequisition::update_many()
            .set(purchase_requisition::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_requisition::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn find_list<C: ConnectionTrait>(
        db: &C,
        query: &RequisitionListQuery,
    ) -> Result<(Vec<purchase_requisition::Model>, u64), DbErr> {
        let page_num = query.page_num.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);

        let mut q = PurchaseRequisition::find()
            .filter(purchase_requisition::Column::Deleted.eq(0));

        if let Some(ref kw) = query.keywords {
            q = q.filter(
                sea_orm::Condition::any()
                    .add(purchase_requisition::Column::PrNo.contains(kw))
                    .add(purchase_requisition::Column::Title.contains(kw)),
            );
        }
        if let Some(s) = query.status {
            q = q.filter(purchase_requisition::Column::Status.eq(s));
        }
        if let Some(d) = query.department_id {
            q = q.filter(purchase_requisition::Column::DepartmentId.eq(d));
        }
        if let Some(r) = query.requester_id {
            q = q.filter(purchase_requisition::Column::RequesterId.eq(r));
        }
        if let Some(ref pt) = query.pr_type {
            q = q.filter(purchase_requisition::Column::PrType.eq(pt));
        }

        let paginator = q
            .order_by_desc(purchase_requisition::Column::CreateTime)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page((page_num - 1) as u64).await?;
        Ok((list, total))
    }

    /// 查询审批待办列表（按审批人ID和业务类型）
    pub async fn find_approval_pending_list<C: ConnectionTrait>(
        db: &C,
        approver_id: i64,
        page_num: u64,
        page_size: u64,
    ) -> Result<(Vec<purchase_requisition::Model>, u64), DbErr> {
        use crate::modules::purchase::entity::purchase_approval_record::{self, Entity as ApprovalRecord};

        // 先查出该审批人待审批的记录ID
        let pending_ids = ApprovalRecord::find()
            .filter(purchase_approval_record::Column::ApproverId.eq(approver_id))
            .filter(purchase_approval_record::Column::Action.eq("pending"))
            .filter(purchase_approval_record::Column::Deleted.eq(0))
            .all(db)
            .await?;

        let biz_ids: Vec<i64> = pending_ids
            .iter()
            .filter_map(|r| r.biz_id)
            .collect();

        if biz_ids.is_empty() {
            return Ok((vec![], 0));
        }

        let q = PurchaseRequisition::find()
            .filter(purchase_requisition::Column::Id.is_in(biz_ids))
            .filter(purchase_requisition::Column::Deleted.eq(0))
            .order_by_desc(purchase_requisition::Column::CreateTime);

        let paginator = q.paginate(db, page_size);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num).await?;
        Ok((list, total))
    }
}

// ==================== 采购申请明细操作 ====================

pub struct RequisitionItemModel;

impl RequisitionItemModel {
    pub async fn batch_insert<C: ConnectionTrait>(
        db: &C,
        pr_id: i64,
        items: &[RequisitionItemDTO],
    ) -> Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        for item in items {
            let active = purchase_requisition_item::ActiveModel {
                pr_id: Set(Some(pr_id)),
                product_id: Set(item.product_id),
                product_name: Set(item.product_name.clone()),
                product_sku: Set(item.product_sku.clone()),
                spec: Set(item.spec.clone()),
                unit: Set(item.unit.clone()),
                quantity: Set(item.quantity),
                estimated_price: Set(item.estimated_price),
                estimated_amount: Set(item.estimated_amount),
                remark: Set(item.remark.clone()),
                deleted: Set(Some(0)),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            PurchaseRequisitionItem::insert(active).exec(db).await?;
        }
        Ok(())
    }

    pub async fn find_by_pr_id<C: ConnectionTrait>(
        db: &C,
        pr_id: i64,
    ) -> Result<Vec<purchase_requisition_item::Model>, DbErr> {
        PurchaseRequisitionItem::find()
            .filter(purchase_requisition_item::Column::PrId.eq(pr_id))
            .filter(purchase_requisition_item::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    pub async fn delete_by_pr_id<C: ConnectionTrait>(db: &C, pr_id: i64) -> Result<(), DbErr> {
        PurchaseRequisitionItem::update_many()
            .set(purchase_requisition_item::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_requisition_item::Column::PrId.eq(pr_id))
            .exec(db)
            .await?;
        Ok(())
    }
}

/// 生成采购申请单号 PR{yyyyMMdd}{0001}
pub fn generate_pr_no(seq: i32) -> String {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    format!("PR{}{:04}", today, seq)
}