<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { Form, FormItem, Input, Modal, Select, Tag, message } from 'ant-design-vue';

import { sendMailApi } from '#/api';

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

const toEmails = ref<string[]>([]);
const ccEmails = ref<string[]>([]);
const subject = ref('');
const body = ref('');
const sending = ref(false);

// 员工主邮箱（列表 VO 的 email 字段）
const defaultEmail = computed(() => props.row?.email || '');

// 简单邮箱格式校验
function isValidEmail(v: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v.trim());
}

// 打开时重置表单，并预填员工邮箱
watch(
  () => props.open,
  (val) => {
    if (!val) return;
    toEmails.value = defaultEmail.value ? [defaultEmail.value] : [];
    ccEmails.value = [];
    subject.value = '';
    body.value = '';
    if (!defaultEmail.value) {
      message.warning('该员工未填写邮箱，请手动输入收件人邮箱');
    }
  },
);

async function handleSend() {
  if (toEmails.value.length === 0) {
    message.warning('请选择收件人');
    return;
  }
  const invalidTo = toEmails.value.find((e) => !isValidEmail(e));
  if (invalidTo) {
    message.warning(`收件人邮箱格式不正确：${invalidTo}`);
    return;
  }
  if (ccEmails.value.length > 0) {
    const invalidCc = ccEmails.value.find((e) => !isValidEmail(e));
    if (invalidCc) {
      message.warning(`抄送邮箱格式不正确：${invalidCc}`);
      return;
    }
  }
  if (!subject.value.trim()) {
    message.warning('请输入邮件主题');
    return;
  }
  if (!body.value.trim()) {
    message.warning('请输入邮件正文');
    return;
  }

  sending.value = true;
  try {
    await sendMailApi({
      toEmails: toEmails.value,
      ccEmails: ccEmails.value,
      subject: subject.value,
      body: body.value,
    });
    message.success('邮件发送成功');
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
    title="发送邮件"
    :width="720"
    :destroy-on-close="true"
    :mask-closable="false"
    :confirm-loading="sending"
    ok-text="发送"
    cancel-text="取消"
    @ok="handleSend"
  >
    <div v-if="row" class="mb-3 text-sm text-gray-500">
      收件员工：<Tag color="blue">{{ row.nickName || row.userName }}</Tag>
    </div>

    <Form layout="vertical">
      <FormItem label="收件人" required>
        <Select
          v-model:value="toEmails"
          mode="tags"
          :token-separators="[',', ' ', ';']"
          placeholder="收件人邮箱（回车添加）"
          style="width: 100%"
        />
      </FormItem>

      <FormItem label="抄送">
        <Select
          v-model:value="ccEmails"
          mode="tags"
          :token-separators="[',', ' ', ';']"
          placeholder="抄送邮箱（回车添加）"
          style="width: 100%"
        />
      </FormItem>

      <FormItem label="主题" required>
        <Input
          v-model:value="subject"
          placeholder="请输入邮件主题"
          allow-clear
        />
      </FormItem>

      <FormItem label="正文" required>
        <Input.TextArea
          v-model:value="body"
          :rows="8"
          placeholder="请输入邮件正文"
          show-count
          :maxlength="2000"
        />
      </FormItem>
    </Form>
  </Modal>
</template>
