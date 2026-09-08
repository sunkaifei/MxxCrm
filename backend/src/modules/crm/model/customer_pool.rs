//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use sea_orm::prelude::{DateTime};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{
    deserialize_string_or_num_vec_to_i64_vec, deserialize_string_or_number_to_i64, deserialize_string_to_u64,
    serialize_option_u64_to_string,
};

/// 客户公海池保存请求（新增/更新共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolSaveRequest {
    /// 池ID（更新时必传）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 池名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 池管理员ID（可看全池含已分配客户）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_admin_id: Option<i64>,
}

/// 客户公海池分页查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomerPoolListQuery {
    /// 页码
    pub page: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（按池名称模糊）
    pub keywords: Option<String>,
    /// 状态（1=启用 2=停用）
    pub status: Option<i16>,
}

/// 客户公海池列表 VO（含成员数与在池客户数统计）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolListVO {
    /// 池ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 池名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 状态（1=启用 2=停用）
    pub status: Option<i16>,
    /// 排序
    pub sort: Option<i32>,
    /// 默认池标识（1=默认池）
    pub is_default: Option<i16>,
    /// 池管理员ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_admin_id: Option<i64>,
    /// 池管理员姓名
    pub pool_admin_name: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 成员数
    pub member_count: Option<i64>,
    /// 在池客户数（未分配）
    pub customer_count: Option<i64>,
    /// 池配置（列表即带出，供抽屉回显）
    pub config: Option<CustomerPoolConfigVO>,
}

/// 客户公海池 ID 查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolIdQuery {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
}

/// 客户公海池状态更新参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolStatusUpdateQuery {
    /// 池ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 目标状态（1=启用 2=停用）
    pub status: Option<i16>,
}

/// 工作台可见池 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MyCustomerPoolVO {
    /// 池ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 池名称
    pub name: Option<String>,
    /// 默认池标识
    pub is_default: Option<i16>,
    /// 领取模式（1=自主领取 2=申领需审批 3=停用领取仅分配）
    pub claim_mode: Option<i16>,
    /// 自动分配开关
    pub auto_assign_enabled: Option<i16>,
    /// 脱敏字段
    pub mask_fields: Option<Vec<String>>,
    /// 我是否为该池管理员
    pub is_manager: Option<bool>,
    /// 在池客户数
    pub customer_count: Option<i64>,
}

/// 客户公海池配置 VO（22 项池级参数）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolConfigVO {
    /// 池ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    /// 默认保护期：未跟进自动回收天数（0=关闭该规则）
    pub recycle_days: Option<i32>,
    /// 最长保护期（NULL=无硬上限）
    pub max_recycle_days: Option<i32>,
    /// 未新增商机回收天数（NULL=关闭该规则）
    pub no_opportunity_days: Option<i32>,
    /// 未签合同回收天数（NULL=关闭该规则）
    pub no_contract_days: Option<i32>,
    /// 回收前 N 天提醒（0=不提醒）
    pub reminder_days: Option<i32>,
    /// 退回后冷却期天数（0=不启用）
    pub cool_down_days: Option<i32>,
    /// 每人每日领取上限（0=不限制）
    pub claim_daily_limit: Option<i32>,
    /// 私海保有量上限（0=不限制）
    pub hold_limit: Option<i32>,
    /// 领取模式（1=自主领取 2=申领需审批 3=停用领取仅分配）
    pub claim_mode: Option<i16>,
    /// 自动分配开关（0=关 1=开）
    pub auto_assign_enabled: Option<i16>,
    /// 自动分配模式（1=轮询 2=权重）
    pub auto_assign_mode: Option<i16>,
    /// 脱敏字段数组
    pub mask_fields: Option<Vec<String>>,
    /// 成单保护开关（0=关 1=开）
    pub deal_protect_enabled: Option<i16>,
    /// 有效合同状态集合（默认 [2,3,4]）
    pub deal_status: Option<Vec<i16>>,
    /// 商机豁免开关（0=关 1=开）
    pub opportunity_protect_enabled: Option<i16>,
    /// 自建客户占保有量（0=不占 1=占）
    pub include_self_built: Option<i16>,
    /// 自建客户按池规则回收（0=仅统计不强制 1=按池规则）
    pub self_built_recycle_enabled: Option<i16>,
    /// 工作台隐藏已领取客户（0=否 1=是）
    pub hide_claimed: Option<i16>,
    /// 工作台隐藏已转化客户（0=否 1=是）
    pub hide_converted: Option<i16>,
    /// 普通成员可查看详情（0=不可 1=可）
    pub allow_detail: Option<i16>,
    /// 退回去向（1=原池 2=选择分组 3=指定分组）
    pub release_back_to: Option<i16>,
    /// 连续被动退回/回收达该值触发冻结（0=不冻结）
    pub freeze_release_count: Option<i32>,
    /// 首次触达 SLA（领取后 N 小时内需首跟）
    pub first_touch_hours: Option<i32>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

