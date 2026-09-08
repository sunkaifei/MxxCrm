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
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 公海池保存请求（新增/更新共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolSaveRequest {
    /// 池ID（更新时必传）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 池名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序
    pub sort: Option<i32>,
}

/// 公海池分页查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PoolListQuery {
    /// 页码
    pub page: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（按池名称模糊）
    pub keywords: Option<String>,
    /// 状态（1=启用 2=停用）
    pub status: Option<i16>,
}

/// 公海池列表 VO（含成员数与在池线索数统计）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolListVO {
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
    /// 默认池标识
    pub is_default: Option<i16>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 成员数
    pub member_count: Option<i64>,
    /// 在池线索数（未分配）
    pub lead_count: Option<i64>,
    /// 池配置（列表即带出，供抽屉回显）
    pub config: Option<PoolConfigVO>,
}

/// 公海池配置 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolConfigVO {
    /// 池ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    /// 未跟进自动回收天数
    pub recycle_days: Option<i32>,
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
    /// 自动分配模式（1=轮询 2=负载均衡）
    pub auto_assign_mode: Option<i16>,
    /// 脱敏字段数组
    pub mask_fields: Option<Vec<String>>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

/// 公海池配置保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolConfigSaveRequest {
    /// 池ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    pub recycle_days: Option<i32>,
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
}

/// 公海池线索分页查询参数（公海优化 §5.3：pool_id 过滤 + §5.9 脱敏）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PoolLeadPageQuery {
    /// 页码
    pub page: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 池ID（为空时返回我可见全部池的线索）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    /// 关键词（公司名称）
    pub keywords: Option<String>,
    /// 线索状态
    pub status: Option<i32>,
    /// 线索等级
    pub level: Option<String>,
    /// 线索来源
    pub source: Option<String>,
    /// 联系人
    pub contact_name: Option<String>,
    /// 手机号
    pub mobile: Option<String>,
    /// 行业
    pub industry: Option<i32>,
}

/// 公海池线索分页返回（含池级渲染参数：领取模式 + 脱敏字段，公海优化 §5.3/§5.9）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolLeadPageVO {
    /// 领取模式（1=自主领取 2=申领需审批 3=停用领取仅分配）
    pub claim_mode: Option<i16>,
    /// 脱敏字段配置
    pub mask_fields: Option<Vec<String>>,
    /// 数据总条数
    pub total: i64,
    /// 线索列表
    pub items: Vec<crate::modules::crm::model::lead::LeadListVO>,
}

/// 池成员 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolMemberVO {
    /// 主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 池ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    /// 用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 用户名
    pub user_name: Option<String>,
    /// 成员类型（1=池管理员 2=普通成员）
    pub member_type: Option<i16>,
    /// 加入时间
    pub create_time: Option<DateTime>,
}

/// 池成员添加请求（批量，含成员类型）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolMemberSaveRequest {
    /// 池ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    /// 成员列表
    pub members: Option<Vec<PoolMemberItem>>,
}

/// 单个成员项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolMemberItem {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub user_id: Option<i64>,
    /// 1=池管理员 2=普通成员
    pub member_type: Option<i16>,
}

/// 池成员移除请求（批量）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolMemberDeleteRequest {
    /// 池ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    /// 用户ID列表
    pub user_ids: Option<Vec<Option<String>>>,
}

/// 池ID查询参数（成员列表/配置详情等 GET 接口）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolIdQuery {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
}

/// 池状态更新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PoolStatusUpdateQuery {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 目标状态（1=启用 2=停用）
    pub status: Option<i16>,
}

/// 公海工作台可见池 VO（我的池∪我管理的池；带领取模式与脱敏配置供前端渲染）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MyPoolVO {
    /// 池ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 池名称
    pub name: Option<String>,
    /// 默认池标识
    pub is_default: Option<i16>,
    /// 状态（1=启用 2=停用）
    pub status: Option<i16>,
    /// 领取模式
    pub claim_mode: Option<i16>,
    /// 自动分配开关
    pub auto_assign_enabled: Option<i16>,
    /// 脱敏字段
    pub mask_fields: Option<Vec<String>>,
    /// 当前用户是否该池管理员
    pub is_manager: Option<bool>,
    /// 在池未分配线索数
    pub lead_count: Option<i64>,
}

/// 可选成员简要 VO（指派/自动分配候选）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolMemberSimpleVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub member_type: Option<i16>,
    /// 当前私海线索数（负载均衡/保有量展示）
    pub holding_count: Option<i64>,
}

/// 批量领取请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolClaimRequest {
    /// 线索ID列表
    pub ids: Option<Vec<Option<String>>>,
}

/// 归属历史查询参数（GET /lead-pool/assign-history?lead_id=）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadHistoryQuery {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub lead_id: Option<i64>,
}

