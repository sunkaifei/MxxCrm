<script lang="ts" setup>
import type { NotificationItem } from '@vben/layouts';

import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { AuthenticationLoginExpiredModal } from '@vben/common-ui';
import { VBEN_GITHUB_URL } from '@vben/constants';
import { useWatermark } from '@vben/hooks';
import { CircleHelp, SvgGithubIcon } from '@vben/icons';
import {
  BasicLayout,
  Notification,
  UserDropdown,
} from '@vben/layouts';
import { preferences, usePreferences } from '@vben/preferences';
import { useAccessStore, useUserStore } from '@vben/stores';
import { openWindow } from '@vben/utils';
import dayjs from 'dayjs';

import { $t } from '#/locales';
import {
  getNotificationListApi,
  readAllNotificationApi,
  readNotificationApi,
  getMyNoticeListApi,
  readNoticeApi,
  readAllNoticeApi,
} from '#/api';
import {
  getUnreadCountApi,
  getSessionListApi,
  markReadApi,
  type ChatSessionDTO,
} from '#/api/core/message/chat';
import { useAuthStore } from '#/store';
import LoginForm from '#/views/_core/authentication/login.vue';

const notifications = ref<NotificationItem[]>([]);
const chatSessions = ref<ChatSessionDTO[]>([]);

// 系统通知类型数字 -> 名称映射（与 message/index.vue NOTIFICATION_TYPES 保持一致）
const NOTIF_TYPE_NAMES: Record<number, string> = {
  1: '公司公告',
  2: '客户分配',
  3: '报价审批',
  4: '订单审批',
  5: '合同审批',
  6: '发货通知',
  7: '回款提醒',
  8: '财务信息',
};

function getNotifTypeName(type: any): string {
  const t = Number(type);
  if (!Number.isNaN(t) && NOTIF_TYPE_NAMES[t]) return NOTIF_TYPE_NAMES[t];
  if (typeof type === 'string' && type) return type;
  return '系统通知';
}

// ===== 聊天总未读数（右上角铃铛实时提醒） =====
const chatUnreadCount = ref(0);
let chatWs: WebSocket | null = null;
let chatWsReconnectTimer: ReturnType<typeof setTimeout> | null = null;
let chatWsReconnectAttempts = 0;
const CHAT_WS_RECONNECT_MAX = 10;
const CHAT_WS_RECONNECT_INTERVAL = 3000;

// 公告提示音元素（懒初始化，复用单例避免每次新建 Audio 对象）
let noticeAudio: HTMLAudioElement | null = null;
function playNoticeSound() {
  try {
    if (!noticeAudio) {
      noticeAudio = new Audio('/sounds/news.mp3');
      noticeAudio.preload = 'auto';
    }
    noticeAudio.currentTime = 0;
    noticeAudio.play().catch((e) => {
      console.debug('[Notice] 提示音播放被阻止', e);
    });
  } catch (e) {
    console.warn('[Notice] 提示音初始化失败', e);
  }
}

const router = useRouter();
const userStore = useUserStore();
const authStore = useAuthStore();
const accessStore = useAccessStore();
const { destroyWatermark, updateWatermark } = useWatermark();
const { isDark } = usePreferences();

// 合并后的总未读数（系统通知未读 + 聊天未读）
const totalUnreadCount = computed(() => {
  const notifUnread = notifications.value.filter((n) => !n.isRead).length;
  return notifUnread + chatUnreadCount.value;
});

// 系统通知按类型分组（仅未读）
const groupedNotifications = computed(() => {
  const groups = new Map<string, { type: string; count: number; items: any[] }>();
  for (const n of notifications.value) {
    if (n.isRead) continue;
    const typeName = getNotifTypeName((n as any).type);
    if (!groups.has(typeName)) {
      groups.set(typeName, { type: typeName, count: 0, items: [] });
    }
    const g = groups.get(typeName)!;
    g.count++;
    g.items.push(n);
  }
  return Array.from(groups.values());
});

