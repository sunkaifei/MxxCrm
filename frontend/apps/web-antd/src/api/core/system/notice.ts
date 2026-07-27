import { requestClient } from '#/api/request';

export const getNoticeListApi = async (params?: any) => {
  return requestClient.get('/api/system/notice/list', { params });
};

export const getNoticeInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/notice/detail/${id}`);
};

export const createNoticeApi = async (param: any) => {
  // 字段对齐后端 NoticeSaveRequest（camelCase）
  const data = {
    title: param.title,
    content: param.content,
    type: param.type,
    level: param.level,
    targetType: param.targetType,
    targetUserIds: param.targetUserIds,
  };
  return requestClient.post('/api/system/notice/add', data);
};

export const updateNoticeApi = async (id: number, param: any) => {
  const data = {
    id,
    title: param.title,
    content: param.content,
    type: param.type,
    level: param.level,
    targetType: param.targetType,
    targetUserIds: param.targetUserIds,
  };
  return requestClient.put(`/api/system/notice/update/${id}`, data);
};

export const deleteNoticeApi = async (id: number) => {
  return requestClient.delete(`/api/system/notice/bath_delete`, {
    data: { ids: [id] },
  });
};

// 发布公告
export const publishNoticeApi = async (id: number) => {
  return requestClient.put(`/api/system/notice/${id}/publish`);
};

// 撤销公告
export const revokeNoticeApi = async (id: number) => {
  return requestClient.put(`/api/system/notice/${id}/revoke`);
};

// 我的公告列表（当前用户收到的已发布公告，含已读/未读状态）
export const getMyNoticeListApi = async (params: {
  page: number;
  pageSize: number;
  isRead?: number;
}) => {
  return requestClient.get('/api/system/notice/my-page', { params });
};

// 标记公告为已读（专用接口 PUT /notice/user/{id}/read）
export const readNoticeApi = async (id: number) => {
  return requestClient.put(`/api/system/notice/user/${id}/read`);
};

// 全部公告标记已读
export const readAllNoticeApi = async () => {
  return requestClient.put('/api/system/notice/read-all');
};

// 字典数据常量（与 mxx_system_dict_data 中 notice_type / notice_level 保持一致）
export const NOTICE_TYPE_OPTIONS = [
  { label: '通知', value: 1 },
  { label: '公告', value: 2 },
  { label: '系统消息', value: 3 },
];

export const NOTICE_LEVEL_OPTIONS = [
  { label: '低', value: 'low' },
  { label: '普通', value: 'normal' },
  { label: '高', value: 'high' },
  { label: '紧急', value: 'urgent' },
];

export const NOTICE_TARGET_TYPE_OPTIONS = [
  { label: '全体用户', value: 1 },
  { label: '指定用户', value: 2 },
];

// 发布状态：0=未发布, 1=已发布, -1=已撤回
export const NOTICE_PUBLISH_STATUS = {
  UNPUBLISHED: 0,
  PUBLISHED: 1,
  REVOKED: -1,
} as const;

export const NOTICE_PUBLISH_STATUS_OPTIONS = [
  { label: '未发布', value: 0, color: 'default' },
  { label: '已发布', value: 1, color: 'success' },
  { label: '已撤回', value: -1, color: 'warning' },
];
