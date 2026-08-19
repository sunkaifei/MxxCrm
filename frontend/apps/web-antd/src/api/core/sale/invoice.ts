import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getInvoiceListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/invoice/list', { params });
};
export const getInvoiceInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/invoice/info', { params: { id } });
};
export const createInvoiceApi = async (param: any) => {
  return requestClient.post('/api/system/sale/invoice/save', param);
};
export const updateInvoiceApi = async (param: any) => {
  return requestClient.put('/api/system/sale/invoice/update', param);
};
export const deleteInvoiceApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/invoice/batch-delete', { ids });
};

// ===== 审批流（invoice_approval：部门主管 → 财务审核） =====
export const submitInvoiceApi = async (id: number) => {
  return requestClient.post(`/api/system/sale/invoice/${id}/submit`);
};

export const approveInvoiceApi = async (id: number, reason?: string) => {
  return requestClient.post(`/api/system/sale/invoice/${id}/approve`, {
    reason,
  });
};

export const rejectInvoiceApi = async (id: number, reason?: string) => {
  return requestClient.post(`/api/system/sale/invoice/${id}/reject`, {
    reason,
  });
};

// 作废/红冲（业务动作，仅已开票 status=3，需理由；终态不可再变）
export const voidInvoiceApi = async (
  id: number,
  action: 1 | 2,
  reason: string,
) => {
  return requestClient.post('/api/system/sale/invoice/void', {
    id,
    action,
    reason,
  });
};

export const getInvoiceApprovalDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/sale/invoice/${id}/approval-detail`);
};

// 审批历史（完整链路追溯：全部审批实例 + 修改留痕，供"流转记录"聚合展示）
export const getInvoiceHistoryApi = async (id: number) => {
  return requestClient.get(`/api/system/sale/invoice/${id}/history`);
};

// 审批流预览（提交审核页展示将经过的审批环节）
export const getInvoiceApprovalPreviewApi = async () => {
  return requestClient.get('/api/system/sale/invoice/approval-preview');
};