// 下拉列表显示的混合内容（聊天会话 + 分组通知）
const mergedNotifications = computed<NotificationItem[]>(() => {
  const list: NotificationItem[] = [];

  // 1. 聊天会话未读（按最后消息时间倒序），显示发送人 + 未读条数
  const unreadSessions = chatSessions.value
    .filter((s) => s.unreadCount > 0)
    .sort((a, b) => {
      const ta = a.lastMessageTime ? new Date(a.lastMessageTime).getTime() : 0;
      const tb = b.lastMessageTime ? new Date(b.lastMessageTime).getTime() : 0;
      return tb - ta;
    });
  for (const s of unreadSessions) {
    list.push({
      id: `chat_${s.sessionId}`,
      avatar: s.avatarUrl || '/static/images/system_avatar.png',
      title: `${s.sessionName} 给您发来 ${s.unreadCount} 条新消息`,
      message: s.lastMessageContent || '',
      date: s.lastMessageTime ? formatRelativeTime(s.lastMessageTime) : '',
      isRead: false,
      link: '/company/message',
    } as NotificationItem);
  }

  // 2. 系统通知按类型分组（避免「发货通知通知」重复，结尾已含「通知」则不再追加）
  for (const g of groupedNotifications.value) {
    const title = g.type.endsWith('通知') ? g.type : `${g.type}通知`;
    list.push({
      id: `group_${g.type}`,
      avatar: '',
      title,
      message: `共 ${g.count} 条未读`,
      date: '',
      isRead: false,
      link: '/company/message',
    } as NotificationItem);
  }

  return list;
});

const showDot = computed(() => totalUnreadCount.value > 0);

function getChatWsUrl(): string {
  const token = accessStore.accessToken || '';
  const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
  // 开发环境连后端 8080，生产环境同源
  const host = import.meta.env.DEV ? '192.168.1.3:8080' : window.location.host;
  return `${protocol}://${host}/ws/message?token=${encodeURIComponent(token)}`;
}

async function refreshChatUnreadCount() {
  try {
    const count = await getUnreadCountApi();
    console.log('[ChatWS] 当前聊天未读总数:', count);
    chatUnreadCount.value = typeof count === 'number' ? count : 0;
  } catch (e) {
    console.warn('[ChatWS] 获取未读数失败', e);
  }
}

function connectChatWs() {
  if (chatWs && (chatWs.readyState === WebSocket.OPEN || chatWs.readyState === WebSocket.CONNECTING)) {
    return;
  }
  try {
    chatWs = new WebSocket(getChatWsUrl());
  } catch (e) {
    console.warn('[ChatWS] 创建失败', e);
    scheduleChatWsReconnect();
    return;
  }

  chatWs.onopen = () => {
    console.log('[ChatWS] 连接已建立');
    chatWsReconnectAttempts = 0;
    // 连接成功后立即拉取一次未读数
    refreshChatUnreadCount();
  };

  chatWs.onmessage = (event) => {
    let payload: any;
    try {
      payload = JSON.parse(event.data);
    } catch (e) {
      return;
    }
    // 收到聊天消息或已读回执时刷新未读总数和会话列表
    if (payload?.type === 'chat_message' || payload?.type === 'message_read') {
      console.log('[ChatWS] 收到推送，刷新未读数和会话列表', payload?.type);
      loadChatSessions();
    }
    // 收到公告发布推送：刷新系统通知列表 + 播放提示音
    if (payload?.type === 'notice_publish') {
      console.log('[ChatWS] 收到公告发布推送', payload?.data);
      loadSystemNotifications();
      playNoticeSound();
    }
  };

  chatWs.onclose = (e) => {
    console.log('[ChatWS] 关闭', e.code, e.reason);
    chatWs = null;
    if (e.code !== 1000) {
      scheduleChatWsReconnect();
    }
  };

  chatWs.onerror = (e) => {
    console.error('[ChatWS] 错误', e);
  };
}

function scheduleChatWsReconnect() {
  if (chatWsReconnectTimer) return;
  if (chatWsReconnectAttempts >= CHAT_WS_RECONNECT_MAX) return;
  chatWsReconnectAttempts++;
  const delay = CHAT_WS_RECONNECT_INTERVAL * Math.min(chatWsReconnectAttempts, 5);
  chatWsReconnectTimer = setTimeout(() => {
    chatWsReconnectTimer = null;
    connectChatWs();
  }, delay);
}

function disconnectChatWs() {
  if (chatWsReconnectTimer) {
    clearTimeout(chatWsReconnectTimer);
    chatWsReconnectTimer = null;
  }
  if (chatWs) {
    chatWs.onclose = null;
    chatWs.close(1000, 'page unmount');
    chatWs = null;
  }
}

