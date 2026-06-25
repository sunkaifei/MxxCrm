use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ContractStatus {
    #[sea_orm(num_value = 1)]
    Draft,
    #[sea_orm(num_value = 2)]
    Signed,
    #[sea_orm(num_value = 3)]
    Executing,
    #[sea_orm(num_value = 4)]
    Completed,
    #[sea_orm(num_value = 5)]
    Terminated,
}

impl fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractStatus::Draft => write!(f, "1"),
            ContractStatus::Signed => write!(f, "2"),
            ContractStatus::Executing => write!(f, "3"),
            ContractStatus::Completed => write!(f, "4"),
            ContractStatus::Terminated => write!(f, "5"),
        }
    }
}
