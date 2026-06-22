import { requestClient } from '#/api/request';

/** SKU模板列表查询参数 */
export interface SkuTemplateListParams {
  page?: number;
  pageSize?: number;
  keywords?: string;
  productType?: string;
}

/** SKU模板列表VO */
export interface SkuTemplateListVO {
  id: string;
  templateName: string;
  productType?: string;
  description?: string;
  specCount: number;
  createTime?: string;
}

/** SKU模板详情VO（含规格） */
export interface SpecValueVO {
  id?: string;
  value?: string;
  sortOrder?: number;
}

export interface SpecVO {
  id?: string;
  name?: string;
  sortOrder?: number;
  values: SpecValueVO[];
}

export interface SkuTemplateDetailVO {
  id: string;
  templateName?: string;
  productType?: string;
  description?: string;
  createdBy?: number;
  createTime?: string;
  specs: SpecVO[];
}

/** SKU模板保存请求 */
export interface SkuTemplateSaveRequest {
  templateName: string;
  productType?: string;
  description?: string;
}

/** SKU模板更新请求 */
export interface SkuTemplateUpdateRequest {
  id: string;
  templateName?: string;
  productType?: string;
  description?: string;
}

/** 规格值保存项 */
export interface SpecValueSaveItem {
  value: string;
  sortOrder?: number;
}

/** 规格保存项 */
export interface SpecSaveItem {
  name: string;
  sortOrder?: number;
  values: SpecValueSaveItem[];
}

/** 模板规格批量保存请求 */
export interface TemplateSpecBatchSaveRequest {
  templateId: number;
  specs: SpecSaveItem[];
}

/** 获取SKU模板列表 */
export const getSkuTemplateListApi = async (params?: SkuTemplateListParams) => {
  return requestClient.get('/api/system/sku/template/list', { params });
};

/** 获取SKU模板详情（含规格） */
export const getSkuTemplateInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sku/template/info', { params: { id } });
};

/** 保存SKU模板 */
export const saveSkuTemplateApi = async (param: SkuTemplateSaveRequest) => {
  return requestClient.post('/api/system/sku/template/save', param);
};

/** 更新SKU模板 */
export const updateSkuTemplateApi = async (param: SkuTemplateUpdateRequest) => {
  return requestClient.put('/api/system/sku/template/update', param);
};

/** 删除SKU模板 */
export const deleteSkuTemplateApi = async (id: number) => {
  return requestClient.post('/api/system/sku/template/delete', { id });
};

/** 保存模板规格 */
export const saveTemplateSpecsApi = async (param: TemplateSpecBatchSaveRequest) => {
  return requestClient.post('/api/system/sku/template/spec/save', param);
};
