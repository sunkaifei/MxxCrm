use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum LeadStatus {
    New = 1,
    Following = 2,
    Converted = 3,
    Invalid = 4,
    Recycled = 5,
    Unchecked = 6,
    Checking = 7,
    Valid = 8,
}

impl fmt::Display for LeadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeadStatus::New => write!(f, "1"),
            LeadStatus::Following => write!(f, "2"),
            LeadStatus::Converted => write!(f, "3"),
            LeadStatus::Invalid => write!(f, "4"),
            LeadStatus::Recycled => write!(f, "5"),
            LeadStatus::Unchecked => write!(f, "6"),
            LeadStatus::Checking => write!(f, "7"),
            LeadStatus::Valid => write!(f, "8"),
        }
    }
}
