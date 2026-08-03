use crate::core::errors::error::Result;
use crate::modules::crm::entity::contract::{self, Entity as Contract};
use crate::modules::crm::entity::customer::{self, Entity as Customer};
use crate::modules::crm::entity::lead::{self, Entity as Lead};
use crate::modules::crm::entity::opportunity::{self, Entity as Opportunity};
use crate::modules::statistics::model::customer_stats::{CustomerTypeStatsVO, CustomerSourceStatsVO, CustomerIndustryStatsVO, CustomerFunnelStatsVO, CustomerFunnelVO};
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter};
use std::collections::HashMap;

fn customer_type_name(t: i32) -> &'static str {
    match t {
        1 => "企业客户",
        2 => "个人客户",
        _ => "未知",
    }
}

fn source_name(s: i32) -> &'static str {
    match s {
        1 => "展会",
        2 => "线上广告",
        3 => "老客户推荐",
        4 => "官网",
        5 => "社交媒体",
        6 => "电话销售",
        7 => "邮件营销",
        8 => "合作伙伴",
        _ => "其他",
    }
}

fn industry_name(i: i32) -> &'static str {
    match i {
        1 => "IT/互联网",
        2 => "制造业",
        3 => "零售业",
        4 => "金融业",
        5 => "医疗健康",
        6 => "教育培训",
        7 => "房地产",
        8 => "交通运输",
        9 => "能源化工",
        10 => "农林牧渔",
        _ => "其他",
    }
}

pub async fn get_customer_type_stats(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<CustomerTypeStatsVO>> {
    // 获取所有未删除客户
    let customers = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 按客户类型分组
    let mut type_map: HashMap<i32, i64> = HashMap::new();
    for c in &customers {
        let t = c.customer_type.unwrap_or(0);
        *type_map.entry(t).or_insert(0) += 1;
    }

    // 获取每个类型客户的合同数
    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut contract_by_customer: HashMap<i64, i64> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            *contract_by_customer.entry(cid).or_insert(0) += 1;
        }
    }

    // 按客户类型聚合合同数
    let mut contract_by_type: HashMap<i32, i64> = HashMap::new();
    for c in &customers {
        let t = c.customer_type.unwrap_or(0);
        let cnt = contract_by_customer.get(&c.id).copied().unwrap_or(0);
        *contract_by_type.entry(t).or_insert(0) += cnt;
    }

    let mut result: Vec<CustomerTypeStatsVO> = type_map.into_iter()
        .map(|(t, count)| CustomerTypeStatsVO {
            customer_type: Some(customer_type_name(t).to_string()),
            total_count: Some(count),
            contract_count: contract_by_type.get(&t).copied(),
            conversion_rate: if count > 0 {
                let c = contract_by_type.get(&t).copied().unwrap_or(0);
                Some(Decimal::from(c) / Decimal::from(count) * Decimal::from(100))
            } else {
                Some(Decimal::ZERO)
            },
        })
        .collect();

    result.sort_by(|a, b| b.total_count.unwrap_or(0).cmp(&a.total_count.unwrap_or(0)));
    Ok(result)
}

pub async fn get_customer_source_stats(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<CustomerSourceStatsVO>> {
    let customers = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 按来源分组
    let mut source_map: HashMap<i32, i64> = HashMap::new();
    for c in &customers {
        let s = c.source.unwrap_or(0);
        *source_map.entry(s).or_insert(0) += 1;
    }

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut contract_by_customer: HashMap<i64, i64> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            *contract_by_customer.entry(cid).or_insert(0) += 1;
        }
    }

    let mut contract_by_source: HashMap<i32, i64> = HashMap::new();
    for c in &customers {
        let s = c.source.unwrap_or(0);
        let cnt = contract_by_customer.get(&c.id).copied().unwrap_or(0);
        *contract_by_source.entry(s).or_insert(0) += cnt;
    }

    let mut result: Vec<CustomerSourceStatsVO> = source_map.into_iter()
        .map(|(s, count)| CustomerSourceStatsVO {
            source: Some(source_name(s).to_string()),
            total_count: Some(count),
            contract_count: contract_by_source.get(&s).copied(),
            conversion_rate: if count > 0 {
                let c = contract_by_source.get(&s).copied().unwrap_or(0);
                Some(Decimal::from(c) / Decimal::from(count) * Decimal::from(100))
            } else {
                Some(Decimal::ZERO)
            },
        })
        .collect();

    result.sort_by(|a, b| b.total_count.unwrap_or(0).cmp(&a.total_count.unwrap_or(0)));
    Ok(result)
}

