import { requestClient } from '#/api/request';

export interface PromotionListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  shopId?: number;
}

export interface PromotionSaveDTO {
  id?: number;
  shopId?: number;
  title: string;
  type: number;
  rules: Record<string, unknown>;
  startTime: string;
  endTime: string;
  status?: number;
}

export interface PromotionVO {
  id: number;
  shopId?: number;
  shopName?: string;
  title: string;
  type: number;
  rules: Record<string, unknown>;
  startTime: string;
  endTime: string;
  status: number;
  createTime?: string;
  updateTime?: string;
}

export const promotionApi = {
  list: (params: PromotionListParams) =>
    requestClient.get('/api/system/promotion/list', { params }),
  save: (data: PromotionSaveDTO) =>
    requestClient.post('/api/system/promotion/save', data),
  update: (data: PromotionSaveDTO) =>
    requestClient.put('/api/system/promotion/update', data),
  delete: (params: { id: number }) =>
    requestClient.delete('/api/system/promotion/delete', { params }),
  detail: (id: number) =>
    requestClient.get(`/api/system/promotion/detail/${id}`),
};
