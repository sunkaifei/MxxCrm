<script lang="tsx" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideBot, LucidePlus, LucideTrash2 } from '@vben/icons';

import {
  Button,
  Card,
  Divider,
  Form,
  Input,
  InputNumber,
  InputPassword,
  message,
  Modal,
  Popconfirm,
  Select,
  Spin,
  Switch,
  Tabs,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useRoute } from 'vue-router';

import {
  deleteIntegrationApi,
  getIntegrationListApi,
  saveIntegrationApi,
  testIntegrationApi,
  toggleIntegrationApi,
} from '#/api';

// ─── 提供商模板（用于添加 AI 提供商弹窗） ───
const PROVIDER_TEMPLATES: Array<{
  id: string;
  name: string;
  defaultModel: string;
  defaultApiUrl: string;
  needSecret?: boolean;
}> = [
  {
    id: 'deepseek',
    name: 'DeepSeek',
    defaultModel: 'deepseek-chat',
    defaultApiUrl: 'https://api.deepseek.com/v1/chat/completions',
  },
  {
    id: 'doubao',
    name: '豆包',
    defaultModel: 'doubao-pro-32k',
    defaultApiUrl:
      'https://ark.cn-beijing.volces.com/api/v3/chat/completions',
  },
  {
    id: 'zhipu',
    name: '智谱',
    defaultModel: 'glm-4-plus',
    defaultApiUrl: 'https://open.bigmodel.cn/api/paas/v4/chat/completions',
  },
  {
    id: 'qwen',
    name: '通义千问',
    defaultModel: 'qwen-plus',
    defaultApiUrl:
      'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions',
  },
  {
    id: 'moonshot',
    name: 'Moonshot',
    defaultModel: 'moonshot-v1-8k',
    defaultApiUrl: 'https://api.moonshot.cn/v1/chat/completions',
  },
  {
    id: 'wenxin',
    name: '文心一言',
    defaultModel: 'ernie-4.0',
    defaultApiUrl:
      'https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat',
    needSecret: true,
  },
  {
    id: 'custom_ai',
    name: '自定义模型',
    defaultModel: '',
    defaultApiUrl: '',
  },
];

// ─── 响应式状态 ───
const activeCategory = ref('payment');
const configList = ref<any[]>([]);
const loading = ref(false);
const testing = reactive<Record<string, boolean>>({});
const saving = reactive<Record<string, boolean>>({});
const deleting = reactive<Record<string, boolean>>({});
const editingConfigs = ref<Record<string, any>>({});

// 支持通过 URL ?category=xxx 直接进入指定 Tab（从其他页面跳转时使用）
const route = useRoute();

onMounted(() => {
  const c = (route.query?.category as string) || '';
  if (c && categories.find((x) => x.key === c)) {
    activeCategory.value = c;
  }
  loadList();
});

// ─── 添加 AI 提供商弹窗 ───
const addModalVisible = ref(false);
const addFormRef = ref();
const addForm = reactive({
  templateId: 'deepseek',
  integrationCode: '',
  integrationName: '',
  apiBaseUrl: '',
  apiKey: '',
  secretKey: '',
  model: '',
  temperature: '0.7',
});

const categories = [
  { key: 'payment', label: '支付配置' },
  { key: 'logistics', label: '物流配置' },
  { key: 'esign', label: '电子签约' },
  { key: 'invoice', label: '开票配置' },
  { key: 'notification', label: '通知配置' },
  { key: 'exchange_rate', label: '汇率配置' },
  { key: 'ai', label: 'AI 配置' },
];

function isPromptItem(item: any): boolean {
  const code: string = item.integrationCode || '';
  return code.startsWith('prompt_');
}

const currentList = computed(() =>
  configList.value.filter((i: any) => i.category === activeCategory.value),
);

// AI 配置拆分：提供商列表 + 提示词列表
const aiProviderList = computed(() =>
  currentList.value.filter((i: any) => !isPromptItem(i)),
);
const aiPromptList = computed(() =>
  currentList.value.filter((i: any) => isPromptItem(i)),
);