pub async fn get_customer_industry_stats(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<Vec<CustomerIndustryStatsVO>> {
    let customers = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 按行业分组
    let mut industry_map: HashMap<i32, i64> = HashMap::new();
    for c in &customers {
        let ind = c.industry.unwrap_or(0);
        *industry_map.entry(ind).or_insert(0) += 1;
    }

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut contract_by_customer: HashMap<i64, (i64, Decimal)> = HashMap::new();
    for c in &contracts {
        if let Some(cid) = c.customer_id {
            let e = contract_by_customer.entry(cid).or_insert((0, Decimal::ZERO));
            e.0 += 1;
            e.1 += c.amount.unwrap_or(Decimal::ZERO);
        }
    }

    let mut contract_by_industry: HashMap<i32, (i64, Decimal)> = HashMap::new();
    for c in &customers {
        let ind = c.industry.unwrap_or(0);
        if let Some((cnt, amt)) = contract_by_customer.get(&c.id) {
            let e = contract_by_industry.entry(ind).or_insert((0, Decimal::ZERO));
            e.0 += cnt;
            e.1 += amt;
        }
    }

    let mut result: Vec<CustomerIndustryStatsVO> = industry_map.into_iter()
        .map(|(ind, count)| {
            let (cc, ca) = contract_by_industry.get(&ind).copied().unwrap_or((0, Decimal::ZERO));
            CustomerIndustryStatsVO {
                industry: Some(industry_name(ind).to_string()),
                total_count: Some(count),
                contract_count: Some(cc),
                conversion_rate: if count > 0 {
                    Some(Decimal::from(cc) / Decimal::from(count) * Decimal::from(100))
                } else {
                    Some(Decimal::ZERO)
                },
                contract_amount: Some(ca),
            }
        })
        .collect();

    result.sort_by(|a, b| b.total_count.unwrap_or(0).cmp(&a.total_count.unwrap_or(0)));
    Ok(result)
}

pub async fn get_customer_funnel(db: &DbConn, _year: Option<i32>, _month: Option<i32>) -> Result<CustomerFunnelStatsVO> {
    let lead_count = Lead::find()
        .filter(lead::Column::Deleted.eq(0))
        .count(db)
        .await?
        as i64;

    let customer_count = Customer::find()
        .filter(customer::Column::Deleted.eq(0))
        .count(db)
        .await?
        as i64;

    let opportunity_count = Opportunity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .count(db)
        .await?
        as i64;

    let contracts = Contract::find()
        .filter(contract::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let contract_count = contracts.len() as i64;

    let total_contract_amount: Decimal = contracts.iter()
        .map(|c| c.amount.unwrap_or(Decimal::ZERO))
        .sum();

    let opportunity_amount: Decimal = Opportunity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .all(db)
        .await?
        .iter()
        .map(|o| o.amount.unwrap_or(Decimal::ZERO))
        .sum();

    let l2c = if lead_count > 0 {
        Decimal::from(customer_count) / Decimal::from(lead_count) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let c2o = if customer_count > 0 {
        Decimal::from(opportunity_count) / Decimal::from(customer_count) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let o2c = if opportunity_count > 0 {
        Decimal::from(contract_count) / Decimal::from(opportunity_count) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let overall = if lead_count > 0 {
        Decimal::from(contract_count) / Decimal::from(lead_count) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let funnel = vec![
        CustomerFunnelVO {
            stage: Some("线索".to_string()),
            count: Some(lead_count),
            amount: None,
            rate: Some(Decimal::from(100)),
        },
        CustomerFunnelVO {
            stage: Some("客户".to_string()),
            count: Some(customer_count),
            amount: None,
            rate: Some(l2c),
        },
        CustomerFunnelVO {
            stage: Some("商机".to_string()),
            count: Some(opportunity_count),
            amount: Some(opportunity_amount),
            rate: Some(c2o),
        },
        CustomerFunnelVO {
            stage: Some("合同".to_string()),
            count: Some(contract_count),
            amount: Some(total_contract_amount),
            rate: Some(o2c),
        },
    ];

    Ok(CustomerFunnelStatsVO {
        total_leads: Some(lead_count),
        total_customers: Some(customer_count),
        total_opportunities: Some(opportunity_count),
        total_contracts: Some(contract_count),
        lead_to_customer_rate: Some(l2c),
        customer_to_opportunity_rate: Some(c2o),
        opportunity_to_contract_rate: Some(o2c),
        overall_conversion_rate: Some(overall),
        funnel: Some(funnel),
    })
}