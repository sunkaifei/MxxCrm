//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 费用申请业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::model::approval::ApprovalSubmitRequest;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::crm::entity::customer::{self as customer_entity, Entity as Customer};
use crate::modules::crm::entity::opportunity::{self as opportunity_entity, Entity as Opportunity};
use crate::modules::finance::entity::expense_type::{self as expense_type_entity, Entity as FinanceExpenseType};
use crate::modules::finance::model::expense::{
    ExpenseDetailVO, ExpenseItemModel, ExpenseItemSaveDTO, ExpenseListQuery, ExpenseListVO,
    ExpenseModel, ExpensePaymentReq, ExpenseSaveDTO, ExpenseSaveRequest, ExpenseTypeModel,
    ExpenseTypeSaveRequest, ExpenseTypeVO,
};
use crate::modules::sale::entity::order::{self as order_entity, Entity as SaleOrder};
use crate::modules::system::entity::admin::{self, Entity as Admin};
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, TransactionTrait};
use std::collections::{HashMap, HashSet};

/// 计算费用明细金额合计
fn calculate_expense_amount(items: &Vec<ExpenseItemSaveDTO>) -> Decimal {
    items.iter()
        .map(|item| item.item_amount.unwrap_or(Decimal::from(0)))
        .fold(Decimal::from(0), |acc, x| acc + x)
}

/// 创建费用申请
pub async fn insert(db: &DbConn, form_data: &ExpenseSaveRequest, created_by: i64) -> Result<i64> {
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("费用明细不能为空"));
    }

    form_data.title.as_ref()
        .ok_or_else(|| Error::from("费用申请标题不能为空".to_string()))?;

    // 生成费用申请单号 EXP{YYYYMMDD}{4位序号}
    let date_prefix = format!("EXP{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = ExpenseModel::get_max_expense_no_today(db, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let expense_no = format!("{}{:04}", date_prefix, seq);

    // 计算金额 = 明细合计
    let amount = calculate_expense_amount(&items);

    // 解析申请日期
    let apply_date = form_data.apply_date.as_ref()
        .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
        .or_else(|| Some(chrono::Local::now().date_naive()));

    let txn = db.begin().await?;

    let mut dto: ExpenseSaveDTO = build_save_dto(form_data);
    dto.expense_no = Some(expense_no);
    dto.amount = Some(amount);
    dto.apply_date = apply_date;
    dto.status = Some(1); // 草稿
    dto.approval_status = Some(0);
    dto.create_by = Some(created_by);
    // 申请人默认为创建者
    if dto.applicant_id.is_none() {
        dto.applicant_id = Some(created_by);
    }

    let expense_id = ExpenseModel::insert(&txn, &dto).await?;
    ExpenseItemModel::insert_batch(&txn, expense_id, &items).await?;

    txn.commit().await?;

    Ok(expense_id)
}

/// 编辑费用申请（仅草稿/已驳回可编辑）
pub async fn update(db: &DbConn, form_data: &ExpenseSaveRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("费用申请ID不能为空"));
    }
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("费用明细不能为空"));
    }

    let existing = ExpenseModel::find_by_id(db, id).await?;
    let existing_expense = existing.ok_or_else(|| Error::from("费用申请不存在"))?;

    // 业务状态校验：仅草稿(1)/已驳回(5)允许编辑
    let status = existing_expense.status.unwrap_or(1);
    if status != 1 && status != 5 {
        return Err(Error::from(format!("当前费用申请状态({})不允许编辑", status)));
    }
    // 审批状态校验：仅草稿(0)/已驳回(4)允许编辑
    let approval_status = existing_expense.approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("当前费用申请审批状态不允许编辑"));
    }

    // 计算金额
    let amount = calculate_expense_amount(&items);
    let apply_date = form_data.apply_date.as_ref()
        .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
        .or(existing_expense.apply_date);

    let txn = db.begin().await?;

    let mut dto: ExpenseSaveDTO = build_save_dto(form_data);
    dto.amount = Some(amount);
    dto.apply_date = apply_date;
    // update_by 未在表中存储，仅作为参数占位
    let _ = updated_by;

    ExpenseModel::update_by_id(&txn, id, &dto).await?;
    ExpenseItemModel::delete_by_expense_id(&txn, id).await?;
    ExpenseItemModel::insert_batch(&txn, id, &items).await?;

    txn.commit().await?;

    Ok(id)
}

