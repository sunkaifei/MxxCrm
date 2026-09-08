use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityStage {
    Qualification,
    NeedsAnalysis,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

impl OpportunityStage {
    /// 将数值转为阶段名称（用于数据库查询等）
    pub fn from_code(code: i32) -> Option<OpportunityStage> {
        match code {
            0 => Some(OpportunityStage::Qualification),
            1 => Some(OpportunityStage::NeedsAnalysis),
            2 => Some(OpportunityStage::Proposal),
            3 => Some(OpportunityStage::Negotiation),
            4 => Some(OpportunityStage::Won),
            5 => Some(OpportunityStage::Lost),
            _ => None,
        }
    }

    /// 将阶段转为数值
    pub fn to_code(&self) -> i32 {
        match self {
            OpportunityStage::Qualification => 0,
            OpportunityStage::NeedsAnalysis => 1,
            OpportunityStage::Proposal => 2,
            OpportunityStage::Negotiation => 3,
            OpportunityStage::Won => 4,
            OpportunityStage::Lost => 5,
        }
    }
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
