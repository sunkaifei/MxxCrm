import { requestClient } from '#/api/request';

export interface ChatSessionDTO {
  sessionId: string;
  sessionType: number;
  sessionName: string;
  avatarUrl: string;
  lastMessageId: string;
  lastMessageContent: string;
  lastMessageTime: string;
  unreadCount: number;
  lastMessageSender: string;
  isPinned?: boolean;
  isMuted?: boolean;
}

export interface ChatMessageDTO {
  messageId: string;
  sessionId: string;
  senderId: string;
  senderNickname: string;
  senderAvatar: string;
  content: string;
  messageType: number;
  contentType?: number;
  fileUrl?: string;
  fileName?: string;
  fileSize?: number;
  isRecalled: boolean;
  sendTime: string;
  isMine: boolean;
  /** 已读状态：0=未读，1=已读（仅对用户消息有意义） */
  readStatus?: number;
  /** 已读时间（RFC3339 字符串，未读时为空） */
  readTime?: string;
}

export interface PageResult<T> {
  list: T[];
  total: number;
  page: number;
  pageSize: number;
}

export const sendMessageApi = async (data: {
  sessionId?: string;
  receiverId?: string;
  content: string;
  contentType?: number;
  fileUrl?: string;
  fileName?: string;
}) => {
  return requestClient.post('/api/system/chat/send', data);
};

export const getSessionListApi = async (params: {
  page: number;
  pageSize: number;
}) => {
  return requestClient.get<PageResult<ChatSessionDTO>>('/api/system/chat/sessions', { params });
};

export const getMessageListApi = async (params: {
  sessionId: string;
  page: number;
  pageSize: number;
}) => {
  return requestClient.get<PageResult<ChatMessageDTO>>('/api/system/chat/messages', { params });
};

export const markReadApi = async (data: { sessionId: string }) => {
  return requestClient.post('/api/system/chat/mark-read', data);
};

export const deleteSessionApi = async (data: { sessionId: string }) => {
  return requestClient.post('/api/system/chat/delete-session', data);
};

export const searchUsersApi = async (params: {
  keyword: string;
  page: number;
  pageSize: number;
}) => {
  return requestClient.get('/api/system/chat/search-users', { params });
};

export const getUnreadCountApi = async (): Promise<number> => {
  const res: any = await requestClient.get('/api/system/chat/unread-count');
  // 后端返回 { unreadCount: number }，这里兼容直接返回 number 的情况
  if (typeof res === 'number') return res;
  if (res && typeof res.unreadCount === 'number') return res.unreadCount;
  if (res && typeof res.unread_count === 'number') return res.unread_count;
  return 0;
};

export const startSessionApi = async (data: { receiverId: string }) => {
  return requestClient.post<ChatSessionDTO>('/api/system/chat/start-session', data);
};

export const recallMessageApi = async (data: { messageId: string }) => {
  return requestClient.post('/api/system/chat/recall', data);
};

export const pinSessionApi = async (data: {
  sessionId: string;
  isPinned: boolean;
}) => {
  return requestClient.post('/api/system/chat/pin', data);
};

export const muteSessionApi = async (data: {
  sessionId: string;
  isMuted: boolean;
}) => {
  return requestClient.post('/api/system/chat/mute', data);
};

export interface ColleagueVO {
  id: string;
  userName?: string;
  nickName?: string;
  avatar?: string;
  depts?: Array<{ deptName?: string }>;
}

export const getColleagueListApi = async (params: {
  keyword?: string;
  page?: number;
  pageSize?: number;
}) => {
  return requestClient.get<ColleagueVO[]>('/api/system/chat/colleague-list', { params });
};
