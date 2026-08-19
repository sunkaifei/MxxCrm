import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

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

// 检查客户名称是否已存在（按 customerType 区分字段：1=企业按 companyName，2=个人按 personName）
export const checkCustomerNameApi = async (params: {
  customerType: number;
  excludeId?: number;
  name: string;
}) => {
  return requestClient.get('/api/system/customer/check-name', { params });
};

// 获取客户下的联系人列表
export const getCustomerContactsApi = async (id: number) => {
  return requestClient.get('/api/system/customer/contacts', { params: { id } });
};

// 获取客户分配历史（负责人时间轴）
export const getCustomerAssignHistoryApi = async (id: number) => {
  return requestClient.get('/api/system/customer/assign-history', {
    params: { id },
  });
};
