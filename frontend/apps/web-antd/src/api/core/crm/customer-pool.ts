import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

// ==================== 工作台（客户公海，v57 客户公海） ====================

/**
 * 我可操作的池列表（成员池 ∪ 管理的池；超管=全部）
 * 返回 MyCustomerPoolVO[]：id/name/isDefault/claimMode/autoAssignEnabled/maskFields/isManager/customerCount
 */
export const getWorkbenchPoolsApi = async () => {
  return requestClient.get('/api/system/customer-pool/workbench/pools');
};

/**
 * 池内客户分页（按 maskFields 脱敏 + hide_claimed/hide_converted 过滤）
 * 返回 CustomerPoolWorkbenchPageVO：total/items/claimMode/maskFields/releaseBackTo/allowDetail/canViewDetail/canClaim
 */
export const getWorkbenchCustomersApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/workbench/customers', {
    params,
  });
};

/** 领取（批量，逐条返回成功/失败明细） */
export const claimWorkbenchApi = async (customerIds: number[]) => {
  return requestClient.post('/api/system/customer-pool/workbench/claim', {
    customerIds,
  });
};

/** 提交申领（claim_mode=申领的池），reason 为申领理由 */
export const applyWorkbenchApi = async (param: {
  customerId: number;
  reason?: string;
}) => {
  return requestClient.post('/api/system/customer-pool/workbench/apply', param);
};

/** 退回公海（本人私海客户；原因类型必选，"其他"需补充说明；去向按池配置三选一） */
export const releaseWorkbenchApi = async (param: {
  customerIds: number[];
  reasonType?: number;
  reason?: string;
  targetPoolId?: number;
}) => {
  return requestClient.post('/api/system/customer-pool/workbench/release', param);
};

/** 申请延期（回收顺延，受最长保护期封顶） */
export const retainApplyWorkbenchApi = async (param: {
  customerId: number;
  extendDays: number;
  reason?: string;
}) => {
  return requestClient.post(
    '/api/system/customer-pool/workbench/retain-apply',
    param,
  );
};

/** 客户公海流水轨迹（时间轴，倒序） */
export const getWorkbenchTraceApi = async (customerId: number | string) => {
  return requestClient.get('/api/system/customer-pool/workbench/trace', {
    params: { customerId },
  });
};

// ==================== 管理端：池 CRUD / 成员 / 配置 ====================

/** 池分页（含成员数/在池客户数/配置回显） */
export const getPoolPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/page', { params });
};

/** 池详情 */
export const getPoolDetailApi = async (id: number | string) => {
  return requestClient.get('/api/system/customer-pool/detail', {
    params: { poolId: id },
  });
};

/** 新建池 */
export const createPoolApi = async (param: any) => {
  return requestClient.post('/api/system/customer-pool/create', param);
};

/** 更新池 */
export const updatePoolApi = async (param: any) => {
  return requestClient.post('/api/system/customer-pool/update', param);
};

/** 删除池（默认池禁删；需池内无客户） */
export const deletePoolApi = async (ids: number[]) => {
  return requestClient.post('/api/system/customer-pool/delete', { ids });
};

/** 启用/停用池（默认池禁停） */
export const updatePoolStatusApi = async (param: {
  id: number;
  status: number;
}) => {
  return requestClient.post('/api/system/customer-pool/status', param);
};

/** 池成员列表 */
export const getPoolMembersApi = async (poolId: number | string) => {
  return requestClient.get('/api/system/customer-pool/member/list', {
    params: { poolId },
  });
};

/** 添加池成员（members: userId+memberType(1=池管理员 2=普通成员)，可附特殊员工覆盖） */
export const addPoolMembersApi = async (param: {
  poolId: number;
  members: Array<{
    userId: number;
    memberType: number;
    recycleDaysOverride?: number;
    claimDailyLimitOverride?: number;
  }>;
}) => {
  return requestClient.post('/api/system/customer-pool/member/add', param);
};

