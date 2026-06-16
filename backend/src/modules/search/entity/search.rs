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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Company {
    /// ID
    pub id: Option<i64>,
    /// 企业所属分类
    pub column_ids: Option<String>,
    /// 短网址
    pub short_url: Option<String>,
    /// 企业名称
    pub company_name: Option<String>,
    /// 曾用名
    pub name_before: Option<String>,
    /// 企业说明
    pub content: Option<String>,
    /// 企业简称
    pub short_name: Option<String>,
    /// 经营模式
    pub mode: Option<String>,
    /// 注册资本
    pub capital: Option<String>,
    /// 注册资本货币
    pub money: Option<String>,
    /// 排序权重
    pub weight: Option<f64>,
    /// 归属地区json
    pub come_region: Option<String>,
    /// 所在省份
    pub province: Option<i32>,
    /// 所在城市
    pub city: Option<i32>,
    /// 县级和区
    pub county: Option<i32>,
    /// 形象图片
    pub image_url: Option<String>,
    /// 工商注册号
    pub register_no: Option<String>,
    /// 发证机关
    pub authority: Option<String>,
    /// 法人
    pub corporation: Option<String>,
    /// 经度
    pub longitude: Option<String>,
    /// 纬度
    pub latitude: Option<String>,
    /// 经营范围
    pub business: Option<String>,
    /// 经营地址
    pub business_address: Option<String>,
    /// 公司官网
    pub homepage: Option<String>,
    /// 成立时间
    pub create_date: Option<String>,
    /// 经营期限开始时间
    pub start_date: Option<String>,
    /// 营业结束期限
    pub end_date: Option<String>,
    /// 营业期限：0否，1长期
    pub long_term: Option<bool>,
    /// 企业类型
    pub company_type: Option<i32>,
    /// 经营状态
    pub operation_status: Option<i32>,
    /// 添加时间
    pub create_time: Option<DateTime>,
    /// 最新更新时间
    pub update_time: Option<DateTime>,
    /// 0未审核，1审核，2未通过
    pub status: Option<i32>,
    /// 拒绝原因
    pub reason: Option<String>,
    /// 0不删除1删除
    pub deleted: Option<i32>,
}



