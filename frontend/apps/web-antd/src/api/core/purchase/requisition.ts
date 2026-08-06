import { requestClient } from '#/api/request';

export const getRequisitionListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/purchase/requisition/list', { params });
};
export const getRequisitionInfoApi = async (id: number) => {
  return requestClient.get('/api/system/purchase/requisition/info', { params: { id } });
};
export const createRequisitionApi = async (param: any) => {
  return requestClient.post('/api/system/purchase/requisition/save', param);
};
export const updateRequisitionApi = async (param: any) => {
  return requestClient.put('/api/system/purchase/requisition/update', param);
};
export const deleteRequisitionApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/purchase/requisition/bath_delete', {
    data: { ids },
  });
};

// ========== 审批流程 ==========

export const submitRequisitionApi = async (id: number) => {
  return requestClient.post('/api/system/purchase/requisition/submit', { id });
};
export const approveRequisitionApi = async (id: number, reason?: string) => {
  return requestClient.post('/api/system/purchase/requisition/approve', { id, reason });
};
export const rejectRequisitionApi = async (id: number, reason?: string) => {
  return requestClient.post('/api/system/purchase/requisition/reject', { id, reason });
};
export const withdrawRequisitionApi = async (id: number) => {
  return requestClient.post('/api/system/purchase/requisition/withdraw', { id });
};

// ========== 业务操作 ==========

export const convertToPoApi = async (data: { prIds: number[]; supplierId: number }) => {
  return requestClient.post('/api/system/purchase/requisition/convert', data);
};