use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_currency_code")]
pub enum CurrencyCode {
    #[sea_orm(string_value = "CNY")]
    CNY,
    #[sea_orm(string_value = "USD")]
    USD,
    #[sea_orm(string_value = "EUR")]
    EUR,
    #[sea_orm(string_value = "GBP")]
    GBP,
    #[sea_orm(string_value = "JPY")]
    JPY,
    #[sea_orm(string_value = "HKD")]
    HKD,
    #[sea_orm(string_value = "AUD")]
    AUD,
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurrencyCode::CNY => write!(f, "CNY"),
            CurrencyCode::USD => write!(f, "USD"),
            CurrencyCode::EUR => write!(f, "EUR"),
            CurrencyCode::GBP => write!(f, "GBP"),
            CurrencyCode::JPY => write!(f, "JPY"),
            CurrencyCode::HKD => write!(f, "HKD"),
            CurrencyCode::AUD => write!(f, "AUD"),
        }
    }
}

impl FromStr for CurrencyCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CNY" => Ok(CurrencyCode::CNY),
            "USD" => Ok(CurrencyCode::USD),
            "EUR" => Ok(CurrencyCode::EUR),
            "GBP" => Ok(CurrencyCode::GBP),
            "JPY" => Ok(CurrencyCode::JPY),
            "HKD" => Ok(CurrencyCode::HKD),
            "AUD" => Ok(CurrencyCode::AUD),
            _ => Err(format!("无效币种: {}", s)),
        }
    }
}