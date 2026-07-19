use crate::modules::ai::entity::company_background_check;
use crate::modules::ai::entity::company_background_check::Entity as CompanyBackgroundCheck;
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundCheckSaveDTO {
    pub company_name: Option<String>,
    pub company_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub risk_score: Option<i32>,
    pub risk_level: Option<String>,
    pub report_data: Option<serde_json::Value>,
    pub ai_model: Option<String>,
    pub prompt_version: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundCheckListVO {
    pub id: i64,
    pub company_name: Option<String>,
    pub company_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub risk_score: Option<i32>,
    pub risk_level: Option<String>,
    pub ai_model: Option<String>,
    pub prompt_version: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundCheckDetailVO {
    pub id: i64,
    pub company_name: Option<String>,
    pub company_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub risk_score: Option<i32>,
    pub risk_level: Option<String>,
    pub report_data: Option<serde_json::Value>,
    pub ai_model: Option<String>,
    pub prompt_version: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

impl From<company_background_check::Model> for BackgroundCheckListVO {
    fn from(item: company_background_check::Model) -> Self {
        Self {
            id: item.id,
            company_name: item.company_name,
            company_id: item.company_id,
            lead_id: item.lead_id,
            risk_score: item.risk_score,
            risk_level: item.risk_level,
            ai_model: item.ai_model,
            prompt_version: item.prompt_version,
            created_by: item.created_by,
            created_at: item.created_at.map(|t| t.and_utc().to_rfc3339()),
        }
    }
}

impl From<company_background_check::Model> for BackgroundCheckDetailVO {
    fn from(item: company_background_check::Model) -> Self {
        Self {
            id: item.id,
            company_name: item.company_name,
            company_id: item.company_id,
            lead_id: item.lead_id,
            risk_score: item.risk_score,
            risk_level: item.risk_level,
            report_data: item.report_data,
            ai_model: item.ai_model,
            prompt_version: item.prompt_version,
            created_by: item.created_by,
            created_at: item.created_at.map(|t| t.and_utc().to_rfc3339()),
        }
    }
}

pub struct BackgroundCheckModel;

impl BackgroundCheckModel {
    pub async fn insert(db: &sea_orm::DbConn, form_data: &BackgroundCheckSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let active_model = company_background_check::ActiveModel {
            company_name: sea_orm::Set(form_data.company_name.clone()),
            company_id: sea_orm::Set(form_data.company_id.clone()),
            lead_id: sea_orm::Set(form_data.lead_id.clone()),
            risk_score: sea_orm::Set(form_data.risk_score.clone()),
            risk_level: sea_orm::Set(form_data.risk_level.clone()),
            report_data: sea_orm::Set(form_data.report_data.clone()),
            ai_model: sea_orm::Set(form_data.ai_model.clone()),
            prompt_version: sea_orm::Set(form_data.prompt_version.clone()),
            created_by: sea_orm::Set(form_data.created_by.clone()),
            created_at: sea_orm::Set(Some(now)),
            ..Default::default()
        };
        let result = CompanyBackgroundCheck::insert(active_model).exec(db).await?;
        Ok(result.last_insert_id)
    }

    pub async fn find_by_id(db: &sea_orm::DbConn, id: i64) -> Result<Option<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find_by_id(id).one(db).await
    }

    pub async fn find_by_lead_id(db: &sea_orm::DbConn, lead_id: i64) -> Result<Vec<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find()
            .filter(company_background_check::Column::LeadId.eq(lead_id))
            .order_by(company_background_check::Column::CreatedAt, sea_orm::Order::Desc)
            .all(db)
            .await
    }

    pub async fn find_by_company_name(db: &sea_orm::DbConn, company_name: &str) -> Result<Vec<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find()
            .filter(company_background_check::Column::CompanyName.eq(company_name))
            .order_by(company_background_check::Column::CreatedAt, sea_orm::Order::Desc)
            .all(db)
            .await
    }

    pub async fn find_latest_by_lead_id(db: &sea_orm::DbConn, lead_id: i64) -> Result<Option<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find()
            .filter(company_background_check::Column::LeadId.eq(lead_id))
            .order_by(company_background_check::Column::CreatedAt, sea_orm::Order::Desc)
            .one(db)
            .await
    }

    pub async fn find_latest_by_company_id(db: &sea_orm::DbConn, company_id: i64) -> Result<Option<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find()
            .filter(company_background_check::Column::CompanyId.eq(company_id))
            .order_by(company_background_check::Column::CreatedAt, sea_orm::Order::Desc)
            .one(db)
            .await
    }

    pub async fn find_by_company_id(db: &sea_orm::DbConn, company_id: i64) -> Result<Vec<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find()
            .filter(company_background_check::Column::CompanyId.eq(company_id))
            .order_by(company_background_check::Column::CreatedAt, sea_orm::Order::Desc)
            .all(db)
            .await
    }

    pub async fn get_timeline_by_company_name(db: &sea_orm::DbConn, company_name: &str) -> Result<Vec<company_background_check::Model>, DbErr> {
        CompanyBackgroundCheck::find()
            .filter(company_background_check::Column::CompanyName.eq(company_name))
            .order_by(company_background_check::Column::CreatedAt, sea_orm::Order::Asc)
            .all(db)
            .await
    }

    pub async fn delete_by_id(db: &sea_orm::DbConn, id: i64) -> Result<i64, DbErr> {
        let result = CompanyBackgroundCheck::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected as i64)
    }
}
