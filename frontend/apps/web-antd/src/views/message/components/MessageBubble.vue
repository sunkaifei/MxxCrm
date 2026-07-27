<script lang="ts" setup>
import type { ChatMessageDTO } from '#/api/core/message/chat';

import dayjs from 'dayjs';

const props = defineProps<{
  message: ChatMessageDTO;
}>();

const formatTime = (time: string) => {
  return dayjs(time).format('HH:mm');
};
</script>

<template>
  <div
    class="flex mb-4"
    :class="message.isMine ? 'justify-end' : 'justify-start'"
  >
    <template v-if="!message.isMine">
      <img
        :src="message.senderAvatar || 'https://api.dicebear.com/7.x/avataaars/svg?seed=default'"
        class="w-9 h-9 rounded-full mr-2 flex-shrink-0 mt-1"
        :alt="message.senderNickname"
      />
      <div class="flex flex-col items-start max-w-[70%]">
        <span class="text-xs text-gray-400 mb-1 ml-1">{{ message.senderNickname }}</span>
        <div
          v-if="message.isRecalled"
          class="text-xs text-gray-400 py-2 px-3"
        >
          撤回了一条消息
        </div>
        <div
          v-else
          class="bg-white text-gray-800 rounded-xl px-4 py-2.5 shadow-sm break-words"
          style="border-top-left-radius: 4px;"
        >
          {{ message.content }}
        </div>
        <span class="text-xs text-gray-400 mt-1 ml-1">{{ formatTime(message.sendTime) }}</span>
      </div>
    </template>

    <template v-else>
      <div class="flex flex-col items-end max-w-[70%]">
        <div
          v-if="message.isRecalled"
          class="text-xs text-gray-400 py-2 px-3"
        >
          你撤回了一条消息
        </div>
        <div
          v-else
          class="bg-[#1677ff] text-white rounded-xl px-4 py-2.5 shadow-sm break-words"
          style="border-top-right-radius: 4px;"
        >
          {{ message.content }}
        </div>
        <span class="text-xs text-gray-400 mt-1 mr-1">{{ formatTime(message.sendTime) }}</span>
      </div>
      <img
        :src="message.senderAvatar || 'https://api.dicebear.com/7.x/avataaars/svg?seed=default'"
        class="w-9 h-9 rounded-full ml-2 flex-shrink-0 mt-1"
        :alt="message.senderNickname"
      />
    </template>
  </div>
</template>
