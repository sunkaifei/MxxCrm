//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::DbConn;
use crate::core::errors::error::{Error, Result};
use crate::modules::website::entity::navigation;
use crate::modules::website::model::navigation::{
    NavigationDetailVO, NavigationModel, NavigationSaveDTO,
};

/// 查询所有导航（按 sort 排序）
pub async fn find_all(db: &DbConn, website_id: i64) -> Result<Vec<NavigationDetailVO>> {
    let list = NavigationModel::find_all(db, website_id).await?;
    Ok(list.into_iter().map(|m| m.into()).collect())
}

/// 根据ID查询导航
pub async fn find_by_id(db: &DbConn, website_id: i64, id: i64) -> Result<NavigationDetailVO> {
    let nav = NavigationModel::find_by_id(db, website_id, id)
        .await?
        .ok_or_else(|| Error::from("导航不存在"))?;
    Ok(nav.into())
}

/// 新增导航
pub async fn insert(db: &DbConn, website_id: i64, dto: NavigationSaveDTO) -> Result<i64> {
    NavigationModel::insert(db, website_id, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 更新导航
pub async fn update(db: &DbConn, website_id: i64, dto: NavigationSaveDTO) -> Result<i64> {
    NavigationModel::update(db, website_id, &dto)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 批量删除导航
pub async fn batch_delete(db: &DbConn, website_id: i64, ids: Vec<i64>) -> Result<i64> {
    NavigationModel::batch_delete_by_ids(db, website_id, ids)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 检查导航名称是否重复
pub async fn find_by_name_unique(
    db: &DbConn,
    website_id: &Option<i64>,
    data_type: &Option<String>,
    name: &Option<String>,
) -> Result<i64> {
    NavigationModel::find_by_name_unique(db, website_id, data_type, name)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 按导航类型查询（header/footer）
pub async fn find_by_type(
    db: &DbConn,
    website_id: i64,
    nav_type: String,
) -> Result<Vec<navigation::Model>> {
    use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
    let list = navigation::Entity::find()
        .filter(navigation::Column::WebsiteId.eq(website_id))
        .filter(navigation::Column::NavType.eq(nav_type))
        .order_by_asc(navigation::Column::Sort)
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(list)
}
