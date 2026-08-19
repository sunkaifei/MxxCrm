import { requestClient } from '#/api/request';

// 注意：API 名称加 Finance 前缀，避免与 sale/payment.ts 的同名 API 在
// core/index.ts 的 `export *` 汇总时产生 star export 冲突。
export const getFinancePaymentListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/payment/list', { params });
};

export const getFinancePaymentDetailApi = async (id: number) => {
  return requestClient.get('/api/system/finance/payment/detail', {
    params: { id },
  });
};

export const applyFinancePaymentApi = async (data: any) => {
  return requestClient.post('/api/system/finance/payment/apply', data);
};

export const approveFinancePaymentApi = async (data: {
  approved: boolean;
  id: number;
  remark?: string;
}) => {
  return requestClient.post('/api/system/finance/payment/approve', data);
};

export const confirmFinancePaymentApi = async (data: {
  id: number;
  paymentDate: string;
}) => {
  return requestClient.post('/api/system/finance/payment/confirm', data);
};

export const cancelFinancePaymentApi = async (data: {
  id: number;
  remark: string;
}) => {
  return requestClient.post('/api/system/finance/payment/cancel', data);
};
