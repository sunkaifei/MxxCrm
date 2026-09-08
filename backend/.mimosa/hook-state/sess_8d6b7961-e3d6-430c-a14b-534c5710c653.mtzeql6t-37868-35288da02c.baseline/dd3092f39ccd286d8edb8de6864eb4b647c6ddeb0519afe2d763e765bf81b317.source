//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::entity::prelude::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::statistics::entity::{access_record, access_record::Entity as AccessRecord};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordSaveRequest{
    // 是否登录访问（0-否  1-是）
    pub is_login: Option<i32>,
    // 登录用户id
    pub login_user_id: Option<i64>,
    // 登录用户名
    pub login_user_name: Option<String>,
    // 会话标识
    pub session_id: Option<String>,
    // cookie标识
    pub cookie_id: Option<String>,
    // 访问来源(1:PC  2:移动端H5  3:微信客户端H5 4:IOS 5:安卓 6:小程序)
    pub access_source_client: Option<i32>,
    // 访问网址
    pub access_url: Option<String>,
    // 来源网址
    pub source_url: Option<String>,
    // 来源域名
    pub source_domain: Option<String>,
    // 来源网站类型 （1-搜索引擎  2-外部链接  3-直接访问）
    pub source_url_type: Option<i32>,
    // 访客ip
    pub access_ip: Option<String>,
    // 访客设备系统（如：Win10 Mac10  Android8）
    pub access_device: Option<String>,
    // 访客浏览器类型
    pub access_browser: Option<String>,
    // 访客所属省份
    pub access_province: Option<String>,
    // 访客所属城市
    pub access_city: Option<String>,

    // 访客所属国家
    pub access_country: Option<String>,
    // 搜索名称
    pub engine_name: Option<String>,
    // 是否新访客（0:否   1:是）
    pub is_new_visitor: Option<i32>,
    // 设备类型
    pub device_type: Option<i32>,
}

impl From<RecordSaveRequest> for AccessRecordSaveDTO {
    fn from(s: RecordSaveRequest) -> Self {
        Self {
            id: None,
            is_login: s.is_login,
            login_user_id: s.login_user_id,
            login_user_name: s.login_user_name,
            session_id: s.session_id,
            cookie_id: s.cookie_id,
            access_source_client: s.access_source_client,
            access_url: s.access_url,
            source_url: s.source_url,
            source_domain: s.source_domain,
            source_url_type: s.source_url_type,
            access_ip: s.access_ip,
            access_device: s.access_device,
            access_browser: s.access_browser,
            access_province: s.access_province,
            access_city: s.access_city,
            access_country: s.access_country,
            engine_name: s.engine_name,
            is_new_visitor: s.is_new_visitor,
            device_type: s.device_type,
            create_time: None,
            deleted: None,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct AccessRecordSaveDTO {
    // id
    pub id: Option<i64>,
    // 是否登录访问（0-否  1-是）
    pub is_login: Option<i32>,
    // 登录用户id
    pub login_user_id: Option<i64>,
    // 登录用户名
    pub login_user_name: Option<String>,
    // 会话标识
    pub session_id: Option<String>,
    // cookie标识
    pub cookie_id: Option<String>,
    // 访问来源(1:PC  2:移动端H5  3:微信客户端H5 4:IOS 5:安卓 6:小程序)
    pub access_source_client: Option<i32>,
    // 访问网址
    pub access_url: Option<String>,
    // 来源网址
    pub source_url: Option<String>,
    // 来源域名
    pub source_domain: Option<String>,
    // 来源网站类型 （1-搜索引擎  2-外部链接  3-直接访问）
    pub source_url_type: Option<i32>,
    // 访客ip
    pub access_ip: Option<String>,
    // 访客设备系统（如：Win10 Mac10  Android8）
    pub access_device: Option<String>,
    // 访客浏览器类型
    pub access_browser: Option<String>,
    // 访客所属省份
    pub access_province: Option<String>,
    // 访客所属城市
    pub access_city: Option<String>,

    // 访客所属国家
    pub access_country: Option<String>,
    // 搜索名称
    pub engine_name: Option<String>,
    // 是否新访客（0:否   1:是）
    pub is_new_visitor: Option<i32>,
    // 设备类型
    pub device_type: Option<i32>,
    // 创建时间
    pub create_time: Option<DateTime>,
    // 删除标识
    pub deleted: Option<i32>,
}

/// 访问量统计视图对象
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct VisitStatsVO {
    /// 今日独立访客数 (UV)
    pub today_uv_count: Option<i64>,
    /// 累计独立访客数 (UV)
    pub total_uv_count: Option<i64>,
    /// 独立访客增长率
    pub uv_growth_rate: Option<Decimal>,
    /// 今日页面浏览量 (PV)
    pub today_pv_count: Option<i64>,
    /// 累计页面浏览量 (PV)
    pub total_pv_count: Option<i64>,
    /// 页面浏览量增长率
    pub pv_growth_rate: Option<Decimal>,
}

/// 访问趋势视图VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct VisitTrendVO {
    /// 日期列表
    pub dates: Option<Vec<String>>,
    /// 浏览量(PV)
    pub pv_list: Option<Vec<i32>>,
    /// IP数
    pub ip_list: Option<Vec<i32>>,
}



pub struct AccessRecordModel;

impl AccessRecordModel {
    pub async fn insert(db: &DbConn, form_data: &AccessRecordSaveDTO)  -> Result<i64, DbErr> {
        let payload = access_record::ActiveModel {
            id:                     Set(form_data.id.unwrap_or_default().to_owned()),
            is_login:               Set(form_data.is_login.to_owned()),
            access_url:             Set(form_data.access_url.to_owned()),
            source_url:             Set(form_data.source_url.to_owned()),
            source_domain:          Set(form_data.source_domain.to_owned()),
            access_ip:              Set(form_data.access_ip.to_owned()),
            access_device:          Set(form_data.access_device.to_owned()),
            create_time:            Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        AccessRecord::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }
    
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<access_record::Model>, DbErr> {
        AccessRecord::find_by_id(id)
            .one(db)
            .await
    }
    
}