/// 批量删除费用申请（仅草稿可删除）
pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    // 状态校验：仅草稿(1)/已驳回(5)允许删除
    for &id in ids_vec {
        let existing = ExpenseModel::find_by_id(db, id).await?;
        if let Some(expense) = existing {
            let status = expense.status.unwrap_or(1);
            if status != 1 && status != 5 {
                return Err(Error::from(format!(
                    "费用申请[{}]当前状态不允许删除",
                    expense.expense_no.unwrap_or_default()
                )));
            }
        }
    }
    let result = ExpenseModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

/// 获取费用申请详情（含明细 + 关联客户/商机/订单名称）
pub async fn get_detail(db: &DbConn, id: i64) -> Result<ExpenseDetailVO> {
    let expense = ExpenseModel::find_by_id(db, id).await?
        .ok_or_else(|| Error::from("费用申请不存在"))?;

    let items = ExpenseItemModel::find_by_expense_id(db, id).await?;

    let mut vo = ExpenseDetailVO {
        id: expense.id.into(),
        expense_no: expense.expense_no.clone(),
        title: expense.title.clone(),
        expense_type: expense.expense_type,
        expense_type_name: None,
        applicant_id: expense.applicant_id,
        applicant_name: None,
        dept_id: expense.dept_id,
        customer_id: expense.customer_id,
        customer_name: None,
        opportunity_id: expense.opportunity_id,
        opportunity_name: None,
        order_id: expense.order_id,
        order_no: None,
        amount: expense.amount,
        currency: expense.currency.clone(),
        apply_date: expense.apply_date,
        status: expense.status,
        approval_status: expense.approval_status,
        instance_id: expense.instance_id,
        remark: expense.remark.clone(),
        attachment: expense.attachment.clone(),
        create_by: expense.create_by,
        create_time: expense.create_time,
        update_time: expense.update_time,
        items: items.iter().map(|i| i.into()).collect(),
    };

    // 查询费用类型名称
    if let Some(t_id) = vo.expense_type {
        if let Some(t) = FinanceExpenseType::find_by_id(t_id)
            .filter(expense_type_entity::Column::Deleted.eq(0))
            .one(db).await? {
            vo.expense_type_name = t.type_name;
        }
    }

    // 实时查询客户名称
    if let Some(cid) = vo.customer_id {
        if let Some(c) = Customer::find_by_id(cid).one(db).await? {
            vo.customer_name = c.company_name.or(c.short_name);
        }
    }

    // 实时查询商机名称
    if let Some(oid) = vo.opportunity_id {
        if let Some(o) = Opportunity::find_by_id(oid).one(db).await? {
            vo.opportunity_name = o.title;
        }
    }

    // 实时查询订单号
    if let Some(oid) = vo.order_id {
        if let Some(o) = SaleOrder::find_by_id(oid).one(db).await? {
            vo.order_no = o.order_no;
        }
    }

    // 实时查询申请人名称
    if let Some(uid) = vo.applicant_id {
        if let Some(a) = Admin::find_by_id(uid).one(db).await? {
            vo.applicant_name = a.nick_name.or(a.user_name);
        }
    }

    Ok(vo)
}

