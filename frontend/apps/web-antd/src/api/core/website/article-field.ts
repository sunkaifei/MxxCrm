import { requestClient } from '#/api/request';

export interface ArticleFieldListQuery {
  page?: number;
  pageSize?: number;
  categoryId?: number;
  fieldName?: string;
  status?: number;
}

export interface ArticleFieldSaveDTO {
  id?: number;
  categoryId: number;
  fieldName: string;
  fieldLabel?: string;
  fieldType?: number;
  fieldOptions?: string;
  defaultValue?: string;
  isRequired?: number;
  sort?: number;
  status?: number;
}

export interface ArticleFieldVO {
  id: number;
  categoryId: number;
  fieldName: string;
  fieldLabel?: string;
  fieldType: number;
  fieldOptions?: string;
  defaultValue?: string;
  isRequired: number;
  sort: number;
  status: number;
  createTime?: string;
  updateTime?: string;
}

export interface ArticleFieldValueDTO {
  fieldId: number;
  fieldValue?: string;
}

export interface ArticleFieldValueBatchDTO {
  articleId: number;
  values: ArticleFieldValueDTO[];
}

export interface ArticleFieldValueVO {
  id?: number;
  articleId?: number;
  fieldId?: number;
  fieldValue?: string;
  fieldName?: string;
  fieldLabel?: string;
  fieldType?: number;
  fieldOptions?: string;
  isRequired?: number;
  createTime?: string;
  updateTime?: string;
}

// Article Field APIs
export const getArticleFieldListApi = async (params?: ArticleFieldListQuery) => {
  return requestClient.get('/api/system/article/field/list', { params });
};

export const getArticleFieldDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/article/field/detail/${id}`);
};

export const getArticleFieldByCategoryApi = async (categoryId: number) => {
  return requestClient.get(`/api/system/article/field/by_category/${categoryId}`);
};

export const addArticleFieldApi = async (data: ArticleFieldSaveDTO) => {
  return requestClient.post('/api/system/article/field/add', data);
};

export const updateArticleFieldApi = async (id: number, data: ArticleFieldSaveDTO) => {
  return requestClient.put(`/api/system/article/field/update/${id}`, data);
};

export const deleteArticleFieldApi = async (ids: (number | string)[]) => {
  return requestClient.delete('/api/system/article/field/batch_delete', {
    data: { ids: ids.map((v) => String(v)) },
  });
};

export const getArticleFieldValuesApi = async (articleId: number) => {
  return requestClient.get(`/api/system/article/field/values/${articleId}`);
};

export const saveArticleFieldValuesApi = async (data: ArticleFieldValueBatchDTO) => {
  return requestClient.post('/api/system/article/field/save_values', data);
};
