<script lang="ts" setup>
import type { ChatSessionDTO } from '#/api/core/message/chat';

import { computed, h, ref } from 'vue';

import {
  LucideFileText,
  LucideMoreHorizontal,
  LucidePlus,
  LucideSearch,
} from '@vben/icons';

import { Badge, Button, Dropdown, Input, Tabs } from 'ant-design-vue';
import dayjs from 'dayjs';

const props = defineProps<{
  activeSessionId: null | string;
  activeTab: string;
  loading?: boolean;
  notificationUnread: number;
  sessions: ChatSessionDTO[];
}>();

const emit = defineEmits<{
  (e: 'update:activeTab', value: string): void;
  (e: 'select', session: ChatSessionDTO): void;
  (e: 'selectNotification'): void;
  (e: 'search', keyword: string): void;
  (e: 'newMessage'): void;
  (e: 'pin', session: ChatSessionDTO, isPinned: boolean): void;
  (e: 'mute', session: ChatSessionDTO, isMuted: boolean): void;
  (e: 'delete', session: ChatSessionDTO): void;
}>();

const { TabPane } = Tabs;

const searchKeyword = ref('');

const tabs = [
  { key: 'all', label: '全部' },
  { key: 'notification', label: '系统通知' },
  { key: 'chat', label: '我的消息' },
];

const filteredSessions = computed(() => {
  let list = props.sessions;
  if (props.activeTab === 'chat') {
    list = list.filter((s) => s.sessionType === 1);
  } else if (props.activeTab === 'notification') {
    list = list.filter((s) => s.sessionType === 2);
  }
  if (searchKeyword.value) {
    const kw = searchKeyword.value.toLowerCase();
    list = list.filter(
      (s) =>
        s.sessionName.toLowerCase().includes(kw) ||
        s.lastMessageContent.toLowerCase().includes(kw),
    );
  }
  return list.toSorted((a, b) => {
    const pinA = a.isPinned ? 1 : 0;
    const pinB = b.isPinned ? 1 : 0;
    if (pinA !== pinB) return pinB - pinA;
    return (
      new Date(b.lastMessageTime).getTime() -
      new Date(a.lastMessageTime).getTime()
    );
  });
});

const formatTime = (time: string) => {
  const now = dayjs();
  const msgTime = dayjs(time);
  if (now.isSame(msgTime, 'day')) {
    return msgTime.format('HH:mm');
  }
  if (now.diff(msgTime, 'day') < 7) {
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    return weekdays[msgTime.day()];
  }
  return msgTime.format('MM/DD');
};

const getNotificationSession = computed(() =>
  props.sessions.find((s) => s.sessionType === 2),
);

function handleSelect(session: ChatSessionDTO) {
  if (session.sessionType === 2) {
    emit('selectNotification');
  } else {
    emit('select', session);
  }
}

function handleMenuClick(session: ChatSessionDTO, action: string) {
  switch (action) {
    case 'delete': {
      emit('delete', session);
      break;
    }
    case 'mute': {
      emit('mute', session, !session.isMuted);
      break;
    }
    case 'pin': {
      emit('pin', session, !session.isPinned);
      break;
    }
  }
}

function handleTabChange(key: number | string) {
  emit('update:activeTab', String(key));
}
</script>

