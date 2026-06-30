import { requestClient } from '#/api/request';

export const getPaymentListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/payment/list', { params });
};

export const getPaymentDetailApi = async (id: number) => {
  return requestClient.get('/api/system/finance/payment/detail', {
    params: { id },
  });
};

export const applyPaymentApi = async (data: any) => {
  return requestClient.post('/api/system/finance/payment/apply', data);
};

export const approvePaymentApi = async (data: {
  id: number;
  approved: boolean;
  remark?: string;
}) => {
  return requestClient.post('/api/system/finance/payment/approve', data);
};

export const confirmPaymentApi = async (data: {
  id: number;
  paymentDate: string;
}) => {
  return requestClient.post('/api/system/finance/payment/confirm', data);
};

export const cancelPaymentApi = async (data: { id: number; remark: string }) => {
  return requestClient.post('/api/system/finance/payment/cancel', data);
};
