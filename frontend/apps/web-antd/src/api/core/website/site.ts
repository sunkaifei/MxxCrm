import { requestClient } from '#/api/request';

export interface SiteListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  keyword?: string;
}

export interface SiteSaveDTO {
  id?: number;
  siteName?: string;
  templateId?: number;
  domain?: string;
  bindDomain?: string;
  logo?: string;
  keywords?: string;
  description?: string;
  siteType?: number;
  client?: number;
  status?: number;
  isDefault?: number;
  sort?: number;
  remark?: string;
  showBanner?: number;
  watermarkEnable?: number;
  watermarkType?: number;
  watermarkText?: string;
  watermarkImage?: string;
  watermarkPosition?: number;
  watermarkOpacity?: number;
  uploadAllowedTypes?: string;
  uploadMaxSize?: number;
  companyName?: string;
  companyPhone?: string;
  companyEmail?: string;
  companyAddress?: string;
  workTimeStart?: string;
  workTimeEnd?: string;
  workDays?: string;
  qq?: string;
  wechat?: string;
  wechatQrcode?: string;
  icp?: string;
  copyright?: string;
  statisticsCode?: string;
  customCss?: string;
  customJs?: string;
  closeReason?: string;
  shareTitle?: string;
  shareDesc?: string;
  shareImage?: string;
}

export interface SiteVO {
  id: number;
  siteName?: string;
  domain?: string;
  bindDomain?: string;
  logo?: string;
  keywords?: string;
  description?: string;
  siteType?: number;
  client?: number;
  status?: number;
  isDefault?: number;
  sort?: number;
  remark?: string;
  userName?: string;
  templateId?: number;
  createTime?: string;
  showBanner?: number;
  watermarkEnable?: number;
  watermarkType?: number;
  watermarkText?: string;
  watermarkImage?: string;
  watermarkPosition?: number;
  watermarkOpacity?: number;
  uploadAllowedTypes?: string;
  uploadMaxSize?: number;
  companyName?: string;
  companyPhone?: string;
  companyEmail?: string;
  companyAddress?: string;
  workTimeStart?: string;
  workTimeEnd?: string;
  workDays?: string;
  qq?: string;
  wechat?: string;
  wechatQrcode?: string;
  icp?: string;
  copyright?: string;
  statisticsCode?: string;
  customCss?: string;
  customJs?: string;
  closeReason?: string;
  shareTitle?: string;
  shareDesc?: string;
  shareImage?: string;
}

export const siteApi = {
  list: (params: SiteListParams) =>
    requestClient.get('/api/system/site/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/site/detail/${id}`),

  add: (data: SiteSaveDTO) =>
    requestClient.post('/api/system/site/add', data),

  update: (id: number, data: SiteSaveDTO) =>
    requestClient.put(`/api/system/site/update/${id}`, data),

  delete: (ids: number[]) =>
    requestClient.delete('/api/system/site/batch_delete', { data: { ids } }),
};
