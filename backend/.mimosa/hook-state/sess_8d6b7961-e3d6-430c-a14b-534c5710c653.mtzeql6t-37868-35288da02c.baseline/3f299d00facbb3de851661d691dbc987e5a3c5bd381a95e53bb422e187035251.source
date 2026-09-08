//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售回款核销明细模型层
//!
//! 版权所有，侵权必究！
//!

use rust_decimal::Decimal;
use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::Serialize;
use crate::modules::sale::entity::{payment_application, payment_application::Entity as PaymentApplication};
use crate::modules::sale::model::payment::PaymentApplyItem;
use crate::utils::string_utils::serialize_option_u64_to_string;

// ==================== 响应 VO ====================

/// 核销明细 VO
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentApplicationVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub payment_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub plan_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub contract_id: Option<i64>,
    pub apply_amount: Option<Decimal>,
    pub create_time: Option<DateTime>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub created_by: Option<i64>,
}

impl From<&payment_application::Model> for PaymentApplicationVO {
    fn from(model: &payment_application::Model) -> Self {
        Self {
            id: model.id.into(),
            payment_id: model.payment_id,
            plan_id: model.plan_id,
            contract_id: model.contract_id,
            apply_amount: model.apply_amount,
            create_time: model.create_time,
            created_by: model.created_by,
        }
    }
}

// ==================== 数据库操作方法 ====================

pub struct PaymentApplicationModel;

impl PaymentApplicationModel {
    /// 查询单条核销记录
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<payment_application::Model>, DbErr> {
        PaymentApplication::find_by_id(id)
            .filter(payment_application::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询回款的核销明细列表
    pub async fn find_by_payment<C: ConnectionTrait>(db: &C, payment_id: i64) -> Result<Vec<payment_application::Model>, DbErr> {
        PaymentApplication::find()
            .filter(payment_application::Column::PaymentId.eq(payment_id))
            .filter(payment_application::Column::Deleted.eq(0))
            .order_by_asc(payment_application::Column::Id)
            .all(db)
            .await
    }

    /// 查询计划的核销明细列表
    pub async fn find_by_plan<C: ConnectionTrait>(db: &C, plan_id: i64) -> Result<Vec<payment_application::Model>, DbErr> {
        PaymentApplication::find()
            .filter(payment_application::Column::PlanId.eq(plan_id))
            .filter(payment_application::Column::Deleted.eq(0))
            .order_by_asc(payment_application::Column::Id)
            .all(db)
            .await
    }

    /// 批量插入核销记录
    pub async fn insert_batch<C: ConnectionTrait>(
        db: &C,
        payment_id: i64,
        contract_id: Option<i64>,
        items: &[PaymentApplyItem],
        created_by: i64,
    ) -> Result<i64, DbErr> {
        if items.is_empty() {
            return Ok(0);
        }
        let now = chrono::Local::now().naive_local().to_owned();
        let models: Vec<payment_application::ActiveModel> = items.iter().map(|item| {
            payment_application::ActiveModel {
                payment_id: Set(Some(payment_id)),
                plan_id: Set(item.plan_id),
                contract_id: Set(contract_id),
                apply_amount: Set(Some(item.apply_amount)),
                create_time: Set(Some(now)),
                created_by: Set(Some(created_by)),
                deleted: Set(Some(0)),
                ..Default::default()
            }
        }).collect();

        let result = PaymentApplication::insert_many(models).exec(db).await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    /// 软删除回款的所有核销记录
    pub async fn delete_by_payment<C: ConnectionTrait>(db: &C, payment_id: i64) -> Result<i64, DbErr> {
        let result = PaymentApplication::update_many()
            .set(payment_application::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(payment_application::Column::PaymentId.eq(payment_id))
            .filter(payment_application::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 软删除单条核销记录
    pub async fn delete_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        let result = PaymentApplication::update_many()
            .set(payment_application::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(payment_application::Column::Id.eq(id))
            .filter(payment_application::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
