import { requestClient } from '#/api/request';

export interface AreaListParams {
  page_num?: number;
  page_size?: number;
  name?: string;
  level?: number;
  country_code?: string;
}

export interface AreaSaveParams {
  parent_id?: number;
  name?: string;
  name_en?: string;
  code?: string;
  level?: number;
  sort?: number;
  country_code?: string;
  latitude?: number;
  longitude?: number;
}

export interface AreaUpdateParams extends AreaSaveParams {
  id?: number;
}

export const getAreaTreeApi = async () => {
  return requestClient.get('/api/system/area/tree');
};

export const getAreaCascaderApi = async () => {
  return requestClient.get('/api/system/area/cascader');
};

export const getCountriesApi = async () => {
  return requestClient.get('/api/system/area/countries');
};

export const getProvincesApi = async (country_code: string) => {
  return requestClient.get('/api/system/area/provinces', { params: { country_code } });
};

export const getChildrenApi = async (parent_id: number) => {
  return requestClient.get('/api/system/area/children', { params: { parent_id } });
};

export const getAreaDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/area/detail/${id}`);
};

export const getAreaListApi = async (params?: AreaListParams) => {
  return requestClient.get('/api/system/area/list', { params });
};

export const createAreaApi = async (param: AreaSaveParams) => {
  return requestClient.post('/api/system/area/save', param);
};

export const updateAreaApi = async (id: number, param: AreaUpdateParams) => {
  return requestClient.put(`/api/system/area/update/${id}`, param);
};

export const deleteAreaApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/area/batch_delete', {
    data: { ids },
  });
};