/// 费用申请列表（支持 全部/我的/下属）
pub async fn get_list(db: &DbConn, query: &ExpenseListQuery, current_user_id: i64) -> Result<ResultPage<Vec<ExpenseListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let applicant_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => Some(vec![current_user_id]),
        "subordinate" => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();

            match data_scope {
                Some(5) => Some(Vec::new()),
                Some(1) | None => {
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                _ => {
                    let user_ids = crate::modules::system::service::data_scope_service::get_accessible_user_ids(db, current_user_id)
                        .await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>();
                    Some(user_ids)
                }
            }
        }
        _ => {
            crate::modules::system::service::data_scope_service::get_accessible_user_ids(db, current_user_id).await?
        }
    };

    let (list, total) = if list_type == "my" {
        ExpenseModel::select_in_page(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.status,
            query.approval_status,
            query.expense_type,
            query.customer_id,
            query.opportunity_id,
            query.order_id,
            Some(current_user_id),
            query.start_date.clone(),
            query.end_date.clone(),
        ).await?
    } else {
        ExpenseModel::select_in_page_by_applicant_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.status,
            query.approval_status,
            query.expense_type,
            query.customer_id,
            query.opportunity_id,
            query.order_id,
            query.start_date.clone(),
            query.end_date.clone(),
            applicant_ids_opt,
        ).await?
    };

    // 批量查询客户名称
    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.customer_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let customer_name_map: HashMap<i64, String> = if !customer_ids.is_empty() {
        Customer::find()
            .filter(customer_entity::Column::Id.is_in(customer_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.company_name.or(c.short_name).unwrap_or_default()))
            .collect()
    } else {
        HashMap::new()
    };

    // 批量查询商机名称
    let opportunity_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.opportunity_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let opportunity_name_map: HashMap<i64, String> = if !opportunity_ids.is_empty() {
        Opportunity::find()
            .filter(opportunity_entity::Column::Id.is_in(opportunity_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|o| o.title.map(|t| (o.id, t)))
            .collect()
    } else {
        HashMap::new()
    };

    // 批量查询订单号
    let order_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.order_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let order_no_map: HashMap<i64, String> = if !order_ids.is_empty() {
        SaleOrder::find()
            .filter(order_entity::Column::Id.is_in(order_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|o| o.order_no.map(|n| (o.id, n)))
            .collect()
    } else {
        HashMap::new()
    };

    // 批量查询费用类型名称
    let type_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.expense_type)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let type_name_map: HashMap<i64, String> = if !type_ids.is_empty() {
        FinanceExpenseType::find()
            .filter(expense_type_entity::Column::Id.is_in(type_ids.clone()))
            .filter(expense_type_entity::Column::Deleted.eq(0))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|t| t.type_name.map(|n| (t.id, n)))
            .collect()
    } else {
        HashMap::new()
    };

    // 批量查询申请人名称
    let applicant_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.applicant_id)
        .collect();
    let applicant_name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, applicant_ids).await;

    let data: Vec<ExpenseListVO> = list.iter().map(|item| {
        let mut vo: ExpenseListVO = item.into();
        if let Some(cid) = vo.customer_id {
            if let Some(name) = customer_name_map.get(&cid) {
                vo.customer_name = Some(name.clone());
            }
        }
        if let Some(oid) = vo.opportunity_id {
            if let Some(name) = opportunity_name_map.get(&oid) {
                vo.opportunity_name = Some(name.clone());
            }
        }
        if let Some(oid) = vo.order_id {
            if let Some(no) = order_no_map.get(&oid) {
                vo.order_no = Some(no.clone());
            }
        }
        if let Some(t_id) = vo.expense_type {
            if let Some(name) = type_name_map.get(&t_id) {
                vo.expense_type_name = Some(name.clone());
            }
        }
        if let Some(uid) = vo.applicant_id {
            if let Some(name) = applicant_name_map.get(&uid) {
                vo.applicant_name = Some(name.clone());
            }
        }
        vo
    }).collect();

    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

/// 提交审批（接入审批引擎）
pub async fn submit_expense(db: &DbConn, expense_id: i64, operator_id: i64, operator_name: &str) -> Result<ExpenseDetailVO> {
    let expense = ExpenseModel::find_by_id(db, expense_id).await?
        .ok_or_else(|| Error::from("费用申请不存在"))?;

    let approval_status = expense.approval_status.unwrap_or(0);
    if approval_status != 0 && approval_status != 4 {
        return Err(Error::from("仅草稿或已驳回状态可提交审批"));
    }

    // 接入审批引擎：提交费用审批流
    let submit_req = ApprovalSubmitRequest {
        flow_code: "expense_approval".to_string(),
        business_type: "expense".to_string(),
        business_id: expense_id,
        business_title: expense.title.clone(),
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({
            "amount": expense.amount.unwrap_or(Decimal::from(0)),
        })),
    };
    let instance_id = ApprovalService::submit(db, &submit_req).await?;

    // 更新费用申请状态为待审批，记录审批实例ID
    let txn = db.begin().await?;
    ExpenseModel::update_approval(&txn, expense_id, 1, Some(instance_id)).await?;
    ExpenseModel::update_status(&txn, expense_id, 2).await?; // 待审批
    txn.commit().await?;

    get_detail(db, expense_id).await
}

/// 审批通过
pub async fn approve_expense(db: &DbConn, expense_id: i64, operator_id: i64, _reason: Option<String>) -> Result<ExpenseDetailVO> {
    let expense = ExpenseModel::find_by_id(db, expense_id).await?
        .ok_or_else(|| Error::from("费用申请不存在"))?;

    let approval_status = expense.approval_status.unwrap_or(0);
    if approval_status != 1 && approval_status != 2 {
        return Err(Error::from("仅待审批或审批中状态可进行审批操作"));
    }

    let txn = db.begin().await?;
    ExpenseModel::update_approval(&txn, expense_id, 3, None).await?;
    ExpenseModel::update_status(&txn, expense_id, 4).await?; // 已通过
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, expense_id).await
}

