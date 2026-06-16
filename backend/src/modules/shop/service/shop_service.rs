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
use crate::core::web::response::{ResultPage, MetaResp};
use crate::modules::shop::model::shop::*;
use sea_orm::{DatabaseConnection as DbConn, ConnectionTrait};

pub async fn save<C: ConnectionTrait>(db: &C, form_data: &ShopSaveDTO) -> Result<i64> {
    Ok(ShopModel::insert(db, form_data).await?)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    Ok(ShopModel::batch_delete_by_ids(db, ids).await?)
}

pub async fn update_by_id(db: &DbConn, form_data: &ShopSaveDTO) -> Result<i64> {
    Ok(ShopModel::update_by_id(db, form_data).await?)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<ShopDetailVO>> {
    let model = ShopModel::find_by_id(db, id).await?;
    Ok(model.map(|m| m.into()))
}

pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<ShopListVO>>> {
    let page_num = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    
    let select_where = PageWhere {
        store_name: query.keyword,
        status: query.status,
        self_operated: query.self_operated,
    };
    let search_where = select_where.format();
    
    let (list, total) = ShopModel::select_in_page(db, page_num, page_size, search_where).await?;
    
    let vo_list: Vec<ShopListVO> = list.into_iter().map(|model| model.into()).collect();
    
    Ok(ResultPage::new(vo_list, total, page_num, page_size))
}
