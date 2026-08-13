import { requestClient } from '#/api/request';

// 服务权益列表
export const getEntitlementListApi = async (params: any) => {
  return requestClient.get('/api/system/sale/entitlement/list', { params });
};

// 权益详情
export const getEntitlementInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/entitlement/info', {
    params: { id },
  });
};

// 新建权益
export const createEntitlementApi = async (data: any) => {
  return requestClient.post('/api/system/sale/entitlement/save', data);
};

// 修改权益状态
export const updateEntitlementApi = async (data: any) => {
  return requestClient.put('/api/system/sale/entitlement/update', data);
};

// 续约
export const renewEntitlementApi = async (data: any) => {
  return requestClient.post('/api/system/sale/entitlement/renew', data);
};

// 批量删除权益
export const deleteEntitlementApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/sale/entitlement/delete', { data: ids });
};

// 按客户查询
export const getEntitlementByCustomerApi = async (customerId: number) => {
  return requestClient.get('/api/system/sale/entitlement/by-customer', {
    params: { id: customerId },
  });
};
