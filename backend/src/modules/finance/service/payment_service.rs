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
use chrono::Utc;

use crate::modules::finance::entity::payment;
use crate::modules::finance::model::payment::{
    PaymentDTO, PaymentQuery, PaymentApplyDTO, PaymentApproveDTO, PaymentConfirmDTO, to_decimal,
};

/// 列表
pub async fn get_list(
    db: &DatabaseConnection,
    query: PaymentQuery,
) -> Result<(Vec<PaymentDTO>, i64), String> {
    let mut stmt = payment::Entity::find()
        .filter(payment::Column::Deleted.eq(0));

    if let Some(payment_no) = &query.payment_no {
        stmt = stmt.filter(payment::Column::PaymentNo.contains(payment_no));
    }
    if let Some(po_no) = &query.purchase_order_no {
        stmt = stmt.filter(payment::Column::PurchaseOrderNo.contains(po_no));
    }
    if let Some(supplier_name) = &query.supplier_name {
        stmt = stmt.filter(payment::Column::SupplierName.contains(supplier_name));
    }
    if let Some(status) = query.status {
        stmt = stmt.filter(payment::Column::Status.eq(status));
    }

    stmt = stmt.order_by_desc(payment::Column::CreateTime);

    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    let dto_list: Vec<PaymentDTO> = items.into_iter().map(PaymentDTO::from).collect();

    Ok((dto_list, total))
}

/// 详情
pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<PaymentDTO, String> {
    let model = payment::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "付款记录不存在".to_string())?;

    Ok(PaymentDTO::from(model))
}

/// 申请付款
pub async fn apply(
    db: &DatabaseConnection,
    dto: PaymentApplyDTO,
    applicant_id: i64,
    applicant_name: String,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let payment_amount = to_decimal(dto.payment_amount);

    let model = payment::ActiveModel {
        payment_no: Set(None),
        purchase_order_id: Set(dto.purchase_order_id),
        purchase_order_no: Set(dto.purchase_order_no),
        supplier_name: Set(dto.supplier_name),
        payment_type: Set(dto.payment_type),
        payment_amount: Set(payment_amount),
        payment_method: Set(dto.payment_method),
        bank_account: Set(dto.bank_account),
        status: Set(Some(0)),
        applicant_id: Set(Some(applicant_id)),
        applicant_name: Set(Some(applicant_name)),
        apply_time: Set(Some(now)),
        remark: Set(dto.remark),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let result = model.insert(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(result.id)
}

/// 审批
pub async fn approve(
    db: &DatabaseConnection,
    dto: PaymentApproveDTO,
    approver_id: i64,
    approver_name: String,
) -> Result<(), String> {
    let record = payment::Entity::find_by_id(dto.id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "付款记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 0 {
        return Err("只有待审批状态的付款记录才能审批".to_string());
    }

    let now = Utc::now().naive_utc();
    // 审批通过 status=1, 审批驳回 status=3(取消)
    let new_status = if dto.approved { 1 } else { 3 };

    let mut model: payment::ActiveModel = record.into();
    model.status = Set(Some(new_status));
    model.approver_id = Set(Some(approver_id));
    model.approver_name = Set(Some(approver_name));
    model.approve_time = Set(Some(now));
    model.approve_remark = Set(dto.remark);
    model.update_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 确认付款
pub async fn confirm(db: &DatabaseConnection, dto: PaymentConfirmDTO) -> Result<(), String> {
    let record = payment::Entity::find_by_id(dto.id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "付款记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 1 {
        return Err("只有已审批状态的付款记录才能确认付款".to_string());
    }

    let payment_date = chrono::NaiveDate::parse_from_str(&dto.payment_date, "%Y-%m-%d")
        .map_err(|e| format!("付款日期格式错误: {}", e))?;

    let now = Utc::now().naive_utc();
    let mut model: payment::ActiveModel = record.into();
    model.payment_date = Set(Some(payment_date));
    model.status = Set(Some(2));
    model.update_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 取消
pub async fn cancel(db: &DatabaseConnection, id: i64, remark: String) -> Result<(), String> {
    let record = payment::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "付款记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status == 3 {
        return Err("该付款记录已取消".to_string());
    }
    if status == 2 {
        return Err("已付款的记录不能取消".to_string());
    }

    let now = Utc::now().naive_utc();
    let mut model: payment::ActiveModel = record.into();
    model.status = Set(Some(3));
    model.approve_remark = Set(Some(remark));
    model.update_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}