<template>
  <div class="flex flex-col h-full bg-gray-50 border-r border-gray-200">
    <div class="p-3 border-b border-gray-200 bg-white">
      <div class="flex items-center gap-2 mb-3">
        <Input
          v-model:value="searchKeyword"
          placeholder="搜索"
          allow-clear
          class="flex-1"
          size="small"
          @input="() => emit('search', searchKeyword)"
        >
          <template #prefix>
            <LucideSearch class="w-4 h-4 text-gray-400" />
          </template>
        </Input>
        <Button
          type="primary"
          size="small"
          :icon="h(LucidePlus)"
          @click="emit('newMessage')"
        >
          新建
        </Button>
      </div>
      <Tabs :active-key="activeTab" size="small" @change="handleTabChange">
        <TabPane v-for="tab in tabs" :key="tab.key" :tab="tab.label" />
      </Tabs>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div
        v-if="activeTab !== 'chat' && getNotificationSession"
        class="flex items-center p-3 cursor-pointer hover:bg-gray-100 transition-colors border-b border-gray-100"
        :class="{
          'bg-blue-50': activeSessionId === getNotificationSession.sessionId,
        }"
        @click="handleSelect(getNotificationSession)"
      >
        <div class="relative">
          <div
            class="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center"
          >
            <LucideFileText class="w-5 h-5 text-[#1677ff]" />
          </div>
          <Badge
            v-if="getNotificationSession.unreadCount > 0"
            :count="getNotificationSession.unreadCount"
            :offset="[-2, 2]"
            class="absolute -top-1 -right-1"
          />
        </div>
        <div class="ml-3 flex-1 min-w-0">
          <div class="flex items-center justify-between">
            <span class="font-medium text-gray-800 truncate">
              {{ getNotificationSession.sessionName }}
            </span>
            <span class="text-xs text-gray-400 flex-shrink-0 ml-2">
              {{ formatTime(getNotificationSession.lastMessageTime) }}
            </span>
          </div>
          <p class="text-sm text-gray-500 truncate mt-0.5">
            {{ getNotificationSession.lastMessageContent }}
          </p>
        </div>
      </div>

      <div
        v-for="session in filteredSessions.filter((s) => s.sessionType === 1)"
        :key="session.sessionId"
        class="flex items-center p-3 cursor-pointer hover:bg-gray-100 transition-colors relative group"
        :class="{ 'bg-blue-50': activeSessionId === session.sessionId }"
        @click="handleSelect(session)"
      >
        <div
          v-if="session.isPinned"
          class="absolute left-0 top-0 bottom-0 w-0.5 bg-[#1677ff]"
        ></div>
        <div class="relative flex-shrink-0">
          <img
            :src="
              session.avatarUrl ||
              `https://api.dicebear.com/7.x/avataaars/svg?seed=${session.sessionId}`
            "
            class="w-10 h-10 rounded-full"
            :alt="session.sessionName"
          />
          <Badge
            v-if="session.unreadCount > 0 && !session.isMuted"
            :count="session.unreadCount"
            :offset="[-2, 2]"
            class="absolute -top-1 -right-1"
          />
        </div>
        <div class="ml-3 flex-1 min-w-0">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1">
              <span class="font-medium text-gray-800 truncate">
                {{ session.sessionName }}
              </span>
            </div>
            <span class="text-xs text-gray-400 flex-shrink-0 ml-2">
              {{ formatTime(session.lastMessageTime) }}
            </span>
          </div>
          <p class="text-sm text-gray-500 truncate mt-0.5">
            {{ session.lastMessageContent }}
          </p>
        </div>
        <Dropdown :trigger="['click']" @click.stop>
          <template #overlay>
            <div class="py-1">
              <div
                class="px-4 py-2 text-sm cursor-pointer hover:bg-gray-100"
                @click.stop="handleMenuClick(session, 'pin')"
              >
                {{ session.isPinned ? '取消置顶' : '置顶' }}
              </div>
              <div
                class="px-4 py-2 text-sm cursor-pointer hover:bg-gray-100"
                @click.stop="handleMenuClick(session, 'mute')"
              >
                {{ session.isMuted ? '取消免打扰' : '消息免打扰' }}
              </div>
              <div
                class="px-4 py-2 text-sm cursor-pointer hover:bg-gray-100 text-red-500"
                @click.stop="handleMenuClick(session, 'delete')"
              >
                删除会话
              </div>
            </div>
          </template>
          <Button
            type="text"
            size="small"
            class="opacity-0 group-hover:opacity-100 transition-opacity"
          >
            <template #icon>
              <LucideMoreHorizontal class="w-4 h-4 text-gray-400" />
            </template>
          </Button>
        </Dropdown>
      </div>

      <div
        v-if="
          filteredSessions.filter((s) =>
            activeTab === 'chat' ? s.sessionType === 1 : true,
          ).length === 0
        "
        class="flex flex-col items-center justify-center py-16 text-gray-400"
      >
        <LucidePlus class="w-12 h-12 mb-2 opacity-50" />
        <p class="text-sm">暂无会话</p>
      </div>
    </div>
  </div>
</template>
