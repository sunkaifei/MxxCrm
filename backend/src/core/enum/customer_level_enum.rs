use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum CustomerLevel {
    None = 1,
    Key = 2,
    Premium = 3,
    Normal = 4,
    Other = 5,
}

impl fmt::Display for CustomerLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomerLevel::None => write!(f, "1"),
            CustomerLevel::Key => write!(f, "2"),
            CustomerLevel::Premium => write!(f, "3"),
            CustomerLevel::Normal => write!(f, "4"),
            CustomerLevel::Other => write!(f, "5"),
        }
    }
}
