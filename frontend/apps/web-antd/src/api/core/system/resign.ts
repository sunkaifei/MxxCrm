import { requestClient } from '#/api/request';

// ============ 离职流程（F4/F5）============

/** 离职申请（admin 端：HR/管理员代发起，JWT 取操作人） */
export const submitResignApplyApi = async (data: {
  adminId: number;
  resignType: number;
  resignDate?: string;
  reason?: string;
  transferToAdminId?: number;
}) => requestClient.post('/api/system/admin/resign/apply', data);

/** 交接单列表（system:resign:view） */
export const getResignListApi = async (params?: any) =>
  requestClient.get('/api/system/resign/list', { params });

/** 交接单详情（system:resign:view） */
export const getResignDetailApi = async (id: number) =>
  requestClient.get(`/api/system/resign/${id}`);

/** 交接项确认（assignee 本人或 HR 代确认） */
export const confirmResignItemApi = async (
  id: number,
  data: { itemId: number; isNa?: boolean; remark?: string },
) => requestClient.post(`/api/system/resign/${id}/confirm-item`, data);

/** 财务结算确认（system:resign:settle，结算即完全离职） */
export const settleResignApi = async (id: number, data: { leaveDate?: string }) =>
  requestClient.post(`/api/system/resign/${id}/settle`, data);

/** 离职中止（发起人本人或 HR，理由必填） */
export const abortResignApi = async (id: number, data: { reason: string }) =>
  requestClient.post(`/api/system/resign/${id}/abort`, data);

/** 交接确认人转派（system:resign:confirm） */
export const transferResignAssigneeApi = async (
  id: number,
  data: { itemId: number; newAssigneeId: number },
) => requestClient.post(`/api/system/resign/${id}/transfer-assignee`, data);
