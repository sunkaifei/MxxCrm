import { requestClient } from '#/api/request';

// ============ Flow Management ============

export const getApprovalFlowListApi = async (params?: any) =>
  requestClient.get('/api/system/approval/flow/list', { params });

export const getApprovalFlowDetailApi = async (id: number) =>
  requestClient.get(`/api/system/approval/flow/detail/${id}`);

export const saveApprovalFlowApi = async (data: any) =>
  requestClient.post('/api/system/approval/flow/save', data);

export const toggleApprovalFlowApi = async (id: number) =>
  requestClient.post(`/api/system/approval/flow/toggle/${id}`);

// ============ Approval Instance ============

export const submitApprovalApi = async (data: any) =>
  requestClient.post('/api/system/approval/submit', data);

export const processApprovalApi = async (data: any) =>
  requestClient.post('/api/system/approval/process', data);

export const getApprovalDetailApi = async (id: number) =>
  requestClient.get(`/api/system/approval/detail/${id}`);

export const getApprovalListApi = async (params?: any) =>
  requestClient.get('/api/system/approval/list', { params });
