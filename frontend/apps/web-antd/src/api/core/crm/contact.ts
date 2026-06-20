import { requestClient } from '#/api/request';

export const getContactListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/contact/list', { params });
};
export const getContactInfoApi = async (id: number) => {
  return requestClient.get('/api/system/contact/info', { params: { id } });
};
export const createContactApi = async (param: any) => {
  return requestClient.post('/api/system/contact/save', param);
};
export const updateContactApi = async (param: any) => {
  return requestClient.put('/api/system/contact/update', param);
};
export const deleteContactApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/contact/bath_delete', {
    data: { ids },
  });
};

// 绑定联系人到客户（入职）
export const bindContactApi = async (param: any) => {
  return requestClient.post('/api/system/contact/bind', param);
};
// 解绑联系人（离职）
export const unbindContactApi = async (param: any) => {
  return requestClient.post('/api/system/contact/unbind', param);
};
// 设置联系人角色/标记
export const setContactRoleApi = async (param: any) => {
  return requestClient.put('/api/system/contact/set_role', param);
};