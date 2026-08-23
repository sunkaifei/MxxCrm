<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRoute } from 'vue-router';

import { Page } from '@vben/common-ui';
import {
  LucideArrowRight,
  LucideImage,
  LucideSearch,
  SvgBellIcon,
} from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';

import {
  Avatar,
  Badge,
  Button,
  Empty,
  Image,
  Input,
  message,
  Spin,
} from 'ant-design-vue';

import { uploadFileApi } from '#/api/core/attachment/file';
import {
  getColleagueListApi,
  getMessageListApi,
  getSessionListApi,
  markReadApi,
  sendMessageApi,
  startSessionApi,
} from '#/api/core/message/chat';
import {
  getNotificationListApi,
  getNotificationUnreadCountApi,
  readAllNotificationApi,
} from '#/api/core/message/notification';
import {
  getMyNoticeListApi,
  readAllNoticeApi,
  readNoticeApi,
} from '#/api/core/system/notice';

defineOptions({ name: 'CompanyMessage' });

const userStore = useUserStore();
const currentUserAvatar = computed(
  () => (userStore.userInfo as any)?.avatar || undefined,
);
const currentUserRealName = computed(
  () =>
    (userStore.userInfo as any)?.realName ||
    (userStore.userInfo as any)?.username ||
    '我',
);

// 通知右上角铃铛（basic.vue）实时刷新未读数与下拉列表
function notifyBellRefresh() {
  window.dispatchEvent(new CustomEvent('chat:unread-updated'));
  window.dispatchEvent(new CustomEvent('notification:read'));
}

const route = useRoute();

// 根据路由 query 参数打开对应窗口
async function openFromRouteQuery() {
  const { tab, sessionId, notifType } = route.query as Record<string, string>;
  if (!tab) return;

  if (tab === 'colleague' && sessionId) {
    leftTab.value = 'colleague';
    // 等待 sessionList 和 allUsers 加载完毕后查找对应用户
    const targetSession = sessionList.value.find(
      (s: any) => String(s.sessionId) === String(sessionId),
    );
    if (targetSession) {
      const targetUser = allUsers.value.find((u: any) => {
        const name = u.nickName || u.userName;
        return name === targetSession.sessionName;
      });
      if (targetUser) {
        await handleSelectChat(targetUser);
        return;
      }
    }
    // 兜底：直接通过 sessionId 加载消息（无左侧用户高亮）
    activeSessionId.value = sessionId;
    await loadMessages(sessionId);
  } else if (tab === 'notification' && notifType) {
    leftTab.value = 'notification';
    const typeNum = NOTIF_NAME_TO_TYPE[notifType];
    if (typeNum) {
      activeNotifType.value = typeNum;
    }
  }
}

type LeftTab = 'colleague' | 'notification';

const CONTENT_TYPE_TEXT = 1;
const CONTENT_TYPE_IMAGE = 2;

const NOTIFICATION_TYPES = [
  { type: 0, name: '全部', icon: 'all', color: '#1677ff' },
  { type: 1, name: '公司公告', icon: 'announce', color: '#1677ff' },
  { type: 2, name: '客户分配', icon: 'customer', color: '#52c41a' },
  { type: 3, name: '报价审批', icon: 'quote', color: '#faad14' },
  { type: 4, name: '订单审批', icon: 'order', color: '#722ed1' },
  { type: 5, name: '合同审批', icon: 'contract', color: '#eb2f96' },
  { type: 6, name: '发货通知', icon: 'ship', color: '#13c2c2' },
  { type: 7, name: '回款提醒', icon: 'money', color: '#fa8c16' },
  { type: 8, name: '财务信息', icon: 'finance', color: '#f5222d' },
  { type: 9, name: '人事审批', icon: 'hr', color: '#2f54eb' },
];

// 通知类型名称 -> 数字 type 反向映射（用于路由参数解析，必须在 NOTIFICATION_TYPES 之后）
const NOTIF_NAME_TO_TYPE: Record<string, number> = {};
for (const item of NOTIFICATION_TYPES) {
  if (item.type !== 0) NOTIF_NAME_TO_TYPE[item.name] = item.type;
}

const leftTab = ref<LeftTab>('notification');
const activeNotifType = ref(0);
const activeChatId = ref<null | number>(null);
const notifLoading = ref(false);
const userLoading = ref(false);
const searchKeyword = ref('');

const notificationList = ref<any[]>([]);
const notificationUnread = ref(0);

// ===== 公告（来自 notice 模块，独立于消息通知） =====
const noticeList = ref<any[]>([]);
const noticeLoading = ref(false);
const expandedNoticeId = ref<null | number>(null);

const noticeUnreadCount = computed(
  () => noticeList.value.filter((n: any) => n.is_read !== 1).length,
);

// 公告按发布时间倒序排列（最新在前）
const sortedNoticeList = computed(() => {
  return noticeList.value.toSorted((a, b) => {
    const ta = a.publish_time
      ? new Date(String(a.publish_time).replace(' ', 'T')).getTime()
      : 0;
    const tb = b.publish_time
      ? new Date(String(b.publish_time).replace(' ', 'T')).getTime()
      : 0;
    return tb - ta;
  });
});

const allUsers = ref<any[]>([]);
const sessionList = ref<any[]>([]);

const messageList = ref<any[]>([]);
const messageInput = ref('');
const messageLoading = ref(false);
const chatContainerRef = ref<HTMLElement | null>(null);
const activeSessionId = ref<null | string>(null);
const imageInputRef = ref<HTMLInputElement | null>(null);
const uploadingImage = ref(false);

const unreadByType = computed(() => {
  const map: Record<number, number> = {};
  for (const n of notificationList.value) {
    if (!n.isRead) {
      const t = n.type || 0;
      map[t] = (map[t] || 0) + 1;
    }
  }
  return map;
});

// 侧栏各类未读数：type=0（全部）= 通知未读 + 公告未读；type=1（公司公告）= 公告未读
function getUnreadCountForTab(type: number): number {
  if (type === 0) return notificationUnread.value + noticeUnreadCount.value;
  if (type === 1) return noticeUnreadCount.value;
  return unreadByType.value[type] || 0;
}

// ===== 实时推送：WebSocket 客户端 =====
let ws: null | WebSocket = null;
let wsReconnectTimer: null | ReturnType<typeof setTimeout> = null;
let wsReconnectAttempts = 0;
const WS_RECONNECT_MAX = 10;
const WS_RECONNECT_INTERVAL = 3000;
const audioElement = ref<HTMLAudioElement | null>(null);

// 初始化音频元素（用户首次交互后才能播放）
function initAudio() {
  if (audioElement.value) return;
  try {
    audioElement.value = new Audio('/sounds/news.mp3');
    audioElement.value.preload = 'auto';
  } catch (error) {
    console.warn('[音频] 初始化失败', error);
  }
}

