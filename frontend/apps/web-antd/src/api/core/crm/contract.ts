import { requestClient } from '#/api/request';

export const getContractListApi = async (params?: PageParams) => {
  return requestClient.get('/api/crm/contract/list', { params });
};
export const getContractInfoApi = async (id: number) => {
  return requestClient.get('/api/crm/contract/info', { params: { id } });
};
export const createContractApi = async (param: any) => {
  return requestClient.post('/api/crm/contract/save', param);
};
export const updateContractApi = async (param: any) => {
  return requestClient.put('/api/crm/contract/update', param);
};
export const deleteContractApi = async (ids: number[]) => {
  return requestClient.delete('/api/crm/contract/bath_delete', {
    data: { ids },
  });
};
