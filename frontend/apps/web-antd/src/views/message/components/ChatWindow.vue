<script lang="ts" setup>
import type { ChatMessageDTO, ChatSessionDTO } from '#/api/core/message/chat';

import { nextTick, onMounted, ref, watch } from 'vue';

import {
  LucideMoreHorizontal,
  LucideSettings,
  LucideTrash2,
  LucideMessageCircle,
} from '@vben/icons';

import {
  Button,
  Dropdown,
  Empty,
} from 'ant-design-vue';
import dayjs from 'dayjs';

import MessageBubble from './MessageBubble.vue';
import MessageInput from './MessageInput.vue';

const props = defineProps<{
  session: ChatSessionDTO | null;
  messages: ChatMessageDTO[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'send', content: string): void;
  (e: 'loadMore'): void;
  (e: 'pin', isPinned: boolean): void;
  (e: 'mute', isMuted: boolean): void;
  (e: 'delete'): void;
}>();

const messageListRef = ref<HTMLDivElement>();
const messageInputRef = ref();

const shouldShowTimeDivider = (index: number) => {
  if (index === 0) return true;
  const current = props.messages[index];
  const prev = props.messages[index - 1];
  if (!current || !prev) return false;
  return dayjs(current.sendTime).diff(dayjs(prev.sendTime), 'minute') >= 5;
};

const formatDividerTime = (time: string) => {
  const now = dayjs();
  const msgTime = dayjs(time);
  if (now.isSame(msgTime, 'day')) {
    return msgTime.format('HH:mm');
  }
  if (now.isSame(msgTime, 'year')) {
    return msgTime.format('MM-DD HH:mm');
  }
  return msgTime.format('YYYY-MM-DD HH:mm');
};

function scrollToBottom() {
  nextTick(() => {
    if (messageListRef.value) {
      messageListRef.value.scrollTop = messageListRef.value.scrollHeight;
    }
  });
}

watch(
  () => props.messages.length,
  () => {
    scrollToBottom();
  },
);

watch(
  () => props.session?.sessionId,
  () => {
    scrollToBottom();
    nextTick(() => {
      messageInputRef.value?.focus();
    });
  },
);

onMounted(() => {
  scrollToBottom();
});

function handleSend(content: string) {
  emit('send', content);
}

function handleMenuClick(action: string) {
  switch (action) {
    case 'pin':
      emit('pin', !props.session?.isPinned);
      break;
    case 'mute':
      emit('mute', !props.session?.isMuted);
      break;
    case 'delete':
      emit('delete');
      break;
  }
}

defineExpose({ scrollToBottom });
</script>

<template>
  <div class="flex flex-col h-full">
    <template v-if="session">
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-white">
        <div class="flex items-center">
          <img
            :src="session.avatarUrl || 'https://api.dicebear.com/7.x/avataaars/svg?seed=' + session.sessionId"
            class="w-9 h-9 rounded-full mr-3"
            :alt="session.sessionName"
          />
          <div>
            <div class="font-medium text-gray-800 flex items-center gap-2">
              {{ session.sessionName }}
              <span class="w-2 h-2 rounded-full bg-green-500"></span>
            </div>
            <div class="text-xs text-gray-400">在线</div>
          </div>
        </div>
        <Dropdown :trigger="['click']">
          <template #overlay>
            <div class="py-1">
              <div
                class="px-4 py-2 text-sm cursor-pointer hover:bg-gray-100 flex items-center gap-2"
                @click="handleMenuClick('pin')"
              >
                <span class="w-4 h-4"></span>
                <span>{{ session.isPinned ? '取消置顶' : '置顶' }}</span>
              </div>
              <div
                class="px-4 py-2 text-sm cursor-pointer hover:bg-gray-100 flex items-center gap-2"
                @click="handleMenuClick('mute')"
              >
                <LucideSettings class="w-4 h-4" />
                <span>{{ session.isMuted ? '取消免打扰' : '消息免打扰' }}</span>
              </div>
              <div
                class="px-4 py-2 text-sm cursor-pointer hover:bg-gray-100 text-red-500 flex items-center gap-2"
                @click="handleMenuClick('delete')"
              >
                <LucideTrash2 class="w-4 h-4" />
                <span>删除会话</span>
              </div>
            </div>
          </template>
          <Button type="text">
            <template #icon>
              <LucideMoreHorizontal class="w-5 h-5 text-gray-500" />
            </template>
          </Button>
        </Dropdown>
      </div>

      <div
        ref="messageListRef"
        class="flex-1 overflow-y-auto px-4 py-4 bg-[#f5f5f5]"
      >
        <template v-if="messages.length > 0">
          <div
            v-for="(message, index) in messages"
            :key="message.messageId"
          >
            <div
              v-if="shouldShowTimeDivider(index)"
              class="flex justify-center my-4"
            >
              <span class="text-xs text-gray-400 bg-gray-200 px-3 py-1 rounded-full">
                {{ formatDividerTime(message.sendTime) }}
              </span>
            </div>
            <MessageBubble :message="message" />
          </div>
        </template>
        <div v-else class="flex items-center justify-center h-full">
          <Empty description="暂无消息，开始聊天吧" />
        </div>
      </div>

      <MessageInput
        ref="messageInputRef"
        @send="handleSend"
      />
    </template>

    <div
      v-else
      class="flex-1 flex flex-col items-center justify-center bg-gray-50 text-gray-400"
    >
      <LucideMessageCircle class="w-16 h-16 mb-4 opacity-30" />
      <p class="text-lg">选择一个会话开始聊天</p>
    </div>
  </div>
</template>
