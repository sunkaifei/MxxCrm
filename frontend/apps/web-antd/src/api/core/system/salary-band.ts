import { requestClient } from '#/api/request';

// 岗位薪资带宽（入口内嵌岗位管理页，权限复用 system:post:*）

export const getSalaryBandListApi = async (params?: any) =>
  requestClient.get('/api/system/salary-band/list', { params });

export const getSalaryBandDetailApi = async (id: number) =>
  requestClient.get(`/api/system/salary-band/detail/${id}`);

export const createSalaryBandApi = async (data: any) =>
  requestClient.post('/api/system/salary-band/save', data);

export const updateSalaryBandApi = async (id: number, data: any) =>
  requestClient.put(`/api/system/salary-band/update/${id}`, data);

export const deleteSalaryBandApi = async (ids: number[]) =>
  requestClient.delete('/api/system/salary-band/bath_delete', {
    data: { ids },
  });

/** 按岗位ID查询启用的带宽（入职定薪参照） */
export const getSalaryBandByPostApi = async (postId: number) =>
  requestClient.get(`/api/system/salary-band/by-post/${postId}`);
