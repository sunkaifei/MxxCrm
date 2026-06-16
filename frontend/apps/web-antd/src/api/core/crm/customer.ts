import { requestClient } from '#/api/request';

export const getCustomerListApi = async (params?: PageParams) => {
  return requestClient.get('/api/crm/customer/list', { params });
};
export const getCustomerInfoApi = async (id: number) => {
  return requestClient.get('/api/crm/customer/info', { params: { id } });
};
export const createCustomerApi = async (param: any) => {
  return requestClient.post('/api/crm/customer/save', param);
};
export const updateCustomerApi = async (param: any) => {
  return requestClient.put('/api/crm/customer/update', param);
};
export const deleteCustomerApi = async (ids: number[]) => {
  return requestClient.delete('/api/crm/customer/bath_delete', {
    data: { ids },
  });
};
