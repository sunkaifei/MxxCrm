import { requestClient } from '#/api/request';

export interface NavigationListParams {
  websiteId?: number;
  navType?: string;
  dataType?: string;
  isShow?: number;
  keywords?: string;
  page?: number;
  pageSize?: number;
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
  /** 可见性：是否对未登录访客可见（0否，1是） */
  visibleGuest?: number;
  /** 可见性：可见设备，逗号分隔（pc,mobile；空=全部） */
  visibleDevices?: string;
  /** SEO：rel 属性 */
  rel?: string;
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
  visibleGuest?: number;
  visibleDevices?: string;
  rel?: string;
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

  /** 拖拽排序落库 */
  updateSort: (id: number, sort: number) =>
    requestClient.put('/api/system/navigation/update_sort', { id, sort }),
};
