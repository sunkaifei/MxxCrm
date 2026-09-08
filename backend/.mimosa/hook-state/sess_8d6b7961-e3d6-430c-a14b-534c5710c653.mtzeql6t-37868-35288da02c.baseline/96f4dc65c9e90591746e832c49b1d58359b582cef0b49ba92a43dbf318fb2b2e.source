//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::modules::company::entity::code_rule;
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::DateTime;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

/// 段位类型枚举字符串
pub const SEG_COMPANY: &str = "company";
pub const SEG_BIZ_TYPE: &str = "biz_type";
pub const SEG_YEAR: &str = "year";
pub const SEG_DEPT: &str = "dept";
pub const SEG_SEQ: &str = "seq";
pub const SEG_VERSION: &str = "version";
pub const SEG_FIXED: &str = "fixed";
pub const SEG_DATE: &str = "date";

/// 年份来源：current=当前年 / business_date=业务日期年 / create_time=创建时间年
pub const YEAR_SRC_CURRENT: &str = "current";
pub const YEAR_SRC_BUSINESS_DATE: &str = "business_date";
pub const YEAR_SRC_CREATE_TIME: &str = "create_time";

/// 单个段位配置
/// type:
///   - company   公司简称（取自企业信息表）
///   - biz_type  业务类型编码（如 KH/HT/JS）
///   - year      年份，source 决定取值方式
///   - dept      部门编码（取自用户所属一级职能部门）
///   - seq        流水号
///   - version   版本号（V1/V2...）
///   - fixed     固定文本
///   - date      日期，format: yyyyMM / yyyyMMdd
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentConfig {
    /// 段位类型：company/biz_type/year/dept/seq/version/fixed/date
    #[serde(rename = "type")]
    pub type_: String,
    /// 段位值，type=fixed/company/biz_type 时使用
    pub value: Option<String>,
    /// 格式：type=year 时 yyyy/yy；type=date 时 yyyyMM/yyyydd
    pub format: Option<String>,
    /// 年份来源：type=year 时使用 current/business_date/create_time
    pub source: Option<String>,
    /// 流水号位数：type=seq 时使用，默认 4
    pub length: Option<i32>,
    /// 排序号
    pub sort: i32,
}

/// 编号规则 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeRuleVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub module_code: Option<String>,
    pub module_name: Option<String>,
    pub rule_name: Option<String>,
    pub company_abbr: Option<String>,
    pub dept_code: Option<String>,
    pub biz_type_code: Option<String>,
    pub separator: Option<String>,
    pub segments: Option<Vec<SegmentConfig>>,
    pub seq_length: Option<i16>,
    pub enabled: Option<i16>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

/// 新增/修改编号规则请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeRuleSaveReq {
    pub id: Option<i64>,
    pub module_code: String,
    pub module_name: String,
    pub rule_name: Option<String>,
    /// 公司简称（管理员自定义，不再从企业信息表读取）
    pub company_abbr: Option<String>,
    /// 部门编码（管理员自定义）
    pub dept_code: Option<String>,
    pub biz_type_code: Option<String>,
    pub separator: Option<String>,
    pub segments: Vec<SegmentConfig>,
    pub seq_length: Option<i16>,
    pub enabled: Option<i16>,
    pub remark: Option<String>,
}

/// 生成编号请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeReq {
    pub module_code: String,
    /// 部门编码（可选，无部门传 None）
    pub dept_code: Option<String>,
    /// 业务日期，用于补录历史文件时取正确年份（type=year 且 source=business_date 时使用）
    pub business_date: Option<String>,
    /// 前一个版本号，用于修订时自动递增（如传入 "V1" 则返回 "V2"）
    /// 仅在规则配置了 version 段位时有效，传 None 或空字符串表示首次创建
    pub previous_version: Option<String>,
}

/// 预览编号请求（不入库，仅根据段位配置生成示例）
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCodeReq {
    /// 模块编码（已存在规则时使用，从规则取 company_abbr 等）
    pub module_code: Option<String>,
    /// 段位配置（编辑时直接预览，未保存的段位）
    pub segments: Vec<SegmentConfig>,
    /// 公司简称（编辑时若用户改动可传）
    pub company_abbr: Option<String>,
    pub biz_type_code: Option<String>,
    pub separator: Option<String>,
    pub seq_length: Option<i16>,
    /// 部门编码
    pub dept_code: Option<String>,
    /// 业务日期
    pub business_date: Option<String>,
    /// 是否模拟流水号（预览时显示 0001）
    pub mock_seq: Option<bool>,
}

/// 一键更新请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRegenerateReq {
    /// 要更新的模块编码列表
    pub module_codes: Vec<String>,
    /// 要更新的年份（仅客户模块使用，按 create_time 年份归类）
    pub years: Option<Vec<i32>>,
}

/// 一键更新进度 VO
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchRegenerateProgressVO {
    pub total: i64,
    pub done: i64,
    pub current_module: Option<String>,
    /// running / success / failed
    pub status: String,
    pub message: Option<String>,
}

/// 一键更新结果 VO
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchRegenerateResultVO {
    /// 受影响的模块列表
    pub modules: Vec<String>,
    /// 总更新记录数
    pub total_affected: i64,
}

impl From<code_rule::Model> for CodeRuleVO {
    fn from(m: code_rule::Model) -> Self {
        let segments = m.segments.as_ref().and_then(|j| {
            serde_json::from_value::<Vec<SegmentConfig>>(j.clone()).ok()
        });
        CodeRuleVO {
            id: Option::from(m.id),
            module_code: m.module_code,
            module_name: m.module_name,
            rule_name: m.rule_name,
            company_abbr: m.company_abbr,
            dept_code: m.dept_code,
            biz_type_code: m.biz_type_code,
            separator: m.separator,
            segments,
            seq_length: m.seq_length,
            enabled: m.enabled,
            remark: m.remark,
            create_time: m.create_time,
            update_time: m.update_time,
        }
    }
}

/// 把 SaveReq 转换为 ActiveModel（用于新增/修改）
pub fn save_req_to_active(req: &CodeRuleSaveReq) -> code_rule::ActiveModel {
    let segments_json = serde_json::to_value(&req.segments).unwrap_or(serde_json::Value::Array(vec![]));
    code_rule::ActiveModel {
        module_code: Set(Some(req.module_code.clone())),
        module_name: Set(Some(req.module_name.clone())),
        rule_name: Set(req.rule_name.clone()),
        company_abbr: Set(req.company_abbr.clone()),
        dept_code: Set(req.dept_code.clone()),
        biz_type_code: Set(req.biz_type_code.clone()),
        separator: Set(req.separator.clone().or_else(|| Some("-".to_string()))),
        segments: Set(Some(segments_json)),
        seq_length: Set(req.seq_length.or(Some(4))),
        enabled: Set(req.enabled.or(Some(1))),
        remark: Set(req.remark.clone()),
        ..Default::default()
    }
}
