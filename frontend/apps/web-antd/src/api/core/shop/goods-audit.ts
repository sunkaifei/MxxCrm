import { requestClient } from '#/api/request';

export interface GoodsAuditListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  shopId?: number;
  keyword?: string;
}

export interface GoodsAuditDTO {
  id: number;
  status: number;
  auditRemark?: string;
}

export interface GoodsAuditVO {
  id: number;
  shopId: number;
  shopName?: string;
  categoryId: number;
  title: string;
  primaryImage?: string;
  minSalePrice: number;
  maxSalePrice: number;
  stockTotal: number;
  soldNum: number;
  status: number;
  auditRemark?: string;
  createTime?: string;
  updateTime?: string;
}

export const goodsAuditApi = {
  list: (params: GoodsAuditListParams) =>
    requestClient.get('/api/system/spu/list', { params }),
  audit: (data: GoodsAuditDTO) =>
    requestClient.put('/api/system/spu/audit', data),
  detail: (id: number) => requestClient.get(`/api/system/spu/detail/${id}`),
};
