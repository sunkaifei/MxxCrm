import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getAlertListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/alert/list', { params });
};

export const getAlertRuleListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/alert/rule/list', { params });
};

export const getAlertRuleInfoApi = async (id: number) => {
  return requestClient.get('/api/system/alert/rule/info', { params: { id } });
};

export const createAlertRuleApi = async (data: any) => {
  return requestClient.post('/api/system/alert/rule/save', data);
};

export const updateAlertRuleApi = async (data: any) => {
  return requestClient.put('/api/system/alert/rule/update', data);
};

export const deleteAlertRuleApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/alert/rule/batch_delete', {
    data: { ids },
  });
};