// 路由切换时也刷新一次未读数
watch(
  () => router.currentRoute.value.path,
  () => {
    refreshChatUnreadCount();
  },
);

const menus = computed(() => [
  {
    handler: () => {
      router.push({ name: 'Profile' });
    },
    icon: 'lucide:user',
    text: $t('page.auth.profile'),
  },
]);

const avatar = computed(() => {
  return userStore.userInfo?.avatar ?? preferences.app.defaultAvatar;
});

async function loadNotifications() {
  // 并行加载系统通知 + 聊天会话列表
  await Promise.all([loadSystemNotifications(), loadChatSessions()]);
}

async function loadSystemNotifications() {
  try {
    // 并行加载：消息通知 + 我的公告
    const [msgRes, noticeRes] = await Promise.all([
      getNotificationListApi({ page: 1, pageSize: 50 }),
      getMyNoticeListApi({ page: 1, pageSize: 50 }),
    ]);

    // 注意：两个接口返回的分页字段不同
    // - my-notification 返回 PageResponse：{ list, total, page, page_size }
    // - notice/my-page 返回 ResultPage：{ items, total, current_page, page_size, total_pages }
    const msgList = msgRes?.list || [];
    const noticeList = noticeRes?.items || noticeRes?.list || [];

    // 消息通知项（系统通知不显示头像）
    const msgItems = msgList.map((item: any) => ({
      id: `msg_${item.id}`,
      avatar: '',
      title: item.title,
      message: item.content || '',
      date: item.createTime ? formatRelativeTime(item.createTime) : '',
      isRead: item.isRead,
      link: item.linkUrl,
      type: item.type,
    }));

    // 公告项（已发布的才会出现在"我的公告"列表）
    // 注意：后端 MyNoticeListVO 字段为 snake_case（无 serde rename_all）
    const noticeItems = noticeList.map((item: any) => ({
      id: `notice_${item.id}`,
      avatar: '',
      title: item.title || '系统公告',
      message: stripHtml(item.content || ''),
      date: item.publish_time ? formatRelativeTime(item.publish_time) : '',
      isRead: item.is_read === 1,
      link: `/system/notice/detail?id=${item.id}`,
      type: 100, // 100=公告类型，用于区分
    }));

    // 合并公告和消息（公告在前，消息在后）
    notifications.value = [...noticeItems, ...msgItems];
  } catch (e) {
    console.warn('加载系统通知失败', e);
  }
}

// 去除 HTML 标签，提取纯文本（公告内容是富文本）
function stripHtml(html: string): string {
  const text = html?.replace(/<[^>]+>/g, '').trim() || '';
  return text.length > 80 ? `${text.slice(0, 80)}...` : text;
}

async function loadChatSessions() {
  try {
    const res: any = await getSessionListApi({ page: 1, pageSize: 50 });
    chatSessions.value = res?.list || [];
    // 重新计算聊天未读总数
    const total = chatSessions.value.reduce(
      (sum: number, s: ChatSessionDTO) => sum + (s.unreadCount || 0),
      0,
    );
    chatUnreadCount.value = total;
  } catch (e) {
    console.warn('加载聊天会话失败', e);
  }
}

function formatRelativeTime(time: string): string {
  const now = dayjs();
  const target = dayjs(time);
  const diffMinutes = now.diff(target, 'minute');
  if (diffMinutes < 1) return '刚刚';
  if (diffMinutes < 60) return `${diffMinutes}分钟前`;
  const diffHours = now.diff(target, 'hour');
  if (diffHours < 24) return `${diffHours}小时前`;
  const diffDays = now.diff(target, 'day');
  if (diffDays < 7) return `${diffDays}天前`;
  return target.format('YYYY-MM-DD');
}

async function handleLogout() {
  await authStore.logout(false);
}

async function handleNoticeClear() {
  notifications.value = [];
  chatSessions.value = [];
  chatUnreadCount.value = 0;
}

