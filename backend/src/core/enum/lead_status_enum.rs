use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_lead_status")]
pub enum LeadStatus {
    #[sea_orm(string_value = "new")]
    New,
    #[sea_orm(string_value = "following")]
    Following,
    #[sea_orm(string_value = "converted")]
    Converted,
    #[sea_orm(string_value = "invalid")]
    Invalid,
    #[sea_orm(string_value = "recycled")]
    Recycled,
    #[sea_orm(string_value = "unchecked")]
    Unchecked,
    #[sea_orm(string_value = "checking")]
    Checking,
    #[sea_orm(string_value = "valid")]
    Valid,
}

impl fmt::Display for LeadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeadStatus::New => write!(f, "new"),
            LeadStatus::Following => write!(f, "following"),
            LeadStatus::Converted => write!(f, "converted"),
            LeadStatus::Invalid => write!(f, "invalid"),
            LeadStatus::Recycled => write!(f, "recycled"),
            LeadStatus::Unchecked => write!(f, "unchecked"),
            LeadStatus::Checking => write!(f, "checking"),
            LeadStatus::Valid => write!(f, "valid"),
        }
    }
}
