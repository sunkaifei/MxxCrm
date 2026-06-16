import { requestClient } from '#/api/request';

export interface ShopListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  keyword?: string;
}

export interface ShopUpdateDTO {
  id: number;
  status?: number;
  commissionRate?: number;
}

export interface ShopVO {
  id: number;
  shopName?: string;
  shopLogo?: string;
  shopDesc?: string;
  contactName?: string;
  contactPhone?: string;
  status: number;
  commissionRate: number;
  settlementCycle: number;
  balance: number;
  createTime?: string;
  updateTime?: string;
}

export const shopApi = {
  list: (params: ShopListParams) =>
    requestClient.get('/api/system/shop/list', { params }),
  update: (data: ShopUpdateDTO) =>
    requestClient.put('/api/system/shop/update', data),
  detail: (id: number) => requestClient.get(`/api/system/shop/detail/${id}`),
};
