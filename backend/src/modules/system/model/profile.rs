//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

/// 本人档案聚合 VO
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyProfileVO {
    pub basic: BasicBlock,
    pub employ: EmployBlock,
    pub id_card: IdCardBlock,
    pub bank: BankBlock,
    pub visibility: VisibilityConfig,
    pub resume: Vec<ResumeItem>,
    pub emergency_contacts: Vec<EmergencyContactItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicBlock {
    pub nick_name: Option<String>,
    pub gender: Option<i32>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub intro: Option<String>,
    pub mobile_masked: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployBlock {
    pub user_name: Option<String>,
    pub dept_names: Vec<String>,
    pub post_names: Vec<String>,
    pub direct_manager_id: Option<i64>,
    pub direct_manager_name: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub probation_months: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdCardBlock {
    /// 脱敏身份证，未填返回 None
    pub masked: Option<String>,
    pub locked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankBlock {
    pub masked_card_no: Option<String>,
    pub bank_name: Option<String>,
    pub masked_account_name: Option<String>,
    pub locked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityConfig {
    pub show_mobile: bool,
    pub show_wechat: bool,
    pub show_skills: bool,
    pub show_birthday: bool,
}

/// 本人基本信息更新（白名单外字段一律忽略）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicUpdateRequest {
    pub nick_name: Option<String>,
    pub gender: Option<i32>,
    pub email: Option<String>,
    pub intro: Option<String>,
    pub visibility: Option<VisibilityConfig>,
    pub wechat: Option<String>,
    pub birthday: Option<NaiveDate>,
}

/// 身份证首填
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdCardRequest {
    pub id_card_no: String,
}

/// 工资卡首填
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankRequest {
    pub bank_card_no: String,
    pub bank_name: Option<String>,
    pub bank_account_name: Option<String>,
}

/// 简历条目
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeItem {
    pub id: Option<i64>,
    pub kind: i32,
    pub title: Option<String>,
    pub org: Option<String>,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    pub remark: Option<String>,
    #[serde(default)]
    pub is_public: Option<i32>,
}

/// 紧急联系人
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyContactItem {
    pub id: Option<i64>,
    pub name: String,
    pub relation: Option<String>,
    pub mobile: String,
    #[serde(default)]
    pub sort: Option<i32>,
}

/// 同事名片 VO（可选字段按公开开关决定是否返回）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardVO {
    pub admin_id: i64,
    pub nick_name: Option<String>,
    pub avatar: Option<String>,
    pub dept_names: Vec<String>,
    pub post_names: Vec<String>,
    pub direct_manager_name: Option<String>,
    pub email: Option<String>,
    pub intro: Option<String>,
    pub mobile: Option<String>,
    pub wechat: Option<String>,
    pub skills: Vec<String>,
    pub birthday: Option<NaiveDate>,
    pub online: bool,
}

/// HR 档案列表 VO
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HrArchiveListVO {
    pub id: i64,
    pub user_name: Option<String>,
    pub nick_name: Option<String>,
    pub avatar: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub dept_names: Vec<String>,
    pub post_names: Vec<String>,
    pub hire_date: Option<NaiveDate>,
    pub id_locked: bool,
    pub bank_locked: bool,
    /// 1=普通员工 2=超管（档案页展示全部账号，用 tag 区分）
    pub user_type: Option<i32>,
    /// 状态：1=启用 0=禁用
    pub status: Option<i32>,
    /// 六项完善明细（true=已填写）
    pub id_filled: bool,
    pub bank_filled: bool,
    pub email_filled: bool,
    pub hire_filled: bool,
    pub resume_filled: bool,
    pub contact_filled: bool,
    pub resume_count: i64,
    pub contact_count: i64,
    /// 档案完整度：0-100（身份证/银行卡/简历/紧急联系人/邮箱/入职日期 六项占比）
    pub completeness: i32,
}

/// HR 档案详情 VO（含完整敏感字段）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HrArchiveDetailVO {
    pub id: i64,
    pub user_name: Option<String>,
    pub nick_name: Option<String>,
    pub gender: Option<i32>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub avatar: Option<String>,
    pub dept_ids: Vec<i64>,
    pub dept_names: Vec<String>,
    pub post_ids: Vec<i64>,
    pub post_names: Vec<String>,
    pub direct_manager_id: Option<i64>,
    pub direct_manager_name: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub probation_months: Option<i32>,
    pub probation_ratio: Option<f64>,
    pub id_card_no: Option<String>,
    pub id_locked: bool,
    pub bank_card_no: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_name: Option<String>,
    pub bank_locked: bool,
    pub status: Option<i32>,
    pub resume: Vec<ResumeItem>,
    pub emergency_contacts: Vec<EmergencyContactItem>,
}

/// HR 代改请求（全字段可选，仅更新出现的字段）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrArchiveUpdateRequest {
    pub nick_name: Option<String>,
    pub gender: Option<i32>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub probation_months: Option<i32>,
    pub probation_ratio: Option<f64>,
    pub direct_manager_id: Option<i64>,
    pub bank_card_no: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_name: Option<String>,
    pub dept_ids: Option<Vec<i64>>,
    pub post_ids: Option<Vec<i64>>,
    pub status: Option<i32>,
}

/// 解锁请求
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequest {
    /// id_card / bank
    pub field: String,
}

/// 变更日志 VO
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileLogVO {
    pub id: i64,
    pub admin_id: i64,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub operate_type: Option<i32>,
    pub operator_name: Option<String>,
    /// 格式 HH:mm:ss（日期在列表按天分组由前端处理）
    pub create_time: Option<NaiveTime>,
    pub create_date: Option<NaiveDate>,
}

/// 日志查询参数
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileLogQuery {
    pub admin_id: Option<i64>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
