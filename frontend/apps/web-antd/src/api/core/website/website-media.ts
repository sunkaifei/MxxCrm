import { requestClient } from '#/api/request';

export interface WebsiteMediaVO {
  id: number;
  originalName: string;
  storageName: string;
  filePath: string;
  fileUrl: string;
  fileExt?: string;
  fileSize?: number;
  fileType: number;
  mimeType?: string;
  width?: number;
  height?: number;
  thumbSmall?: string;
  thumbMedium?: string;
  thumbLarge?: string;
  altText?: string;
  title?: string;
  caption?: string;
  description?: string;
  categoryId?: number;
  tags?: string[];
  refCount: number;
  hasWatermark: number;
  sort: number;
  status: number;
  attachmentId?: number;
  uploadedBy?: number;
  uploadedName?: string;
  createTime: string;
}

export interface MediaListQuery {
  keywords?: string;
  fileType?: number;
  categoryId?: number;
  page?: number;
  pageSize?: number;
}

export interface MediaSaveDTO {
  id?: number;
  originalName?: string;
  storageName?: string;
  filePath?: string;
  fileUrl?: string;
  fileExt?: string;
  fileSize?: number;
  fileType?: number;
  mimeType?: string;
  width?: number;
  height?: number;
  thumbSmall?: string;
  thumbMedium?: string;
  thumbLarge?: string;
  altText?: string;
  title?: string;
  caption?: string;
  description?: string;
  categoryId?: number;
  tags?: string[];
  sort?: number;
  status?: number;
  attachmentId?: number;
}

export interface MediaCategoryVO {
  id: number;
  categoryName: string;
  parentId: number;
  sort: number;
  children?: MediaCategoryVO[];
}

export interface MediaCategorySaveDTO {
  id?: number;
  categoryName?: string;
  parentId?: number;
  sort?: number;
}

export const getMediaListApi = async (params?: MediaListQuery) => {
  return requestClient.get('/api/system/website/media/list', { params });
};

export const getMediaDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/website/media/detail/${id}`);
};

export const addMediaApi = async (data: MediaSaveDTO) => {
  return requestClient.post('/api/system/website/media/add', data);
};

export const updateMediaApi = async (id: number, data: MediaSaveDTO) => {
  return requestClient.put(`/api/system/website/media/update/${id}`, data);
};

export const deleteMediaApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/website/media/batch_delete', {
    data: { ids },
  });
};

export const getMediaCategoryAllApi = async () => {
  return requestClient.get('/api/system/website/media/category/all');
};

export const addMediaCategoryApi = async (data: MediaCategorySaveDTO) => {
  return requestClient.post('/api/system/website/media/category/add', data);
};

export const updateMediaCategoryApi = async (
  id: number,
  data: MediaCategorySaveDTO,
) => {
  return requestClient.put(
    `/api/system/website/media/category/update/${id}`,
    data,
  );
};

export const deleteMediaCategoryApi = async (id: number) => {
  return requestClient.delete(
    `/api/system/website/media/category/delete/${id}`,
  );
};
