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
use sea_orm::DbConn;
use crate::modules::website::model::template_revision::{TemplateRevisionListVO, TemplateRevisionModel, TemplateRevisionSaveDTO};

/// 保存模板版本历史
///
/// 当模板数据发生修改时，调用本方法记录历史版本。
pub async fn save_revision(db: &DbConn, dto: &TemplateRevisionSaveDTO) -> Result<i64> {
    if dto.template_data_id.is_none() {
        return Err(Error::from("模板数据id不能为空"));
    }
    if dto.temptext.is_none() {
        return Err(Error::from("模板内容不能为空"));
    }
    let result = TemplateRevisionModel::insert(db, dto).await?;
    Ok(result)
}

/// 获取某模板数据的版本历史列表
pub async fn get_revisions(db: &DbConn, template_data_id: &Option<i64>) -> Result<Vec<TemplateRevisionListVO>> {
    let list = TemplateRevisionModel::select_by_template_data_id(db, template_data_id).await?;
    let list_data: Vec<TemplateRevisionListVO> = list.into_iter().map(|item| TemplateRevisionListVO::from(item)).collect();
    Ok(list_data)
}

/// 获取版本详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<TemplateRevisionListVO> {
    let result = TemplateRevisionModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "模板版本不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result = TemplateRevisionListVO::from(result);
    Ok(result)
}