// 播放消息提醒音
function playNotificationSound() {
  if (!audioElement.value) initAudio();
  if (audioElement.value) {
    audioElement.value.currentTime = 0;
    audioElement.value.play().catch((error) => {
      console.warn('[音频] 播放被阻止', error);
    });
  }
}

// 获取 WebSocket URL（与后端 /ws/message 路由对应）
function getWsUrl(): string {
  const accessStore = useAccessStore();
  const token = accessStore.accessToken || '';
  const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
  // 与 HTTP API 同源：开发环境经 vite proxy 转发 /ws 到后端（端口见 backend/config/config.ini），生产环境同源
  return `${protocol}://${window.location.host}/ws/message?token=${encodeURIComponent(token)}`;
}

// WebSocket 关闭处理（命名以便 disconnectWebSocket 成对注销）
function handleWsClose(e: CloseEvent) {
  console.warn('[WebSocket] 关闭', e.code, e.reason);
  ws = null;
  if (e.code !== 1000) {
    // 非正常关闭，尝试重连
    scheduleReconnect();
  }
}

// 建立 WebSocket 连接
function connectWebSocket() {
  if (
    ws &&
    (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)
  ) {
    return;
  }
  try {
    ws = new WebSocket(getWsUrl());
  } catch (error) {
    console.error('[WebSocket] 创建失败', error);
    scheduleReconnect();
    return;
  }

  ws.addEventListener('open', () => {
    console.warn('[WebSocket] 连接已建立');
    wsReconnectAttempts = 0;
  });

  ws.addEventListener('message', (event) => {
    handleWsMessage(event.data);
  });

  ws.addEventListener('error', (e) => {
    console.error('[WebSocket] 错误', e);
  });

  ws.addEventListener('close', handleWsClose);
}

// 重连调度
function scheduleReconnect() {
  if (wsReconnectTimer) return;
  if (wsReconnectAttempts >= WS_RECONNECT_MAX) {
    console.warn('[WebSocket] 达到最大重连次数，停止重连');
    return;
  }
  wsReconnectAttempts++;
  const delay = WS_RECONNECT_INTERVAL * Math.min(wsReconnectAttempts, 5);
  console.warn(`[WebSocket] ${delay}ms 后第 ${wsReconnectAttempts} 次重连`);
  wsReconnectTimer = setTimeout(() => {
    wsReconnectTimer = null;
    connectWebSocket();
  }, delay);
}

// 关闭 WebSocket
function disconnectWebSocket() {
  if (wsReconnectTimer) {
    clearTimeout(wsReconnectTimer);
    wsReconnectTimer = null;
  }
  if (ws) {
    ws.removeEventListener('close', handleWsClose);
    ws.close(1000, 'page unmount');
    ws = null;
  }
}

// 处理 WebSocket 推送消息
async function handleWsMessage(data: any) {
  let payload: any;
  try {
    payload = JSON.parse(data);
  } catch {
    console.warn('[WebSocket] 消息解析失败', data);
    return;
  }

  const { type, data: msgData } = payload || {};
  if (!type) return;

  switch (type) {
    case 'chat_message': {
      await handleIncomingChatMessage(msgData);

      break;
    }
    case 'message_read': {
      // 对方已读回执：更新自己发送的消息为已读状态
      handleMessageReadReceipt(msgData);

      break;
    }
    case 'notice_publish': {
      // 公告发布：刷新公告列表
      await loadNotices();
      playNotificationSound();

      break;
    }
    case 'system_notification': {
      // 系统通知：刷新未读数和通知列表
      await Promise.all([loadNotificationUnread(), loadNotifications()]);
      playNotificationSound();

      break;
    }
    // No default
  }
}

// 处理"对方已读"回执
function handleMessageReadReceipt(msg: any) {
  if (!msg || !Array.isArray(msg.messageIds) || msg.messageIds.length === 0)
    return;

  // 如果当前正在查看该会话，更新消息列表中匹配的消息为已读
  if (
    activeSessionId.value &&
    String(msg.sessionId) === String(activeSessionId.value)
  ) {
    const idSet = new Set(msg.messageIds.map(String));
    for (const m of messageList.value) {
      if (idSet.has(String(m.messageId || m.id))) {
        m.readStatus = 1;
        m.readTime = msg.readTime || new Date().toISOString();
      }
    }
  }
}

// 处理新收到的聊天消息
async function handleIncomingChatMessage(msg: any) {
  if (!msg) return;

  // 判断是否为自己发送的消息（多端同步场景下，自己其他端的发送也会推送过来）
  // 后端 UserLoginVO 返回的字段是 id（非 userId），这里同时兼容两种字段名
  const userInfo: any = userStore.userInfo;
  const currentUserId = userInfo?.id ?? userInfo?.userId;
  const isFromSelf =
    currentUserId !== null &&
    currentUserId !== undefined &&
    String(msg.senderId) === String(currentUserId);

  // 仅当他人发来的消息才播放提醒音
  if (!isFromSelf) {
    playNotificationSound();
  }

  // 如果推送附带了 unreadCount，直接更新对应会话的未读数（避免再拉一次列表）
  if (typeof msg.unreadCount === 'number' && !isFromSelf) {
    const session = sessionList.value.find(
      (s: any) => s.sessionId === msg.sessionId,
    );
    if (session) {
      session.unreadCount = msg.unreadCount;
    } else {
      // 会话不在本地列表中，拉取一次
      await loadSessions();
    }
  } else {
    // 自己发的消息只需刷新会话最后一条消息预览
    await loadSessions();
  }

  // 如果当前正在查看该消息所属会话，追加到消息列表
  if (
    activeSessionId.value &&
    String(msg.sessionId) === String(activeSessionId.value)
  ) {
    // 检查消息是否已在列表中（避免重复）
    const exists = messageList.value.some(
      (m: any) => String(m.messageId || m.id) === String(msg.messageId),
    );
    if (!exists) {
      messageList.value.push({
        messageId: msg.messageId,
        sessionId: msg.sessionId,
        senderId: msg.senderId,
        senderNickname: msg.senderNickname,
        senderAvatar: msg.senderAvatar,
        content: msg.content,
        contentType: msg.contentType,
        message_type: msg.messageType,
        fileUrl: msg.fileUrl,
        fileName: msg.fileName,
        isMine: isFromSelf,
        senderType: msg.messageType === 2 ? 1 : 2,
        sendTime: msg.sendTime,
        readStatus: msg.readStatus ?? 0,
        readTime: msg.readTime || '',
      });
      scrollToBottom();
    }
    // 当前会话标记已读（接收方打开了会话）
    if (!isFromSelf) {
      try {
        await markReadApi({ sessionId: activeSessionId.value });
        const session = sessionList.value.find(
          (s: any) => s.sessionId === activeSessionId.value,
        );
        if (session) session.unreadCount = 0;
        // 通知右上角铃铛实时刷新
        notifyBellRefresh();
      } catch {
        // 忽略
      }
    }
  }
}

