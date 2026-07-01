import { requestClient } from '#/api/request';

// 获取企业信息
export const getCompanyInfoApi = async () => {
  return requestClient.get('/api/system/company/info');
};

// 更新企业信息（超管权限）
export const updateCompanyInfoApi = async (data: any) => {
  return requestClient.put('/api/system/company/update', data);
};

// 获取银行账户列表
export const getCompanyAccountListApi = async () => {
  return requestClient.get('/api/system/company/account/list');
};

// 保存银行账户（新增/编辑）
export const saveCompanyAccountApi = async (data: any) => {
  return requestClient.post('/api/system/company/account/save', data);
};

// 删除银行账户
export const deleteCompanyAccountApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/company/account/delete', { data: { ids: ids.map(String) } });
};
