import { requestClient } from '#/api/request';

// 分页查询附件列表
export const getFileListApi = async (params?: any) => {
  return requestClient.get('/api/system/attachment/list', { params });
};

// 获取附件详情
export const getFileInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/attachment/detail/${id}`);
};

// 上传文件（multipart/form-data）
export const uploadFileApi = async (
  file: File,
  entityType: string,
  entityId?: number,
  typeId?: number,
) => {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('entity_type', entityType);
  if (entityId) formData.append('entity_id', String(entityId));
  if (typeId) formData.append('type_id', String(typeId));
  return requestClient.post('/api/system/attachment/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
};

// 安全下载/预览文件（返回blob）
export const downloadFileApi = (id: number, mode = 'preview') => {
  return requestClient.get(`/api/system/attachment/download/${id}`, {
    params: { mode },
    responseType: 'blob',
    responseReturn: 'body',
  });
};

// 按业务实体查询附件
export const getAttachmentsByEntityApi = (entityType: string, entityId: number) => {
  return requestClient.get('/api/system/attachment/by-entity', {
    params: { entityType, entityId },
  });
};

// 绑定附件到业务实体
export const bindAttachmentApi = (data: {
  attachmentIds: number[];
  entityType: string;
  entityId: number;
}) => {
  return requestClient.post('/api/system/attachment/bind', data);
};

// 解绑附件
export const unbindAttachmentApi = (data: {
  attachmentIds: number[];
  entityType: string;
  entityId: number;
}) => {
  return requestClient.post('/api/system/attachment/unbind', data);
};

// 更新附件信息
export const updateFileApi = async (id: number, param: any) => {
  return requestClient.put(`/api/system/attachment/update/${id}`, param);
};

// 批量删除附件
export const deleteFileApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/attachment/batch_delete', {
    data: { ids },
  });
};