/// 客户公海池配置保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolConfigSaveRequest {
    /// 池ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    pub recycle_days: Option<i32>,
    pub max_recycle_days: Option<i32>,
    pub no_opportunity_days: Option<i32>,
    pub no_contract_days: Option<i32>,
    pub reminder_days: Option<i32>,
    pub cool_down_days: Option<i32>,
    pub claim_daily_limit: Option<i32>,
    pub hold_limit: Option<i32>,
    /// 领取模式
    pub claim_mode: Option<i16>,
    pub auto_assign_enabled: Option<i16>,
    pub auto_assign_mode: Option<i16>,
    /// 脱敏字段数组（空数组=不脱敏）
    pub mask_fields: Option<Vec<String>>,
    pub deal_protect_enabled: Option<i16>,
    /// 有效合同状态集合（空数组视为非法）
    pub deal_status: Option<Vec<i16>>,
    pub opportunity_protect_enabled: Option<i16>,
    pub include_self_built: Option<i16>,
    pub self_built_recycle_enabled: Option<i16>,
    pub hide_claimed: Option<i16>,
    pub hide_converted: Option<i16>,
    pub allow_detail: Option<i16>,
    pub release_back_to: Option<i16>,
    pub freeze_release_count: Option<i32>,
    pub first_touch_hours: Option<i32>,
}

/// 池成员 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolMemberVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 成员姓名
    pub user_name: Option<String>,
    /// 成员类型（1=池管理员 2=普通成员）
    pub member_type: Option<i16>,
    /// 特殊员工回收天数覆盖（NULL=用池级）
    pub recycle_days_override: Option<i32>,
    /// 特殊员工领取上限覆盖（NULL=用池级）
    pub claim_daily_limit_override: Option<i32>,
    pub create_time: Option<DateTime>,
}

/// 池成员条目（批量保存用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolMemberItem {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub user_id: i64,
    /// 1=池管理员 2=普通成员
    pub member_type: i16,
    /// 特殊员工回收天数覆盖
    pub recycle_days_override: Option<i32>,
    /// 特殊员工领取上限覆盖
    pub claim_daily_limit_override: Option<i32>,
}

/// 池成员批量保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolMemberSaveRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    pub members: Option<Vec<CustomerPoolMemberItem>>,
}

/// 池成员批量移除请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolMemberDeleteRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    pub user_ids: Option<Vec<i64>>,
}

/// 候选成员简单 VO（指派/自动分配/负载均衡展示）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolMemberSimpleVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub member_type: Option<i16>,
    /// 特殊员工回收天数覆盖
    pub recycle_days_override: Option<i32>,
    /// 特殊员工领取上限覆盖
    pub claim_daily_limit_override: Option<i32>,
    /// 当前私海客户数
    pub holding_count: Option<i64>,
}

/// 客户公海工作台查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolWorkbenchQuery {
    /// 页码
    pub page: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 池ID（不传=跨池视图）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    /// 关键词（公司名/简称/个人姓名模糊）
    pub keywords: Option<String>,
}

