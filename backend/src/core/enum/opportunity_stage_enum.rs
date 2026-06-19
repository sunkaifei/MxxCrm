use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mxx_opportunity_stage")]
pub enum OpportunityStage {
    #[sea_orm(string_value = "qualification")]
    Qualification,
    #[sea_orm(string_value = "needs_analysis")]
    NeedsAnalysis,
    #[sea_orm(string_value = "proposal")]
    Proposal,
    #[sea_orm(string_value = "negotiation")]
    Negotiation,
    #[sea_orm(string_value = "won")]
    Won,
    #[sea_orm(string_value = "lost")]
    Lost,
}

impl fmt::Display for OpportunityStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpportunityStage::Qualification => write!(f, "qualification"),
            OpportunityStage::NeedsAnalysis => write!(f, "needs_analysis"),
            OpportunityStage::Proposal => write!(f, "proposal"),
            OpportunityStage::Negotiation => write!(f, "negotiation"),
            OpportunityStage::Won => write!(f, "won"),
            OpportunityStage::Lost => write!(f, "lost"),
        }
    }
}