// 前端需要但 DB 中尚未预置的「占位提示词」（无 ID，只提示用户去后端接口或后续手动加）
// （若后端初始化脚本已创建对应 prompt_* 行，这里不会重复出现）
const MISSING_PROMPT_PLACEHOLDERS: Array<{ code: string; name: string; desc: string }> = [
  {
    code: 'prompt_customer_insight',
    name: '客户画像分析提示词',
    desc: '用于 AI 生成客户画像（背调/业务模式/财务风险/客户分级等）',
  },
  {
    code: 'prompt_sales_reply',
    name: '销售回复话术提示词',
    desc: '用于基于客户画像生成销售跟进邮件/消息的回复建议',
  },
];

const effectivePromptList = computed(() => {
  const existing = new Set(aiPromptList.value.map((i) => i.integrationCode));
  const dynamic: any[] = [...aiPromptList.value];
  for (const ph of MISSING_PROMPT_PLACEHOLDERS) {
    if (!existing.has(ph.code)) {
      dynamic.push({
        id: 0,
        category: 'ai',
        integrationCode: ph.code,
        integrationName: ph.name,
        enabled: 1,
        remark: ph.desc,
        isPlaceholder: true,
        apiBaseUrl: '',
      });
    }
  }
  return dynamic;
});

// ─── 初始化 / 加载 ───
async function loadList() {
  loading.value = true;
  try {
    const res = await getIntegrationListApi();
    configList.value = res || [];
    configList.value.forEach((item: any) => initEditForm(item));
  } catch {
    // global interceptor
  } finally {
    loading.value = false;
  }
}

function initEditForm(item: any) {
  if (!editingConfigs.value[item.id]) {
    let cj = item.configJson;
    if (typeof cj === 'string') {
      try {
        cj = JSON.parse(cj || '{}');
      } catch {
        cj = {};
      }
    }
    // 占位提示词：默认 content 字段为空，等待第一次保存
    if (item.isPlaceholder) {
      editingConfigs.value[item.integrationCode] = { content: '', ...(cj || {}) };
    } else {
      editingConfigs.value[item.id] = { ...(cj || {}) };
    }
  }
}

function getEditFormRef(item: any) {
  if (item.isPlaceholder) return editingConfigs.value[item.integrationCode];
  return editingConfigs.value[item.id];
}

// ─── 保存 ───
async function handleSave(item: any) {
  // 占位提示词：先创建，再进入编辑
  if (item.isPlaceholder) {
    await handleCreatePrompt(item);
    return;
  }
  saving[item.id] = true;
  try {
    await saveIntegrationApi({
      id: item.id,
      category: item.category,
      integrationCode: item.integrationCode,
      integrationName: item.integrationName,
      configJson: getEditFormRef(item),
      apiBaseUrl: item.apiBaseUrl,
      enabled: item.enabled,
      remark: item.remark,
    });
    message.success(`${item.integrationName} 配置已保存`);
    await loadList();
  } catch {
    // global interceptor
  } finally {
    saving[item.id] = false;
  }
}

// ─── 开关 ───
async function handleToggle(item: any, checked: boolean) {
  try {
    await toggleIntegrationApi(item.id, checked ? 1 : 0);
    item.enabled = checked ? 1 : 0;
    message.success(`${item.integrationName} 已${checked ? '启用' : '禁用'}`);
  } catch {
    // global interceptor
  }
}

// ─── 测试 ───
async function handleTest(item: any) {
  testing[item.id] = true;
  try {
    const res: any = await testIntegrationApi(item.id);
    if (res?.success) {
      message.success(`${item.integrationName} 连接成功`);
    } else {
      message.error(
        `${item.integrationName} 连接失败：${res?.message ?? '未知错误'}`,
      );
    }
    await loadList();
  } catch {
    // global interceptor
  } finally {
    testing[item.id] = false;
  }
}

async function handleTestAll() {
  const items = activeCategory.value === 'ai' ? aiProviderList.value : currentList.value;
  for (const item of items) {
    if (item.isPlaceholder) continue;
    // eslint-disable-next-line no-await-in-loop
    await handleTest(item);
  }
}

// ─── 删除 ───
async function handleDelete(item: any) {
  deleting[item.id] = true;
  try {
    await deleteIntegrationApi(item.id);
    message.success('配置已删除');
    await loadList();
  } catch {
    // global interceptor
  } finally {
    deleting[item.id] = false;
  }
}