const filteredNotifications = computed(() => {
  if (activeNotifType.value === 0) return notificationList.value;
  return notificationList.value.filter((n) => n.type === activeNotifType.value);
});

const chatList = computed(() => {
  const users = [...allUsers.value];
  const sessionMap = new Map<string, any>();
  for (const s of sessionList.value) {
    sessionMap.set(s.sessionName, s);
  }

  const list = users.map((user) => {
    const name = user.nickName || user.userName;
    const session = sessionMap.get(name);
    return {
      ...user,
      displayName: name,
      deptName: user.depts?.[0]?.deptName || '',
      lastMessage: session?.lastMessageContent || '',
      lastMessageTime: session?.lastMessageTime || '',
      unreadCount: session?.unreadCount || 0,
      sessionId: session?.sessionId || null,
    };
  });

  list.sort((a, b) => {
    if (a.lastMessageTime && !b.lastMessageTime) return -1;
    if (!a.lastMessageTime && b.lastMessageTime) return 1;
    if (a.lastMessageTime && b.lastMessageTime) {
      return (
        new Date(b.lastMessageTime).getTime() -
        new Date(a.lastMessageTime).getTime()
      );
    }
    return 0;
  });

  const keyword = searchKeyword.value.trim().toLowerCase();
  if (!keyword) return list;
  return list.filter(
    (u) =>
      (u.displayName || '').toLowerCase().includes(keyword) ||
      (u.userName || '').toLowerCase().includes(keyword) ||
      (u.deptName || '').toLowerCase().includes(keyword),
  );
});

async function loadNotifications() {
  notifLoading.value = true;
  try {
    const res = await getNotificationListApi({
      page: 1,
      pageSize: 100,
    });
    notificationList.value = res.list || [];
  } catch (error) {
    console.error('加载通知失败', error);
  } finally {
    notifLoading.value = false;
  }
}

async function loadNotificationUnread() {
  try {
    const res = await getNotificationUnreadCountApi();
    notificationUnread.value = res || 0;
  } catch (error) {
    console.error('加载未读通知数失败', error);
  }
}

async function loadUsers() {
  userLoading.value = true;
  try {
    const res: any = await getColleagueListApi({
      page: 1,
      pageSize: 200,
    });
    allUsers.value = Array.isArray(res) ? res : res.list || res.records || [];
  } catch (error) {
    console.error('加载同事列表失败', error);
  } finally {
    userLoading.value = false;
  }
}

async function loadSessions() {
  try {
    const res: any = await getSessionListApi({ page: 1, pageSize: 100 });
    sessionList.value = res.list || [];
  } catch (error) {
    console.error('加载会话列表失败', error);
  }
}

async function handleSelectChat(user: any) {
  activeChatId.value = user.id;
  messageList.value = [];
  leftTab.value = 'colleague';

  try {
    const res: any = await startSessionApi({ receiverId: user.id });
    if (res?.sessionId) {
      activeSessionId.value = res.sessionId;
      await loadMessages(res.sessionId);
      // 标记已读：清除该会话的未读红点
      try {
        await markReadApi({ sessionId: res.sessionId });
        // 更新本地会话列表中的未读数
        const session = sessionList.value.find(
          (s: any) => s.sessionId === res.sessionId,
        );
        if (session) {
          session.unreadCount = 0;
        }
        // 通知右上角铃铛实时刷新
        notifyBellRefresh();
      } catch (error) {
        console.warn('[标记已读] 失败', error);
      }
      await loadSessions();
    } else {
      activeSessionId.value = null;
      message.error('创建会话失败：未返回会话ID');
    }
  } catch (error: any) {
    console.error('[选择同事] 加载消息失败', error);
    const errMsg =
      error?.response?.data?.msg || error?.message || '创建会话失败';
    message.error(errMsg);
    activeSessionId.value = null;
  }
}

async function loadMessages(sessionId: string) {
  messageLoading.value = true;
  try {
    const res = await getMessageListApi({
      sessionId,
      page: 1,
      pageSize: 50,
    });
    messageList.value = (res.list || []).toReversed();
    scrollToBottom();
  } catch (error) {
    console.error('加载消息失败', error);
  } finally {
    messageLoading.value = false;
  }
}

function scrollToBottom() {
  setTimeout(() => {
    if (chatContainerRef.value) {
      chatContainerRef.value.scrollTop = chatContainerRef.value.scrollHeight;
    }
  }, 100);
}

function handleChooseImage() {
  imageInputRef.value?.click();
}

async function handleImageChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file || !activeSessionId.value) return;

  if (!file.type.startsWith('image/')) {
    message.warning('请选择图片文件');
    return;
  }

  uploadingImage.value = true;
  try {
    const res: any = await uploadFileApi(file, 'chat');
    const fileUrl = res?.url || res?.fileUrl || '';
    if (!fileUrl) {
      message.error('上传失败');
      return;
    }

    const tempId = `temp_${Date.now()}`;
    messageList.value.push({
      messageId: tempId,
      id: tempId,
      content: '[图片]',
      contentType: CONTENT_TYPE_IMAGE,
      fileUrl,
      fileName: file.name,
      isMine: true,
      senderType: 2,
      messageType: 2,
      sendTime: new Date().toISOString(),
      readStatus: 0,
      readTime: '',
    });
    scrollToBottom();

    await sendMessageApi({
      sessionId: activeSessionId.value,
      content: '[图片]',
      contentType: CONTENT_TYPE_IMAGE,
      fileUrl,
      fileName: file.name,
    });
    await loadMessages(activeSessionId.value);
    await loadSessions();
  } catch (error) {
    console.error('发送图片失败', error);
    message.error('发送图片失败');
  } finally {
    uploadingImage.value = false;
    if (imageInputRef.value) {
      imageInputRef.value.value = '';
    }
  }
}

async function handleSendMessage() {
  const content = messageInput.value.trim();
  if (!content || !activeChatId.value || !activeSessionId.value) return;

  try {
    const tempId = `temp_${Date.now()}`;
    messageList.value.push({
      messageId: tempId,
      id: tempId,
      content,
      contentType: CONTENT_TYPE_TEXT,
      isMine: true,
      senderType: 2,
      messageType: 2,
      sendTime: new Date().toISOString(),
      readStatus: 0,
      readTime: '',
    });
    messageInput.value = '';
    scrollToBottom();

    await sendMessageApi({
      sessionId: activeSessionId.value,
      content,
    });
    await loadMessages(activeSessionId.value);
    await loadSessions();
  } catch (error) {
    console.error('发送消息失败', error);
    window.$message?.error('发送失败');
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSendMessage();
  }
}

