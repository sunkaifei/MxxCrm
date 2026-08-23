import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getPaymentListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/payment/list', { params });
};
export const getPaymentInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/payment/info', { params: { id } });
};
export const createPaymentApi = async (param: any) => {
  return requestClient.post('/api/system/sale/payment/save', param);
};
export const updatePaymentApi = async (param: any) => {
  return requestClient.put('/api/system/sale/payment/update', param);
};
export const deletePaymentApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/payment/bath_delete', { ids });
};

// 确认回款：status→2，联动订单 paid_amount
export const confirmPaymentApi = async (id: number) => {
  return requestClient.post('/api/system/sale/payment/confirm', { id });
};

// 按回款计划登记回款并核销（业务人员在回款计划上直接操作）
export const registerPaymentByPlanApi = async (data: {
  amount: number;
  paymentDate?: string;
  paymentMethod?: number;
  payer?: string;
  payerAccount?: string;
  planId: number;
  remark?: string;
}) => {
  return requestClient.post('/api/system/sale/payment/register', data);
};

// 驳回回款：status→3
export const rejectPaymentApi = async (id: number) => {
  return requestClient.post('/api/system/sale/payment/reject', { id });
};

// 核销：一笔回款核销多个计划
export const applyPaymentApi = async (data: {
  applications: Array<{ applyAmount: number; planId: null | number }>;
  paymentId: number;
}) => {
  return requestClient.post('/api/system/sale/payment/application/apply', data);
};

// 取消核销
export const cancelPaymentApplicationApi = async (id: number) => {
  return requestClient.post('/api/system/sale/payment/application/cancel', {
    id,
  });
};

// 查询回款未核销金额及可核销计划
export const getPaymentUnappliedApi = async (id: number) => {
  return requestClient.get('/api/system/sale/payment/unapplied', {
    params: { id },
  });
};

// 查询回款核销明细
export const getPaymentApplicationsApi = async (paymentId: number) => {
  return requestClient.get('/api/system/sale/payment/application/list', {
    params: { id: paymentId },
  });
};
