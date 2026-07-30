import { requestClient } from '#/api/request';

// ============ 待办中心 API ============

/** 待办汇总统计 */
export const getTodoSummaryApi = async () =>
  requestClient.get('/api/system/todo/summary');

/** 审批待办列表 */
export const getTodoApprovalListApi = async (params?: any) =>
  requestClient.get('/api/system/todo/approval', { params });

/** 跟进待办列表 */
export const getTodoFollowUpListApi = async (params?: any) =>
  requestClient.get('/api/system/todo/follow-up', { params });

/** 待回款提醒列表 */
export const getTodoPaymentListApi = async (params?: any) =>
  requestClient.get('/api/system/todo/payment', { params });

/** 合同到期提醒列表 */
export const getTodoContractListApi = async (params?: any) =>
  requestClient.get('/api/system/todo/contract', { params });

/** 停滞商机列表 */
export const getTodoOpportunityListApi = async (params?: any) =>
  requestClient.get('/api/system/todo/opportunity', { params });