/** 移除池成员 */
export const removePoolMembersApi = async (param: {
  poolId: number;
  userIds: number[];
}) => {
  return requestClient.post('/api/system/customer-pool/member/remove', param);
};

/** 池配置读取（22 项池级规则） */
export const getPoolConfigApi = async (poolId: number | string) => {
  return requestClient.get('/api/system/customer-pool/config', {
    params: { poolId },
  });
};

/** 池配置保存 */
export const savePoolConfigApi = async (param: any) => {
  return requestClient.post('/api/system/customer-pool/config/save', param);
};

// ==================== 管理端：指派 / 审批 / 冻结 ====================

/** 管理员指派（池内客户 → 指定池成员，批量逐条返回明细） */
export const assignPoolCustomersApi = async (param: {
  poolId: number;
  items: Array<{ customerId: number; toUserId: number }>;
}) => {
  return requestClient.post('/api/system/customer-pool/assign', param);
};

/** 申领审批分页（status 1=待审批 2=通过 3=拒绝） */
export const getCustomerPoolApplyAuditPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/apply/page', { params });
};

/** 申领审批（pass true=通过 false=拒绝） */
export const auditCustomerPoolApplyApi = async (param: {
  id: number;
  pass: boolean;
  remark?: string;
}) => {
  return requestClient.post('/api/system/customer-pool/apply/audit', param);
};

/** 延期审批分页 */
export const getCustomerPoolRetainAuditPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/retain/page', { params });
};

/** 延期审批 */
export const auditCustomerPoolRetainApi = async (param: {
  id: number;
  pass: boolean;
  remark?: string;
}) => {
  return requestClient.post('/api/system/customer-pool/retain/audit', param);
};

/** 冻结客户分页 */
export const getFrozenPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/freeze/page', { params });
};

/** 恢复冻结客户 */
export const restoreFrozenApi = async (customerId: number) => {
  return requestClient.post('/api/system/customer-pool/freeze/restore', {
    customerId,
  });
};

// ==================== 查重规则 ====================
export const getCustomerPoolDupRulePageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/dup-rule/page', { params });
};
export const saveCustomerPoolDupRuleApi = async (param: any) => {
  return requestClient.post('/api/system/customer-pool/dup-rule/save', param);
};
export const deleteCustomerPoolDupRuleApi = async (ids: number[]) => {
  return requestClient.post('/api/system/customer-pool/dup-rule/delete', { ids });
};
export const checkCustomerPoolDupApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/dup-rule/check', { params });
};

// ==================== 自动分配规则 ====================
export const getCustomerPoolAssignRulePageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/assign-rule/page', { params });
};
export const saveCustomerPoolAssignRuleApi = async (param: any) => {
  return requestClient.post('/api/system/customer-pool/assign-rule/save', param);
};
export const deleteCustomerPoolAssignRuleApi = async (ids: number[]) => {
  return requestClient.post('/api/system/customer-pool/assign-rule/delete', { ids });
};

// ==================== 考核统计 ====================
export const getCustomerPoolStatsMineApi = async () => {
  return requestClient.get('/api/system/customer-pool/stats/mine');
};
export const getCustomerPoolStatsAdminApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/stats', { params });
};

// ==================== 存量兼容接口（旧公海页使用，逐步下线） ====================

/** 旧公海客户列表（保留兼容，内部转发新服务） */
export const getCustomerPoolListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/list', { params });
};

/** 旧领取（保留兼容） */
export const claimCustomerApi = async (id: number) => {
  return requestClient.put('/api/system/customer/claim', null, {
    params: { id },
  });
};

// 退回公海（原因类型必选；类型为"其他"时需补充说明，后端校验）
export const addCustomerToPoolApi = async (param: {
  id: number;
  reason?: string;
  reasonType: number;
}) => {
  return requestClient.put('/api/system/customer/add-to-pool', param);
};
