use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_industry_type")]
pub enum IndustryType {
    #[sea_orm(string_value = "retail")]
    Retail,
    #[sea_orm(string_value = "wholesale")]
    Wholesale,
    #[sea_orm(string_value = "manufacturer")]
    Manufacturer,
    #[sea_orm(string_value = "trade_agent")]
    TradeAgent,
    #[sea_orm(string_value = "ecommerce")]
    Ecommerce,
    #[sea_orm(string_value = "wechat_business")]
    WechatBusiness,
    #[sea_orm(string_value = "other")]
    Other,
    #[sea_orm(string_value = "social")]
    Social,
}

impl fmt::Display for IndustryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndustryType::Retail => write!(f, "retail"),
            IndustryType::Wholesale => write!(f, "wholesale"),
            IndustryType::Manufacturer => write!(f, "manufacturer"),
            IndustryType::TradeAgent => write!(f, "trade_agent"),
            IndustryType::Ecommerce => write!(f, "ecommerce"),
            IndustryType::WechatBusiness => write!(f, "wechat_business"),
            IndustryType::Other => write!(f, "other"),
            IndustryType::Social => write!(f, "social"),
        }
    }
}