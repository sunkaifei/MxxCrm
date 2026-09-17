import { requestClient } from '#/api/request';

// 路由 scope 为 /scheduler，注册在 /api/system 下，
// 完整路径为 /api/system/scheduler/...
const BASE = '/api/system/scheduler';

export const getSchedulerJobListApi = async (params?: any) => {
  return requestClient.get(`${BASE}/list`, { params });
};

export const getSchedulerJobDetailApi = async (id: number) => {
  return requestClient.get(`${BASE}/detail`, { params: { id } });
};

export const updateSchedulerJobApi = async (data: any) => {
  return requestClient.post(`${BASE}/update`, data);
};

export const toggleSchedulerJobApi = async (data: {
  enabled: number;
  id: number;
}) => {
  return requestClient.post(`${BASE}/toggle`, data);
};

export const triggerSchedulerJobApi = async (data: { id: number }) => {
  return requestClient.post(`${BASE}/trigger`, data);
};

export const getSchedulerLogListApi = async (params?: any) => {
  return requestClient.get(`${BASE}/log/list`, { params });
};

// 调度告警日志（独立表，仅管理员可见；不含公告表数据）
export const getSchedulerAlertListApi = async (params?: any) => {
  return requestClient.get(`${BASE}/alert/list`, { params });
};

export const clearSchedulerAlertApi = async () => {
  return requestClient.post(`${BASE}/alert/clear`);
};
