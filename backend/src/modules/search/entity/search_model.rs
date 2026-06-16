//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexSaveRequest {
    /// 信息ID
    pub id: Option<String>,
    /// 信息类型，1文章，2企业，3百科
    pub info_type: Option<i32>,
    /// 地区ID列表
    pub region_ids: Option<String>,
    /// 栏目ID列表
    pub column_ids: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 内容
    pub content: Option<String>,
    /// 基础信息
    pub basics: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchPageRequest {
    // 搜索关键词
    pub q: Option<String>,
    // 区域筛选
    pub area: Option<String>,
    // 行业筛选
    pub trade: Option<String>,
    /// 信息类型，1文章，2企业，3百科
    pub info_type: Option<String>,
    // 当前页码数
    pub p: Option<i64>,
    // 不包含的ID
    pub not_id: Option<i64>,
    /// 三个排名id
    pub top_id: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchPageVO {
    /// ID
    pub id: Option<String>,
    /// 短网址
    pub short_url: Option<String>,
    /// 信息类型，1文章，2企业，3百科
    pub info_type: Option<i32>,
    /// 地区ID列表
    pub region_ids: Option<String>,
    /// 栏目ID列表
    pub column_ids: Option<String>,
    /// 是否是竞价关键词
    pub bidding: bool,
    /// 竞价关键词排名顺序
    pub top: i32,
    /// 标题
    pub title: Option<String>,
    /// 内容
    pub content: Option<String>,
    /// 基础信息
    pub basics: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
}

/// 主要是企业和化学字典的基本信息
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BasicsVO {
    /// 短网址
    pub short_url: Option<String>,
    /// 形象图片
    pub image_url: Option<String>,
    /// 法人
    pub corporation: Option<String>,
    /// 注册资本
    pub capital: Option<String>,
    /// 注册资本货币
    pub money: Option<String>,
    /// 经营范围
    pub business: Option<String>,
    /// 成立时间
    pub create_date: Option<String>,
    /// 经营地址
    pub business_address: Option<String>,
    /// 经营状态
    pub operation_status: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatedQueryRequest {
    /// 信息类型，1文章，2企业，3百科
    pub info_type: Option<i32>,
    /// 搜索关键词
    pub key: Option<String>,
    /// 区域筛选
    pub region_id: Option<String>,
    /// 行业筛选
    pub column_id: Option<String>,
    /// 不包含的ID
    pub not_id: Option<i64>,
    /// 起始页码
    pub from_num: Option<i64>,
    /// 每页数量
    pub page_size: Option<i64>,
}

impl RelatedQueryRequest {
    pub fn new(from_num: Option<i64>,page_size: Option<i64>) -> Self {
        Self {
            info_type: None,
            key: None,
            region_id: None,
            column_id: None,
            not_id: None,
            from_num: Option::from(from_num.unwrap_or(0)),
            page_size: Option::from(page_size.unwrap_or(10)),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Keywords {
    pub name: Option<String>,
    pub age: Option<i32>,
}