function getAvatarText(user: any) {
  return user.nickName?.charAt(0) || user.userName?.charAt(0) || '?';
}

function getAvatarColor(userId: number) {
  const colors = [
    '#1890ff',
    '#52c41a',
    '#faad14',
    '#f5222d',
    '#722ed1',
    '#13c2c2',
    '#eb2f96',
    '#fa8c16',
  ];
  return colors[userId % colors.length];
}

function formatTime(timeStr: string) {
  if (!timeStr) return '';
  const d = new Date(timeStr);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  const oneDay = 24 * 60 * 60 * 1000;

  if (diff < 60 * 1000) return '刚刚';
  if (diff < 60 * 60 * 1000) return `${Math.floor(diff / 60_000)}分钟前`;
  if (diff < oneDay)
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`;
  if (diff < oneDay * 2) return '昨天';
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

// 格式化消息发送时间（微信 PC 风格）
function formatMessageTime(timeStr: string): string {
  if (!timeStr) return '';
  let d: Date;
  try {
    // 后端可能返回 "2026-07-25 05:15:08" 或 ISO 8601 格式
    d = new Date(timeStr.replace(' ', 'T'));
  } catch {
    return '';
  }
  if (Number.isNaN(d.getTime())) return '';

  const pad = (n: number) => n.toString().padStart(2, '0');
  const hhmm = `${pad(d.getHours())}:${pad(d.getMinutes())}`;

  const now = new Date();
  const todayStart = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate(),
  ).getTime();
  const msgStart = new Date(
    d.getFullYear(),
    d.getMonth(),
    d.getDate(),
  ).getTime();
  const dayDiff = Math.round((todayStart - msgStart) / (24 * 60 * 60 * 1000));

  if (dayDiff === 0) return hhmm; // 今天
  if (dayDiff === 1) return `昨天 ${hhmm}`; // 昨天
  if (dayDiff === 2) return `前天 ${hhmm}`; // 前天
  if (now.getFullYear() === d.getFullYear()) {
    return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${hhmm}`; // 今年
  }
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${hhmm}`; // 更早
}

// 判断是否需要显示时间分隔条（与上一条消息间隔超过 5 分钟）
function shouldShowTimeSeparator(currentMsg: any, prevMsg: any): boolean {
  if (!prevMsg) return true;
  const cur = new Date((currentMsg.sendTime || '').replace(' ', 'T')).getTime();
  const prev = new Date((prevMsg.sendTime || '').replace(' ', 'T')).getTime();
  if (Number.isNaN(cur) || Number.isNaN(prev)) return true;
  return cur - prev > 5 * 60 * 1000; // 5 分钟
}

function getNotifTypeName(type: number) {
  const item = NOTIFICATION_TYPES.find((t) => t.type === type);
  return item?.name || '系统消息';
}

function getNotifTypeColor(type: number) {
  const item = NOTIFICATION_TYPES.find((t) => t.type === type);
  return item?.color || '#1677ff';
}

async function handleReadAllNotif() {
  try {
    await readAllNotificationApi();
    notificationList.value.forEach((n: any) => (n.isRead = 1));
    notificationUnread.value = 0;
    // 通知右上角铃铛实时刷新
    notifyBellRefresh();
  } catch (error) {
    console.error('全部已读失败', error);
  }
}

// ===== 公告相关函数 =====

async function loadNotices() {
  noticeLoading.value = true;
  try {
    const res: any = await getMyNoticeListApi({ page: 1, pageSize: 100 });
    // 后端 ResultPage 返回 items 字段（非 list）
    noticeList.value = res?.items || res?.list || [];
  } catch (error) {
    console.error('加载公告失败', error);
  } finally {
    noticeLoading.value = false;
  }
}

// 展开/折叠公告，首次查看时标记已读：左侧"公司公告"徽标 + 顶部铃铛各减1
async function toggleNoticeExpand(item: any) {
  const itemId = Number(item.id);
  if (expandedNoticeId.value === itemId) {
    expandedNoticeId.value = null;
    return;
  }
  expandedNoticeId.value = itemId;
  // 未读公告：本地立即标记已读 → 左侧徽标减1 + 顶部铃铛减1
  if (item.is_read !== 1) {
    item.is_read = 1; // 本地立即更新，noticeUnreadCount 立即减1 → 左侧徽标减1
    try {
      await readNoticeApi(itemId);
      // API 成功后再通知顶部铃铛刷新 → 顶部未读数减1
      notifyBellRefresh();
    } catch (error) {
      console.error('[Notice] 标记公告已读失败', error);
      // 失败时回滚本地状态
      item.is_read = 0;
    }
  }
}

async function handleReadAllNotices() {
  try {
    await readAllNoticeApi();
    noticeList.value.forEach((n: any) => (n.is_read = 1));
    notifyBellRefresh();
  } catch (error) {
    console.error('公告全部已读失败', error);
  }
}

onMounted(async () => {
  await Promise.all([
    loadNotificationUnread(),
    loadNotifications(),
    loadNotices(),
    loadUsers(),
    loadSessions(),
  ]);
  // 建立 WebSocket 连接
  connectWebSocket();
  // 根据路由 query 参数（来自右上角铃铛点击）打开对应窗口
  await openFromRouteQuery();
});

onUnmounted(() => {
  disconnectWebSocket();
});

watch(
  () => [leftTab.value, chatList.value.length],
  async () => {
    if (
      leftTab.value === 'colleague' &&
      chatList.value.length > 0 &&
      !activeChatId.value
    ) {
      const firstUser = chatList.value[0];
      if (firstUser) {
        await handleSelectChat(firstUser);
      }
    }
  },
  { immediate: true },
);

function ensureString(val: any): string {
  if (val === null || val === undefined) return '';
  if (typeof val === 'string') return val;
  if (typeof val === 'object') {
    try {
      return JSON.stringify(val);
    } catch {
      return String(val);
    }
  }
  return String(val);
}
</script>

<template>
  <Page auto-content-height class="!p-0 !m-0">
    <div class="msg-container">
      <div class="msg-sidebar">
        <div class="sidebar-header">
          <div class="header-tabs">
            <div
              class="header-tab"
              :class="{ active: leftTab === 'notification' }"
              @click="leftTab = 'notification'"
            >
              <SvgBellIcon :size="16" />
              <span>系统消息</span>
              <Badge
                v-if="notificationUnread > 0"
                :count="notificationUnread"
                size="small"
                :number-style="{
                  backgroundColor: '#ff4d4f',
                  transform: 'scale(0.8)',
                }"
                :offset="['-4px', 0]"
              />
            </div>
            <div
              class="header-tab"
              :class="{ active: leftTab === 'colleague' }"
              @click="leftTab = 'colleague'"
            >
              <span>同事消息</span>
            </div>
          </div>
        </div>

        <div v-if="leftTab === 'notification'" class="notif-sidebar">
          <div class="notif-type-list">
            <div
              v-for="tab in NOTIFICATION_TYPES"
              :key="tab.type"
              class="notif-type-item"
              :class="{ active: activeNotifType === tab.type }"
              @click="activeNotifType = tab.type"
            >
              <span
                class="notif-type-dot"
                :style="{ backgroundColor: tab.color }"
              ></span>
              <span class="notif-type-name">{{ tab.name }}</span>
              <Badge
                v-if="getUnreadCountForTab(tab.type) > 0"
                :count="getUnreadCountForTab(tab.type)"
                size="small"
                :number-style="{
                  backgroundColor: '#ff4d4f',
                  transform: 'scale(0.8)',
                }"
              />
            </div>
          </div>
        </div>

        <div v-else class="colleague-sidebar">
          <div class="search-bar">
            <Input v-model:value="searchKeyword" placeholder="搜索同事">
              <template #prefix>
                <LucideSearch :size="16" />
              </template>
            </Input>
          </div>
          <div class="user-list">
            <Spin v-if="userLoading" size="small" class="list-loading" />
            <template v-else>
              <div
                v-for="user in chatList"
                :key="user.id"
                class="msg-list-item"
                :class="{ active: activeChatId === user.id }"
                @click="handleSelectChat(user)"
              >
                <Avatar
                  class="msg-avatar"
                  :src="user.avatar || undefined"
                  :style="{
                    backgroundColor: user.avatar
                      ? 'transparent'
                      : getAvatarColor(user.id),
                  }"
                >
                  {{ getAvatarText(user) }}
                </Avatar>
                <div class="msg-content">
                  <div class="msg-title-row">
                    <span class="msg-title">
                      {{ user.displayName }}
                      <span v-if="user.deptName" class="dept-tag">{{
                        user.deptName
                      }}</span>
                    </span>
                    <span class="msg-time">{{
                      formatTime(user.lastMessageTime)
                    }}</span>
                  </div>
                  <div class="msg-desc">
                    {{ user.lastMessage || '暂无消息' }}
                  </div>
                </div>
                <div v-if="user.unreadCount > 0" class="unread-badge">
                  {{ user.unreadCount > 99 ? '99+' : user.unreadCount }}
                </div>
              </div>
              <Empty
                v-if="chatList.length === 0"
                description="暂无同事"
                class="list-empty"
              />
            </template>
          </div>
        </div>
      </div>

      <div class="msg-main">
        <template v-if="leftTab === 'notification'">
          <!-- ===== 公告列表（activeNotifType === 1） ===== -->
          <template v-if="activeNotifType === 1">
            <div class="chat-header">
              <div class="chat-title">
                <SvgBellIcon class="mr-2" />
                公司公告
              </div>
              <div class="header-actions">
                <Button
                  type="link"
                  size="small"
                  :disabled="noticeUnreadCount === 0"
                  @click="handleReadAllNotices"
                >
                  全部已读
                </Button>
              </div>
            </div>
            <div class="notification-detail notice-list-detail">
              <Spin v-if="noticeLoading" class="notif-loading" />
              <template v-else>
                <div
                  v-for="item in sortedNoticeList"
                  :key="item.id"
                  class="notice-card"
                  :class="{
                    'is-unread': item.is_read !== 1,
                    'is-expanded': expandedNoticeId === Number(item.id),
                  }"
                >
                  <div
                    class="notice-card-header"
                    @click="toggleNoticeExpand(item)"
                  >
                    <div class="notice-card-title-row">
                      <span
                        v-if="item.is_read !== 1"
                        class="notice-unread-dot"
                      ></span>
                      <span class="notice-card-title">{{
                        item.title || '系统公告'
                      }}</span>
                      <span class="notice-expand-icon">
                        {{
                          expandedNoticeId === Number(item.id) ? '收起' : '查看'
                        }}
                      </span>
                    </div>
                    <div class="notice-card-meta">
                      <span class="notice-publisher">{{
                        item.publish_name || '系统'
                      }}</span>
                      <span class="notice-card-time">{{
                        formatTime(item.publish_time)
                      }}</span>
                    </div>
                  </div>
                  <!-- 展开状态：富文本内容 -->
                  <transition name="notice-expand">
                    <div
                      v-show="expandedNoticeId === Number(item.id)"
                      class="notice-card-content"
                    >
                      <!-- eslint-disable-next-line vue/no-v-html -- 后端富文本公告内容，可信来源 -->
                      <div v-html="item.content"></div>
                    </div>
                  </transition>
                </div>
                <Empty
                  v-if="sortedNoticeList.length === 0"
                  description="暂无公告"
                  class="list-empty"
                />
              </template>
            </div>
          </template>

          <!-- ===== 其他通知列表 ===== -->
          <template v-else>
            <div class="chat-header">
              <div class="chat-title">
                <SvgBellIcon class="mr-2" />
                {{ getNotifTypeName(activeNotifType) }}
              </div>
              <div class="header-actions">
                <Button type="link" size="small" @click="handleReadAllNotif">
                  全部已读
                </Button>
              </div>
            </div>
            <div class="notification-detail">
              <Spin v-if="notifLoading" class="notif-loading" />
              <template v-else>
                <div
                  v-for="item in filteredNotifications"
                  :key="item.id"
                  class="notification-item"
                >
                  <div
                    class="notif-icon"
                    :style="{
                      backgroundColor: `${getNotifTypeColor(item.type)}20`,
                      color: getNotifTypeColor(item.type),
                    }"
                  >
                    <SvgBellIcon :size="16" />
                  </div>
                  <div class="notif-body">
                    <div class="notif-title-row">
                      <span class="notif-title">{{ item.title }}</span>
                      <Badge v-if="!item.isRead" color="red" size="small" />
                      <span class="notif-time">{{
                        formatTime(item.createTime)
                      }}</span>
                    </div>
                    <div class="notif-content">{{ item.content }}</div>
                    <div
                      class="notif-type-tag"
                      :style="{
                        color: getNotifTypeColor(item.type),
                        backgroundColor: `${getNotifTypeColor(item.type)}15`,
                      }"
                    >
                      {{ getNotifTypeName(item.type) }}
                    </div>
                    <div v-if="item.linkUrl" class="notif-link">
                      <a :href="item.linkUrl" target="_blank">查看详情 →</a>
                    </div>
                  </div>
                </div>
                <Empty
                  v-if="filteredNotifications.length === 0"
                  description="暂无通知"
                />
              </template>
            </div>
          </template>
        </template>

        <template v-else>
          <div v-if="activeChatId" class="chat-header">
            <Avatar
              class="chat-header-avatar"
              :src="
                allUsers.find((u) => u.id === activeChatId)?.avatar || undefined
              "
              :style="{
                backgroundColor: allUsers.find((u) => u.id === activeChatId)
                  ?.avatar
                  ? 'transparent'
                  : getAvatarColor(activeChatId as number),
              }"
            >
              {{ getAvatarText(allUsers.find((u) => u.id === activeChatId)) }}
            </Avatar>
            <div>
              <div class="chat-title">
                {{
                  allUsers.find((u) => u.id === activeChatId)?.nickName ||
                  allUsers.find((u) => u.id === activeChatId)?.userName
                }}
              </div>
              <div class="chat-subtitle">
                {{
                  allUsers.find((u) => u.id === activeChatId)?.depts?.[0]
                    ?.deptName || ''
                }}
              </div>
            </div>
          </div>

          <div v-else class="empty-chat">
            <Empty description="选择一位同事开始聊天" />
          </div>

          <div v-if="activeChatId" class="chat-body">
            <div ref="chatContainerRef" class="chat-messages">
              <Spin v-if="messageLoading" class="chat-loading" />
              <template v-else>
                <template
                  v-for="(msg, idx) in messageList"
                  :key="msg.messageId || msg.id"
                >
                  <!-- 时间分隔条（首条消息或距上一条超过 5 分钟时显示） -->
                  <div
                    v-if="shouldShowTimeSeparator(msg, messageList[idx - 1])"
                    class="msg-time-separator"
                  >
                    {{ formatMessageTime(msg.sendTime) }}
                  </div>
                  <div
                    class="chat-msg-row"
                    :class="
                      msg.isMine || msg.senderType === 2
                        ? 'is-mine'
                        : 'is-other'
                    "
                  >
                    <Avatar
                      v-if="!(msg.isMine || msg.senderType === 2)"
                      class="msg-avatar-small"
                      :style="{
                        backgroundColor: getAvatarColor(activeChatId as number),
                      }"
                    >
                      {{
                        getAvatarText(
                          allUsers.find((u) => u.id === activeChatId),
                        )
                      }}
                    </Avatar>

                    <div
                      class="msg-bubble-wrap"
                      :class="{
                        'mine-wrap': msg.isMine || msg.senderType === 2,
                      }"
                    >
                      <template v-if="msg.contentType === 2 || msg.fileUrl">
                        <div
                          class="msg-image-wrapper"
                          :class="{
                            'mine-img': msg.isMine || msg.senderType === 2,
                          }"
                        >
                          <Image :src="msg.fileUrl" class="msg-image" />
                        </div>
                      </template>
                      <template v-else>
                        <div
                          class="msg-bubble"
                          :class="
                            msg.isMine || msg.senderType === 2
                              ? 'mine'
                              : 'other'
                          "
                        >
                          {{ ensureString(msg.content) }}
                        </div>
                      </template>
                      <!-- 自己发的消息显示已读/未读状态（不再显示时间） -->
                      <div
                        v-if="msg.isMine || msg.senderType === 2"
                        class="msg-read-status"
                        :class="{
                          'is-read': msg.readStatus === 1,
                          'is-unread': msg.readStatus !== 1,
                        }"
                      >
                        <span>{{
                          msg.readStatus === 1 ? '已读' : '未读'
                        }}</span>
                      </div>
                      <!-- 对方消息不再显示时间 -->
                    </div>

                    <Avatar
                      v-if="msg.isMine || msg.senderType === 2"
                      class="msg-avatar-small own-avatar"
                      :src="currentUserAvatar"
                      :style="{
                        backgroundColor: currentUserAvatar
                          ? 'transparent'
                          : '#07c160',
                      }"
                    >
                      {{ currentUserRealName.charAt(0) }}
                    </Avatar>
                  </div>
                </template>
                <Empty
                  v-if="messageList.length === 0"
                  description="暂无消息，开始聊天吧"
                  class="chat-empty"
                />
              </template>
            </div>

            <input
              ref="imageInputRef"
              type="file"
              accept="image/*"
              style="display: none"
              @change="handleImageChange"
            />

            <div class="chat-input-area">
              <div class="input-toolbar">
                <Button
                  type="text"
                  size="small"
                  :disabled="uploadingImage"
                  @click="handleChooseImage"
                >
                  <LucideImage :size="16" style="margin-right: 4px" />
                  图片
                </Button>
              </div>
              <Input.TextArea
                v-model:value="messageInput"
                :rows="3"
                placeholder="输入消息，按 Enter 发送"
                @keydown="handleKeydown"
              />
              <div class="input-actions">
                <Button
                  type="primary"
                  :loading="uploadingImage"
                  @click="handleSendMessage"
                >
                  发送
                  <LucideArrowRight :size="16" style="margin-left: 4px" />
                </Button>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </Page>
</template>

<style scoped>
/* ===== WeChat PC Design System ===== */

/* Color tokens:
   Primary: #07C160 (WeChat green)
   Sidebar: #F4F4F4  | Main: #FFFFFF
   Hover: #E8E8E8    | Active: #D6D6D6
   Border: #E5E5E5   | My bubble: #95EC69
   Text: #1A1A1A / #888 / #B2B2B2
*/

.msg-container {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family:
    'PingFang SC',
    'Microsoft YaHei',
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    Roboto,
    'Helvetica Neue',
    Arial,
    sans-serif;
  background: #fff;
}

/* ===== Sidebar (white background for design consistency) ===== */
.msg-sidebar {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  width: 280px;
  background: #fff;
  border-right: 1px solid #e5e5e5;
}

.sidebar-header {
  background: #fff;
  border-bottom: 1px solid #e5e5e5;
}

.header-tabs {
  display: flex;
}

.header-tab {
  position: relative;
  display: flex;
  flex: 1;
  gap: 5px;
  align-items: center;
  justify-content: center;
  padding: 16px 0;
  font-size: 13px;
  color: #888;
  text-align: center;
  cursor: pointer;
  transition: color 0.15s ease;
}

.header-tab:hover {
  color: #07c160;
}

.header-tab.active {
  font-weight: 500;
  color: #07c160;
}

.header-tab.active::after {
  position: absolute;
  bottom: 0;
  left: 50%;
  width: 24px;
  height: 3px;
  content: '';
  background: #07c160;
  border-radius: 2px;
  transform: translateX(-50%);
}

/* ===== Notification Type List ===== */
.notif-sidebar {
  flex: 1;
  padding: 6px 0;
  overflow-y: auto;
}

.notif-type-list {
  display: flex;
  flex-direction: column;
}

.notif-type-item {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 20px;
  font-size: 13px;
  color: #333;
  cursor: pointer;
  transition: background 0.12s;
}

.notif-type-item:hover {
  background: #e8e8e8;
}

.notif-type-item.active {
  font-weight: 500;
  color: #07c160;
  background: #d6d6d6;
}

.notif-type-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.notif-type-name {
  flex: 1;
}

/* ===== Colleague Sidebar ===== */
.colleague-sidebar {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
}

.search-bar {
  padding: 8px 12px;
  background: #fff;
  border-bottom: 1px solid #e5e5e5;
}

.search-bar :deep(.ant-input-affix-wrapper) {
  background: #fff;
  border-color: #e0e0e0;
  border-radius: 6px;
}

.search-bar :deep(.ant-input-affix-wrapper:focus),
.search-bar :deep(.ant-input-affix-wrapper-focused) {
  border-color: #07c160;
  box-shadow: 0 0 0 2px rgb(7 193 96 / 12%);
}

.user-list {
  flex: 1;
  overflow-y: auto;
}

.list-loading {
  display: flex;
  justify-content: center;
  padding: 20px;
}

.list-empty {
  padding: 40px 0;
}

/* Chat list item - WeChat PC hover/active */
.msg-list-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.12s;
}

.msg-list-item:hover {
  background: #e8e8e8;
}

.msg-list-item.active {
  background: #d6d6d6;
}

/* WeChat PC avatars: 6px radius (not fully round) */
.msg-avatar {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  margin-right: 12px;
  font-size: 14px;
  font-weight: 500;
  color: #fff;
  border-radius: 6px;
}

.msg-avatar :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 6px;
}

.msg-content {
  display: flex;
  flex: 1;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
}

.msg-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.msg-title {
  display: flex;
  gap: 6px;
  align-items: center;
  max-width: 130px;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 14px;
  font-weight: 400;
  color: #1a1a1a;
  white-space: nowrap;
}

.dept-tag {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: normal;
  color: #b2b2b2;
}

.msg-time {
  flex-shrink: 0;
  margin-left: 8px;
  font-size: 11px;
  color: #b2b2b2;
}

.msg-desc {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 12px;
  color: #888;
  white-space: nowrap;
}

.unread-dot {
  position: absolute;
  top: 50%;
  right: 12px;
  min-width: 8px;
  height: 8px;
  background: #fa5151;
  border-radius: 50%;
  transform: translateY(-50%);
}

/* 未读消息数字徽标（替代原 unread-dot 红点） */
.unread-badge {
  position: absolute;
  top: 50%;
  right: 12px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  font-size: 11px;
  font-weight: 500;
  line-height: 18px;
  color: #fff;
  text-align: center;
  background: #fa5151;
  border-radius: 9px;
  box-shadow: 0 1px 3px rgb(250 81 81 / 40%);
  transform: translateY(-50%);
}

/* ===== Main Area ===== */
.msg-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  background: #fff;
}

/* Chat header */
.chat-header {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  padding: 14px 20px;
  background: #fff;
  border-bottom: 1px solid #e5e5e5;
}

.chat-header-avatar {
  flex-shrink: 0;
  width: 38px;
  height: 38px;
  margin-right: 12px;
  font-size: 13px;
  border-radius: 6px;
}

.chat-header-avatar :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 6px;
}

.chat-title {
  display: flex;
  align-items: center;
  font-size: 15px;
  font-weight: 500;
  color: #1a1a1a;
}

.header-actions {
  display: flex;
  align-items: center;
  margin-left: auto;
}

.header-actions :deep(.ant-btn-link) {
  color: #07c160;
}

.chat-subtitle {
  margin-top: 2px;
  font-size: 12px;
  color: #b2b2b2;
}

.empty-chat {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
}

/* ===== Notification Detail ===== */
.notification-detail {
  flex: 1;
  padding: 16px 20px;
  overflow-y: auto;
  background: #fff;
}

.notif-loading {
  display: flex;
  justify-content: center;
  padding: 40px;
}

.notification-item {
  display: flex;
  gap: 12px;
  padding: 14px 16px;
  margin-bottom: 10px;
  background: #f7f7f7;
  border-radius: 8px;
  transition: background 0.12s;
}

.notification-item:hover {
  background: #f0f0f0;
}

.notif-icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
}

.notif-body {
  flex: 1;
  min-width: 0;
}

.notif-title-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}

.notif-title {
  font-size: 14px;
  font-weight: 500;
  color: #1a1a1a;
}

.notif-time {
  flex-shrink: 0;
  margin-left: auto;
  font-size: 12px;
  color: #b2b2b2;
}

.notif-content {
  font-size: 13px;
  line-height: 1.6;
  color: #555;
}

.notif-link {
  margin-top: 8px;
  font-size: 12px;
}

.notif-link a {
  color: #07c160;
}

.notif-type-tag {
  display: inline-block;
  padding: 2px 8px;
  margin-top: 6px;
  font-size: 11px;
  border-radius: 4px;
}

/* ===== Chat Body (white background per requirement) ===== */
.chat-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  background: #fff;
}

/* Chat messages - white background */
.chat-messages {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  background: #fff;
}

.chat-loading {
  display: flex;
  justify-content: center;
  padding: 20px;
}

.chat-empty {
  padding: 60px 0;
}

/* Message rows */
.chat-msg-row {
  display: flex;
  align-items: flex-start;
  margin-bottom: 16px;
  animation: bubble-fade-in 0.2s ease;
}

@keyframes bubble-fade-in {
  from {
    opacity: 0;
    transform: translateY(4px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.chat-msg-row.is-mine {
  justify-content: flex-end;
}

.chat-msg-row.is-other {
  justify-content: flex-start;
}

/* Small avatars in chat - WeChat PC 6px radius */
.msg-avatar-small {
  flex-shrink: 0;
  width: 38px;
  height: 38px;
  margin-right: 10px;
  font-size: 13px;
  border-radius: 6px;
}

.msg-avatar-small :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 6px;
}

/* Own avatar on right side */
.own-avatar {
  order: 2;
  margin-right: 0;
  margin-left: 10px;
}

/* Message bubbles - WeChat PC style with tails */
.msg-bubble {
  position: relative;
  padding: 9px 13px;
  font-size: 14px;
  line-height: 1.6;
  word-break: break-all;
  overflow-wrap: break-word;
  border-radius: 6px;
}

/* Mine bubble - WeChat green */
.msg-bubble.mine {
  color: #1a1a1a;
  background: #95ec69;
}

/* Other bubble - white with subtle border */
.msg-bubble.other {
  color: #1a1a1a;
  background: #fff;
  border: 1px solid #ebebeb;
  box-shadow: 0 1px 1px rgb(0 0 0 / 3%);
}

/* Bubble tails (small triangles pointing to avatars) */
.msg-bubble.other::before {
  position: absolute;
  top: 13px;
  left: -6px;
  width: 0;
  height: 0;
  content: '';
  border-top: 5px solid transparent;
  border-right: 6px solid #fff;
  border-bottom: 5px solid transparent;
  filter: drop-shadow(-1px 0 0 #ebebeb);
}

.msg-bubble.mine::before {
  position: absolute;
  top: 13px;
  right: -6px;
  width: 0;
  height: 0;
  content: '';
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
  border-left: 6px solid #95ec69;
}

/* 消息气泡包装容器：用于承载气泡 + 已读状态标签 */
.msg-bubble-wrap {
  display: flex;
  flex-direction: column;
  max-width: 60%;
}

.msg-bubble-wrap.mine-wrap {
  align-items: flex-end;
}

/* 自己发送消息的已读/未读状态标签 */
.msg-read-status {
  display: flex;
  gap: 4px;
  align-items: center;
  padding: 0 4px;
  margin-top: 4px;
  font-size: 11px;
  line-height: 1;
  user-select: none;
}

.msg-read-status.is-read {
  color: #b2b2b2;
}

.msg-read-status.is-unread {
  color: #b2b2b2;
}

/* 对方消息的时间标签靠左 */
.msg-other-time {
  align-self: flex-start;
}

.msg-send-time {
  font-size: 11px;
}

.msg-read-divider {
  color: #d6d6d6;
}

/* 消息时间分隔条（微信风格：居中灰色文字） */
.msg-time-separator {
  margin: 12px 0 8px;
  font-size: 12px;
  color: #b2b2b2;
  text-align: center;
  user-select: none;
}

/* ===== Chat Input Area ===== */
.chat-input-area {
  padding: 10px 16px 12px;
  background: #fff;
  border-top: 1px solid #e5e5e5;
}

.chat-input-area :deep(.ant-input) {
  border-color: #e0e0e0;
}

.chat-input-area :deep(.ant-input:focus),
.chat-input-area :deep(.ant-input-focused) {
  border-color: #07c160;
  box-shadow: 0 0 0 2px rgb(7 193 96 / 10%);
}

.input-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

/* WeChat green send button */
.input-actions :deep(.ant-btn-primary) {
  background: #07c160;
  border-color: #07c160;
}

.input-actions :deep(.ant-btn-primary:hover) {
  background: #06ad56 !important;
  border-color: #06ad56 !important;
}

.input-toolbar {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.input-toolbar :deep(.ant-btn-text) {
  color: #555;
}

.input-toolbar :deep(.ant-btn-text:hover) {
  color: #07c160;
  background: #f0f0f0 !important;
}

/* Image messages */
.msg-image-wrapper {
  max-width: 240px;
  overflow: hidden;
  cursor: pointer;
  border-radius: 6px;
}

.msg-image-wrapper.mine-img {
  order: 1;
}

.msg-image {
  display: block;
  width: 100%;
  border-radius: 6px;
}

/* ===== Notice Cards (collapsible announcement list) ===== */
.notice-list-detail {
  padding: 16px 20px;
}

.notice-card {
  margin-bottom: 10px;
  overflow: hidden;
  background: #f7f7f7;
  border-left: 3px solid transparent;
  border-radius: 8px;
  transition:
    background 0.2s ease,
    box-shadow 0.2s ease;
}

.notice-card:hover {
  background: #f0f0f0;
}

.notice-card.is-unread {
  background: #f0fdf4;
  border-left-color: #07c160;
}

.notice-card.is-unread:hover {
  background: #e8fce8;
}

.notice-card.is-expanded {
  background: #fff;
  box-shadow: 0 2px 8px rgb(0 0 0 / 6%);
}

.notice-card-header {
  padding: 14px 16px;
  cursor: pointer;
  user-select: none;
}

.notice-card-title-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}

.notice-unread-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  background: #fa5151;
  border-radius: 50%;
}

.notice-card-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 14px;
  font-weight: 500;
  color: #1a1a1a;
  white-space: nowrap;
}

.notice-expand-icon {
  flex-shrink: 0;
  padding: 2px 8px;
  font-size: 12px;
  color: #07c160;
  background: rgb(7 193 96 / 8%);
  border-radius: 4px;
  transition: background 0.15s;
}

.notice-expand-icon:hover {
  background: rgb(7 193 96 / 15%);
}

.notice-card-meta {
  display: flex;
  gap: 12px;
  align-items: center;
  font-size: 12px;
  color: #b2b2b2;
}

.notice-publisher {
  color: #888;
}

.notice-card-time {
  margin-left: auto;
}

.notice-card-content {
  padding: 0 16px 16px;
  padding-top: 14px;
  font-size: 14px;
  line-height: 1.8;
  color: #333;
  border-top: 1px solid #f0f0f0;
}

.notice-card-content :deep(img) {
  max-width: 100%;
  margin: 8px 0;
  border-radius: 4px;
}

.notice-card-content :deep(p) {
  margin: 8px 0;
}

/* 展开/折叠过渡动画 */
.notice-expand-enter-active,
.notice-expand-leave-active {
  overflow: hidden;
  transition: all 0.25s ease;
}

.notice-expand-enter-from,
.notice-expand-leave-to {
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
  opacity: 0;
}

.notice-expand-enter-to,
.notice-expand-leave-from {
  max-height: 2000px;
  opacity: 1;
}

/* ===== Scrollbars (WeChat PC style: thin and subtle) ===== */
.chat-messages::-webkit-scrollbar,
.user-list::-webkit-scrollbar,
.notification-detail::-webkit-scrollbar,
.notice-list-detail::-webkit-scrollbar,
.notif-sidebar::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.chat-messages::-webkit-scrollbar-thumb,
.user-list::-webkit-scrollbar-thumb,
.notification-detail::-webkit-scrollbar-thumb,
.notice-list-detail::-webkit-scrollbar-thumb,
.notif-sidebar::-webkit-scrollbar-thumb {
  background: #c8c8c8;
  border-radius: 3px;
}

.chat-messages::-webkit-scrollbar-thumb:hover,
.user-list::-webkit-scrollbar-thumb:hover,
.notification-detail::-webkit-scrollbar-thumb:hover,
.notice-list-detail::-webkit-scrollbar-thumb:hover,
.notif-sidebar::-webkit-scrollbar-thumb:hover {
  background: #b0b0b0;
}

.chat-messages::-webkit-scrollbar-track,
.user-list::-webkit-scrollbar-track,
.notification-detail::-webkit-scrollbar-track,
.notice-list-detail::-webkit-scrollbar-track,
.notif-sidebar::-webkit-scrollbar-track {
  background: transparent;
}
</style>
