import { requestClient } from '#/api/request';

export interface TemplateListParams {
  page?: number;
  pageSize?: number;
  keywords?: string;
  status?: number;
  categoryId?: number;
}

export interface TemplateListVO {
  id: number;
  categoryId?: number;
  name?: string;
  templateFolder?: string;
  previewUrl?: string;
  price?: string;
  promotionPrice?: string;
  terminalMobile?: number;
  terminalPc?: number;
  terminalIpad?: number;
  terminalDisplay?: number;
  previewPic?: string;
  createTime?: string;
  sort?: number;
  status?: number;
}

export interface TemplateDetailVO {
  id: number;
  categoryId?: number;
  name?: string;
  templateFolder?: string;
  userId?: number;
  remark?: string;
  previewUrl?: string;
  companyname?: string;
  username?: string;
  siteurl?: string;
  terminalMobile?: number;
  terminalPc?: number;
  terminalIpad?: number;
  terminalDisplay?: number;
  iscommon?: number;
  sort?: number;
  wscsoDownUrl?: string;
  zipDownUrl?: string;
  previewPic?: string;
  resourceImport?: string;
  createTime?: string;
  updateTime?: string;
  status?: number;
}

export interface TemplateSaveDTO {
  categoryId?: number;
  name?: string;
  templateFolder?: string;
  remark?: string;
  previewUrl?: string;
  price?: number;
  promotionPrice?: number;
  companyname?: string;
  username?: string;
  siteurl?: string;
  terminalMobile?: number;
  terminalPc?: number;
  terminalIpad?: number;
  terminalDisplay?: number;
  iscommon?: number;
  sort?: number;
  wscsoDownUrl?: string;
  zipDownUrl?: string;
  previewPic?: string;
  resourceImport?: string;
  status?: number;
}

export const templateApi = {
  /** 分页获取模板列表 */
  list: (params: TemplateListParams) =>
    requestClient.get('/api/system/template/list', { params }),

  /** 模板详情 */
  detail: (id: number) =>
    requestClient.get(`/api/system/template/detail/${id}`),

  /** 新增模板 */
  add: (data: TemplateSaveDTO) =>
    requestClient.post('/api/system/template/add', data),

  /** 更新模板 */
  update: (id: number, data: TemplateSaveDTO) =>
    requestClient.post('/api/system/template/update', { ...data, id }),

  /** 删除模板 */
  delete: (ids: number[]) =>
    requestClient.delete('/api/system/template/batch_delete', { data: { ids } }),

  /** 公共模板下拉选项 */
  commonOptions: () =>
    requestClient.get('/api/system/template/common_options'),
};
