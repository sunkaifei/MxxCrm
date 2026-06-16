import { requestClient } from '#/api/request';

export const getContactListApi = async (params?: PageParams) => {
  return requestClient.get('/api/crm/contact/list', { params });
};
export const getContactInfoApi = async (id: number) => {
  return requestClient.get('/api/crm/contact/info', { params: { id } });
};
export const createContactApi = async (param: any) => {
  return requestClient.post('/api/crm/contact/save', param);
};
export const updateContactApi = async (param: any) => {
  return requestClient.put('/api/crm/contact/update', param);
};
export const deleteContactApi = async (ids: number[]) => {
  return requestClient.delete('/api/crm/contact/bath_delete', {
    data: { ids },
  });
};
