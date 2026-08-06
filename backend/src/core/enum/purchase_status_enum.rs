use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_purchase_status")]
pub enum PurchaseStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "pending_audit")]
    PendingAudit,
    #[sea_orm(string_value = "audited")]
    Audited,
    #[sea_orm(string_value = "ordered")]
    Ordered,
    #[sea_orm(string_value = "in_transit")]
    InTransit,
    #[sea_orm(string_value = "partial_received")]
    PartialReceived,
    #[sea_orm(string_value = "received")]
    Received,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
    #[sea_orm(string_value = "rejected")]
    Rejected,
}

impl fmt::Display for PurchaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurchaseStatus::Draft => write!(f, "draft"),
            PurchaseStatus::PendingAudit => write!(f, "pending_audit"),
            PurchaseStatus::Audited => write!(f, "audited"),
            PurchaseStatus::Ordered => write!(f, "ordered"),
            PurchaseStatus::InTransit => write!(f, "in_transit"),
            PurchaseStatus::PartialReceived => write!(f, "partial_received"),
            PurchaseStatus::Received => write!(f, "received"),
            PurchaseStatus::Completed => write!(f, "completed"),
            PurchaseStatus::Cancelled => write!(f, "cancelled"),
            PurchaseStatus::Rejected => write!(f, "rejected"),
        }
    }
}