import { requestClient } from '#/api/request';

export const getAiConfigListApi = async () => {
  return requestClient.get('/api/system/ai-config/list');
};

export const getAiConfigDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/ai-config/detail/${id}`);
};

export const addAiConfigApi = async (params: any) => {
  return requestClient.post('/api/system/ai-config/add', params);
};

export const updateAiConfigApi = async (params: any) => {
  return requestClient.put('/api/system/ai-config/update', params);
};

export const deleteAiConfigApi = async (id: number) => {
  return requestClient.delete(`/api/system/ai-config/delete/${id}`);
};

export const batchDeleteAiConfigApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/ai-config/batch-delete', { ids });
};

export const performBackgroundCheckApi = async (params: { company_name: string; lead_id?: number; company_id?: number }) => {
  return requestClient.post('/api/system/background-check/perform', params, { timeout: 120_000 });
};

export const getBackgroundCheckByLeadIdApi = async (leadId: number) => {
  return requestClient.get(`/api/system/background-check/lead/${leadId}`);
};

export const getLatestBackgroundCheckApi = async (leadId: number) => {
  return requestClient.get(`/api/system/background-check/latest/${leadId}`);
};

export const getBackgroundCheckByCompanyIdApi = async (companyId: number) => {
  return requestClient.get(`/api/system/background-check/company/${companyId}`);
};

export const getLatestBackgroundCheckByCompanyIdApi = async (companyId: number) => {
  return requestClient.get(`/api/system/background-check/company-latest/${companyId}`);
};

export const getBackgroundCheckDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/background-check/detail/${id}`);
};

export const getBackgroundCheckTimelineApi = async (companyName: string) => {
  return requestClient.get('/api/system/background-check/timeline', { params: { company_name: companyName } });
};

export const getLatestBackgroundCheckByCompanyApi = async (companyName: string) => {
  const list = await getBackgroundCheckTimelineApi(companyName);
  const arr = Array.isArray(list) ? list : (Array.isArray((list as any)?.data) ? (list as any).data : []);
  if (arr.length === 0) return null;
  const latest = arr[arr.length - 1];
  if (latest?.id) {
    return getBackgroundCheckDetailApi(latest.id);
  }
  return latest;
};

export const deleteBackgroundCheckApi = async (id: number) => {
  return requestClient.delete(`/api/system/background-check/delete/${id}`);
};
