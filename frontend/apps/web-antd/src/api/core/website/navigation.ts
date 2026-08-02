import { requestClient } from '#/api/request';

export interface NavigationListParams {
  websiteId?: number;
  navType?: string;
}

export interface NavigationSaveDTO {
  id?: number;
  websiteId?: number;
  parentId?: number;
  name?: string;
  webUrl?: string;
  value?: number;
  dataType?: string;
  navType?: string;
  sort?: number;
  isShow?: number;
  isNewWindowOpen?: number;
  target?: string;
  icon?: string;
}

export interface NavigationVO {
  id: number;
  websiteId?: number;
  parentId?: number;
  name?: string;
  webUrl?: string;
  value?: number;
  dataType?: string;
  navType?: string;
  sort?: number;
  isShow?: number;
  isNewWindowOpen?: number;
  target?: string;
  icon?: string;
  createTime?: string;
}

export const navigationApi = {
  list: (params?: NavigationListParams) =>
    requestClient.get('/api/system/navigation/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/navigation/detail/${id}`),

  add: (data: NavigationSaveDTO) =>
    requestClient.post('/api/system/navigation/add', data),

  update: (id: number, data: NavigationSaveDTO) =>
    requestClient.put(`/api/system/navigation/update/${id}`, data),

  delete: (ids: number[]) =>
    requestClient.delete('/api/system/navigation/batch_delete', {
      data: { ids },
    }),
};