// ─── 新建占位提示词：首次点击「创建并保存」时写入 DB ───
async function handleCreatePrompt(item: any) {
  const savingKey = `ph_${item.integrationCode}`;
  saving[savingKey] = true;
  try {
    const content = (getEditFormRef(item) || {}).content || '';
    await saveIntegrationApi({
      category: 'ai',
      integrationCode: item.integrationCode,
      integrationName: item.integrationName,
      configJson: { content },
      apiBaseUrl: '',
      enabled: 1,
      remark: item.remark,
    });
    message.success(`${item.integrationName} 已创建并保存`);
    await loadList();
  } catch {
    // global interceptor
  } finally {
    saving[savingKey] = false;
  }
}

// ─── 状态标签 ───
function statusTag(item: any) {
  if (!item.enabled) return { color: 'default', text: '已禁用' };
  if (item.lastTestResult === 1) return { color: 'success', text: '已连通' };
  if (item.lastTestResult === 2) return { color: 'error', text: '连接失败' };
  return { color: 'warning', text: '未测试' };
}

// ─── 表单字段定义 ───
interface ConfigField {
  key: string;
  label: string;
  type: string;
  required?: boolean;
}

function getConfigFields(code: string): ConfigField[] {
  // 提示词：只渲染 content 长文本
  if (code.startsWith('prompt_')) {
    return [{ key: 'content', label: '提示词内容', type: 'textarea', required: true }];
  }
  const map: Record<string, ConfigField[]> = {
    wechat_pay: [
      { key: 'app_id', label: 'AppID', type: 'text', required: true },
      { key: 'mchid', label: '商户号', type: 'text', required: true },
      { key: 'serial_no', label: '证书序列号', type: 'text' },
      { key: 'private_key', label: 'API私钥', type: 'password', required: true },
      { key: 'cert_path', label: 'API证书路径', type: 'text' },
      { key: 'platform_cert_path', label: '平台证书路径', type: 'text' },
    ],
    alipay: [
      { key: 'app_id', label: 'AppID', type: 'text', required: true },
      { key: 'private_key', label: '应用私钥', type: 'password', required: true },
      { key: 'alipay_public_key', label: '支付宝公钥', type: 'password' },
      { key: 'sign_type', label: '签名类型', type: 'text' },
    ],
    stripe: [
      { key: 'secret_key', label: 'Secret Key', type: 'password', required: true },
      { key: 'publishable_key', label: 'Publishable Key', type: 'text' },
      { key: 'webhook_secret', label: 'Webhook Secret', type: 'password' },
    ],
    kuaidi100: [
      { key: 'customer', label: 'Customer', type: 'text', required: true },
      { key: 'key', label: 'Key', type: 'password', required: true },
      { key: 'callback_url', label: '回调URL', type: 'text' },
    ],
    sf_express: [
      { key: 'check_word', label: '校验码', type: 'password', required: true },
      { key: 'custid', label: '客户号', type: 'text', required: true },
      { key: 'service_code', label: '服务代码', type: 'text' },
    ],
    esign_cn: [
      { key: 'app_id', label: 'AppID', type: 'text', required: true },
      { key: 'app_secret', label: 'AppSecret', type: 'password', required: true },
      { key: 'org_id', label: '机构ID', type: 'text' },
    ],
    fdd: [
      { key: 'app_id', label: 'AppID', type: 'text', required: true },
      { key: 'app_secret', label: 'AppSecret', type: 'password', required: true },
    ],
    baiwang: [
      { key: 'device_no', label: '设备号', type: 'text', required: true },
      { key: 'tax_no', label: '税号', type: 'text', required: true },
      { key: 'private_key', label: '私钥', type: 'password' },
    ],
    aisino: [
      { key: 'tax_no', label: '税号', type: 'text', required: true },
      { key: 'user_token', label: '用户Token', type: 'password', required: true },
    ],
    smtp_email: [
      { key: 'host', label: 'SMTP主机', type: 'text', required: true },
      { key: 'port', label: '端口', type: 'number', required: true },
      { key: 'username', label: '用户名', type: 'text', required: true },
      { key: 'password', label: '密码', type: 'password', required: true },
      { key: 'from_email', label: '发件邮箱', type: 'text', required: true },
      { key: 'from_name', label: '发件人名称', type: 'text' },
      { key: 'is_ssl', label: 'SSL(1=是)', type: 'number' },
    ],
    sms_aliyun: [
      { key: 'access_key', label: 'AccessKey', type: 'text', required: true },
      { key: 'secret_key', label: 'SecretKey', type: 'password', required: true },
      { key: 'sign_name', label: '签名名称', type: 'text', required: true },
      { key: 'template_code', label: '模板Code', type: 'text' },
    ],
    sms_tencent: [
      { key: 'secret_id', label: 'SecretId', type: 'text', required: true },
      { key: 'secret_key', label: 'SecretKey', type: 'password', required: true },
      { key: 'sdk_appid', label: 'SdkAppId', type: 'text', required: true },
      { key: 'sign_name', label: '签名名称', type: 'text' },
    ],
    wecom_webhook: [
      { key: 'webhook_url', label: 'Webhook URL', type: 'text', required: true },
    ],
    dingtalk_webhook: [
      { key: 'webhook_url', label: 'Webhook URL', type: 'text', required: true },
      { key: 'secret', label: '加签密钥', type: 'password' },
    ],
    feishu_webhook: [
      { key: 'webhook_url', label: 'Webhook URL', type: 'text', required: true },
      { key: 'secret', label: '加签密钥', type: 'password' },
    ],
    wenxin: [
      { key: 'api_key', label: 'API Key', type: 'password', required: true },
      { key: 'secret_key', label: 'Secret Key', type: 'password', required: true },
      { key: 'model', label: '模型', type: 'text' },
    ],
    ecb: [] as ConfigField[],
  };
  // 通用 AI 提供商兜底（如果没列在上面，且不是 prompt_*，则默认字段）
  if (!map[code] && !code.startsWith('prompt_')) {
    const tpl = PROVIDER_TEMPLATES.find((t) => t.id === code);
    if (tpl?.needSecret) {
      return [
        { key: 'api_key', label: 'API Key', type: 'password', required: true },
        { key: 'secret_key', label: 'Secret Key', type: 'password', required: true },
        { key: 'model', label: '模型', type: 'text' },
        { key: 'temperature', label: 'Temperature', type: 'text' },
      ];
    }
    return [
      { key: 'api_key', label: 'API Key', type: 'password', required: true },
      { key: 'model', label: '模型', type: 'text' },
      { key: 'temperature', label: 'Temperature', type: 'text' },
    ];
  }
  return map[code] || [];
}

