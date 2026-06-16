//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use std::fmt::Debug;
use serde::{Deserialize, Serialize};


pub const DEFAULT_PAGE_SIZE: i64 = 10;

///
/// # 分页设置
/// # Example
/// 
/// ```
///     // 创建一个 Page 对象并设置一些值
///     let mut page: Page<i32> = Page::new_total(1, 10, 50);
///     println!("{:?}", page);
/// 
///     // 更新 total 并查看 total_pages 的变化
///     page = page.set_total(100);
///     println!("{:?}", page);
/// 
///     // 更新 page_size 并查看 total_pages 的变化
///     page = page.set_page_size(20);
///     println!("{:?}", page);
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page<T: Send + Sync> {
    pub list: Vec<T>,
    /// 记录总数
    pub total: i64,
    /// 当前页数
    pub page_no: i64,
    /// 每页记录条数
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}

impl<T: Send + Sync> Page<T> {
    pub fn new(page_no: i64, page_size: i64) -> Self {
        Page::new_total(page_no, page_size, 0)
    }

    pub fn new_option(page_no: Option<i64>, page_size: Option<i64>) -> Self {
        Page::new(page_no.unwrap_or(1), page_size.unwrap_or(DEFAULT_PAGE_SIZE))
    }

    pub fn new_total(page_no: i64, page_size: i64, total: i64) -> Self {
        let page_no = if page_no < 1 { 1 } else { page_no };
        let total_pages = Self::calculate_total_pages(total, page_size);
        Self {
            list: vec![],
            total,
            page_size,
            page_no,
            total_pages,
        }
    }

    // 新的方法，接受 records 参数
    pub fn with_list(list: Vec<T>, page_no: i64, page_size: i64, total: i64) -> Self {
        let page_no = if page_no < 1 { 1 } else { page_no };
        let total_pages = Self::calculate_total_pages(total, page_size);
        Self {
            list,
            total,
            page_size,
            page_no,
            total_pages,
        }
    }

    pub fn set_total(mut self, total: i64) -> Self {
        self.total = total;
        self.total_pages = Self::calculate_total_pages(total, self.page_size);
        self
    }

    pub fn set_page_size(mut self, page_size: i64) -> Self {
        self.page_size = page_size;
        self.total_pages = Self::calculate_total_pages(self.total, page_size);
        self
    }
    pub fn set_page_no(mut self, page_no: i64) -> Self {
        self.page_no = if page_no < 1 { 1 } else { page_no };
        self
    }
    
    fn calculate_total_pages(total: i64, page_size: i64) -> i64 {
        if page_size == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        }
    }
}

impl<T: Send + Sync> Default for Page<T> {
    fn default() -> Self {
        Page {
            list: vec![],
            total: 0,
            page_size: DEFAULT_PAGE_SIZE,
            page_no: 1,
            total_pages: 0,
        }
    }
}
