import { requestClient } from '#/api/request';

export interface ProductSkuRequest {
  id?: number | string;
  specs?: string;
  imageUrl?: string | null;
  price?: number;
  costPrice?: number;
  originalPrice?: number;
  stock?: number;
  skuCode?: string;
  weight?: number;
  volume?: number;
  isDefault?: boolean;
  isActive?: boolean;
}

export interface ProductSaveRequest {
  id?: number | string;
  name: string;
  productNo?: string;
  categoryId?: number;
  keywords?: string;
  unit?: string;
  barcode?: string;
  salePrice?: number;
  marketPrice?: number;
  costPrice?: number;
  stock?: number;
  weight?: number;
  dimensions?: string;
  isActive?: boolean;
  description?: string;
  detail?: string;
  currency?: string;
  coverImage?: string | null;
  carouselImages?: string[];
  specType?: 'single' | 'multiple';
  templateId?: number;
  skus?: ProductSkuRequest[];
}

export type ProductUpdateRequest = ProductSaveRequest;

export const getProductListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/product/product/list', { params });
};
export const getProductInfoApi = async (id: number) => {
  return requestClient.get('/api/system/product/product/info', { params: { id } });
};
export const createProductApi = async (param: ProductSaveRequest) => {
  return requestClient.post('/api/system/product/product/save', param);
};
export const updateProductApi = async (param: ProductUpdateRequest) => {
  return requestClient.put('/api/system/product/product/update', param);
};
export const deleteProductApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/product/product/batchDelete', {
    data: { ids },
  });
};