// ─── 已存在的提供商 integrationCode 集合（用于新增去重） ───
const existingProviderCodes = computed(() =>
  new Set(aiProviderList.value.map((i: any) => i.integrationCode)),
);

// ─── 添加 AI 提供商：打开弹窗 / 模板切换 ───
function openAddProviderModal() {
  addForm.templateId = 'deepseek';
  applyTemplate('deepseek');
  addModalVisible.value = true;
}

function applyTemplate(tid: string) {
  const tpl = PROVIDER_TEMPLATES.find((t) => t.id === tid);
  if (!tpl) return;
  addForm.model = tpl.defaultModel;
  addForm.apiBaseUrl = tpl.defaultApiUrl;
  if (tid === 'custom_ai') {
    addForm.integrationCode = '';
    addForm.integrationName = '';
  } else {
    addForm.integrationCode = tid;
    addForm.integrationName = tpl.name;
  }
  addForm.apiKey = '';
  addForm.secretKey = '';
  addForm.temperature = '0.7';
}

function watchTemplate(val: any) {
  applyTemplate(String(val ?? ''));
}

async function handleAddProviderSubmit() {
  try {
    await addFormRef.value?.validate();
  } catch {
    return;
  }
  // 检查 code 是否已存在
  if (existingProviderCodes.value.has(addForm.integrationCode)) {
    message.error('该提供商代码已存在，请在下方卡片中直接编辑');
    return;
  }
  // 构造 config_json
  const cfgJson: Record<string, string> = {
    api_key: addForm.apiKey,
  };
  if (addForm.model) cfgJson.model = addForm.model;
  if (addForm.temperature) cfgJson.temperature = addForm.temperature;
  if (addForm.secretKey) cfgJson.secret_key = addForm.secretKey;

  saving['add_provider'] = true;
  try {
    await saveIntegrationApi({
      category: 'ai',
      integrationCode: addForm.integrationCode,
      integrationName: addForm.integrationName,
      configJson: cfgJson,
      apiBaseUrl: addForm.apiBaseUrl,
      enabled: 1,
      sortOrder: 0,
      remark: `AI 提供商：${addForm.integrationName}`,
    });
    message.success(`${addForm.integrationName} 已添加`);
    addModalVisible.value = false;
    await loadList();
  } catch {
    // global interceptor
  } finally {
    saving['add_provider'] = false;
  }
}