/// 审批驳回
pub async fn reject_expense(db: &DbConn, expense_id: i64, operator_id: i64, _reason: Option<String>) -> Result<ExpenseDetailVO> {
    let expense = ExpenseModel::find_by_id(db, expense_id).await?
        .ok_or_else(|| Error::from("费用申请不存在"))?;

    let approval_status = expense.approval_status.unwrap_or(0);
    if approval_status != 1 && approval_status != 2 {
        return Err(Error::from("仅待审批或审批中状态可进行驳回操作"));
    }

    let txn = db.begin().await?;
    ExpenseModel::update_approval(&txn, expense_id, 4, None).await?;
    ExpenseModel::update_status(&txn, expense_id, 5).await?; // 已驳回
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, expense_id).await
}

/// 财务打款（status -> 6 已打款）
pub async fn make_payment(db: &DbConn, req: &ExpensePaymentReq, operator_id: i64) -> Result<ExpenseDetailVO> {
    let expense = ExpenseModel::find_by_id(db, req.expense_id).await?
        .ok_or_else(|| Error::from("费用申请不存在"))?;

    let status = expense.status.unwrap_or(0);
    // 仅已通过(4)状态可进行打款
    if status != 4 {
        return Err(Error::from("仅已通过状态可进行打款操作"));
    }

    let txn = db.begin().await?;
    ExpenseModel::update_status(&txn, req.expense_id, 6).await?; // 已打款
    txn.commit().await?;

    let _ = operator_id;
    get_detail(db, req.expense_id).await
}

// ==================== 费用类型 ====================

/// 费用类型列表
pub async fn get_type_list(db: &DbConn) -> Result<Vec<ExpenseTypeVO>> {
    let list = ExpenseTypeModel::find_all(db).await?;
    Ok(list.iter().map(|m| m.into()).collect())
}

/// 费用类型新建/编辑
pub async fn save_type(db: &DbConn, req: &ExpenseTypeSaveRequest) -> Result<i64> {
    let type_name = req.type_name.as_ref()
        .ok_or_else(|| Error::from("费用类型名称不能为空".to_string()))?;
    let type_code = req.type_code.as_ref()
        .ok_or_else(|| Error::from("费用类型编码不能为空".to_string()))?;

    // 编码唯一性校验
    let existing = ExpenseTypeModel::find_by_code(db, type_code, req.id).await?;
    if existing.is_some() {
        return Err(Error::from("费用类型编码已存在"));
    }

    let _ = type_name;

    if let Some(id) = req.id {
        if id > 0 {
            let existing = ExpenseTypeModel::find_by_id(db, id).await?
                .ok_or_else(|| Error::from("费用类型不存在"))?;
            // 系统内置类型不允许修改编码
            if existing.is_system.unwrap_or(0) == 1 {
                return Err(Error::from("系统内置费用类型不允许编辑"));
            }
            ExpenseTypeModel::update_by_id(db, id, req).await?;
            return Ok(id);
        }
    }

    let id = ExpenseTypeModel::insert(db, req).await?;
    Ok(id)
}

/// 批量删除费用类型（系统内置类型受保护，不会删除）
pub async fn batch_delete_type(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let affected = ExpenseTypeModel::batch_delete_by_ids(db, ids).await?;
    if affected == 0 {
        return Err(Error::from("未删除任何费用类型，可能因选中了系统内置类型或不存在的记录"));
    }
    Ok(affected)
}

// ==================== 内部工具 ====================

fn build_save_dto(req: &ExpenseSaveRequest) -> ExpenseSaveDTO {
    ExpenseSaveDTO {
        expense_no: None,
        title: req.title.clone(),
        expense_type: req.expense_type,
        applicant_id: req.applicant_id,
        dept_id: req.dept_id,
        customer_id: req.customer_id,
        opportunity_id: req.opportunity_id,
        order_id: req.order_id,
        amount: None,
        currency: req.currency.clone(),
        apply_date: None,
        status: None,
        approval_status: None,
        remark: req.remark.clone(),
        attachment: req.attachment.clone(),
        create_by: None,
    }
}
