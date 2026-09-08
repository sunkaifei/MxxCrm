use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum CurrencyCode {
    #[sea_orm(num_value = 1)]
    CNY,
    #[sea_orm(num_value = 2)]
    USD,
    #[sea_orm(num_value = 3)]
    EUR,
    #[sea_orm(num_value = 4)]
    GBP,
    #[sea_orm(num_value = 5)]
    JPY,
    #[sea_orm(num_value = 6)]
    HKD,
    #[sea_orm(num_value = 7)]
    AUD,
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurrencyCode::CNY => write!(f, "1"),
            CurrencyCode::USD => write!(f, "2"),
            CurrencyCode::EUR => write!(f, "3"),
            CurrencyCode::GBP => write!(f, "4"),
            CurrencyCode::JPY => write!(f, "5"),
            CurrencyCode::HKD => write!(f, "6"),
            CurrencyCode::AUD => write!(f, "7"),
        }
    }
}

impl FromStr for CurrencyCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i32>() {
            Ok(1) => Ok(CurrencyCode::CNY),
            Ok(2) => Ok(CurrencyCode::USD),
            Ok(3) => Ok(CurrencyCode::EUR),
            Ok(4) => Ok(CurrencyCode::GBP),
            Ok(5) => Ok(CurrencyCode::JPY),
            Ok(6) => Ok(CurrencyCode::HKD),
            Ok(7) => Ok(CurrencyCode::AUD),
            Ok(n) => Err(format!("无效的货币代码: {}", n)),
            Err(e) => Err(format!("货币代码解析失败: {}", e)),
        }
    }
}

impl CurrencyCode {
    /// 转换为数值（用于前端数据传输，遵循项目规则：不使用枚举序列化）
    pub fn to_i32(self) -> i32 {
        match self {
            CurrencyCode::CNY => 1,
            CurrencyCode::USD => 2,
            CurrencyCode::EUR => 3,
            CurrencyCode::GBP => 4,
            CurrencyCode::JPY => 5,
            CurrencyCode::HKD => 6,
            CurrencyCode::AUD => 7,
        }
    }

    /// 从数值转换为 CurrencyCode
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            1 => Some(CurrencyCode::CNY),
            2 => Some(CurrencyCode::USD),
            3 => Some(CurrencyCode::EUR),
            4 => Some(CurrencyCode::GBP),
            5 => Some(CurrencyCode::JPY),
            6 => Some(CurrencyCode::HKD),
            7 => Some(CurrencyCode::AUD),
            _ => None,
        }
    }
}
