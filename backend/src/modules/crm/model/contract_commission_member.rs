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
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::crm::entity::contract_commission_member::{self, Entity as ContractCommissionMember};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractCommissionMemberVO {
    pub id: i64,
    pub contract_id: Option<i64>,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub role_type: i32,
    pub role_name: Option<String>,
    pub share_ratio: f64,
    pub sort: i32,
}

impl From<contract_commission_member::Model> for ContractCommissionMemberVO {
    fn from(item: contract_commission_member::Model) -> Self {
        ContractCommissionMemberVO {
            id: item.id,
            contract_id: Some(item.contract_id),
            user_id: item.user_id,
            user_name: item.user_name,
            role_type: item.role_type,
            role_name: None,
            share_ratio: item.share_ratio.to_f64().unwrap_or_default(),
            sort: item.sort,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractCommissionMemberSaveDTO {
    pub id: Option<i64>,
    pub contract_id: Option<i64>,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub role_type: i32,
    pub share_ratio: f64,
    pub sort: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractCommissionMemberBatchSaveDTO {
    pub contract_id: i64,
    pub members: Vec<ContractCommissionMemberSaveDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractCommissionConfigVO {
    pub rule_id: Option<i64>,
    pub mode: Option<i32>,
    pub members: Vec<ContractCommissionMemberVO>,
}

pub struct ContractCommissionMemberModel;

impl ContractCommissionMemberModel {
    pub async fn find_by_contract_id<C: ConnectionTrait>(db: &C, contract_id: i64) -> Result<Vec<contract_commission_member::Model>, DbErr> {
        ContractCommissionMember::find()
            .filter(contract_commission_member::Column::ContractId.eq(contract_id))
            .order_by_asc(contract_commission_member::Column::Sort)
            .all(db)
            .await
    }

    pub async fn insert_batch<C: ConnectionTrait>(db: &C, contract_id: i64, members: &[ContractCommissionMemberSaveDTO], created_by: Option<i64>) -> Result<(), DbErr> {
        Self::delete_by_contract_id(db, contract_id).await?;

        if members.is_empty() {
            return Ok(());
        }

        let now = chrono::Local::now().naive_local().to_owned();
        let mut active_models: Vec<contract_commission_member::ActiveModel> = Vec::with_capacity(members.len());
        for member in members {
            let share_ratio = Decimal::from_f64(member.share_ratio).unwrap_or_default();
            let am = contract_commission_member::ActiveModel {
                contract_id: Set(contract_id),
                user_id: Set(member.user_id),
                user_name: Set(member.user_name.clone()),
                role_type: Set(member.role_type),
                share_ratio: Set(share_ratio),
                sort: Set(member.sort.unwrap_or(0)),
                created_by: Set(created_by),
                create_time: Set(Some(now)),
                updated_by: Set(created_by),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            active_models.push(am);
        }

        let _ = ContractCommissionMember::insert_many(active_models).exec(db).await?;
        Ok(())
    }

    pub async fn delete_by_contract_id<C: ConnectionTrait>(db: &C, contract_id: i64) -> Result<u64, DbErr> {
        let result = ContractCommissionMember::delete_many()
            .filter(contract_commission_member::Column::ContractId.eq(contract_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
