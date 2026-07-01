import { requestClient } from '#/api/request';

export const getCustomerPoolListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/list', { params });
};

export const claimCustomerApi = async (id: number) => {
  return requestClient.put('/api/system/customer/claim', null, {
    params: { id },
  });
};

export const addCustomerToPoolApi = async (id: number) => {
  return requestClient.put('/api/system/customer/add-to-pool', null, {
    params: { id },
  });
};