/// 单条线索操作结果（批量领取/退回等部分成功明细）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolOpResultVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub lead_id: Option<i64>,
    pub success: bool,
    pub message: Option<String>,
}

/// 批量分配请求（单人多条=逐条；多人=按顺序均摊）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolAssignRequest {
    /// 线索ID列表
    pub ids: Option<Vec<Option<String>>>,
    /// 单个目标负责人（与 to_user_ids 二选一）
    pub to_user_id: Option<String>,
    /// 多目标负责人列表（按顺序均摊）
    pub to_user_ids: Option<Vec<Option<String>>>,
}

/// 批量退回请求（默认回 source_pool_id，可指定 target_pool_id）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolReleaseBatchRequest {
    /// 线索ID列表
    pub ids: Option<Vec<Option<String>>>,
    /// 退回原因类型（1=无意向 2=无法联系 3=重复线索 4=区域不符 其他=其他）
    pub reason_type: Option<i16>,
    /// 补充说明
    pub reason: Option<String>,
    /// 目标池ID（不传回 source_pool_id）
    pub target_pool_id: Option<String>,
}

/// 管理员收回请求（私海→本池）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolRecycleRequest {
    /// 线索ID列表
    pub ids: Option<Vec<Option<String>>>,
    /// 收回备注
    pub remark: Option<String>,
}

/// 线索归属历史VO（时间轴展示）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeadAssignHistoryVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub lead_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub admin_id: Option<i64>,
    /// 负责人名称
    pub admin_name: Option<String>,
    /// 操作类型：1=领取 2=退回 3=管理员分配 4=转移 5=自动回收 6=管理员收回 7=申领通过
    pub action_type: Option<i16>,
    /// 操作类型文案
    pub action_label: Option<String>,
    pub start_time: Option<DateTime>,
    pub end_time: Option<DateTime>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    /// 公海池名称
    pub pool_name: Option<String>,
    pub remark: Option<String>,
    pub reason_type: Option<i16>,
    pub reason: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub operated_by: Option<i64>,
    /// 操作人名称
    pub operated_by_name: Option<String>,
    pub create_time: Option<DateTime>,
}

/// 批量申领请求（claim_mode=2 的池）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolApplyRequest {
    /// 线索ID列表
    pub ids: Option<Vec<Option<String>>>,
    /// 申领理由
    pub reason: Option<String>,
}

/// 申领审批请求（通过=执行领取，拒绝=留痕）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolApplyAuditRequest {
    /// 申请ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 是否通过
    pub approved: Option<bool>,
    /// 审批备注
    pub remark: Option<String>,
}

/// 申领待办/已办分页查询
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolApplyPageQuery {
    /// 池ID（可选）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    /// 状态（1=待审批 2=通过 3=拒绝；不传=全部）
    pub status: Option<i16>,
    /// 申请人ID（可选）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub user_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 申领申请VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeadPoolApplyVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub pool_id: Option<i64>,
    /// 公海池名称
    pub pool_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub lead_id: Option<i64>,
    /// 线索名称
    pub lead_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 申请人名称
    pub user_name: Option<String>,
    /// 状态（1=待审批 2=通过 3=拒绝）
    pub status: Option<i16>,
    pub reason: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub audit_by: Option<i64>,
    /// 审批人名称
    pub audit_by_name: Option<String>,
    pub audit_time: Option<DateTime>,
    pub audit_remark: Option<String>,
    pub create_time: Option<DateTime>,
}

/// 延期申请请求（回收时点顺延）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolRetainApplyRequest {
    /// 线索ID列表
    pub ids: Option<Vec<Option<String>>>,
    /// 延期天数（1~365）
    pub days: Option<i32>,
    /// 申请理由
    pub reason: Option<String>,
}

/// 延期审批请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolRetainAuditRequest {
    /// 申请ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 是否通过
    pub approved: Option<bool>,
    /// 审批备注
    pub remark: Option<String>,
}

/// 延期待办/已办分页查询
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LeadPoolRetainPageQuery {
    /// 线索ID（可选）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub lead_id: Option<i64>,
    /// 状态（1=待审批 2=通过 3=拒绝；不传=全部）
    pub status: Option<i16>,
    /// 申请人ID（可选）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub user_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 延期申请VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeadPoolRetainVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub lead_id: Option<i64>,
    /// 线索名称
    pub lead_name: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 申请人名称
    pub user_name: Option<String>,
    /// 延期天数
    pub extend_days: Option<i32>,
    /// 状态（1=待审批 2=通过 3=拒绝）
    pub status: Option<i16>,
    pub reason: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub audit_by: Option<i64>,
    /// 审批人名称
    pub audit_by_name: Option<String>,
    pub audit_time: Option<DateTime>,
    pub audit_remark: Option<String>,
    pub create_time: Option<DateTime>,
}
