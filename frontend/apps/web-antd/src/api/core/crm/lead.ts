import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getLeadListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/lead/list', { params });
};
export const getLeadInfoApi = async (id: number) => {
  return requestClient.get('/api/system/lead/info', { params: { id } });
};
export const createLeadApi = async (param: any) => {
  return requestClient.post('/api/system/lead/save', param);
};
export const updateLeadApi = async (param: any) => {
  return requestClient.put('/api/system/lead/update', param);
};
export const deleteLeadApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/lead/bath_delete', {
    data: { ids },
  });
};
export const updateLeadStatusApi = async (
  id: number,
  status: number | string,
) => {
  return requestClient.put('/api/system/lead/update-status', { id, status });
};
// 退回线索池（原因类型必选；类型为"其他"时需补充说明，后端校验）
export const addLeadToPoolApi = async (param: {
  id: number;
  reason?: string;
  reasonType: number;
}) => {
  return requestClient.put('/api/system/lead/add-to-pool', param);
};
export const claimLeadApi = async (id: number) => {
  return requestClient.put('/api/system/lead/claim', { id });
};
export const convertLeadToCustomerApi = async (id: number) => {
  return requestClient.post('/api/system/lead/convert-to-customer', { id });
};
export const saveFollowupApi = async (params: any) => {
  return requestClient.post('/api/system/followup/save', params);
};
