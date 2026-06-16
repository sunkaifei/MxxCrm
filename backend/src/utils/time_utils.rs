//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::str::FromStr;
use std::time::SystemTime;

use chrono::{prelude::*};
use chrono_tz::Asia::Shanghai;
use chrono_tz::{Tz};
use crate::core::errors::error::{Error, Result };
use crate::core::kit::config;

/// get current time stamp
#[inline]
pub fn timestamp() -> i64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => { n.as_secs() as i64 }
        Err(_) => { 0 }
    }
}

/// 要求输入: 2019-11-11 10:10:10
#[inline]
pub fn from_str(datetime_str: &str) -> DateTime<Local> {
    let datetime = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
        .expect("Failed to parse datetime");
    Local.from_local_datetime(&datetime).unwrap()
}

/// 当前的时间字符串
#[inline]
pub fn to_string() -> String {
    let local: DateTime<Local> = Local::now();
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 格式化时间
#[inline]
pub fn format(format_str: &str) -> String {
    let local: DateTime<Local> = Local::now();
    local.format(format_str).to_string()
}

/// 当前时间
#[inline]
pub fn now() -> DateTime<Tz> {
    let tz = timezone();
    Utc::now().with_timezone(&tz)
}

/// 得到时分秒
#[inline]
pub fn time() -> (u32, u32, u32) {
    let now = now();
    (now.hour(), now.minute(), now.second())
}

/// 得到年月日
#[inline]
pub fn date() -> (u32, u32, u32) {
    let now = now();
    (now.year() as u32, now.month() as u32, now.day() as u32)
}

// 时区
pub fn timezone() -> Tz {
    let timezone = &config::section::<String>("time", "timezone", "Asia/Shanghai".to_string());
    match Tz::from_str(timezone) {
        Ok(tz) => tz,
        Err(e) => {
            log::warn!("时区配置解析失败: {}, 使用默认时区 Asia/Shanghai", e);
            chrono_tz::Asia::Shanghai
        }
    }
}

/// 获取当前日期
/// 格式：2020/01/01
pub fn current_date() -> String {
    // 获取当前本地时间
    let tz = timezone();
    let now = Utc::now().with_timezone(&tz);
    // 格式化时间为字符串
    let date_str = now.format("%Y/%m/%d").to_string();
    date_str
}

pub fn current_date_time() -> String {
    // 获取当前本地时间
    let tz = timezone();
    let now = Utc::now().with_timezone(&tz);
    // 格式化时间为字符串
    let date_time_str = now.format("%Y/%m/%d %H:%M:%S").to_string();
    date_time_str
}

/// #格式化NaiveDateTime时间字符串
/// * `datetime` 要转换的NaiveDateTime时间
pub fn format_datetime(datetime: Option<NaiveDateTime>) -> Result<String> {
    match datetime {
        Some(dt) => Ok(dt.format("%Y/%m/%d").to_string()),
        None => Err(Error::from("NaiveDateTime is None")),
    }
}


// 解析时间
pub fn parse(d: &str) -> DateTime<Tz> {
    let tz = timezone();

    let date = NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S").unwrap_or_default();

    tz.from_utc_datetime(&date)
}

// 解析时间戳
pub fn from_timestamp(t: i64) -> DateTime<Tz> {
    let tz = timezone();
    let date = DateTime::from_timestamp(t, 0).unwrap_or_default();
    tz.from_utc_datetime(&date.naive_utc())
}


#[inline]
pub fn time_difference(input_time: &str) -> String {
    match NaiveDateTime::parse_from_str(input_time, "%Y-%m-%d %H:%M:%S") {
        Ok(dt) => {
            let parsed_time = dt.and_local_timezone(Shanghai).unwrap();
            let current_time_shanghai = Local::now().with_timezone(&Shanghai);
            let duration = current_time_shanghai.signed_duration_since(parsed_time);
            if duration.num_seconds() < 60 {
                "刚刚".to_string()
            } else if duration.num_hours() < 1 {
                format!("{} 分钟前", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                format!("{} 小时前", duration.num_hours())
            } else if duration.num_days() < 30 {
                format!("{} 天前", duration.num_days())
            } else if duration.num_days() < 365 {
                let months = (duration.num_days() as f64 + 14.5).floor() / 30.0; // 更准确地计算月份
                format!("{} 个月前", months.round() as i32)
            } else {
                parsed_time.format("%Y-%m-%d %H:%M:%S").to_string()
            }
        },
        Err(e) => {
            eprintln!("Error parsing datetime: {:?}", e);
            String::from("时间格式错误")
        }
    }
}


/// 将日期字符串从 "2025-03-03 23:22:25" 格式转换为 "2025/03/03" 格式
pub fn convert_date_format(date_str: &str) -> Result<String> {
    // 定义输入日期时间格式
    let input_format = "%Y-%m-%d %H:%M:%S";
    // 定义输出日期格式
    let output_format = "%Y/%m/%d";

    // 解析输入字符串为 NaiveDateTime
    let naive_datetime = NaiveDateTime::parse_from_str(date_str, input_format)?;

    // 格式化为输出字符串
    Ok(naive_datetime.format(output_format).to_string())
}

/// 计算从给定日期到当前日期的天数差
/// * `date` 要计算的日期（NaiveDate类型）
/// * 返回值：Some(天数) 或 None（日期为空时返回None，日期在未来时返回Some(0)）
pub fn days_since(date: Option<NaiveDate>) -> Option<i64> {
    date.map(|d| {
        let today = Local::now().date_naive();
        let days = (today - d).num_days();
        // 如果日期在未来，返回0
        if days < 0 { 0 } else { days }
    })
}

/// 计算年龄描述（超过一岁显示几岁多少天，不满一岁显示多少天）
/// * `birthday` 出生日期（NaiveDate类型）
/// * 返回值：年龄描述字符串，如 "2岁15天" 或 "120天"
pub fn age_description(birthday: Option<NaiveDate>) -> Option<String> {
    birthday.map(|b| {
        let today = Local::now().date_naive();
        let days = (today - b).num_days();
        
        if days < 0 {
            return "未出生".to_string();
        }
        
        let years = days / 365;
        let remaining_days = days % 365;
        
        if years >= 1 {
            format!("{}岁{}天", years, remaining_days)
        } else {
            format!("{}天", days)
        }
    })
}