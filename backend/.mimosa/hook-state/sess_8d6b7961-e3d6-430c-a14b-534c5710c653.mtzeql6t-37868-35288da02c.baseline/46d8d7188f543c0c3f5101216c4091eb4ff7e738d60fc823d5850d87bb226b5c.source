use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_payment_status")]
pub enum PaymentStatus {
    #[sea_orm(string_value = "unpaid")]
    Unpaid,
    #[sea_orm(string_value = "partial")]
    Partial,
    #[sea_orm(string_value = "paid")]
    Paid,
    #[sea_orm(string_value = "overdue")]
    Overdue,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Unpaid => write!(f, "unpaid"),
            PaymentStatus::Partial => write!(f, "partial"),
            PaymentStatus::Paid => write!(f, "paid"),
            PaymentStatus::Overdue => write!(f, "overdue"),
        }
    }
}