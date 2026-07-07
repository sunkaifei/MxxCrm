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
    #[sea_orm(string_value = "social")]
    Social,
    #[sea_orm(string_value = "other")]
    Other,
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
            IndustryType::Social => write!(f, "social"),
            IndustryType::Other => write!(f, "other"),
        }
    }
}

impl IndustryType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "retail" => Some(IndustryType::Retail),
            "wholesale" => Some(IndustryType::Wholesale),
            "manufacturer" => Some(IndustryType::Manufacturer),
            "trade_agent" => Some(IndustryType::TradeAgent),
            "ecommerce" => Some(IndustryType::Ecommerce),
            "wechat_business" => Some(IndustryType::WechatBusiness),
            "social" => Some(IndustryType::Social),
            "other" => Some(IndustryType::Other),
            _ => None,
        }
    }

    /// 转换为数值（用于 customer 表的 industry 字段，customer 表已改为 INTEGER 存储）
    pub fn to_i32(self) -> i32 {
        match self {
            IndustryType::Retail => 1,
            IndustryType::Wholesale => 2,
            IndustryType::Manufacturer => 3,
            IndustryType::TradeAgent => 4,
            IndustryType::Ecommerce => 5,
            IndustryType::WechatBusiness => 6,
            IndustryType::Social => 7,
            IndustryType::Other => 8,
        }
    }
}
