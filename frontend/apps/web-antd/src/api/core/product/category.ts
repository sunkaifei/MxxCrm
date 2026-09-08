import { useAccessStore } from '@vben/stores';

import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getCategoryListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/product/category/list', { params });
};
export const getCategoryInfoApi = async (id: number) => {
  return requestClient.get('/api/system/product/category/info', {
    params: { id },
  });
};
export const createCategoryApi = async (param: any) => {
  return requestClient.post('/api/system/product/category/save', param);
};
export const updateCategoryApi = async (param: any) => {
  return requestClient.put('/api/system/product/category/update', param);
};
export const deleteCategoryApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/product/category/batch_delete', {
    data: { ids },
  });
};

/** 上传图片到附件系统 */
export const uploadCategoryImageApi = async (file: File): Promise<string> => {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('type_id', '1'); // 产品分类图片使用附件分类 id=1
  formData.append('entity_type', 'product'); // 后端必填：业务类型（缺失时报"entity_type 不能为空"）

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

  // 修复：上传接口返回 JSON（JsonResp{code,msg,data}），此前误用 msgpack 解析导致
  // "Extra 56 of 57 byte(s) found at buffer[1]"（JSON 文本被当作 msgpack 帧）
  const json: any = await resp.json();
  if (json?.code !== 200) {
    throw new Error(json?.msg || '上传失败');
  }
  const data = json?.data;
  // data 兼容字符串 URL 或对象（{url}）两种形态
  if (typeof data === 'string') return data;
  return data?.url || data?.fileUrl || '';
};
