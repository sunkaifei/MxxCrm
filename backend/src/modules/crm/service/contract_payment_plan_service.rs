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
use crate::modules::crm::model::contract_payment_plan::{PaymentPlanModel, PaymentPlanSaveRequest, PaymentPlanVO};
use sea_orm::{DbConn, DbErr, TransactionTrait};

/// 查询合同的回款计划列表
///
/// # 参数
/// * `db` - 数据库连接
/// * `contract_id` - 合同ID
///
/// # 返回
/// * `Result<Vec<PaymentPlanVO>>` - 回款计划列表
pub async fn list(db: &DbConn, contract_id: i64) -> Result<Vec<PaymentPlanVO>> {
    let list = PaymentPlanModel::find_by_contract(db, contract_id).await?;
    Ok(list)
}

/// 批量保存回款计划（事务：先删后插）
///
/// # 参数
/// * `db` - 数据库连接
/// * `req` - 批量保存请求
///
/// # 返回
/// * `Result<i64>` - 插入的记录数
pub async fn save(db: &DbConn, req: &PaymentPlanSaveRequest) -> Result<i64> {
    let contract_id = req.contract_id;
    let plans = req.plans.clone();

    // 先删后插必须事务化，避免中途失败导致数据丢失
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                let inserted = PaymentPlanModel::save_batch(txn, contract_id, plans).await?;
                Ok(inserted)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 删除合同下所有回款计划
///
/// # 参数
/// * `db` - 数据库连接
/// * `contract_id` - 合同ID
///
/// # 返回
/// * `Result<i64>` - 删除的记录数
pub async fn delete(db: &DbConn, contract_id: i64) -> Result<i64> {
    let result = PaymentPlanModel::delete_by_contract(db, contract_id).await?;
    Ok(result)
}
