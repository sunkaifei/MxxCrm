<script lang="ts" setup>
import type { SelectProps } from 'ant-design-vue';

import { computed, defineAsyncComponent, ref, watch } from 'vue';

import { message, Modal, Select, Input, Form, FormItem, Tag } from 'ant-design-vue';

import {
  getCustomerContactsApi,
  getCustomerInfoApi,
  getMailTemplateInfoApi,
  getMailTemplateOptionsApi,
  sendMailApi,
} from '#/api';

// 异步加载富文本编辑器
const RichTextEditor = defineAsyncComponent(
  () => import('#/components/RichTextEditor/index.vue'),
);

const props = defineProps<{
  visible: boolean;
  customerId?: number;
  customerName?: string;
}>();

const emit = defineEmits<{
  (e: 'update:visible', v: boolean): void;
  (e: 'success', id: number): void;
}>();

const loading = ref(false);
const sending = ref(false);
const templateLoading = ref(false);

// 表单
const toEmails = ref<string[]>([]);
const ccEmails = ref<string[]>([]);
const subject = ref('');
const body = ref('');
const docUrl = ref('');
const templateId = ref<number | undefined>(undefined);

// 收件人选项（客户邮箱 + 联系人邮箱）
const emailOptions = ref<{ label: string; value: string }[]>([]);
// 模板选项
const templateOptions = ref<{ label: string; value: number }[]>([]);

const open = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
});

const customerIdNum = computed(() => Number(props.customerId));

// 简单邮箱格式校验
function isValidEmail(v: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v.trim());
}

// 加载收件人邮箱选项
async function loadEmailOptions() {
  if (!customerIdNum.value) return;
  loading.value = true;
  try {
    const [customerRes, contactsRes] = await Promise.all([
      getCustomerInfoApi(customerIdNum.value),
      getCustomerContactsApi(customerIdNum.value),
    ]);

    const customer: any = customerRes;
    const customerEmail = customer?.personalEmail || customer?.email || '';
    const options: { label: string; value: string }[] = [];
    const seenEmails = new Set<string>();
    const defaultSelected: string[] = [];

    if (customerEmail) {
      options.push({
        label: `${customerEmail}（${props.customerName || '客户主邮箱'}）`,
        value: customerEmail,
      });
      seenEmails.add(customerEmail);
      defaultSelected.push(customerEmail);
    }

    // 联系人去重：后端返回 { current: [], history: [] } 结构
    let contacts: any[] = [];
    if (Array.isArray(contactsRes)) {
      contacts = contactsRes;
    } else if (contactsRes && (contactsRes.current || contactsRes.history)) {
      const current = Array.isArray(contactsRes.current) ? contactsRes.current : [];
      const history = Array.isArray(contactsRes.history) ? contactsRes.history : [];
      const seenIds = new Set<unknown>();
      for (const c of current) {
        const key = c?.id;
        if (key != null && seenIds.has(key)) continue;
        if (key != null) seenIds.add(key);
        contacts.push(c);
      }
      for (const c of history) {
        const key = c?.id;
        if (key != null && seenIds.has(key)) continue;
        if (key != null) seenIds.add(key);
        contacts.push(c);
      }
    } else if (contactsRes && Array.isArray((contactsRes as any).items)) {
      contacts = (contactsRes as any).items;
    }

    for (const c of contacts) {
      const email = c?.email || c?.mail || '';
      if (email && !seenEmails.has(email)) {
        seenEmails.add(email);
        const name = c?.name || c?.contactName || c?.personName || '';
        options.push({
          label: name ? `${email}（${name}）` : email,
          value: email,
        });
      }
    }

    emailOptions.value = options;
    toEmails.value = defaultSelected;
  } catch {
    emailOptions.value = [];
  } finally {
    loading.value = false;
  }
}

// 加载模板选项
async function loadTemplateOptions() {
  templateLoading.value = true;
  try {
    const res: any = await getMailTemplateOptionsApi();
    const list = Array.isArray(res) ? res : (res?.items || []);
    templateOptions.value = list.map((t: any) => ({
      label: t.name,
      value: t.id,
    }));
  } catch {
    templateOptions.value = [];
  } finally {
    templateLoading.value = false;
  }
}

