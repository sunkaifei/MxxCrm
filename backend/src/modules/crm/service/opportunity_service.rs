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
use crate::modules::crm::entity::{contact, customer};
use crate::modules::crm::model::opportunity::{OpportunityDetailVO, OpportunityListQuery, OpportunityListVO, OpportunityModel, OpportunitySaveDTO, OpportunitySaveRequest, OpportunityUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin, dept as dept_entity};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::admin_service::build_admin_name_map;
use crate::modules::system::service::role_service;
use crate::modules::system::service::sales_flow_config_service;
use crate::modules::sale::entity::{quotation, quotation::Entity as Quotation};
use crate::modules::sale::model::order::{OrderModel, OrderSaveDTO};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QuerySelect, sea_query::Expr, TransactionTrait};
use std::collections::HashMap;

/// 根据用户ID获取其数据权限范围内的所有用户ID
///
/// 已迁移至 [`data_scope_service::get_accessible_user_ids`]，支持多角色合并。
/// 参数 `data_scope` 已弃用，内部会自动查询用户所有角色并合并权限。
async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    _data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    crate::modules::system::service::data_scope_service::get_accessible_user_ids(db, current_user_id).await
}

/// 递归收集子部门ID


pub async fn insert(db: &DbConn, form_data: &OpportunitySaveRequest, created_by: i64) -> Result<i64> {
    if let (Some(customer_id), Some(name)) = (form_data.customer_id, form_data.title.as_deref()) {
        let existing = OpportunityModel::find_by_customer_and_name(db, customer_id, name, None).await
            .map_err(|e| Error::from(format!("查询商机失败: {}", e)))?;
        if existing.is_some() {
            return Err(Error::from("该客户下已存在相同名称的商机".to_string()));
        }
    }

    // 校验联系人存在性
    if let Some(contact_id) = form_data.contact_id {
        let contact_exists = contact::Entity::find_by_id(contact_id)
            .one(db)
            .await
            .map_err(|e| Error::from(format!("查询联系人失败: {}", e)))?
            .is_some();
        if !contact_exists {
            return Err(Error::from("选择的联系人不存在".to_string()));
        }
    }

    let mut dto: OpportunitySaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = OpportunityModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &OpportunityUpdateRequest, updated_by: i64) -> Result<i64> {
    if let (Some(customer_id), Some(name)) = (form_data.customer_id, form_data.title.as_deref()) {
        let existing = OpportunityModel::find_by_customer_and_name(db, customer_id, name, form_data.id).await
            .map_err(|e| Error::from(format!("查询商机失败: {}", e)))?;
        if existing.is_some() {
            return Err(Error::from("该客户下已存在相同名称的商机".to_string()));
        }
    }

    // 校验联系人存在性
    if let Some(contact_id) = form_data.contact_id {
        let contact_exists = contact::Entity::find_by_id(contact_id)
            .one(db)
            .await
            .map_err(|e| Error::from(format!("查询联系人失败: {}", e)))?
            .is_some();
        if !contact_exists {
            return Err(Error::from("选择的联系人不存在".to_string()));
        }
    }

    let mut dto: OpportunitySaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = OpportunityModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = OpportunityModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<OpportunityDetailVO> {
    let result = OpportunityModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => {
            let customer_id = item.customer_id;
            let contact_id = item.contact_id;
            let created_by = item.created_by;
            let assigned_to = item.assigned_to;
            let mut vo: OpportunityDetailVO = item.into();
            if let Some(cid) = customer_id {
                if let Ok(Some(cust)) = customer::Entity::find_by_id(cid).one(db).await {
                    vo.customer_name = cust.company_name;
                    vo.customer_industry = cust.industry;
                    vo.customer_level = cust.level;
                    vo.customer_country = cust.country;
                    vo.customer_address = cust.address;
                    vo.customer_website = cust.website;
                    vo.customer_no = cust.customer_no;
                    vo.customer_short_name = cust.short_name;
                    vo.customer_credit_limit = cust.credit_limit;
                    vo.customer_credit_days = cust.credit_days;
                }
            }
            if let Some(ctid) = contact_id {
                if let Ok(Some(ct)) = contact::Entity::find_by_id(ctid).one(db).await {
                    vo.contact_name = ct.name;
                    vo.contact_title = ct.title;
                    vo.contact_mobile = ct.mobile;
                    vo.contact_email = ct.email;
                    vo.contact_phone = ct.phone;
                    vo.contact_wechat = ct.wechat;
                }
            }
            // 批量查询 created_by + assigned_to 用户名（合并为一次 IN 查询）
            let user_ids: Vec<i64> = vec![created_by, assigned_to].into_iter().flatten().collect();
            let name_map = build_admin_name_map(db, user_ids).await;
            if let Some(uid) = created_by {
                vo.created_by_name = name_map.get(&uid).cloned();
            }
            if let Some(uid) = assigned_to {
                vo.assignee = name_map.get(&uid).cloned();
            }
            Ok(vo)
        },
        None => Err(Error::from("商机不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &OpportunityListQuery, current_user_id: i64) -> Result<ResultPage<Vec<OpportunityListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    // 计算负责人ID集合（用于 my / subordinate / all 过滤）
    let assigned_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            // 我的商机：仅看自己负责的
            Some(vec![current_user_id])
        }
        "subordinate" => {
            // 下属商机：获取数据权限范围内的其他用户（排除自己）
            let accessible = crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?;
            match accessible {
                None => {
                    // 全部数据权限：获取所有用户，排除自己
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                Some(ids) => {
                    // 部门/仅本人权限：排除自己
                    Some(ids.into_iter().filter(|id| *id != current_user_id).collect())
                }
            }
        }
        _ => {
            // all：按多角色合并后的数据权限过滤
            crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?
        }
    };

    let (list, total) = if list_type == "my" {
        // my：直接用 select_in_page，按 assigned_to = current_user_id 过滤
        OpportunityModel::select_in_page(
            &db,
            page,
            page_size,
            query.keywords.clone(),
            query.stage.clone(),
            Some(current_user_id),
            query.customer_id,
        ).await?
    } else if list_type == "customer" {
        // customer：客户详情页使用，不过滤数据权限，按 customer_id 查询该客户下所有商机
        OpportunityModel::select_in_page_by_assigned_ids(
            &db,
            page,
            page_size,
            query.keywords.clone(),
            query.stage.clone(),
            None,
            query.customer_id,
        ).await?
    } else {
        // subordinate / all：按 assigned_ids 过滤
        OpportunityModel::select_in_page_by_assigned_ids(
            &db,
            page,
            page_size,
            query.keywords.clone(),
            query.stage.clone(),
            assigned_ids_opt,
            query.customer_id,
        ).await?
    };

    // 批量查询客户名称
    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.customer_id)
        .collect();
    let mut customer_map: HashMap<i64, String> = HashMap::new();
    if !customer_ids.is_empty() {
        let customers = customer::Entity::find()
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db)
            .await?;
        for c in customers {
            if let Some(name) = c.company_name {
                customer_map.insert(c.id, name);
            }
        }
    }

    // 批量查询创建人名称（统一调用共用方法）
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect();
    let creator_map = build_admin_name_map(db, creator_ids).await;

    // 批量查询负责人名称
    let assigned_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.assigned_to)
        .collect();
    let assigned_map = build_admin_name_map(db, assigned_ids).await;

    // 批量查询联系人姓名
    let contact_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.contact_id)
        .collect();
    let mut contact_map: HashMap<i64, String> = HashMap::new();
    if !contact_ids.is_empty() {
        let contacts = contact::Entity::find()
            .filter(contact::Column::Id.is_in(contact_ids))
            .all(db)
            .await?;
        for ct in contacts {
            if let Some(name) = ct.name {
                contact_map.insert(ct.id, name);
            }
        }
    }

    // 批量统计报价次数（关联 mxx_sale_quotation 表）
    let opp_ids: Vec<i64> = list.iter().map(|item| item.id).collect();
    let mut quote_count_map: HashMap<i64, i64> = HashMap::new();
    if !opp_ids.is_empty() {
        let quote_rows = Quotation::find()
            .select_only()
            .column(quotation::Column::OpportunityId)
            .column_as(quotation::Column::Id.count(), "cnt")
            .filter(quotation::Column::OpportunityId.is_in(opp_ids.clone()))
            .filter(quotation::Column::Deleted.eq(0))
            .group_by(quotation::Column::OpportunityId)
            .into_tuple::<(Option<i64>, i64)>()
            .all(db)
            .await?;
        for (oid, cnt) in quote_rows {
            if let Some(id) = oid {
                quote_count_map.insert(id, cnt);
            }
        }
    }

    let data: Vec<OpportunityListVO> = list.into_iter().map(|item| {
        let customer_id = item.customer_id;
        let created_by = item.created_by;
        let assigned_to = item.assigned_to;
        let opp_id = item.id;
        let ct_id = item.contact_id;
        let mut vo: OpportunityListVO = item.into();
        vo.customer_name = customer_id.and_then(|id| customer_map.get(&id).cloned());
        vo.created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        vo.assignee_name = assigned_to.and_then(|id| assigned_map.get(&id).cloned());
        vo.quote_count = quote_count_map.get(&opp_id).copied();
        vo.contact_name = ct_id.and_then(|id| contact_map.get(&id).cloned());
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

/// 商机转报价单
///
/// 根据商机信息创建报价单草稿，并将商机阶段推进到"已报价"
/// 使用事务保证一致性
pub async fn convert_to_quotation(db: &DbConn, opportunity_id: i64, user_id: i64) -> Result<i64> {
    use crate::modules::sale::entity::quotation::{self, ActiveModel as QuotationActiveModel};
    use crate::modules::crm::entity::opportunity::{self as opp_entity, ActiveModel as OppActiveModel};
    use sea_orm::ActiveValue::Set;

    // 1. 校验企业配置：模式 A 或 both 才允许走标准流程
    let mode = sales_flow_config_service::get_mode(db).await;
    if !sales_flow_config_service::allows_standard_flow(&mode) {
        return Err(Error::from(
            "当前企业销售流程模式不允许走标准流程（需经过报价单），请联系管理员修改配置".to_string(),
        ));
    }

    // 2. 查询商机
    let opp = OpportunityModel::find_by_id(&db, opportunity_id)
        .await?
        .ok_or_else(|| Error::from("商机不存在".to_string()))?;

    // 3. 校验：未删除且尚未转报价单
    if opp.quote_status.unwrap_or(0) == 1 {
        return Err(Error::from("该商机已转报价单，请勿重复操作".to_string()));
    }

    // 3.1 校验：商机客户必填（保证数据完整性）
    if opp.customer_id.is_none() {
        return Err(Error::from("商机缺少客户信息，请先完善商机客户字段后再转报价单".to_string()));
    }

    // 4. 开启事务
    let txn = db.begin().await?;

    let now = chrono::Local::now().naive_local().to_owned();
    let quotation_no = format!("QT{}{:06}", now.format("%Y%m%d"), opportunity_id);

    let quotation_model = QuotationActiveModel {
        quotation_no: Set(Some(quotation_no)),
        customer_id: Set(opp.customer_id),
        opportunity_id: Set(Some(opportunity_id)),
        opportunity_title: Set(opp.title.clone()),
        title: Set(opp.title.clone()),
        total_amount: Set(opp.amount),
        currency: Set(opp.currency.map(|c| match c {
            crate::core::r#enum::currency_code_enum::CurrencyCode::CNY => 1,
            crate::core::r#enum::currency_code_enum::CurrencyCode::USD => 2,
            crate::core::r#enum::currency_code_enum::CurrencyCode::EUR => 3,
            crate::core::r#enum::currency_code_enum::CurrencyCode::GBP => 4,
            crate::core::r#enum::currency_code_enum::CurrencyCode::JPY => 5,
            crate::core::r#enum::currency_code_enum::CurrencyCode::HKD => 6,
            crate::core::r#enum::currency_code_enum::CurrencyCode::AUD => 7,
        })),
        status: Set(Some(0)), // 草稿
        owner_user_id: Set(Some(user_id)),
        create_by: Set(Some(user_id.to_string())),
        create_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    let quotation_result = Quotation::insert(quotation_model)
        .exec(&txn)
        .await?;

    let quotation_id = quotation_result.last_insert_id;

    // 5. 更新商机阶段为"已报价"（4）和报价状态
    let _ = opp_entity::Entity::update_many()
        .set(opp_entity::ActiveModel {
            stage: Set(Some(4)),
            quote_status: Set(Some(1)),
            update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        })
        .filter(opp_entity::Column::Id.eq(opportunity_id))
        .exec(&txn)
        .await?;

    // 6. 提交事务
    txn.commit().await?;

    Ok(quotation_id)
}

/// 商机直接转订单（简易流程模式 B）
///
/// 跳过报价单，直接创建订单草稿，并将商机阶段推进到"已下单"
/// 使用事务保证一致性
pub async fn convert_to_order(db: &DbConn, opportunity_id: i64, user_id: i64) -> Result<i64> {
    use crate::modules::crm::entity::opportunity::{self as opp_entity, ActiveModel as OppActiveModel};
    use sea_orm::ActiveValue::Set;
    use rust_decimal::Decimal;

    // 1. 校验企业配置：模式 B 或 both 才允许跳过报价单
    let mode = sales_flow_config_service::get_mode(db).await;
    if !sales_flow_config_service::can_skip_quotation(&mode) {
        return Err(Error::from(
            "当前企业销售流程模式不允许跳过报价单，请先转报价单再转订单".to_string(),
        ));
    }

    // 2. 查询商机
    let opp = OpportunityModel::find_by_id(&db, opportunity_id)
        .await?
        .ok_or_else(|| Error::from("商机不存在".to_string()))?;

    // 3. 校验：尚未转订单
    if opp.order_status.unwrap_or(0) == 1 {
        return Err(Error::from("该商机已转订单，请勿重复操作".to_string()));
    }

    // 3.1 校验：商机客户必填（保证数据完整性）
    if opp.customer_id.is_none() {
        return Err(Error::from("商机缺少客户信息，请先完善商机客户字段后再转订单".to_string()));
    }

    // 4. 开启事务
    let txn = db.begin().await?;

    // 5. 生成订单编号：SO{YYYYMMDD}{4位序号}
    let date_prefix = format!("SO{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = OrderModel::get_max_order_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let order_no = format!("{}{:04}", date_prefix, seq);

    // 6. 构造订单 DTO（从商机带出客户、金额等，无商品明细）
    let opp_amount = opp.amount.unwrap_or_else(|| Decimal::from(0));
    let currency_i32 = opp.currency.map(|c| match c {
        crate::core::r#enum::currency_code_enum::CurrencyCode::CNY => 1,
        crate::core::r#enum::currency_code_enum::CurrencyCode::USD => 2,
        crate::core::r#enum::currency_code_enum::CurrencyCode::EUR => 3,
        crate::core::r#enum::currency_code_enum::CurrencyCode::GBP => 4,
        crate::core::r#enum::currency_code_enum::CurrencyCode::JPY => 5,
        crate::core::r#enum::currency_code_enum::CurrencyCode::HKD => 6,
        crate::core::r#enum::currency_code_enum::CurrencyCode::AUD => 7,
    });

    let order_dto = OrderSaveDTO {
        order_no: Some(order_no),
        title: opp.title.clone(),
        order_type: Some(1),
        order_status: Some(1), // 已确认
        customer_id: opp.customer_id,
        customer_name: None,
        contact_id: opp.contact_id,
        contact_name: None,
        opportunity_id: Some(opportunity_id),
        quotation_id: None,  // 跳过报价单
        contract_id: None,
        order_date: Some(chrono::Local::now().naive_local().date()),
        delivery_date: None,
        currency: currency_i32,
        exchange_rate: Some(Decimal::from(1)),
        product_amount: Some(opp_amount),
        discount_amount: Some(Decimal::from(0)),
        shipping_fee: Some(Decimal::from(0)),
        tax_amount: Some(Decimal::from(0)),
        other_fee: Some(Decimal::from(0)),
        total_amount: Some(opp_amount),
        paid_amount: Some(Decimal::from(0)),
        unpaid_amount: Some(opp_amount),
        pay_status: Some(1), // 未支付
        payment_method: None,
        payment_due_date: None,
        shipping_method: None,
        tracking_no: None,
        shipping_time: None,
        complete_time: None,
        receiver_name: None,
        receiver_phone: None,
        shipping_address: None,
        billing_address: None,
        buyer_company_name: None,
        buyer_account_name: None,
        buyer_bank_name: None,
        buyer_account_number: None,
        seller_company_name: None,
        seller_bank_name: None,
        seller_account_name: None,
        seller_account_number: None,
        remark: opp.description.clone(),
        owner_user_id: Some(user_id),
        dept_id: None,
        approval_status: Some(0), // 草稿
        instance_id: None,
        create_by: Some(user_id),
        update_by: None,
    };

    let order_id = OrderModel::insert(&txn, &order_dto).await?;

    // 7. 更新商机阶段为"已报价"（4，等同于已下单阶段）和订单状态
    let now = chrono::Local::now().naive_local();
    let _ = opp_entity::Entity::update_many()
        .set(opp_entity::ActiveModel {
            stage: Set(Some(4)),
            order_status: Set(Some(1)),
            update_time: Set(Some(now)),
            ..Default::default()
        })
        .filter(opp_entity::Column::Id.eq(opportunity_id))
        .exec(&txn)
        .await?;

    // 8. 提交事务
    txn.commit().await?;

    Ok(order_id)
}