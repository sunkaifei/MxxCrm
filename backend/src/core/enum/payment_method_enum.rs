use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_payment_method")]
pub enum PaymentMethod {
    #[sea_orm(string_value = "cash")]
    Cash,
    #[sea_orm(string_value = "bank_transfer")]
    BankTransfer,
    #[sea_orm(string_value = "credit_card")]
    CreditCard,
    #[sea_orm(string_value = "paypal")]
    Paypal,
    #[sea_orm(string_value = "wechat_pay")]
    WechatPay,
    #[sea_orm(string_value = "alipay")]
    Alipay,
    #[sea_orm(string_value = "tt")]
    Tt,
    #[sea_orm(string_value = "lc")]
    Lc,
    #[sea_orm(string_value = "other")]
    Other,
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::BankTransfer => write!(f, "bank_transfer"),
            PaymentMethod::CreditCard => write!(f, "credit_card"),
            PaymentMethod::Paypal => write!(f, "paypal"),
            PaymentMethod::WechatPay => write!(f, "wechat_pay"),
            PaymentMethod::Alipay => write!(f, "alipay"),
            PaymentMethod::Tt => write!(f, "tt"),
            PaymentMethod::Lc => write!(f, "lc"),
            PaymentMethod::Other => write!(f, "other"),
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cash" => Ok(PaymentMethod::Cash),
            "bank_transfer" => Ok(PaymentMethod::BankTransfer),
            "credit_card" => Ok(PaymentMethod::CreditCard),
            "paypal" => Ok(PaymentMethod::Paypal),
            "wechat_pay" => Ok(PaymentMethod::WechatPay),
            "alipay" => Ok(PaymentMethod::Alipay),
            "tt" => Ok(PaymentMethod::Tt),
            "lc" => Ok(PaymentMethod::Lc),
            "other" => Ok(PaymentMethod::Other),
            _ => Err(format!("无效支付方式: {}", s)),
        }
    }
}
