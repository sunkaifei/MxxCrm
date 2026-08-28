import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getCustomerPoolListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/customer-pool/list', { params });
};

export const claimCustomerApi = async (id: number) => {
  return requestClient.put('/api/system/customer/claim', null, {
    params: { id },
  });
};

// 退回公海（原因类型必选；类型为"其他"时需补充说明，后端校验）
export const addCustomerToPoolApi = async (param: {
  id: number;
  reason?: string;
  reasonType: number;
}) => {
  return requestClient.put('/api/system/customer/add-to-pool', param);
};
