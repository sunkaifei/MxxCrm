//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 访问来源
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_statistics_source")]
pub struct Model {
    /// id
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 统计日期（格式:yyyy-MM-dd）
    pub statistics_day: Option<String>,
    /// 来源网站类型 （1-搜索引擎  2-外部链接  3-直接访问）
    pub sorce_url_type: Option<i32>,
    /// 是否新客户 （0-否   1-是）
    pub is_new_visitor: Option<i32>,
    /// 访客设备类型（1-计算机   2-移动设备）
    pub visitor_device_type: Option<i32>,
    /// 来源域名
    pub source_domain: Option<String>,
    /// 来源外部链接网站地址或网站名称（如：百度/http://www.maxxshop.com）
    pub sorce_url: Option<String>,
    /// 搜索引擎
    pub engine_name: Option<String>,
    /// 浏览量
    pub pvs: Option<i32>,
    /// 访客数
    pub uvs: Option<i32>,
    /// ip数
    pub ips: Option<i32>,
    /// 总访问时长(单位：秒)
    pub access_houre_long: Option<i32>,
    /// 只访问一次页面的访问次数
    pub only_one_pv: Option<i32>,
    /// 时间段
    pub statistics_hour: Option<i32>,
    /// 删除标识
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}