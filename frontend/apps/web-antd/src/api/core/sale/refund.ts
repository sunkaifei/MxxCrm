import { requestClient } from '#/api/request';

// ==================== 退货单基础 CRUD ====================

export const getRefundListApi = async (params?: any) => {
  return requestClient.get('/api/system/sale/refund/list', { params });
};

export const getRefundInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/refund/info', { params: { id } });
};

export const createRefundApi = async (param: any) => {
  return requestClient.post('/api/system/sale/refund/save', param);
};

export const updateRefundApi = async (param: any) => {
  return requestClient.put('/api/system/sale/refund/update', param);
};

export const deleteRefundApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/refund/batch-delete', { ids });
};

// ==================== 退货流程操作 ====================

// 提交审批
export const submitRefundApi = async (refundId: number) => {
  return requestClient.post('/api/system/sale/refund/submit', { id: refundId });
};

// 审批通过
export const approveRefundApi = async (refundId: number, reason?: string) => {
  return requestClient.post('/api/system/sale/refund/approve', {
    refundId,
    reason,
  });
};

// 审批驳回
export const rejectRefundApi = async (refundId: number, reason?: string) => {
  return requestClient.post('/api/system/sale/refund/reject', {
    refundId,
    reason,
  });
};

// 仓库收货
export const receiveRefundApi = async (param: {
  refundId: number;
  logisticsNo?: string;
  logisticsCompany?: string;
}) => {
  return requestClient.post('/api/system/sale/refund/receive', param);
};

// 质检完成
export const qualityCheckRefundApi = async (param: {
  refundId: number;
  qualityCheckResult: number;
  qualityCheckRemark?: string;
}) => {
  return requestClient.post('/api/system/sale/refund/quality-check', param);
};

// 取消退货单
export const cancelRefundApi = async (refundId: number) => {
  return requestClient.post('/api/system/sale/refund/cancel', { id: refundId });
};

// 发起退款
export const createRefundPaymentApi = async (param: {
  refundId: number;
  paymentMethod?: number;
  paymentAmount?: number;
  paymentAccount?: string;
  transactionNo?: string;
  remark?: string;
}) => {
  return requestClient.post('/api/system/sale/refund/payment', param);
};
