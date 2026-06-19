use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_activity_type")]
pub enum ActivityType {
    #[sea_orm(string_value = "call")]
    Call,
    #[sea_orm(string_value = "visit")]
    Visit,
    #[sea_orm(string_value = "email")]
    Email,
    #[sea_orm(string_value = "meeting")]
    Meeting,
    #[sea_orm(string_value = "demo")]
    Demo,
    #[sea_orm(string_value = "whatsapp")]
    Whatsapp,
    #[sea_orm(string_value = "wechat")]
    Wechat,
    #[sea_orm(string_value = "other")]
    Other,
}

impl fmt::Display for ActivityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityType::Call => write!(f, "call"),
            ActivityType::Visit => write!(f, "visit"),
            ActivityType::Email => write!(f, "email"),
            ActivityType::Meeting => write!(f, "meeting"),
            ActivityType::Demo => write!(f, "demo"),
            ActivityType::Whatsapp => write!(f, "whatsapp"),
            ActivityType::Wechat => write!(f, "wechat"),
            ActivityType::Other => write!(f, "other"),
        }
    }
}
