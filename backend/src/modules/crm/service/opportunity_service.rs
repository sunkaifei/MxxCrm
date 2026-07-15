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
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::sale::entity::{quotation, quotation::Entity as Quotation};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QuerySelect, sea_query::Expr};
use std::collections::HashMap;

pub async fn insert(db: &DbConn, form_data: &OpportunitySaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: OpportunitySaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = OpportunityModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &OpportunityUpdateRequest, updated_by: i64) -> Result<i64> {
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
            if let Some(uid) = created_by {
                if let Ok(Some(u)) = Admin::find_by_id(uid).one(db).await {
                    vo.created_by_name = u.user_name;
                }
            }
            if let Some(uid) = assigned_to {
                if let Ok(Some(u)) = Admin::find_by_id(uid).one(db).await {
                    vo.assignee = u.user_name;
                }
            }
            Ok(vo)
        },
        None => Err(Error::from("商机不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &OpportunityListQuery) -> Result<ResultPage<Vec<OpportunityListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = OpportunityModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.stage.clone(),
        query.assigned_to,
        query.customer_id,
    ).await?;
    
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
    
    // 批量查询创建人名称
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect();
    let mut creator_map: HashMap<i64, String> = HashMap::new();
    if !creator_ids.is_empty() {
        let admins = Admin::find()
            .filter(admin::Column::Id.is_in(creator_ids))
            .all(db)
            .await?;
        for a in admins {
            if let Some(name) = a.nick_name.or(a.user_name) {
                creator_map.insert(a.id, name);
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
        let opp_id = item.id;
        let mut vo: OpportunityListVO = item.into();
        vo.customer_name = customer_id.and_then(|id| customer_map.get(&id).cloned());
        vo.created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        vo.quote_count = quote_count_map.get(&opp_id).copied();
        vo
    }).collect();
    
    Ok(ResultPage::new(data, total, page, page_size))
}

/// 商机转报价单
///
/// 根据商机信息创建报价单草稿，并将商机阶段推进到"已报价"
pub async fn convert_to_quotation(db: &DbConn, opportunity_id: i64, user_id: i64) -> Result<i64> {
    use crate::modules::sale::entity::quotation::{self, ActiveModel as QuotationActiveModel};
    use crate::modules::crm::entity::opportunity::{self as opp_entity, ActiveModel as OppActiveModel};
    use sea_orm::ActiveValue::Set;

    // 1. 查询商机
    let opp = OpportunityModel::find_by_id(&db, opportunity_id)
        .await?
        .ok_or_else(|| Error::from("商机不存在".to_string()))?;

    // 2. 创建报价单草稿
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
        .exec(db)
        .await?;

    let quotation_id = quotation_result.last_insert_id;

    // 3. 更新商机阶段为"已报价"（4）和报价状态
    let _ = opp_entity::Entity::update_many()
        .set(opp_entity::ActiveModel {
            stage: Set(Some(4)),
            quote_status: Set(Some(1)),
            update_time: Set(Some(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        })
        .filter(opp_entity::Column::Id.eq(opportunity_id))
        .exec(db)
        .await?;

    Ok(quotation_id)
}