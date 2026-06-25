use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ContractType {
    #[sea_orm(num_value = 1)]
    Sales,
    #[sea_orm(num_value = 2)]
    Purchase,
    #[sea_orm(num_value = 3)]
    Service,
    #[sea_orm(num_value = 4)]
    Cooperation,
    #[sea_orm(num_value = 5)]
    Other,
}

impl fmt::Display for ContractType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractType::Sales => write!(f, "1"),
            ContractType::Purchase => write!(f, "2"),
            ContractType::Service => write!(f, "3"),
            ContractType::Cooperation => write!(f, "4"),
            ContractType::Other => write!(f, "5"),
        }
    }
}
