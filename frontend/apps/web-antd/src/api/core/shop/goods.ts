import { requestClient } from '#/api/request';

export interface GoodsListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  shopId?: number;
  categoryId?: number;
  keyword?: string;
}

export interface GoodsVO {
  id: number;
  shopId: number;
  shopName?: string;
  categoryId: number;
  title: string;
  subtitle?: string;
  primaryImage?: string;
  minSalePrice: number;
  maxSalePrice: number;
  minLinePrice: number;
  maxLinePrice: number;
  stockTotal: number;
  soldNum: number;
  status: number;
  isCommission: number;
  createTime?: string;
  updateTime?: string;
}

export const goodsApi = {
  list: (params: GoodsListParams) =>
    requestClient.get('/api/system/spu/list', { params }),
  detail: (id: number) => requestClient.get(`/api/system/spu/detail/${id}`),
  offline: (id: number) => requestClient.put(`/api/system/spu/offline/${id}`),
};
