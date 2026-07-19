use crate::core::errors::error::{Error, Result};
use crate::modules::ai::model::ai_config::{AiConfigDetailVO, AiConfigListVO, AiConfigModel, AiConfigSaveDTO};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &AiConfigSaveDTO) -> Result<i64> {
    let config = AiConfigModel::insert(db, form_data).await?;
    Ok(config)
}

pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &AiConfigSaveDTO) -> Result<i64> {
    let config = AiConfigModel::update_by_id(db, id, form_data).await?;
    Ok(config)
}

pub async fn find_by_key_unique(db: &DbConn, key: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result = AiConfigModel::find_by_key_unique(db, key, id).await?;
    Ok(result)
}

pub async fn get_by_key(db: &DbConn, key: &str) -> Result<String> {
    let config = AiConfigModel::find_by_key(db, key).await?;
    match config {
        Some(c) => Ok(c.config_value.unwrap_or_default()),
        None => Err(Error::from(format!("配置项 {} 不存在", key))),
    }
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<AiConfigDetailVO>> {
    let config = AiConfigModel::find_by_id(db, id).await?;
    let result = match config {
        Some(c) => Some(AiConfigDetailVO::from(c)),
        None => None,
    };
    Ok(result)
}

pub async fn get_all(db: &DbConn) -> Result<Vec<AiConfigListVO>> {
    let list = AiConfigModel::select_all(db).await?;
    let result: Vec<AiConfigListVO> = list.into_iter().map(|item| AiConfigListVO::from(item)).collect();
    Ok(result)
}

pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64> {
    let result = AiConfigModel::delete_by_id(db, id).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    let result = AiConfigModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}
