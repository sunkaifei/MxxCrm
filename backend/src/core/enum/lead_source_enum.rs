use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_lead_source")]
pub enum LeadSource {
    #[sea_orm(string_value = "website")]
    Website,
    #[sea_orm(string_value = "exhibition")]
    Exhibition,
    #[sea_orm(string_value = "social")]
    Social,
    #[sea_orm(string_value = "referral")]
    Referral,
    #[sea_orm(string_value = "cold_call")]
    ColdCall,
    #[sea_orm(string_value = "customs")]
    Customs,
    #[sea_orm(string_value = "email")]
    Email,
    #[sea_orm(string_value = "alibaba")]
    Alibaba,
    #[sea_orm(string_value = "amazon")]
    Amazon,
    #[sea_orm(string_value = "tiktok")]
    Tiktok,
    #[sea_orm(string_value = "wechat")]
    Wechat,
    #[sea_orm(string_value = "other")]
    Other,
}

impl fmt::Display for LeadSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeadSource::Website => write!(f, "website"),
            LeadSource::Exhibition => write!(f, "exhibition"),
            LeadSource::Social => write!(f, "social"),
            LeadSource::Referral => write!(f, "referral"),
            LeadSource::ColdCall => write!(f, "cold_call"),
            LeadSource::Customs => write!(f, "customs"),
            LeadSource::Email => write!(f, "email"),
            LeadSource::Alibaba => write!(f, "alibaba"),
            LeadSource::Amazon => write!(f, "amazon"),
            LeadSource::Tiktok => write!(f, "tiktok"),
            LeadSource::Wechat => write!(f, "wechat"),
            LeadSource::Other => write!(f, "other"),
        }
    }
}

impl LeadSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "website" => Some(LeadSource::Website),
            "exhibition" => Some(LeadSource::Exhibition),
            "social" => Some(LeadSource::Social),
            "referral" => Some(LeadSource::Referral),
            "cold_call" => Some(LeadSource::ColdCall),
            "customs" => Some(LeadSource::Customs),
            "email" => Some(LeadSource::Email),
            "alibaba" => Some(LeadSource::Alibaba),
            "amazon" => Some(LeadSource::Amazon),
            "tiktok" => Some(LeadSource::Tiktok),
            "wechat" => Some(LeadSource::Wechat),
            "other" => Some(LeadSource::Other),
            _ => None,
        }
    }
}