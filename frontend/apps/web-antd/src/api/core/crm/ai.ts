import { requestClient } from '#/api/request';

// AI 模型提供商 / 提示词配置：已统一迁移到「第三方接口配置」(integration-config.ts → AI 分类)
// 下方仅保留企业背调相关接口（消费方）

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
