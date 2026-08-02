import { requestClient } from '#/api/request';

export interface CategoryListParams {
  page?: number;
  pageSize?: number;
}

export interface CategorySaveDTO {
  id?: number;
  parentId?: number;
  name?: string;           // maps to category_name
  shortUrl?: string;
  sortOrder?: number;      // maps to sort
  isShow?: number;         // maps to is_show
  status?: number;
  // New unified fields
  pageType?: number;       // 1=封面模式, 2=列表模式
  pageTemplateDataId?: number;
  bannerImage?: string;
  description?: string;
  contentType?: number;    // 1=文章, 2=产品, 3=自定义链接
  linkUrl?: string;        // content_type=3时使用
  // SEO fields
  seoTitle?: string;
  seoKeywords?: string;
  seoDescription?: string;
}

export interface CategoryVO {
  id: number;
  parentId: number;
  name: string;
  shortUrl?: string;
  sortOrder: number;
  level: number;
  isShow: number;
  children?: CategoryVO[];
  createTime?: string;
  updateTime?: string;
  // New unified fields
  pageType?: number;
  pageTemplateDataId?: number;
  bannerImage?: string;
  description?: string;
  contentType?: number;
  linkUrl?: string;
  // SEO fields
  seoTitle?: string;
  seoKeywords?: string;
  seoDescription?: string;
}

export const categoryApi = {
  tree: () => requestClient.get('/api/system/category/tree'),
  save: (data: CategorySaveDTO) =>
    requestClient.post('/api/system/category/save', data),
  update: (data: CategorySaveDTO) =>
    requestClient.put('/api/system/category/update', data),
  delete: (params: { id: number }) =>
    requestClient.delete('/api/system/category/delete', { params }),
};
