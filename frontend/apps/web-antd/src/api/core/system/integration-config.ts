import { requestClient } from '#/api/request';

// 接口配置列表（按分组）
export const getIntegrationListApi = async (category?: string) => {
  return requestClient.get('/api/system/system/integration/list', {
    params: { category },
  });
};

// 接口配置详情
export const getIntegrationInfoApi = async (id: number) => {
  return requestClient.get('/api/system/system/integration/info', {
    params: { id },
  });
};

// 保存接口配置
export const saveIntegrationApi = async (data: any) => {
  return requestClient.post('/api/system/system/integration/save', data);
};

// 启用/禁用
export const toggleIntegrationApi = async (id: number, enabled: number) => {
  return requestClient.post('/api/system/system/integration/toggle', {
    id,
    enabled,
  });
};

// 测试连接
export const testIntegrationApi = async (id: number) => {
  return requestClient.post(
    '/api/system/system/integration/test',
    {},
    { params: { id } },
  );
};

// 批量测试
export const testAllIntegrationApi = async () => {
  return requestClient.post('/api/system/system/integration/test-all', {});
};

// 删除配置（软删除）
export const deleteIntegrationApi = async (id: number) => {
  return requestClient.delete('/api/system/system/integration/delete', {
    params: { id },
  });
};
