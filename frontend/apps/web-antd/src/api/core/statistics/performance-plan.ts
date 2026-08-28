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

/** 查询计划列表 */
export const getPlanListApi = async (params?: {
  employeeId?: number;
  /** 待我审批模式：true=查询当前用户作为审批人的待审计划 */
  pendingMyApproval?: boolean;
  status?: number;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/plan/list', {
    params,
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
