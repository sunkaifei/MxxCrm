import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

/**
 * 公海线索列表（v3.0：pool_id 过滤 + 返回 claimMode/maskFields 池级渲染参数）
 * 返回 PoolLeadPageVO { claimMode, maskFields, total, items }
 */
export const getLeadPoolListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/lead-pool/list', { params });
};
export const getLeadPoolInfoApi = async (id: number | string) => {
  return requestClient.get('/api/system/lead-pool/info', { params: { id } });
};
export const deleteLeadPoolApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/lead-pool/bath_delete', {
    data: { ids },
  });
};

/** 批量领取（claim_mode=1 自主领取） */
export const claimPoolLeadsApi = async (ids: (number | string)[]) => {
  return requestClient.post('/api/system/lead-pool/claim', { ids });
};

/** 批量申领（claim_mode=2 申领需审批） */
export const applyPoolLeadsApi = async (
  ids: (number | string)[],
  reason?: string,
) => {
  return requestClient.post('/api/system/lead-pool/apply', { ids, reason });
};

/** 批量分配（管理员；单人逐条 / 多人按顺序均摊） */
export const assignPoolLeadsApi = async (
  ids: (number | string)[],
  toUserIds: (number | string)[],
) => {
  return requestClient.post('/api/system/lead-pool/assign', {
    ids,
    toUserIds,
  });
};

/** 批量退回公海（默认回 source_pool_id，可指定目标池） */
export const releasePoolLeadsApi = async (param: {
  ids: (number | string)[];
  reason?: string;
  reasonType: number;
  targetPoolId?: number | string;
}) => {
  return requestClient.post('/api/system/lead-pool/release', param);
};

/** 管理员批量收回（私海 → 本池） */
export const recyclePoolLeadsApi = async (
  ids: (number | string)[],
  remark?: string,
) => {
  return requestClient.post('/api/system/lead-pool/recycle', { ids, remark });
};

/** 线索归属历史（时间轴） */
export const getAssignHistoryApi = async (leadId: number | string) => {
  return requestClient.get('/api/system/lead-pool/assign-history', {
    params: { leadId },
  });
};

/** 批量提交延期保留申请（私海） */
export const submitRetainApplyApi = async (
  ids: (number | string)[],
  days: number,
  reason?: string,
) => {
  return requestClient.post('/api/system/lead-pool/retain/apply', {
    days,
    ids,
    reason,
  });
};

/** 延期审批分页（待办/已办） */
export const getRetainPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/lead-pool/retain/page', { params });
};

/** 延期审批（通过=延长保留天数） */
export const auditRetainApi = async (
  id: number | string,
  approved: boolean,
  remark?: string,
) => {
  return requestClient.post('/api/system/lead-pool/retain/audit', {
    approved,
    id,
    remark,
  });
};

/** 申领审批分页（待办/已办） */
export const getPoolApplyPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/lead-pool/apply/page', { params });
};

/** 申领审批（通过=执行领取到名下） */
export const auditPoolApplyApi = async (
  id: number | string,
  approved: boolean,
  remark?: string,
) => {
  return requestClient.post('/api/system/lead-pool/apply/audit', {
    approved,
    id,
    remark,
  });
};
