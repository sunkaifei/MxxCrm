import { requestClient } from '#/api/request';

export const getNoticeListApi = async (params?: any) => {
  return requestClient.get('/api/system/notice/list', { params });
};

export const getNoticeInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/notice/detail/${id}`);
};

export const createNoticeApi = async (param: any) => {
  const data = {
    title: param.noticeTitle,
    content: param.content,
    r#type: param.noticeType,
    publishStatus: param.status,
  };
  return requestClient.post('/api/system/notice/add', data);
};

export const updateNoticeApi = async (id: number, param: any) => {
  const data = {
    title: param.noticeTitle,
    content: param.content,
    r#type: param.noticeType,
    publishStatus: param.status,
  };
  return requestClient.put(`/api/system/notice/update/${id}`, data);
};

export const deleteNoticeApi = async (id: number) => {
  return requestClient.delete(`/api/system/notice/bath_delete`, {
    data: { ids: [id] },
  });
};
