//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DbConn;
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::ip_address::{IpAddressModel, IpListVO, ListQuery, PageWhere};

/// 查询角色列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<IpListVO>>> {

    let select_where = PageWhere {
        start_ip: None,
        end_ip: None,
        country: None,
        province: None,
        city: None,
        county: None,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = IpAddressModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<IpListVO> = list.into_iter().map(|item| IpListVO::from(item)).collect();

    let count = IpAddressModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}