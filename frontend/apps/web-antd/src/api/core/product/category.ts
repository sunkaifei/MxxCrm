import { requestClient } from '#/api/request';
import { useAccessStore } from '@vben/stores';

export const getCategoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/product/category/list', { params });
};
export const getCategoryInfoApi = async (id: number) => {
  return requestClient.get('/api/system/category/info', { params: { id } });
};
export const createCategoryApi = async (param: any) => {
  return requestClient.post('/api/system/product/category/save', param);
};
export const updateCategoryApi = async (param: any) => {
  return requestClient.put('/api/system/product/category/update', param);
};
export const deleteCategoryApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/category/bath_delete', {
    data: { ids },
  });
};

/** 上传图片到附件系统 */
export const uploadCategoryImageApi = async (file: File): Promise<string> => {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('type_id', '1'); // 产品分类图片使用附件分类 id=1

  const accessStore = useAccessStore();
  const token = accessStore.accessToken;

  const resp = await fetch('/api/system/attachment/upload', {
    method: 'POST',
    headers: {
      Authorization: token ? `Bearer ${token}` : '',
    },
    body: formData,
  });

  if (!resp.ok) {
    throw new Error(`Upload failed: ${resp.status} ${resp.statusText}`);
  }

  const blob = await resp.arrayBuffer();
  const { decode } = await import('@msgpack/msgpack');
  const decoded: any = decode(new Uint8Array(blob));
  return decoded?.data?.url || '';
};
