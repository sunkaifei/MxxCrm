use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ActivityType {
    #[sea_orm(num_value = 1)]
    Call = 1,
    #[sea_orm(num_value = 2)]
    Visit = 2,
    #[sea_orm(num_value = 3)]
    Email = 3,
    #[sea_orm(num_value = 4)]
    Meeting = 4,
    #[sea_orm(num_value = 5)]
    Whatsapp = 5,
    #[sea_orm(num_value = 6)]
    Wechat = 6,
    #[sea_orm(num_value = 7)]
    Other = 7,
}

impl TryFrom<i32> for ActivityType {
    type Error = String;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(ActivityType::Call),
            2 => Ok(ActivityType::Visit),
            3 => Ok(ActivityType::Email),
            4 => Ok(ActivityType::Meeting),
            5 => Ok(ActivityType::Whatsapp),
            6 => Ok(ActivityType::Wechat),
            7 => Ok(ActivityType::Other),
            _ => Err(format!("无效的跟进方式值: {}", v)),
        }
    }
}

impl fmt::Display for ActivityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as i32)
    }
}
