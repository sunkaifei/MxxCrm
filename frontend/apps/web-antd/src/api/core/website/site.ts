import { requestClient } from '#/api/request';

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
  /** URL伪静态规则：0=默认动态 1=短URL 2=目录模式 3=自定义 */
  urlRule?: number;
  /** URL伪静态规则模板（urlRule=3 时生效） */
  urlRulePattern?: string;
  /** 站点模式：1=展示型 2=交易型 3=混合型 */
  siteMode?: number;
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
  /** URL伪静态规则：0=默认动态 1=短URL 2=目录模式 3=自定义 */
  urlRule?: number;
  /** URL伪静态规则模板（urlRule=3 时生效） */
  urlRulePattern?: string;
  /** 站点模式：1=展示型 2=交易型 3=混合型 */
  siteMode?: number;
}

/**
 * 站点 API（单站模式）
 *
 * 单站模式下仅保留 `/site/current` 的 GET/PUT 两个接口：
 * - `getCurrent()`   → GET  /api/system/site/current
 * - `updateCurrent`  → PUT  /api/system/site/current
 *
 * 多站遗留接口（list/detail/add/update/delete）已全部移除。
 */
export const siteApi = {
  /** 获取当前（默认）站点配置 —— 单站模式专用 */
  getCurrent: () => requestClient.get('/api/system/site/current'),

  /** 更新当前（默认）站点配置 —— 单站模式专用 */
  updateCurrent: (data: SiteSaveDTO) =>
    requestClient.put('/api/system/site/current', data),
};
