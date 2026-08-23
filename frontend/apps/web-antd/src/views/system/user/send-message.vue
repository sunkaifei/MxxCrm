<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { Form, FormItem, Input, Modal, Tag, message } from 'ant-design-vue';

import { sendMessageApi } from '#/api/core/message/chat';

const props = defineProps<{
  open: boolean;
  row?: any;
}>();

const emit = defineEmits<{
  (e: 'update:open', v: boolean): void;
  (e: 'success'): void;
}>();

const open = computed({
  get: () => props.open,
  set: (v) => emit('update:open', v),
});

const content = ref('');
const sending = ref(false);

// 打开时清空输入
watch(
  () => props.open,
  (val) => {
    if (val) content.value = '';
  },
);

async function handleSend() {
  if (!props.row) return;
  if (!content.value.trim()) {
    message.warning('请输入消息内容');
    return;
  }
  sending.value = true;
  try {
    // receiverId 传员工ID字符串，后端已兼容（session_id/receiver_id 支持字符串反序列化）
    await sendMessageApi({
      receiverId: String(props.row.id),
      content: content.value.trim(),
    });
    message.success('消息发送成功');
    open.value = false;
    emit('success');
  } catch {
    // 错误由全局拦截器处理，保留弹窗以便修改后重试
  } finally {
    sending.value = false;
  }
}
</script>

<template>
  <Modal
    v-model:open="open"
    title="发送消息"
    :width="480"
    :destroy-on-close="true"
    :mask-closable="false"
    :confirm-loading="sending"
    ok-text="发送"
    cancel-text="取消"
    @ok="handleSend"
  >
    <div v-if="row" class="mb-3 text-sm text-gray-500">
      发送给：<Tag color="blue">{{ row.nickName || row.userName }}</Tag>
    </div>

    <Form layout="vertical">
      <FormItem label="消息内容" required>
        <Input.TextArea
          v-model:value="content"
          :rows="6"
          placeholder="请输入要发送的站内消息"
          show-count
          :maxlength="1000"
        />
      </FormItem>
    </Form>
  </Modal>
</template>
