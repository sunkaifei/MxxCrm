import { requestClient } from '#/api/request';

export const getPaymentListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/payment/list', { params });
};
export const getPaymentInfoApi = async (id: number) => {
  return requestClient.get('/api/system/payment/info', { params: { id } });
};
export const createPaymentApi = async (param: any) => {
  return requestClient.post('/api/system/payment/save', param);
};
export const updatePaymentApi = async (param: any) => {
  return requestClient.put('/api/system/payment/update', param);
};
export const deletePaymentApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/payment/bath_delete', {
    data: { ids },
  });
};