// ─── 通用渲染：单张配置卡片 ───
function renderConfigCard(item: any, opts: { deletable: boolean } = { deletable: false }) {
  return (
    <Card size="small" class="mb-4" key={item.id || item.integrationCode}>
      {{
        title: () => (
          <div class="flex items-center gap-2">
            {activeCategory.value === 'ai' && !isPromptItem(item) ? (
              <LucideBot class="h-4 w-4 text-indigo-500" />
            ) : null}
            <span>{item.integrationName}</span>
            {!item.isPlaceholder ? (
              <Tag color={statusTag(item).color}>{statusTag(item).text}</Tag>
            ) : (
              <Tag color="default">未创建</Tag>
            )}
            {item.remark && (
              <Tooltip title={item.remark}>
                <span class="text-xs text-gray-400">({item.remark})</span>
              </Tooltip>
            )}
          </div>
        ),
        extra: () => (
          <div class="flex items-center gap-2">
            {!item.isPlaceholder ? (
              <Switch
                checked={item.enabled === 1}
                checked-children="启用"
                un-checked-children="禁用"
                onChange={(checked: any) => handleToggle(item, checked)}
              />
            ) : null}
            {opts.deletable && !item.isPlaceholder ? (
              <Popconfirm
                title={`确认删除「${item.integrationName}」配置吗？`}
                ok-text="确认删除"
                cancel-text="取消"
                ok-type="danger"
                onConfirm={() => handleDelete(item)}
              >
                <Button
                  type="text"
                  size="small"
                  danger
                  loading={!!deleting[item.id]}
                  v-slots={{ icon: () => <LucideTrash2 class="h-4 w-4" /> }}
                />
              </Popconfirm>
            ) : null}
          </div>
        ),
        default: () => {
          const fields = getConfigFields(item.integrationCode);
          const formRef = getEditFormRef(item) || {};
          initEditForm(item);

          // 提示词：只有 content 长文本 → 占满宽度
          const gridCols =
            item.integrationCode.startsWith('prompt_') ? 'grid-cols-1' : 'grid-cols-2';

          return (
            <>
              {/* 配置表单 */}
              {fields.length > 0 ? (
                <div class={`grid gap-x-6 gap-y-3 ${gridCols}`}>
                  {fields.map((field) => (
                    <div class="flex flex-col gap-1" key={field.key}>
                      <label class="text-xs text-gray-500">
                        {field.label}
                        {field.required ? (
                          <span class="text-red-500">*</span>
                        ) : null}
                      </label>
                      {field.type === 'number' ? (
                        <InputNumber
                          value={formRef[field.key]}
                          size="small"
                          class="w-full"
                          placeholder={`请输入${field.label}…`}
                          onUpdate:value={(v: any) => (formRef[field.key] = v)}
                        />
                      ) : field.type === 'password' ? (
                        <InputPassword
                          value={formRef[field.key]}
                          size="small"
                          placeholder={`请输入${field.label}…`}
                          allow-clear
                          onUpdate:value={(v: any) => (formRef[field.key] = v)}
                        />
                      ) : field.type === 'textarea' ? (
                        <Input.TextArea
                          value={formRef[field.key]}
                          placeholder={`请输入${field.label}…`}
                          auto-size={{ minRows: 6, maxRows: 16 }}
                          onUpdate:value={(v: any) => (formRef[field.key] = v)}
                        />
                      ) : (
                        <Input
                          value={formRef[field.key]}
                          size="small"
                          placeholder={`请输入${field.label}…`}
                          allow-clear
                          onUpdate:value={(v: any) => (formRef[field.key] = v)}
                        />
                      )}
                    </div>
                  ))}
                </div>
              ) : null}

              {/* API 地址（始终展示，AI 提供商必填） */}
              {!item.integrationCode.startsWith('prompt_') ? (
                <div class="mt-4 flex flex-col gap-1">
                  <label class="text-xs text-gray-500">
                    API 基础地址
                    {activeCategory.value === 'ai' && !isPromptItem(item) ? (
                      <span class="text-red-500">*</span>
                    ) : null}
                  </label>
                  <Input
                    value={item.apiBaseUrl}
                    size="small"
                    placeholder="例如 https://api.deepseek.com/v1/chat/completions"
                    onUpdate:value={(v: any) => (item.apiBaseUrl = v)}
                  />
                </div>
              ) : null}

              {/* 上次测试信息 */}
              {!item.isPlaceholder && item.lastTestMessage ? (
                <Tooltip title={item.lastTestMessage}>
                  <div
                    class={[
                      'mt-3 truncate text-xs',
                      item.lastTestResult === 2 ? 'text-red-400' : 'text-gray-400',
                    ]}
                  >
                    {item.lastTestMessage}
                  </div>
                </Tooltip>
              ) : null}

              {/* 操作按钮 */}
              <div class="mt-4 flex justify-end gap-2">
                {!item.isPlaceholder && !isPromptItem(item) ? (
                  <Button
                    size="small"
                    loading={!!testing[item.id]}
                    disabled={item.enabled !== 1}
                    onClick={() => handleTest(item)}
                  >
                    测试连接
                  </Button>
                ) : null}
                <Button
                  type="primary"
                  size="small"
                  loading={
                    item.isPlaceholder
                      ? !!saving[`ph_${item.integrationCode}`]
                      : !!saving[item.id]
                  }
                  onClick={() => handleSave(item)}
                >
                  {item.isPlaceholder ? '创建并保存' : '保存'}
                </Button>
              </div>
            </>
          );
        },
      }}
    </Card>
  );
}
</script>

