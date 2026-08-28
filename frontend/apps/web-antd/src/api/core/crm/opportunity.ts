import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getOpportunityListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/opportunity/list', { params });
};
export const getOpportunityInfoApi = async (id: number) => {
  return requestClient.get('/api/system/opportunity/info', { params: { id } });
};
export const createOpportunityApi = async (param: any) => {
  return requestClient.post('/api/system/opportunity/save', param);
};
export const updateOpportunityApi = async (param: any) => {
  return requestClient.put('/api/system/opportunity/update', param);
};
export const deleteOpportunityApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/opportunity/bath_delete', {
    data: { ids },
  });
};

export const convertOpportunityToQuotationApi = async (id: number) => {
  return requestClient.post('/api/system/opportunity/convert_to_quotation', {
    id,
  });
};

export const convertOpportunityToOrderApi = async (id: number) => {
  return requestClient.post('/api/system/opportunity/convert_to_order', { id });
};

// 作废商机（原因必填，负责人/管理员可操作；终态 stage=5/6 禁止）
export const voidOpportunityApi = async (id: number, reason: string) => {
  return requestClient.post('/api/system/opportunity/void', { id, reason });
};

// 恢复已作废商机（仅管理员，回到作废前阶段）
export const recoverOpportunityApi = async (id: number) => {
  return requestClient.post('/api/system/opportunity/recover', { id });
};
