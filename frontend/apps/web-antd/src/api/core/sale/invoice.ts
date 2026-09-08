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

/** 回收站恢复（仅创建人） */
export const restoreInvoiceApi = async (id: number) => {
  return requestClient.put(`/api/system/sale/invoice/restore/${id}`);
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

// 发票文件列表（entity_type=invoice 附件元信息，最新在前）。
// 申请人/财务均由此取得附件 id，再经 downloadFileApi 下载或预览——
// /attachment/by-entity 需 attachment:list 权限，业务人员会被守卫拦截。
export const getInvoiceFilesApi = async (id: number) => {
  return requestClient.get(`/api/system/sale/invoice/${id}/invoice-file`);
};

// T2.2 发票-回款勾稽：合同维度"开票 vs 回款"对照（合同额/计划/已回款/已开票/差额）
export const getInvoiceReconciliationApi = async (contractId: number) => {
  return requestClient.get('/api/system/sale/invoice/reconciliation', {
    params: { contract_id: contractId },
  });
};
