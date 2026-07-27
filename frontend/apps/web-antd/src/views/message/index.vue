<script lang="ts" setup>
import type { ChatMessageDTO, ChatSessionDTO } from '#/api/core/message/chat';

import {
  getSessionListApi,
  getMessageListApi,
  sendMessageApi,
  markReadApi,
  deleteSessionApi,
  pinSessionApi,
  muteSessionApi,
  getUnreadCountApi,
} from '#/api/core/message/chat';
import { getNotificationUnreadCountApi } from '#/api/core/message/notification';

import { h, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideMoreHorizontal,
} from '@vben/icons';

import { Button } from 'ant-design-vue';

import SessionList from './components/SessionList.vue';
import ChatWindow from './components/ChatWindow.vue';
import NotificationList from './components/NotificationList.vue';
import UserSearchModal from './components/UserSearchModal.vue';

defineOptions({ name: 'MessageCenter' });

const sessions = ref<ChatSessionDTO[]>([]);
const activeSessionId = ref<string | null>(null);
const activeTab = ref('all');
const messages = ref<ChatMessageDTO[]>([]);
const showNotification = ref(false);
const searchModalVisible = ref(false);
const sidebarVisible = ref(true);
const notificationUnread = ref(0);
const chatUnread = ref(0);

let pollTimer: any = null;
const messagePageSize = 50;

const activeSession = ref<ChatSessionDTO | null>(null);
const notificationListRef = ref();

async function loadSessions() {
  try {
    const res = await getSessionListApi({ page: 1, pageSize: 100 });
    sessions.value = res.list || [];
  } catch (e) {
    console.error('加载会话列表失败', e);
  }
}

async function loadMessages(sessionId: string) {
  try {
    const res = await getMessageListApi({
      sessionId,
      page: 1,
      pageSize: messagePageSize,
    });
    messages.value = res.list || [];
  } catch (e) {
    console.error('加载消息失败', e);
  }
}

async function loadUnreadCount() {
  try {
    const chatRes = await getUnreadCountApi();
    chatUnread.value = chatRes || 0;
  } catch (e) {
    console.error(e);
  }
  try {
    const notifRes = await getNotificationUnreadCountApi();
    notificationUnread.value = notifRes || 0;
  } catch (e) {
    console.error(e);
  }
}

async function handleSelectSession(session: ChatSessionDTO) {
  activeSessionId.value = session.sessionId;
  activeSession.value = session;
  showNotification.value = false;
  messages.value = [];
  await loadMessages(session.sessionId);
  if (session.unreadCount > 0) {
    try {
      await markReadApi({ sessionId: session.sessionId });
      session.unreadCount = 0;
    } catch (e) {
      console.error(e);
    }
  }
}

function handleSelectNotification() {
  const notifSession = sessions.value.find((s) => s.sessionType === 2);
  if (notifSession) {
    activeSessionId.value = notifSession.sessionId;
    activeSession.value = notifSession;
  }
  showNotification.value = true;
  messages.value = [];
  notificationListRef.value?.loadNotifications();
}

async function handleSendMessage(content: string) {
  if (!activeSession.value) return;
  try {
    await sendMessageApi({
      sessionId: activeSession.value.sessionId,
      content,
    });
    await loadMessages(activeSession.value.sessionId);
    await loadSessions();
  } catch (e) {
    console.error('发送消息失败', e);
    window.$message?.error('发送失败');
  }
}

async function handlePinSession(session: ChatSessionDTO, isPinned: boolean) {
  try {
    await pinSessionApi({ sessionId: session.sessionId, isPinned });
    session.isPinned = isPinned;
    window.$message?.success(isPinned ? '已置顶' : '已取消置顶');
  } catch (e) {
    console.error(e);
  }
}

async function handleMuteSession(session: ChatSessionDTO, isMuted: boolean) {
  try {
    await muteSessionApi({ sessionId: session.sessionId, isMuted });
    session.isMuted = isMuted;
    window.$message?.success(isMuted ? '已开启免打扰' : '已关闭免打扰');
  } catch (e) {
    console.error(e);
  }
}

async function handleDeleteSession(session: ChatSessionDTO) {
  try {
    await deleteSessionApi({ sessionId: session.sessionId });
    sessions.value = sessions.value.filter((s) => s.sessionId !== session.sessionId);
    if (activeSessionId.value === session.sessionId) {
      activeSessionId.value = null;
      activeSession.value = null;
      messages.value = [];
    }
    window.$message?.success('删除成功');
  } catch (e) {
    console.error(e);
  }
}

function handleNewMessage() {
  searchModalVisible.value = true;
}

function handleUserSelect(user: any) {
  nextTick(async () => {
    await loadSessions();
    const newSession = sessions.value.find(
      (s) => s.sessionName === (user.nickName || user.realName || user.userName),
    );
    if (newSession) {
      handleSelectSession(newSession);
    }
  });
}

function handleNotificationUnreadChange(delta: number) {
  notificationUnread.value = Math.max(0, notificationUnread.value + delta);
}

function startPolling() {
  pollTimer = setInterval(async () => {
    await loadSessions();
    if (activeSessionId.value && !showNotification.value) {
      await loadMessages(activeSessionId.value);
    }
    await loadUnreadCount();
  }, 3000);
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

function toggleSidebar() {
  sidebarVisible.value = !sidebarVisible.value;
}

onMounted(async () => {
  await loadSessions();
  await loadUnreadCount();
  startPolling();
});

onBeforeUnmount(() => {
  stopPolling();
});
</script>

<template>
  <Page auto-content-height class="!p-0 !m-0">
    <div class="flex h-full w-full overflow-hidden bg-white rounded-lg shadow-sm">
      <div
        v-if="sidebarVisible"
        class="w-80 flex-shrink-0 md:w-80"
      >
        <SessionList
          :sessions="sessions"
          :active-session-id="activeSessionId"
          v-model:active-tab="activeTab"
          :notification-unread="notificationUnread"
          @select="handleSelectSession"
          @select-notification="handleSelectNotification"
          @new-message="handleNewMessage"
          @pin="handlePinSession"
          @mute="handleMuteSession"
          @delete="handleDeleteSession"
        />
      </div>

      <div class="flex-1 flex flex-col min-w-0">
        <div
          v-if="!sidebarVisible"
          class="p-2 border-b border-gray-200"
        >
          <Button type="text" :icon="h(LucideMoreHorizontal)" @click="toggleSidebar">
            菜单
          </Button>
        </div>
        <div class="flex-1 min-h-0">
          <NotificationList
            v-if="showNotification"
            ref="notificationListRef"
            @unread-change="handleNotificationUnreadChange"
          />
          <ChatWindow
            v-else
            :session="activeSession"
            :messages="messages"
            @send="handleSendMessage"
            @pin="(v: boolean) => activeSession && handlePinSession(activeSession, v)"
            @mute="(v: boolean) => activeSession && handleMuteSession(activeSession, v)"
            @delete="() => activeSession && handleDeleteSession(activeSession)"
          />
        </div>
      </div>
    </div>

    <UserSearchModal
      v-model:visible="searchModalVisible"
      @select="handleUserSelect"
    />
  </Page>
</template>