// 选择模板后自动填充主题和正文
async function handleTemplateChange(value: any) {
  const id = Number(value);
  if (!id) {
    subject.value = '';
    body.value = '';
    return;
  }
  try {
    const detail: any = await getMailTemplateInfoApi(id);
    subject.value = detail?.subject || '';
    body.value = detail?.body || '';
  } catch {
    // 错误由全局拦截器处理
  }
}

// 重置表单
function resetForm() {
  toEmails.value = [];
  ccEmails.value = [];
  subject.value = '';
  body.value = '';
  docUrl.value = '';
  templateId.value = undefined;
  emailOptions.value = [];
  templateOptions.value = [];
}

// 监听 visible 打开
watch(
  () => props.visible,
  (val) => {
    if (val) {
      resetForm();
      if (customerIdNum.value) {
        loadEmailOptions();
      }
      loadTemplateOptions();
    }
  },
);

async function handleSend() {
  if (!customerIdNum.value) {
    message.warning('缺少客户信息');
    return;
  }
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
  const bodyText = body.value?.replace(/<[^>]+>/g, '').trim() || '';
  if (!bodyText) {
    message.warning('请输入邮件正文');
    return;
  }

  sending.value = true;
  try {
    const payload = {
      customerId: customerIdNum.value,
      toEmails: toEmails.value,
      ccEmails: ccEmails.value,
      subject: subject.value,
      body: body.value,
      docUrl: docUrl.value?.trim() || undefined,
    };
    const res: any = await sendMailApi(payload);
    const logId = typeof res === 'number' ? res : res?.id;
    message.success('邮件发送成功');
    emit('success', logId);
    open.value = false;
  } catch {
    // 错误由全局拦截器处理
  } finally {
    sending.value = false;
  }
}

function handleCancel() {
  open.value = false;
}

// 抄送人手动输入回车添加
const selectProps: SelectProps = {
  mode: 'tags',
  tokenSeparators: [',', ' ', ';'],
  allowClear: true,
  placeholder: '请输入抄送邮箱（回车添加）',
};
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
    @cancel="handleCancel"
  >
    <div v-if="customerName" class="mb-3 text-sm text-gray-500">
      收件客户：<Tag color="blue">{{ customerName }}</Tag>
    </div>

    <Form layout="vertical">
      <FormItem label="收件人" required>
        <Select
          v-model:value="toEmails"
          mode="multiple"
          :options="emailOptions"
          :loading="loading"
          placeholder="请选择收件人邮箱"
          :filter-option="(input: string, option: any) =>
            option.label.toLowerCase().includes(input.toLowerCase())
          "
          :max-tag-count="10"
          style="width: 100%"
        />
      </FormItem>

      <FormItem label="抄送">
        <Select
          v-bind="selectProps"
          v-model:value="ccEmails"
          style="width: 100%"
        />
      </FormItem>

      <FormItem label="邮件模板">
        <Select
          v-model:value="templateId"
          :options="templateOptions"
          :loading="templateLoading"
          placeholder="选择模板后自动填充主题和正文"
          allow-clear
          show-search
          :filter-option="(input: string, option: any) =>
            option.label.toLowerCase().includes(input.toLowerCase())
          "
          style="width: 100%"
          @change="handleTemplateChange"
        />
      </FormItem>

      <FormItem label="主题" required>
        <Input
          v-model:value="subject"
          placeholder="请输入邮件主题"
          allow-clear
        />
      </FormItem>

      <FormItem label="在线文档URL">
        <Input
          v-model:value="docUrl"
          placeholder="可选，发送时将随请求传给后端抓取文档内容"
          allow-clear
        />
      </FormItem>

      <FormItem label="正文" required>
        <RichTextEditor
          v-model="body"
          placeholder="请输入邮件正文..."
          :height="320"
        />
      </FormItem>
    </Form>
  </Modal>
</template>

<style scoped>
:deep(.ant-form-item) {
  margin-bottom: 16px;
}
</style>
