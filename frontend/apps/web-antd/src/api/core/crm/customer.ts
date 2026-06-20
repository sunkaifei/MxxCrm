import { requestClient } from '#/api/request';

export const getCustomerListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer/list', { params });
};
export const getCustomerInfoApi = async (id: number) => {
  return requestClient.get('/api/system/customer/info', { params: { id } });
};
export const createCustomerApi = async (param: any) => {
  return requestClient.post('/api/system/customer/save', param);
};
export const updateCustomerApi = async (param: any) => {
  return requestClient.put('/api/system/customer/update', param);
};
export const deleteCustomerApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/customer/bath_delete', {
    data: { ids },
  });
};

// 获取客户下的联系人列表
export const getCustomerContactsApi = async (id: number) => {
  return requestClient.get('/api/system/customer/contacts', { params: { id } });
};