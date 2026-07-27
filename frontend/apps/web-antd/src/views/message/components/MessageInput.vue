<script lang="ts" setup>
import { h, ref } from 'vue';

import {
  LucideImage,
  LucideFile,
  LucideFilePenLine,
} from '@vben/icons';

import { Button, Input } from 'ant-design-vue';

const { TextArea } = Input;

const emit = defineEmits<{
  (e: 'send', content: string): void;
}>();

const content = ref('');
const textareaRef = ref();

function handleSend() {
  const text = content.value.trim();
  if (!text) return;
  emit('send', text);
  content.value = '';
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

function focus() {
  textareaRef.value?.focus();
}

defineExpose({ focus });
</script>

<template>
  <div class="border-t border-gray-200 bg-white p-3">
    <div class="flex items-center gap-2 mb-2 text-gray-500">
      <Button
        type="text"
        size="small"
        class="hover:text-[#1677ff]"
      >
        <template #icon>
          <span class="text-lg">😊</span>
        </template>
      </Button>
      <Button
        type="text"
        size="small"
        class="hover:text-[#1677ff]"
      >
        <template #icon>
          <LucideImage class="w-5 h-5" />
        </template>
      </Button>
      <Button
        type="text"
        size="small"
        class="hover:text-[#1677ff]"
      >
        <template #icon>
          <LucideFile class="w-5 h-5" />
        </template>
      </Button>
    </div>
    <div class="flex items-end gap-2">
      <TextArea
        v-model:value="content"
        ref="textareaRef"
        :rows="3"
        placeholder="输入消息，Enter 发送，Shift+Enter 换行"
        class="resize-none"
        @keydown="handleKeydown"
      />
      <Button
        type="primary"
        :icon="h(LucideFilePenLine)"
        :disabled="!content.trim()"
        @click="handleSend"
      >
        发送
      </Button>
    </div>
  </div>
</template>
