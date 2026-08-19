import { requestClient } from '#/api/request';

export interface ContentModelVO {
  id: number;
  modelCode: string;
  modelName: string;
  modelIcon?: string;
  description?: string;
  hasTitle: number;
  hasContent: number;
  hasCover: number;
  hasAuthor: number;
  hasSummary: number;
  hasSeo: number;
  hasImages: number;
  hasAttachment: number;
  listTemplateId?: number;
  detailTemplateId?: number;
  sort: number;
  status: number;
  isSystem: number;
  createTime: string;
}

export interface ContentModelListQuery {
  keywords?: string;
  status?: number;
  page?: number;
  pageSize?: number;
}

export interface ContentModelSaveDTO {
  id?: number;
  modelCode?: string;
  modelName?: string;
  modelIcon?: string;
  description?: string;
  hasTitle?: number;
  hasContent?: number;
  hasCover?: number;
  hasAuthor?: number;
  hasSummary?: number;
  hasSeo?: number;
  hasImages?: number;
  hasAttachment?: number;
  listTemplateId?: number;
  detailTemplateId?: number;
  sort?: number;
  status?: number;
}

export interface ContentModelFieldVO {
  id: number;
  modelId: number;
  fieldName: string;
  fieldLabel?: string;
  fieldType: number;
  fieldOptions?: string;
  defaultValue?: string;
  placeholder?: string;
  isRequired: number;
  isSearchable: number;
  isListShow: number;
  isDetailShow: number;
  sort: number;
  status: number;
  createTime: string;
}

export interface ContentModelFieldSaveDTO {
  id?: number;
  modelId?: number;
  fieldName?: string;
  fieldLabel?: string;
  fieldType?: number;
  fieldOptions?: string;
  defaultValue?: string;
  placeholder?: string;
  isRequired?: number;
  isSearchable?: number;
  isListShow?: number;
  isDetailShow?: number;
  sort?: number;
  status?: number;
}

// Content Model APIs
export const getContentModelListApi = async (
  params?: ContentModelListQuery,
) => {
  return requestClient.get('/api/system/content/model/list', { params });
};

export const getContentModelDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/content/model/detail/${id}`);
};

export const addContentModelApi = async (data: ContentModelSaveDTO) => {
  return requestClient.post('/api/system/content/model/add', data);
};

export const updateContentModelApi = async (
  id: number,
  data: ContentModelSaveDTO,
) => {
  return requestClient.put(`/api/system/content/model/update/${id}`, data);
};

export const deleteContentModelApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/content/model/batch_delete', {
    data: { ids },
  });
};

// Content Model Field APIs
export const getContentModelFieldListApi = async (params?: {
  modelId?: number;
  page?: number;
  pageSize?: number;
}) => {
  return requestClient.get('/api/system/content/model/field/list', { params });
};

export const addContentModelFieldApi = async (
  data: ContentModelFieldSaveDTO,
) => {
  return requestClient.post('/api/system/content/model/field/add', data);
};

export const updateContentModelFieldApi = async (
  id: number,
  data: ContentModelFieldSaveDTO,
) => {
  return requestClient.put(
    `/api/system/content/model/field/update/${id}`,
    data,
  );
};

export const deleteContentModelFieldApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/content/model/field/batch_delete', {
    data: { ids },
  });
};
