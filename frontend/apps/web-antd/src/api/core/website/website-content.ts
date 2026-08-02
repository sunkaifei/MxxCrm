import { requestClient } from '#/api/request';

export interface WebsiteBannerVO {
  id: number;
  title: string;
  imageUrl: string;
  linkUrl?: string;
  altText?: string;
  position: string;
  target: string;
  sort: number;
  startTime?: string;
  endTime?: string;
  status: number;
  createTime: string;
}

export interface BannerSaveDTO {
  id?: number;
  title?: string;
  imageUrl?: string;
  linkUrl?: string;
  altText?: string;
  position?: string;
  target?: string;
  sort?: number;
  startTime?: string;
  endTime?: string;
  status?: number;
}

export interface WebsiteBlockVO {
  id: number;
  blockCode: string;
  blockName: string;
  blockType: number;
  content?: string;
  imageUrl?: string;
  linkUrl?: string;
  sort: number;
  status: number;
  createTime: string;
}

export interface BlockSaveDTO {
  id?: number;
  blockCode?: string;
  blockName?: string;
  blockType?: number;
  content?: string;
  imageUrl?: string;
  linkUrl?: string;
  sort?: number;
  status?: number;
}

export interface WebsitePageVO {
  id: number;
  pageCode: string;
  pageName: string;
  pageTitle?: string;
  pageContent?: string;
  seoKeywords?: string;
  seoDescription?: string;
  templateId?: number;
  sort: number;
  status: number;
  createTime: string;
}

export interface PageSaveDTO {
  id?: number;
  pageCode?: string;
  pageName?: string;
  pageTitle?: string;
  pageContent?: string;
  seoKeywords?: string;
  seoDescription?: string;
  templateId?: number;
  sort?: number;
  status?: number;
}

// Banner APIs
export const getBannerListApi = async (params?: { keywords?: string; position?: string; page?: number; pageSize?: number }) => {
  return requestClient.get('/api/system/website/banner/list', { params });
};
export const getBannerDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/website/banner/detail/${id}`);
};
export const addBannerApi = async (data: BannerSaveDTO) => {
  return requestClient.post('/api/system/website/banner/add', data);
};
export const updateBannerApi = async (id: number, data: BannerSaveDTO) => {
  return requestClient.put(`/api/system/website/banner/update/${id}`, data);
};
export const deleteBannerApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/website/banner/batch_delete', { data: { ids } });
};

// Block APIs
export const getBlockListApi = async (params?: { keywords?: string; page?: number; pageSize?: number }) => {
  return requestClient.get('/api/system/website/block/list', { params });
};
export const getBlockDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/website/block/detail/${id}`);
};
export const addBlockApi = async (data: BlockSaveDTO) => {
  return requestClient.post('/api/system/website/block/add', data);
};
export const updateBlockApi = async (id: number, data: BlockSaveDTO) => {
  return requestClient.put(`/api/system/website/block/update/${id}`, data);
};
export const deleteBlockApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/website/block/batch_delete', { data: { ids } });
};

// Page APIs
export const getPageListApi = async (params?: { keywords?: string; page?: number; pageSize?: number }) => {
  return requestClient.get('/api/system/website/page/list', { params });
};
export const getPageDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/website/page/detail/${id}`);
};
export const addPageApi = async (data: PageSaveDTO) => {
  return requestClient.post('/api/system/website/page/add', data);
};
export const updatePageApi = async (id: number, data: PageSaveDTO) => {
  return requestClient.put(`/api/system/website/page/update/${id}`, data);
};
export const deletePageApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/website/page/batch_delete', { data: { ids } });
};