// 标记单条未读项为已读（支持系统通知 ID 和聊天会话 ID 前缀）
async function markRead(id: number | string) {
  const idStr = String(id);

  // 聊天会话：id 形如 "chat_<sessionId>"
  if (idStr.startsWith('chat_')) {
    const sessionId = idStr.slice(5);
    const session = chatSessions.value.find((s) => String(s.sessionId) === sessionId);
    if (session && session.unreadCount > 0) {
      session.unreadCount = 0;
      // 重新计算总数
      chatUnreadCount.value = chatSessions.value.reduce(
        (sum: number, s: ChatSessionDTO) => sum + (s.unreadCount || 0),
        0,
      );
      try {
        await markReadApi({ sessionId });
      } catch (e) {
        console.warn('标记聊天会话已读失败', e);
      }
    }
    return;
  }

  // 系统通知分组：id 形如 "group_<typeName>"
  if (idStr.startsWith('group_')) {
    const typeName = idStr.slice(6);
    // 把该类型所有未读通知都标记为已读（用类型名称匹配，避免数字 type 不一致）
    const items = notifications.value.filter(
      (n) => !n.isRead && getNotifTypeName((n as any).type) === typeName,
    );
    for (const item of items) {
      const itemIdStr = String(item.id ?? '');
      item.isRead = true;
      try {
        // 公告项：notice_<numericId>，调用公告已读接口
        if (itemIdStr.startsWith('notice_')) {
          const noticeId = Number(itemIdStr.slice(7));
          if (!Number.isNaN(noticeId)) await readNoticeApi(noticeId);
        } else if (item.id) {
          await readNotificationApi({ id: String(item.id) });
        }
      } catch (e) {
        console.warn('标记通知已读失败', e);
      }
    }
    return;
  }

  // 公告项：id 形如 "notice_<numericId>"
  if (idStr.startsWith('notice_')) {
    const noticeId = Number(idStr.slice(7));
    const item = notifications.value.find((n) => n.id === id);
    if (item) item.isRead = true;
    try {
      if (!Number.isNaN(noticeId)) await readNoticeApi(noticeId);
    } catch (e) {
      console.warn('标记公告已读失败', e);
    }
    return;
  }

  // 单条系统通知
  const item = notifications.value.find((item) => item.id === id);
  if (item) {
    item.isRead = true;
    try {
      await readNotificationApi({ id: String(id) });
    } catch (e) {
      console.warn('标记已读失败', e);
    }
  }
}

function remove(id: number | string) {
  const idStr = String(id);
  if (idStr.startsWith('chat_')) {
    const sessionId = idStr.slice(5);
    chatSessions.value = chatSessions.value.filter(
      (s) => String(s.sessionId) !== sessionId,
    );
    chatUnreadCount.value = chatSessions.value.reduce(
      (sum: number, s: ChatSessionDTO) => sum + (s.unreadCount || 0),
      0,
    );
    return;
  }
  if (idStr.startsWith('group_')) return;  // 分组项不支持单条移除
  notifications.value = notifications.value.filter((item) => item.id !== id);
}

async function handleMakeAll() {
  // 全部已读：系统通知 + 公告 + 所有聊天会话
  notifications.value.forEach((item) => (item.isRead = true));
  try {
    await readAllNotificationApi();
  } catch (e) {
    console.warn('系统通知全部已读失败', e);
  }
  // 公告全部已读
  try {
    await readAllNoticeApi();
  } catch (e) {
    console.warn('公告全部已读失败', e);
  }
  // 把所有聊天会话未读清零
  const unreadSessions = chatSessions.value.filter((s) => s.unreadCount > 0);
  for (const s of unreadSessions) {
    s.unreadCount = 0;
    try {
      await markReadApi({ sessionId: String(s.sessionId) });
    } catch (e) {
      // 忽略单条失败
    }
  }
  chatUnreadCount.value = 0;
}

const viewAll = () => {
  router.push('/company/message');
};

const handleClick = (item: NotificationItem) => {
  if (item.id) {
    markRead(item.id);
  }
  if (item.link) {
    const idStr = String(item.id ?? '');
    const query: Record<string, any> = { ...(item.query || {}) };
    // 聊天会话项：携带 sessionId，让消息页自动打开对应会话
    if (idStr.startsWith('chat_')) {
      query.sessionId = idStr.slice(5);
      query.tab = 'colleague';
    } else if (idStr.startsWith('group_')) {
      // 系统通知分组：携带通知类型名称，让消息页切换到对应类型
      query.notifType = idStr.slice(6);
      query.tab = 'notification';
    }
    navigateTo(item.link, query, item.state);
  }
};

