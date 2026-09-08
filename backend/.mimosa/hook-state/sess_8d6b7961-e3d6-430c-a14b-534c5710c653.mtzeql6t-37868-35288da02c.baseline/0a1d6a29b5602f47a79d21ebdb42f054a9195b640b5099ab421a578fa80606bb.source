use crate::core::errors::error::Result;
use crate::modules::ai::model::ai_config::{AiConfigDetailVO, AiConfigListVO, AiConfigModel, AiConfigSaveDTO};
use sea_orm::DbConn;

/// 对 API 密钥做脱敏处理：保留前4位和后4位，中间用 **** 替换
/// 长度 <= 8 时完全遮蔽，防止通过脱敏值反推
fn mask_api_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }
    let prefix: String = chars[..4].iter().collect();
    let suffix: String = chars[chars.len() - 4..].iter().collect();
    format!("{}****{}", prefix, suffix)
}

/// 判断配置项是否为 API 密钥类（config_key 含 api_key）
fn is_api_key_config(key: &Option<String>) -> bool {
    key.as_ref().map_or(false, |k| k.contains("api_key"))
}

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

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<AiConfigDetailVO>> {
    let config = AiConfigModel::find_by_id(db, id).await?;
    let result = match config {
        Some(c) => {
            let mut vo = AiConfigDetailVO::from(c);
            // 对 api_key 类配置做脱敏，防止明文泄露
            if is_api_key_config(&vo.config_key) {
                if let Some(ref v) = vo.config_value {
                    vo.config_value = Some(mask_api_key(v));
                }
            }
            Some(vo)
        }
        None => None,
    };
    Ok(result)
}

pub async fn get_all(db: &DbConn) -> Result<Vec<AiConfigListVO>> {
    let list = AiConfigModel::select_all(db).await?;
    let result: Vec<AiConfigListVO> = list
        .into_iter()
        .map(|item| {
            let mut vo = AiConfigListVO::from(item);
            // 对 api_key 类配置做脱敏，防止明文泄露
            if is_api_key_config(&vo.config_key) {
                if let Some(ref v) = vo.config_value {
                    vo.config_value = Some(mask_api_key(v));
                }
            }
            vo
        })
        .collect();
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