/// 工作台在池客户行 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolWorkbenchCustomerVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub person_name: Option<String>,
    pub industry: Option<i32>,
    pub level: Option<i32>,
    pub source: Option<i32>,
    pub wechat: Option<String>,
    pub personal_mobile: Option<String>,
    pub personal_email: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    /// 最近进入公海时间
    pub entered_pool_at: Option<DateTime>,
    /// 累计被动退回/回收次数
    pub release_count: Option<i32>,
}

/// 工作台分页 VO（含页级操作权限与配置回显）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolWorkbenchPageVO {
    /// 数据总条数
    pub total: i64,
    pub items: Vec<CustomerPoolWorkbenchCustomerVO>,
    /// 领取模式（单池视图回显；1=自主领取 2=申领需审批 3=停用领取仅分配）
    pub claim_mode: Option<i16>,
    /// 脱敏字段配置回显
    pub mask_fields: Vec<String>,
    /// 退回去向（1=原池 2=选择分组 3=指定分组）
    pub release_back_to: i16,
    /// 普通成员可查看详情（池配置）
    pub allow_detail: bool,
    /// 当前用户是否可查看详情
    pub can_view_detail: bool,
    /// 当前用户是否可领取
    pub can_claim: bool,
}

/// 批量操作单条结果 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolOpResultVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub success: bool,
    pub message: String,
}

/// 工作台批量领取请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerWorkbenchClaimRequest {
    #[serde(default, deserialize_with = "deserialize_string_or_num_vec_to_i64_vec")]
    pub customer_ids: Vec<i64>,
}

/// 工作台申领请求（claim_mode=2 池）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerWorkbenchApplyRequest {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub customer_id: i64,
    /// 申领理由
    pub reason: Option<String>,
}

/// 工作台退回公海请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerWorkbenchReleaseRequest {
    #[serde(default, deserialize_with = "deserialize_string_or_num_vec_to_i64_vec")]
    pub customer_ids: Vec<i64>,
    /// 退回原因类型：1=跟进无回应 2=客户无意向 3=客户信息无效 4=换业务方向 9=其他
    pub reason_type: Option<i16>,
    /// 退回补充说明（原因类型为"其他"时必填）
    pub reason: Option<String>,
    /// 目标池（release_back_to=2/3 时必传）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub target_pool_id: Option<i64>,
}

/// 客户延期（回收顺延）申请请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerWorkbenchRetainRequest {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub customer_id: i64,
    /// 延期天数（0 < 天数 ≤ 池回收天数）
    pub extend_days: i32,
    /// 申请理由
    pub reason: Option<String>,
}

/// 客户公海审批分页查询参数（申领/延期共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolAuditPageQuery {
    /// 页码
    pub page: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 池ID（延期申请列表无池维度，忽略该参数）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    /// 状态（1=待审批 2=通过 3=拒绝，默认查待审批）
    pub status: Option<i16>,
}

/// 公海客户申领审批 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerPoolApplyVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    pub pool_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 申请人姓名
    pub user_name: Option<String>,
    /// 申领理由
    pub reason: Option<String>,
    /// 状态（1=待审批 2=通过 3=拒绝）
    pub status: Option<i16>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub audit_by: Option<i64>,
    pub audit_by_name: Option<String>,
    pub audit_remark: Option<String>,
    pub audit_time: Option<DateTime>,
    pub create_time: Option<DateTime>,
}

/// 客户延期审批 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerRetainApplyVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    /// 客户当前所在池（经客户行联查）
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    pub pool_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 申请人（现负责人）姓名
    pub user_name: Option<String>,
    /// 延期天数
    pub extend_days: Option<i32>,
    /// 申请理由
    pub reason: Option<String>,
    /// 状态（1=待审批 2=通过 3=拒绝）
    pub status: Option<i16>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub audit_by: Option<i64>,
    pub audit_by_name: Option<String>,
    pub audit_remark: Option<String>,
    pub audit_time: Option<DateTime>,
    pub create_time: Option<DateTime>,
}

