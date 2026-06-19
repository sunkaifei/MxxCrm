use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_customer_level")]
pub enum CustomerLevel {
    #[sea_orm(string_value = "potential")]
    Potential,
    #[sea_orm(string_value = "normal")]
    Normal,
    #[sea_orm(string_value = "vip")]
    Vip,
    #[sea_orm(string_value = "svip")]
    Svip,
    #[sea_orm(string_value = "lost")]
    Lost,
}

impl fmt::Display for CustomerLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomerLevel::Potential => write!(f, "potential"),
            CustomerLevel::Normal => write!(f, "normal"),
            CustomerLevel::Vip => write!(f, "vip"),
            CustomerLevel::Svip => write!(f, "svip"),
            CustomerLevel::Lost => write!(f, "lost"),
        }
    }
}