use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_contract_status")]
pub enum ContractStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "signed")]
    Signed,
    #[sea_orm(string_value = "executing")]
    Executing,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "terminated")]
    Terminated,
}

impl fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractStatus::Draft => write!(f, "draft"),
            ContractStatus::Signed => write!(f, "signed"),
            ContractStatus::Executing => write!(f, "executing"),
            ContractStatus::Completed => write!(f, "completed"),
            ContractStatus::Terminated => write!(f, "terminated"),
        }
    }
}
