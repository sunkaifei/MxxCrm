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
use crate::modules::shop::model::supplier_apply::*;
use sea_orm::DbConn;

pub async fn submit_apply(db: &DbConn, user_id: i64, req: ApplyRequest) -> Result<i64> {
    let existing = SupplierApplyModel::find_by_user_id(db, user_id).await
        .map_err(|e| Error::from(format!("查询失败: {:?}", e)))?;
    if existing.is_some() {
        return Err(Error::from("您已提交入驻申请"));
    }
    let mut dto: ApplyDTO = req.into();
    dto.user_id = user_id;
    SupplierApplyModel::insert(db, &dto).await
        .map_err(|e| Error::from(format!("提交失败: {:?}", e)))
}

pub async fn audit_apply(db: &DbConn, id: i64, status: i16, audit_remark: Option<String>) -> Result<i64> {
    SupplierApplyModel::update_status(db, id, status, audit_remark).await
        .map_err(|e| Error::from(format!("审核失败: {:?}", e)))
}

pub async fn get_apply_by_id(db: &DbConn, id: i64) -> Result<Option<ApplyVO>> {
    let model = SupplierApplyModel::find_by_id(db, id).await
        .map_err(|e| Error::from(format!("查询失败: {:?}", e)))?;
    Ok(model.map(|m| m.into()))
}

pub async fn get_apply_by_user(db: &DbConn, user_id: i64) -> Result<Option<ApplyVO>> {
    let model = SupplierApplyModel::find_by_user_id(db, user_id).await
        .map_err(|e| Error::from(format!("查询失败: {:?}", e)))?;
    Ok(model.map(|m| m.into()))
}

pub async fn get_apply_page(db: &DbConn, query: ApplyPageQuery) -> Result<(Vec<ApplyVO>, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let (list, total) = SupplierApplyModel::find_page(db, page_num, page_size, query.status).await
        .map_err(|e| Error::from(format!("查询失败: {:?}", e)))?;

    let vo_list: Vec<ApplyVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}
