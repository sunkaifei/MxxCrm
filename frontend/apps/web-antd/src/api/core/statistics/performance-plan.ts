import { requestClient } from '#/api/request';

/** 创建草稿计划（employeeId 为代建目标员工：集中管理入口，缺省=为自己申报） */
export const createPlanApi = async (params: {
  employeeId?: number;
  monthlyTargets: Array<{
    contractTargetAmount?: number;
    contractTargetCount?: number;
    month: number;
    paymentTargetAmount?: number;
  }>;
  year: number;
}) => {
  return requestClient.post(
    '/api/system/statistics/performance/plan/create',
    params,
  );
};

/** 提交计划（草稿→待审批） */
export const submitPlanApi = async (planId: number) => {
  return requestClient.post('/api/system/statistics/performance/plan/submit', {
    planId,
  });
};

/** 审批通过 */
export const approvePlanApi = async (planId: number, reason?: string) => {
  return requestClient.post('/api/system/statistics/performance/plan/approve', {
    planId,
    reason,
  });
};

/** 驳回 */
export const rejectPlanApi = async (planId: number, reason?: string) => {
  return requestClient.post('/api/system/statistics/performance/plan/reject', {
    planId,
    reason,
  });
};

/** 申请修改 */
export const modifyPlanApi = async (params: {
  monthlyTargets: Array<{
    contractTargetAmount?: number;
    contractTargetCount?: number;
    month: number;
    paymentTargetAmount?: number;
  }>;
  planId: number;
  reason: string;
}) => {
  return requestClient.post(
    '/api/system/statistics/performance/plan/modify',
    params,
  );
};

/** 更新草稿/驳回状态的月度目标（不走审批流） */
export const updatePlanTargetsApi = async (params: {
  monthlyTargets: Array<{
    contractTargetAmount?: number;
    contractTargetCount?: number;
    month: number;
    paymentTargetAmount?: number;
  }>;
  planId: number;
}) => {
  return requestClient.post(
    '/api/system/statistics/performance/plan/update-targets',
    params,
  );
};

/** 计划列表项（摘要，含完成度） */
export interface PlanListItem {
  id?: number;
  employeeId?: number;
  employeeName?: string;
  deptName?: string;
  year?: number;
  status?: number;
  version?: number;
  totalContractTarget?: number | string;
  totalPaymentTarget?: number | string;
  totalContractActual?: number | string;
  totalPaymentActual?: number | string;
  completionRate?: number | string;
  paymentCompletionRate?: number | string;
  applyReason?: string;
  createTime?: string;
  updateTime?: string;
  currentApproverId?: number;
  currentApproverName?: string;
  approvalLevel?: number;
  totalLevels?: number;
  submitTime?: string;
  isFrozen?: number;
}

/** 计划分页列表结果 */
export interface PlanListPageResult {
  total: number;
  page: number;
  pageSize: number;
  items: PlanListItem[];
}

/** 查询计划列表（分页 + 排序白名单） */
export const getPlanListApi = async (params?: {
  /** 按部门筛选 */
  deptId?: number;
  employeeId?: number;
  /** 员工姓名关键词（昵称/用户名模糊匹配） */
  keyword?: string;
  /** 排序字段：submitTime/totalContractTarget/completionRate/paymentCompletionRate/employeeName/status */
  orderBy?: string;
  /** 排序方向，缺省 desc */
  orderDir?: 'asc' | 'desc';
  /** 页码，从 1 开始 */
  page?: number;
  /** 每页条数，缺省 20，最大 200 */
  pageSize?: number;
  /** 待我审批模式：true=查询当前用户作为审批人的待审计划 */
  pendingMyApproval?: boolean;
  status?: number;
  /** 状态多选筛选（团队列表状态复选） */
  statusList?: number[];
  year?: number;
}): Promise<PlanListPageResult> => {
  return requestClient.get('/api/system/statistics/performance/plan/list', {
    params,
    // 数组参数用重复键序列化（statusList=1&statusList=2），对齐后端 serde_urlencoded
    paramsSerializer: 'repeat',
  });
};

/** 查询计划详情 */
export const getPlanDetailApi = async (planId: number) => {
  return requestClient.get('/api/system/statistics/performance/plan/detail', {
    params: { planId },
  });
};

/** 获取修改详情（编辑回显） */
export const getPlanModifyDetailApi = async (planId: number) => {
  return requestClient.get(
    '/api/system/statistics/performance/plan/modify-detail',
    { params: { planId } },
  );
};

/** 获取进度汇总（个人 + 团队，自下而上汇总） */
export const getPlanProgressSummaryApi = async (params: { year?: number }) => {
  return requestClient.get(
    '/api/system/statistics/performance/plan/progress-summary',
    { params },
  );
};

/** 年度计划覆盖度（集中管理视角：哪些员工还缺当年销售计划） */
export const getPlanCoverageApi = async (params: { year?: number }) => {
  return requestClient.get(
    '/api/system/statistics/performance/plan/coverage',
    { params },
  );
};

/** 单员工逐月"目标 vs 实际"钻取（团队列表行操作打开） */
export const getMonthlyCompareApi = async (params: {
  employeeId: number;
  year?: number;
}) => {
  return requestClient.get(
    '/api/system/statistics/performance/plan/monthly-compare',
    { params },
  );
};

/** 撤回计划（待审批→草稿，仅第一级未审可撤） */
export const withdrawPlanApi = async (planId: number) => {
  return requestClient.post('/api/system/statistics/performance/plan/withdraw', {
    planId,
  });
};

/** 站内催办：对未提交当年计划的员工发送提醒（manage 权限），返回催办人数 */
export const remindPlanApi = async (params: { year?: number }) => {
  return requestClient.post(
    '/api/system/statistics/performance/plan/remind',
    params,
  );
};