function navigateTo(
  link: string,
  query?: Record<string, any>,
  state?: Record<string, any>,
) {
  if (link.startsWith('http://') || link.startsWith('https://')) {
    window.open(link, '_blank');
  } else {
    router.push({
      path: link,
      query: query || {},
      state,
    });
  }
}

// 消息页面查看/已读后派发的事件监听：实时刷新铃铛未读数和列表
function handleUnreadUpdated() {
  loadChatSessions();
  loadSystemNotifications();
  refreshChatUnreadCount();
}

onMounted(() => {
  loadNotifications();
  // 初始化：拉取一次聊天未读总数 + 建立 WebSocket 实时监听
  refreshChatUnreadCount();
  connectChatWs();
  // 监听消息页面的已读事件，实时同步铃铛数字与下拉列表
  window.addEventListener('chat:unread-updated', handleUnreadUpdated);
  window.addEventListener('notification:read', handleUnreadUpdated);
});

onUnmounted(() => {
  disconnectChatWs();
  window.removeEventListener('chat:unread-updated', handleUnreadUpdated);
  window.removeEventListener('notification:read', handleUnreadUpdated);
});

watch(
  () => ({
    enable: preferences.app.watermark,
    content: preferences.app.watermarkContent,
    isDark: isDark.value,
  }),
  async ({ enable, content, isDark: isDarkValue }) => {
    if (enable) {
      const watermarkColor = isDarkValue
        ? 'rgba(255, 255, 255, 0.12)'
        : 'rgba(0, 0, 0, 0.12)';

      await updateWatermark({
        advancedStyle: {
          colorStops: [
            {
              color: watermarkColor,
              offset: 0,
            },
            {
              color: watermarkColor,
              offset: 1,
            },
          ],
          type: 'linear',
        },
        content:
          content ||
          `${userStore.userInfo?.username} - ${userStore.userInfo?.realName}`,
      });
    } else {
      destroyWatermark();
    }
  },
  {
    immediate: true,
  },
);

// 监听锁屏状态：变 true 时自动跳转到 /lock
watch(
  () => accessStore.isLockScreen,
  (locked) => {
    if (locked && router.currentRoute.value.path !== '/lock') {
      router.replace('/lock');
    }
  },
);
</script>

<template>
  <BasicLayout @clear-preferences-and-logout="handleLogout">
    <template #user-dropdown>
      <UserDropdown
        :avatar
        :menus
        :text="userStore.userInfo?.realName"
        description="ann.vben@gmail.com"
        tag-text="Pro"
        @logout="handleLogout"
      />
    </template>
    <template #notification>
      <div class="chat-notification-wrap">
        <Notification
          :dot="false"
          :notifications="mergedNotifications"
          @clear="handleNoticeClear"
          @read="(item) => item.id && markRead(item.id)"
          @remove="(item) => item.id && remove(item.id)"
          @make-all="handleMakeAll"
          @on-click="handleClick"
          @view-all="viewAll"
        />
        <div
          v-if="totalUnreadCount > 0"
          class="chat-unread-badge"
          @click="viewAll"
          title="您有新的未读消息，点击查看"
        >
          {{ totalUnreadCount > 99 ? '99+' : totalUnreadCount }}
        </div>
      </div>
    </template>
    <template #extra>
      <AuthenticationLoginExpiredModal
        v-model:open="accessStore.loginExpired"
        :avatar
      >
        <LoginForm />
      </AuthenticationLoginExpiredModal>
    </template>
  </BasicLayout>
</template>

<style scoped>
.chat-notification-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.chat-unread-badge {
  position: absolute;
  top: -2px;
  right: 0px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: #fa5151;
  color: #fff;
  font-size: 11px;
  line-height: 18px;
  text-align: center;
  border-radius: 9px;
  box-shadow: 0 1px 3px rgba(250, 81, 81, 0.4);
  font-weight: 500;
  cursor: pointer;
  pointer-events: auto;
  z-index: 10;
}

/* 隐藏通知下拉列表中每条未读项的蓝色小圆点（改用红色数字徽标提示） */
:deep(li .absolute.top-2.right-2.size-2.rounded-sm.bg-primary) {
  display: none !important;
}
</style>
