import { requestClient } from '#/api/request';

export interface ProductListParams {
  page?: number;
  pageSize?: number;
  keywords?: string;
  categoryId?: number;
  status?: number;
}

export interface ProductVO {
  id: number;
  categoryId?: number;
  categoryName?: string;
  productName?: string;
  productCode?: string;
  productImage?: string;
  productImages?: string[];
  price?: number;
  marketPrice?: number;
  costPrice?: number;
  stock?: number;
  salesCount?: number;
  status?: number;
  sort?: number;
  description?: string;
  seoKeywords?: string;
  seoDescription?: string;
  createTime?: string;
  updateTime?: string;
}

export interface ProductSaveDTO {
  id?: number;
  categoryId?: number;
  productName?: string;
  productCode?: string;
  productImage?: string;
  productImages?: string[];
  price?: number;
  marketPrice?: number;
  costPrice?: number;
  stock?: number;
  status?: number;
  sort?: number;
  description?: string;
  seoKeywords?: string;
  seoDescription?: string;
}

export const productApi = {
  list: (params: ProductListParams) =>
    requestClient.get('/api/system/website/product/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/website/product/detail/${id}`),

  add: (data: ProductSaveDTO) =>
    requestClient.post('/api/system/website/product/add', data),

  update: (id: number, data: ProductSaveDTO) =>
    requestClient.put(`/api/system/website/product/update/${id}`, data),

  batchDelete: (ids: number[]) =>
    requestClient.delete('/api/system/website/product/batch_delete', { data: { ids } }),
};