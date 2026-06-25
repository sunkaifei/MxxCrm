use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_purchase_status")]
pub enum PurchaseStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "ordered")]
    Ordered,
    #[sea_orm(string_value = "in_transit")]
    InTransit,
    #[sea_orm(string_value = "received")]
    Received,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

impl fmt::Display for PurchaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurchaseStatus::Draft => write!(f, "draft"),
            PurchaseStatus::Ordered => write!(f, "ordered"),
            PurchaseStatus::InTransit => write!(f, "in_transit"),
            PurchaseStatus::Received => write!(f, "received"),
            PurchaseStatus::Completed => write!(f, "completed"),
            PurchaseStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}