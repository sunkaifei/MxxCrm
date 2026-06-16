//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DatabaseConnection;
use crate::modules::finance::model::finance_statistics::{FinanceStatisticsDTO, FinanceStatisticsQuery, FinanceSummary, FinanceStatisticsModel};

pub async fn get_list(db: &DatabaseConnection, query: FinanceStatisticsQuery) -> Result<Vec<FinanceStatisticsDTO>, String> {
    FinanceStatisticsModel::find_list(db, query).await.map_err(|e| e.to_string())
}

pub async fn get_summary(db: &DatabaseConnection) -> Result<FinanceSummary, String> {
    FinanceStatisticsModel::get_summary(db).await.map_err(|e| e.to_string())
}

pub async fn generate_daily_statistics(db: &DatabaseConnection) -> Result<FinanceStatisticsDTO, String> {
    FinanceStatisticsModel::generate_daily_statistics(db).await.map_err(|e| e.to_string())
}
