import { requestClient } from '#/api/request';

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
  return requestClient.delete('/api/system/sale/payment/bath_delete', {
    data: { ids },
  });
};
