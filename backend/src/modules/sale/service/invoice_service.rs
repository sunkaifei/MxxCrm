//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::customer::{Entity as Customer, Column as CustomerColumn};
use crate::modules::sale::model::invoice::{InvoiceDetailVO, InvoiceListQuery, InvoiceListVO, InvoiceModel, InvoiceSaveDTO, InvoiceSaveRequest, InvoiceUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use sea_orm::{DbConn, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

pub async fn insert(db: &DbConn, form_data: &InvoiceSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let date_prefix = format!("INV{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = InvoiceModel::get_max_invoice_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let invoice_no = format!("{}{:04}", date_prefix, seq);

    let amount = form_data.amount.unwrap_or(Decimal::from(0));
    let tax_rate = form_data.tax_rate.unwrap_or(Decimal::from(0));
    let hundred = Decimal::from(100);
    let tax_amount = form_data.tax_amount.unwrap_or(amount * tax_rate / hundred);

    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.invoice_no = Some(invoice_no);
    dto.status = Some(1);
    dto.tax_amount = Some(tax_amount);
    dto.create_by = Some(created_by.to_string());

    let invoice_id = InvoiceModel::insert(&txn, &dto).await?;

    txn.commit().await?;

    Ok(invoice_id)
}

pub async fn update(db: &DbConn, form_data: &InvoiceUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("发票ID不能为空"));
    }

    let existing = InvoiceModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("发票不存在"));
    }

    let amount = form_data.amount.unwrap_or(Decimal::from(0));
    let tax_rate = form_data.tax_rate.unwrap_or(Decimal::from(0));
    let hundred = Decimal::from(100);
    let tax_amount = form_data.tax_amount.unwrap_or(amount * tax_rate / hundred);

    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.tax_amount = Some(tax_amount);
    dto.update_by = Some(updated_by.to_string());

    InvoiceModel::update_by_id(db, id, &dto).await?;

    Ok(id)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = InvoiceModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<InvoiceDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, id).await?;
    match invoice {
        Some(i) => Ok((&i).into()),
        None => Err(Error::from("发票不存在")),
    }
}

async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    match data_scope {
        Some(1) => Ok(None),
        Some(5) => Ok(Some(vec![current_user_id])),
        Some(3) | Some(4) | Some(2) | Some(0) | Some(_) => {
            let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
                .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;

            let mut target_dept_ids = Vec::new();

            if data_scope == Some(2) {
                let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
                for role in roles {
                    if role.data_scope == Some(2) {
                        if let Some(role_id) = role.id {
                            let dept_result = crate::modules::system::model::role_dept_merge::RoleDeptMergeModel::find_by_role_id(db, &Some(role_id)).await
                                .map_err(|e| Error::from(format!("查询角色部门关联失败: {}", e)))?;
                            for merge in dept_result {
                                if let Some(dept_id) = merge.dept_id {
                                    target_dept_ids.push(dept_id);
                                }
                            }
                        }
                    }
                }
            } else {
                for merge in &user_depts {
                    if let Some(dept_id) = merge.dept_id {
                        target_dept_ids.push(dept_id);
                    }
                }
            }

            if target_dept_ids.is_empty() {
                return Ok(Some(vec![current_user_id]));
            }

            let all_depts = DeptModel::find_all(db).await
                .map_err(|e| Error::from(format!("查询部门列表失败: {}", e)))?;

            let mut all_target_ids = Vec::new();
            for dept_id in &target_dept_ids {
                if data_scope == Some(4) || data_scope == Some(2) {
                    all_target_ids.extend(collect_child_dept_ids(&all_depts, *dept_id));
                } else {
                    all_target_ids.push(*dept_id);
                }
            }

            all_target_ids.sort();
            all_target_ids.dedup();

            let dept_merges = AdminDeptMergeModel::find_by_dept_id(db, all_target_ids).await
                .map_err(|e| Error::from(format!("查询部门用户失败: {}", e)))?;

            let mut user_ids: Vec<i64> = dept_merges.iter()
                .filter_map(|m| m.admin_id)
                .collect();
            user_ids.sort();
            user_ids.dedup();

            if user_ids.is_empty() {
                Ok(Some(vec![current_user_id]))
            } else {
                Ok(Some(user_ids))
            }
        }
        None => Ok(None),
    }
}

fn collect_child_dept_ids(all_depts: &[crate::modules::system::entity::dept::Model], parent_id: i64) -> Vec<i64> {
    let mut result = vec![parent_id];
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            result.extend(collect_child_dept_ids(all_depts, dept.id));
        }
    }
    result
}

pub async fn get_list(db: &DbConn, query: &InvoiceListQuery, current_user_id: i64) -> Result<ResultPage<Vec<InvoiceListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            Some(vec![current_user_id])
        }
        "subordinate" => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();

            match data_scope {
                Some(5) => {
                    Some(Vec::new())
                }
                Some(1) | None => {
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                _ => {
                    let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>();
                    Some(user_ids)
                }
            }
        }
        _ => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();
            get_accessible_user_ids(db, current_user_id, data_scope).await?
        }
    };

    let (list, total) = if list_type == "my" {
        InvoiceModel::select_in_page(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.invoice_type,
            query.status,
            query.customer_id,
            Some(current_user_id),
        ).await?
    } else {
        InvoiceModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.invoice_type,
            query.status,
            query.customer_id,
            owner_user_ids_opt,
        ).await?
    };

    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|c| c.customer_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let customer_name_map: HashMap<i64, String> = if !customer_ids.is_empty() {
        Customer::find()
            .filter(CustomerColumn::Id.is_in(customer_ids.clone()))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.company_name.or(c.short_name).unwrap_or_default()))
            .collect()
    } else {
        HashMap::new()
    };

    let data: Vec<InvoiceListVO> = list.iter().map(|item| {
        let mut vo: InvoiceListVO = item.into();
        if let Some(cid) = vo.customer_id {
            if let Some(name) = customer_name_map.get(&cid) {
                vo.customer_name = Some(name.clone());
            }
        }
        vo
    }).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}
