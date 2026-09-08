use crate::modules::ai::entity::ai_config;
use crate::modules::ai::entity::ai_config::Entity as AiConfig;
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigSaveDTO {
    pub id: Option<i64>,
    pub config_key: Option<String>,
    pub config_name: Option<String>,
    pub config_value: Option<String>,
    pub config_type: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_by: Option<String>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigListVO {
    pub id: i64,
    pub config_key: Option<String>,
    pub config_name: Option<String>,
    pub config_value: Option<String>,
    pub config_type: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_by: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigDetailVO {
    pub id: i64,
    pub config_key: Option<String>,
    pub config_name: Option<String>,
    pub config_value: Option<String>,
    pub config_type: Option<String>,
    pub remark: Option<String>,
    pub sort: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_by: Option<String>,
    pub updated_at: Option<String>,
}

impl From<ai_config::Model> for AiConfigListVO {
    fn from(item: ai_config::Model) -> Self {
        Self {
            id: item.id,
            config_key: item.config_key,
            config_name: item.config_name,
            config_value: item.config_value,
            config_type: item.config_type,
            remark: item.remark,
            sort: item.sort,
            created_by: item.created_by,
            created_at: item.created_at.map(|t| t.and_utc().to_rfc3339()),
            updated_by: item.updated_by,
            updated_at: item.updated_at.map(|t| t.and_utc().to_rfc3339()),
        }
    }
}

impl From<ai_config::Model> for AiConfigDetailVO {
    fn from(item: ai_config::Model) -> Self {
        Self {
            id: item.id,
            config_key: item.config_key,
            config_name: item.config_name,
            config_value: item.config_value,
            config_type: item.config_type,
            remark: item.remark,
            sort: item.sort,
            created_by: item.created_by,
            created_at: item.created_at.map(|t| t.and_utc().to_rfc3339()),
            updated_by: item.updated_by,
            updated_at: item.updated_at.map(|t| t.and_utc().to_rfc3339()),
        }
    }
}

pub struct AiConfigModel;

impl AiConfigModel {
    pub async fn insert(db: &sea_orm::DbConn, form_data: &AiConfigSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let active_model = ai_config::ActiveModel {
            config_key: sea_orm::Set(form_data.config_key.clone()),
            config_name: sea_orm::Set(form_data.config_name.clone()),
            config_value: sea_orm::Set(form_data.config_value.clone()),
            config_type: sea_orm::Set(form_data.config_type.clone()),
            remark: sea_orm::Set(form_data.remark.clone()),
            sort: sea_orm::Set(form_data.sort.clone()),
            created_by: sea_orm::Set(form_data.created_by.clone()),
            created_at: sea_orm::Set(Some(now)),
            updated_by: sea_orm::Set(form_data.updated_by.clone()),
            updated_at: sea_orm::Set(Some(now)),
            ..Default::default()
        };
        let result = AiConfig::insert(active_model).exec(db).await?;
        Ok(result.last_insert_id)
    }

    pub async fn update_by_id(db: &sea_orm::DbConn, id: &Option<i64>, form_data: &AiConfigSaveDTO) -> Result<i64, DbErr> {
        if let Some(id_val) = id {
            let now = chrono::Utc::now().naive_utc();
            let active_model = ai_config::ActiveModel {
                id: sea_orm::Set(*id_val),
                config_key: sea_orm::Set(form_data.config_key.clone()),
                config_name: sea_orm::Set(form_data.config_name.clone()),
                config_value: sea_orm::Set(form_data.config_value.clone()),
                config_type: sea_orm::Set(form_data.config_type.clone()),
                remark: sea_orm::Set(form_data.remark.clone()),
                sort: sea_orm::Set(form_data.sort.clone()),
                updated_by: sea_orm::Set(form_data.updated_by.clone()),
                updated_at: sea_orm::Set(Some(now)),
                ..Default::default()
            };
            let result = AiConfig::update(active_model).exec(db).await?;
            Ok(result.id)
        } else {
            Err(DbErr::Custom("ID不能为空".to_string()))
        }
    }

    pub async fn find_by_id(db: &sea_orm::DbConn, id: &Option<i64>) -> Result<Option<ai_config::Model>, DbErr> {
        if let Some(id_val) = id {
            AiConfig::find_by_id(*id_val).one(db).await
        } else {
            Ok(None)
        }
    }

    pub async fn find_by_key_unique(db: &sea_orm::DbConn, key: &Option<String>, id: &Option<i64>) -> Result<bool, DbErr> {
        if let Some(key_val) = key {
            let mut query = AiConfig::find().filter(ai_config::Column::ConfigKey.eq(key_val));
            if let Some(id_val) = id {
                query = query.filter(ai_config::Column::Id.ne(*id_val));
            }
            let count = query.count(db).await?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    pub async fn select_all(db: &sea_orm::DbConn) -> Result<Vec<ai_config::Model>, DbErr> {
        AiConfig::find()
            .order_by(ai_config::Column::Sort, sea_orm::Order::Asc)
            .all(db)
            .await
    }

    pub async fn delete_by_id(db: &sea_orm::DbConn, id: i64) -> Result<i64, DbErr> {
        let result = AiConfig::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids(db: &sea_orm::DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result = AiConfig::delete_many()
            .filter(ai_config::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