<template>
  <Page>
    <div class="rounded-md bg-gray-50 p-4">
      <div class="mb-4 flex items-center justify-between">
        <span class="text-gray-500">
          共 {{ currentList.length }} 个配置项，已启用
          {{ currentList.filter((i: any) => i.enabled === 1).length }} 个
        </span>
        <Button type="primary" ghost @click="handleTestAll">
          测试全部
        </Button>
      </div>

      <Spin :spinning="loading">
        <Tabs v-model:active-key="activeCategory">
          <!-- 非 AI 分类：普通列表渲染 -->
          <Tabs.TabPane
            v-for="cat in categories.filter((c) => c.key !== 'ai')"
            :key="cat.key"
            :tab="cat.label"
          >
            <div
              v-if="currentList.length === 0"
              class="py-20 text-center text-gray-300"
            >
              暂无接口数据
            </div>
            <template v-else>
              <component
                v-for="item in currentList"
                :is="renderConfigCard(item)"
                :key="item.id"
              />
            </template>
          </Tabs.TabPane>

          <!-- AI 分类：模型提供商 + 提示词 两个分区 -->
          <Tabs.TabPane key="ai" tab="AI 配置">
            <div
              v-if="aiProviderList.length === 0 && effectivePromptList.length === 0"
              class="py-20 text-center text-gray-300"
            >
              暂无 AI 配置数据
            </div>
            <template v-else>
              <!-- 分区 1：模型提供商 -->
              <div class="mb-6">
                <div class="mb-3 flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-gray-700">模型提供商</span>
                    <span class="text-xs text-gray-400">
                      （按「从上到下」的顺序自动调用第一个已启用的可用提供商）
                    </span>
                  </div>
                  <Button
                    type="primary"
                    size="small"
                    @click="openAddProviderModal"
                  >
                    <template #icon>
                      <LucidePlus class="h-4 w-4" />
                    </template>
                    添加提供商
                  </Button>
                </div>
                <div
                  v-if="aiProviderList.length === 0"
                  class="rounded-md border border-dashed border-gray-200 py-10 text-center text-sm text-gray-400"
                >
                  还没有 AI 提供商，点击右上角「添加提供商」开始配置
                </div>
                <template v-else>
                  <component
                    v-for="item in aiProviderList"
                    :is="renderConfigCard(item, { deletable: true })"
                    :key="item.id"
                  />
                </template>
              </div>

              <Divider class="my-2" />

              <!-- 分区 2：提示词配置 -->
              <div>
                <div class="mb-3 flex items-center gap-2">
                  <span class="text-sm font-medium text-gray-700">提示词配置</span>
                  <span class="text-xs text-gray-400">
                    （定义不同业务场景下的 AI 指令，如客户背调、销售回复等）
                  </span>
                </div>
                <component
                  v-for="item in effectivePromptList"
                  :is="renderConfigCard(item, { deletable: false })"
                  :key="item.id || item.integrationCode"
                />
              </div>
            </template>
          </Tabs.TabPane>
        </Tabs>
      </Spin>
    </div>

    <!-- ─── 添加 AI 提供商 Modal ─── -->
    <Modal
      v-model:open="addModalVisible"
      title="添加 AI 提供商"
      :mask-closable="false"
      :confirm-loading="!!saving['add_provider']"
      ok-text="保存"
      cancel-text="取消"
      @ok="handleAddProviderSubmit"
    >
      <Form ref="addFormRef" layout="vertical" :model="addForm">
        <Form.Item
          label="提供商模板"
          name="templateId"
          :rules="[{ required: true, message: '请选择提供商模板' }]"
        >
          <Select
            v-model:value="addForm.templateId"
            placeholder="选择模板后将自动填充默认模型和 API 地址"
            @change="watchTemplate"
          >
            <Select.Option
              v-for="t in PROVIDER_TEMPLATES"
              :key="t.id"
              :value="t.id"
            >
              {{ t.name }}
            </Select.Option>
          </Select>
        </Form.Item>
        <Form.Item
          label="集成编码 (code)"
          name="integrationCode"
          :rules="[
            { required: true, message: '请输入集成编码' },
            {
              pattern: /^[a-z0-9_]+$/,
              message: '只能使用小写字母、数字和下划线',
            },
          ]"
        >
          <Input
            v-model:value="addForm.integrationCode"
            placeholder="例如 deepseek / custom_azure_openai"
          />
        </Form.Item>
        <Form.Item
          label="显示名称"
          name="integrationName"
          :rules="[{ required: true, message: '请输入显示名称' }]"
        >
          <Input v-model:value="addForm.integrationName" placeholder="例如 DeepSeek" />
        </Form.Item>
        <Form.Item
          label="API 基础地址"
          name="apiBaseUrl"
          :rules="[{ required: true, message: '请输入 API 基础地址' }]"
        >
          <Input
            v-model:value="addForm.apiBaseUrl"
            placeholder="例如 https://api.deepseek.com/v1/chat/completions"
          />
        </Form.Item>
        <Form.Item
          label="API Key"
          name="apiKey"
          :rules="[{ required: true, message: '请输入 API Key' }]"
        >
          <InputPassword v-model:value="addForm.apiKey" />
        </Form.Item>
        <Form.Item
          v-if="
            PROVIDER_TEMPLATES.find((t) => t.id === addForm.templateId)?.needSecret
          "
          label="Secret Key"
          name="secretKey"
          :rules="[{ required: true, message: '请输入 Secret Key' }]"
        >
          <InputPassword v-model:value="addForm.secretKey" />
        </Form.Item>
        <Form.Item label="模型名称" name="model">
          <Input
            v-model:value="addForm.model"
            placeholder="例如 deepseek-chat / gpt-4o-mini 等"
          />
        </Form.Item>
        <Form.Item label="Temperature (默认 0.7)" name="temperature">
          <Input v-model:value="addForm.temperature" placeholder="0.7" />
        </Form.Item>
      </Form>
    </Modal>
  </Page>
</template>

<style scoped>
.mb-4 {
  margin-bottom: 16px;
}

.mb-3 {
  margin-bottom: 12px;
}

.mb-6 {
  margin-bottom: 24px;
}

.mt-4 {
  margin-top: 16px;
}

.mt-3 {
  margin-top: 12px;
}

.my-2 {
  margin-top: 8px;
  margin-bottom: 8px;
}

.gap-x-6 {
  column-gap: 24px;
}

.gap-y-3 {
  row-gap: 12px;
}

.grid-cols-1 {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.grid-cols-2 {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.grid {
  display: grid;
}
</style>
