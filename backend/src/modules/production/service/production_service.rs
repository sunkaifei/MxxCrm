//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::company::entity::company_info::{self, Entity as CompanyInfo};
use actix_web::HttpResponse;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

/// 检查企业类型是否为生产型或混合型
/// 如果企业类型为 sales（销售型），返回 403 Forbidden
pub async fn check_production_access(db: &DbConn) -> Result<()> {
    let company = CompanyInfo::find()
        .filter(company_info::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if let Some(c) = company {
        if let Some(bt) = c.business_type {
            if bt == "sales" {
                return Err(Error::from("当前企业类型为销售型，无法使用生产管理功能"));
            }
        }
    }

    Ok(())
}

/// 检查企业类型是否为生产型或混合型，返回403响应
/// 用于控制器中直接返回HttpResponse
pub async fn check_production_access_response(db: &DbConn) -> Option<HttpResponse> {
    match check_production_access(db).await {
        Ok(_) => None,
        Err(e) => Some(
            HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(403, &e.to_string(), "local")),
        ),
    }
}