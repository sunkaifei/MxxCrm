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

impl TryFrom<i32> for LeadStatus {
    type Error = String;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(LeadStatus::New),
            2 => Ok(LeadStatus::Following),
            3 => Ok(LeadStatus::Converted),
            4 => Ok(LeadStatus::Invalid),
            5 => Ok(LeadStatus::Recycled),
            6 => Ok(LeadStatus::Unchecked),
            7 => Ok(LeadStatus::Checking),
            8 => Ok(LeadStatus::Valid),
            _ => Err(format!("无效的状态值: {}", v)),
        }
    }
}

impl fmt::Display for LeadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as i32)
    }
}
