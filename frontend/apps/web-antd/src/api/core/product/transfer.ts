import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getTransferListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/transfer/list', { params });
};

export const getTransferInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inventory/transfer/info', {
    params: { id },
  });
};

export const createTransferApi = async (data: any) => {
  return requestClient.post('/api/system/inventory/transfer/save', data);
};

export const transferOutboundApi = async (id: number) => {
  return requestClient.post('/api/system/inventory/transfer/outbound', {
    params: { id },
  });
};

export const transferInboundApi = async (id: number) => {
  return requestClient.post('/api/system/inventory/transfer/inbound', {
    params: { id },
  });
};

export const deleteTransferApi = async (ids: number[]) => {
  // 后端从 query string 读取 id=1,2,3（逗号分隔）
  return requestClient.delete('/api/system/inventory/transfer/batch_delete', {
    params: { id: ids.join(',') },
  });
};

/** 编辑调拨单（仅草稿/驳回态；后端 PUT /update） */
export const updateTransferApi = async (data: any) => {
  return requestClient.put('/api/system/inventory/transfer/update', data);
};

/** 点选方案之外: 调拨审批——提交审批（草稿/已驳回 → 审批中） */
export const submitTransferApprovalApi = async (id: number) => {
  return requestClient.put(`/api/system/inventory/transfer/submit_approval/${id}`);
};

/** 调拨审批: 审批通过（仓管/CEO 节点审批人） */
export const transferApproveApi = async (id: number, comment?: string) => {
  return requestClient.put(`/api/system/inventory/transfer/approve/${id}`, {
    comment,
  });
};

/** 调拨审批: 审批驳回 */
export const transferRejectApi = async (id: number, comment?: string) => {
  return requestClient.put(`/api/system/inventory/transfer/reject/${id}`, {
    comment,
  });
};

/** W12 调出方确认调整（申请模式首节点：可修改明细+留痕，随后通过节点） */
export const transferConfirmAdjustApi = async (
  id: number,
  data: { reason?: string; items: any[] },
) => {
  return requestClient.post(
    `/api/system/inventory/transfer/confirm_adjust/${id}`,
    data,
  );
};

/** W12 调拨修改留痕（详情"流转记录"） */
export const getTransferChangeLogsApi = async (id: number) => {
  return requestClient.get(`/api/system/inventory/transfer/${id}/change_logs`);
};

/** 回收站恢复 */
export const restoreTransferApi = async (id: number) => {
  return requestClient.put(`/api/system/transfer/restore/${id}`);
};

/** 回收站彻底删除（仅管理层/超管） */
export const purgeTransferApi = async (id: number) => {
  return requestClient.delete(`/api/system/transfer/purge/${id}`);
};