/// 审批操作请求（申领/延期共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolAuditRequest {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub id: i64,
    /// true=通过 false=拒绝
    pub pass: bool,
    /// 审批备注
    pub remark: Option<String>,
}

/// 客户归属轨迹 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerAssignHistoryVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub admin_id: Option<i64>,
    /// 负责人姓名
    pub admin_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    pub pool_name: Option<String>,
    /// 操作类型：1=领取 2=退回 3=管理员分配 4=转移 5=自动分配 6=自动回收 7=管理员收回 8=延期获批 9=冻结 10=解冻
    pub action_type: Option<i16>,
    pub action_label: String,
    /// 开始负责时间
    pub start_time: Option<DateTime>,
    /// 结束负责时间（NULL=正在负责）
    pub end_time: Option<DateTime>,
    pub remark: Option<String>,
    pub reason_type: Option<i16>,
    pub reason: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub operated_by: Option<i64>,
    pub operated_by_name: Option<String>,
    pub create_time: Option<DateTime>,
}

/// 自动分配规则保存请求（规则 + 条目整体保存，方案 §5.3）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AssignRuleSaveRequest {
    /// 规则ID（编辑时传，新增为空）
    pub id: Option<i64>,
    /// 池ID
    pub pool_id: i64,
    /// 规则名称
    pub name: String,
    /// 触发事件（1=进池；V1 引擎仅消费进池触发，2/3 预留）
    pub trigger_event: Option<i16>,
    /// 优先级（越小越先评估）
    pub priority: Option<i32>,
    /// 启用状态（1=启用 0=停用）
    pub enabled: Option<i16>,
    /// 规则条目（保存时整体覆盖）
    pub entries: Vec<AssignRuleEntrySaveItem>,
}

/// 自动分配规则条目保存项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AssignRuleEntrySaveItem {
    /// 条目ID（编辑时传，新增为空）
    pub id: Option<i64>,
    /// 筛选条件（[{field,op,value}] AND 组合；空=无条件全命中）
    pub conditions: Option<serde_json::Value>,
    /// 分配目标类型（1=用户 2=职位 3=群组 4=池成员；3 群组 V1 不支持）
    pub target_type: Option<i16>,
    /// 分配目标 ID（用户/职位；池成员时可为空）
    pub target_ids: Option<Vec<i64>>,
    /// 分配方式（1=指定 2=轮询 3=权重）
    pub mode: Option<i16>,
    /// 权重分配（{userId: weight}；mode=3 生效）
    pub weight: Option<serde_json::Value>,
    /// 条目排序（升序，首个匹配生效）
    pub sort: Option<i32>,
}

/// 自动分配规则列表查询
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AssignRuleListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    /// 池ID过滤
    pub pool_id: Option<i64>,
    /// 启用状态过滤
    pub enabled: Option<i16>,
    /// 名称关键词
    pub keywords: Option<String>,
}

/// 自动分配规则（含条目）VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AssignRuleVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    pub pool_name: Option<String>,
    pub name: Option<String>,
    pub trigger_event: Option<i16>,
    pub priority: Option<i32>,
    pub enabled: Option<i16>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub entries: Vec<AssignRuleEntryVO>,
}

/// 自动分配规则条目 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AssignRuleEntryVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub rule_id: Option<i64>,
    pub conditions: Option<serde_json::Value>,
    pub target_type: Option<i16>,
    pub target_ids: Option<Vec<i64>>,
    pub mode: Option<i16>,
    pub weight: Option<serde_json::Value>,
    pub sort: Option<i32>,
    pub create_time: Option<DateTime>,
}

/// 自动分配规则 ID 请求（删除）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AssignRuleIdRequest {
    #[serde(deserialize_with = "deserialize_string_or_number_to_i64")]
    pub id: i64,
}

/// 手动触发一轮自动分配请求（不传池=全部启用池）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AssignRuleRunRequest {
    pub pool_id: Option<i64>,
}
