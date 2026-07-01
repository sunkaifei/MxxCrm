//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::Result;
use crate::modules::company::model::company;
use crate::modules::company::model::company::{CompanyAccountSaveRequest, CompanyAccountVO, CompanyInfoSaveRequest, CompanyInfoVO};
use sea_orm::DatabaseConnection;

/// 获取企业信息，mask_sensitive=true 时法人电话脱敏
pub async fn get_info(db: &DatabaseConnection, mask_sensitive: bool) -> Result<CompanyInfoVO> {
    let model = company::find_info(db).await?
        .ok_or_else(|| "企业信息不存在".to_string())?;
    let mut vo: CompanyInfoVO = model.into();
    if mask_sensitive {
        if let Some(phone) = vo.legal_phone.clone() {
            vo.legal_phone = Some(company::mask_phone(&phone));
        }
    }
    Ok(vo)
}

/// 获取银行账户列表
pub async fn get_accounts(db: &DatabaseConnection) -> Result<Vec<CompanyAccountVO>> {
    let list = company::find_accounts(db).await?;
    let items = list.into_iter().map(|m| m.into()).collect();
    Ok(items)
}

/// 更新企业信息
pub async fn update_info(db: &DatabaseConnection, req: &CompanyInfoSaveRequest, updated_by: i64) -> Result<i64> {
    let rows = company::update_info(db, req, updated_by).await?;
    Ok(rows)
}

/// 保存银行账户（有id则update，无id则insert）
pub async fn save_account(db: &DatabaseConnection, req: &CompanyAccountSaveRequest, updated_by: i64) -> Result<i64> {
    match req.id {
        Some(id) if id > 0 => {
            let rows = company::update_account(db, id, req, updated_by).await?;
            Ok(rows)
        }
        _ => {
            let id = company::insert_account(db, req, updated_by).await?;
            Ok(id)
        }
    }
}

/// 删除银行账户（软删除）
pub async fn delete_account(db: &DatabaseConnection, id: i64) -> Result<i64> {
    let rows = company::delete_account(db, id).await?;
    Ok(rows)
}
