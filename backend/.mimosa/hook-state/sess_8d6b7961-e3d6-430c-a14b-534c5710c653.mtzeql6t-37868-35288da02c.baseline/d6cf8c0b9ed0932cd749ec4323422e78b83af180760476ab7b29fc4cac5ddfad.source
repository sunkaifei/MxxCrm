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
#[allow(unused_imports)]
use crate::modules::crm::entity::{contract, contract_commission_member};
use crate::modules::crm::model::contract_commission_member::{ContractCommissionMemberVO, ContractCommissionMemberSaveDTO, ContractCommissionMemberModel, ContractCommissionConfigVO};
#[allow(unused_imports)]
use crate::modules::system::entity::admin;
#[allow(unused_imports)]
use sea_orm::{DbConn, TransactionTrait, Set, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};
#[allow(unused_imports)]
use rust_decimal::prelude::ToPrimitive;
#[allow(unused_imports)]
use std::cmp::Ordering;

pub async fn get_commission_config(db: &DbConn, contract_id: i64) -> Result<ContractCommissionConfigVO> {
    let contract = contract::Entity::find_by_id(contract_id)
        .filter(contract::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("合同不存在".to_string()))?;

    let list = ContractCommissionMemberModel::find_by_contract_id(db, contract_id).await?;
    let vos: Vec<ContractCommissionMemberVO> = list.into_iter().map(|item| item.into()).collect();

    Ok(ContractCommissionConfigVO {
        rule_id: contract.commission_rule_id,
        mode: contract.commission_mode,
        members: vos,
    })
}

pub async fn save_contract_members(db: &DbConn, contract_id: i64, members: &[ContractCommissionMemberSaveDTO], user_id: i64) -> Result<()> {
    let total_ratio: f64 = members.iter().map(|m| m.share_ratio).sum();
    let diff = (total_ratio - 1.0).abs();
    if diff > 0.001 {
        return Err(Error::from("所有成员提成比例之和必须等于100%".to_string()));
    }

    let members_cloned = members.to_vec();

    let result = db.transaction::<_, (), sea_orm::DbErr>(move |txn| {
        Box::pin(async move {
            ContractCommissionMemberModel::insert_batch(txn, contract_id, &members_cloned, Some(user_id)).await?;

            let now = chrono::Local::now().naive_local();
            let active_model = contract::ActiveModel {
                commission_mode: Set(Some(2)),
                updated_by: Set(Some(user_id)),
                update_time: Set(Some(now)),
                ..Default::default()
            };

            contract::Entity::update_many()
                .set(active_model)
                .filter(contract::Column::Id.eq(contract_id))
                .exec(txn)
                .await?;

            Ok(())
        })
    }).await;

    result.map_err(|e| Error::from(e.to_string()))
}

pub async fn set_commission_rule(db: &DbConn, contract_id: i64, rule_id: Option<i64>) -> Result<()> {
    let rule_id_value = match rule_id {
        Some(id) if id > 0 => Some(id),
        _ => None,
    };

    let mut active_model: contract::ActiveModel = Default::default();
    active_model.commission_rule_id = Set(rule_id_value);
    active_model.commission_mode = Set(Some(1));

    contract::Entity::update_many()
        .set(active_model)
        .filter(contract::Column::Id.eq(contract_id))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(())
}
