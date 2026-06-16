use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
