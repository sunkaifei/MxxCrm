use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_lead_source")]
pub enum LeadSource {
    #[sea_orm(string_value = "website")]
    Website,
    #[sea_orm(string_value = "social_media")]
    SocialMedia,
    #[sea_orm(string_value = "email_marketing")]
    EmailMarketing,
    #[sea_orm(string_value = "referral")]
    Referral,
    #[sea_orm(string_value = "trade_show")]
    TradeShow,
    #[sea_orm(string_value = "cold_call")]
    ColdCall,
    #[sea_orm(string_value = "direct_mail")]
    DirectMail,
    #[sea_orm(string_value = "search_engine")]
    SearchEngine,
    #[sea_orm(string_value = "partner")]
    Partner,
    #[sea_orm(string_value = "other")]
    Other,
}

impl fmt::Display for LeadSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeadSource::Website => write!(f, "website"),
            LeadSource::SocialMedia => write!(f, "social_media"),
            LeadSource::EmailMarketing => write!(f, "email_marketing"),
            LeadSource::Referral => write!(f, "referral"),
            LeadSource::TradeShow => write!(f, "trade_show"),
            LeadSource::ColdCall => write!(f, "cold_call"),
            LeadSource::DirectMail => write!(f, "direct_mail"),
            LeadSource::SearchEngine => write!(f, "search_engine"),
            LeadSource::Partner => write!(f, "partner"),
            LeadSource::Other => write!(f, "other"),
        }
    }
}

impl LeadSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "website" => Some(LeadSource::Website),
            "social_media" => Some(LeadSource::SocialMedia),
            "email_marketing" => Some(LeadSource::EmailMarketing),
            "referral" => Some(LeadSource::Referral),
            "trade_show" => Some(LeadSource::TradeShow),
            "cold_call" => Some(LeadSource::ColdCall),
            "direct_mail" => Some(LeadSource::DirectMail),
            "search_engine" => Some(LeadSource::SearchEngine),
            "partner" => Some(LeadSource::Partner),
            "other" => Some(LeadSource::Other),
            _ => None,
        }
    }
}