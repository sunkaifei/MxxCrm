import type { PageResult } from './chat';

import { requestClient } from '#/api/request';

export interface NotificationDTO {
  id: string;
  title: string;
  content: string;
  type: number;
  bizType: string;
  bizId: string;
  senderId: string;
  receiverId: string;
  isRead: boolean;
  readTime: string;
  linkUrl: string;
  createTime: string;
}

export const getNotificationListApi = async (params: {
  page: number;
  pageSize: number;
  type?: number;
  isRead?: boolean;
}) => {
  return requestClient.get<PageResult<NotificationDTO>>('/api/system/my-notification/list', { params });
};

export const readNotificationApi = async (data: { id: string }) => {
  return requestClient.post('/api/system/my-notification/read', data);
};

export const readAllNotificationApi = async () => {
  return requestClient.post('/api/system/my-notification/read-all');
};

export const getNotificationUnreadCountApi = async () => {
  return requestClient.get<number>('/api/system/my-notification/unread-count');
};

export const deleteNotificationApi = async (data: { id: string }) => {
  return requestClient.post('/api/system/my-notification/delete', data);
};
