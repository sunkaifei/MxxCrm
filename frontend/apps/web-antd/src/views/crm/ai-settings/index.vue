<script lang="ts" setup>
import { h, ref, reactive, computed, nextTick, onMounted } from 'vue';
import { Button, Card, Modal, Form, Input, Select, InputNumber, message, Tag, Spin, Tooltip, Divider, Row, Col } from 'ant-design-vue';
import { Page } from '@vben/common-ui';
import {
  LucidePlus, LucideTrash2, LucideSettings2, LucideCheckCircle,
  LucideXCircle, LucideKeyRound, LucideGlobe, LucideBot,
  LucideMessageSquare,
  LucideFileText, LucidePencil,
} from '@vben/icons';
import { getAiConfigListApi, addAiConfigApi, updateAiConfigApi, deleteAiConfigApi } from '#/api';

function toCamelCase(obj: any): any {
  if (obj === null || obj === undefined) return obj;
  if (Array.isArray(obj)) return obj.map(toCamelCase);
  if (typeof obj !== 'object') return obj;
  const result: any = {};
  for (const key of Object.keys(obj)) {
    const camelKey = key.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
    result[camelKey] = toCamelCase(obj[key]);
  }
  return result;
}

// ─── 提供商模板 ───
const PROVIDER_TEMPLATES = [
  {
    id: 'deepseek', name: 'DeepSeek', color: '#4F46E5', lightColor: '#EEF2FF',
    model: 'deepseek-chat', apiUrl: 'https://api.deepseek.com/v1/chat/completions',
    docUrl: 'https://platform.deepseek.com/api_keys',
  },
  {
    id: 'doubao', name: '豆包', color: '#0891B2', lightColor: '#ECFEFF',
    model: 'doubao-pro-32k', apiUrl: 'https://ark.cn-beijing.volces.com/api/v3/chat/completions',
    docUrl: 'https://console.volcengine.com/ark',
  },
  {
    id: 'zhipu', name: '智谱', color: '#7C3AED', lightColor: '#F5F3FF',
    model: 'glm-4-plus', apiUrl: 'https://open.bigmodel.cn/api/paas/v4/chat/completions',
    docUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
  },
  {
    id: 'qwen', name: '通义千问', color: '#EA580C', lightColor: '#FFF7ED',
    model: 'qwen-plus', apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions',
    docUrl: 'https://help.aliyun.com/zh/model-studio/',
  },
  {
    id: 'moonshot', name: 'Moonshot', color: '#16A34A', lightColor: '#F0FDF4',
    model: 'moonshot-v1-8k', apiUrl: 'https://api.moonshot.cn/v1/chat/completions',
    docUrl: 'https://platform.moonshot.cn/console/api-keys',
  },
  {
    id: 'baidu', name: '文心一言', color: '#2563EB', lightColor: '#EFF6FF',
    model: 'ernie-4.0', apiUrl: 'https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat',
    docUrl: 'https://console.bce.baidu.com/qianfan/',
  },
  {
    id: 'custom', name: '自定义', color: '#6B7280', lightColor: '#F9FAFB',
    model: '', apiUrl: '', docUrl: '',
  },
];

// ─── 状态 ───
const allConfigs = ref<any[]>([]);
const loading = ref(false);
const activeTab = ref<'provider' | 'prompt'>('provider');

async function loadConfigs() {
  loading.value = true;
  try {
    const res = await getAiConfigListApi();
    const rawData = Array.isArray(res) ? res : (res?.data || res?.items || []);
    allConfigs.value = toCamelCase(rawData);
  } catch { /* ignore */ }
  finally { loading.value = false; }
}

onMounted(loadConfigs);

const promptConfigs = computed(() =>
  allConfigs.value.filter((item: any) => item.configKey.includes('prompt'))
);

// ─── 提供商解析 ───
interface ProviderInfo {
  id: string;
  name: string;
  template: any;
  items: any[];
  apiKey: string;
  model: string;
  apiUrl: string;
  temperature: string;
  apiKeyId?: number;
  modelId?: number;
  apiUrlId?: number;
  tempId?: number;
  status: 'connected' | 'disconnected' | 'untested';
  lastTestTime?: string;
}

const providers = computed(() => {
  const modelItems = allConfigs.value.filter((item: any) => !item.configKey.includes('prompt'));
  const groups: Record<string, any[]> = {};
  for (const item of modelItems) {
    const key = item.configKey || '';
    const match = key.match(/^ai_([a-z]+)_/);
    const prefix = match ? match[1] : 'custom';
    if (!groups[prefix]) groups[prefix] = [];
    groups[prefix].push(item);
  }
  return Object.entries(groups).map(([prefix, items]): ProviderInfo => {
    const apiKeyItem = items.find((i: any) => i.configKey.includes('api_key'));
    const modelItem = items.find((i: any) => i.configKey.includes('model') && !i.configKey.includes('api_key') && !i.configKey.includes('prompt'));
    const apiUrlItem = items.find((i: any) => i.configKey.includes('api_url'));
    const tempItem = items.find((i: any) => i.configKey.includes('temperature'));
    const template = PROVIDER_TEMPLATES.find(t => t.id === prefix) || PROVIDER_TEMPLATES[PROVIDER_TEMPLATES.length - 1];
    const hasApiKey = !!apiKeyItem?.configValue;
    return {
      id: prefix,
      name: template?.name || prefix.charAt(0).toUpperCase() + prefix.slice(1),
      template,
      items,
      apiKey: apiKeyItem?.configValue || '',
      model: modelItem?.configValue || '',
      apiUrl: apiUrlItem?.configValue || '',
      temperature: tempItem?.configValue || '',
      apiKeyId: apiKeyItem?.id,
      modelId: modelItem?.id,
      apiUrlId: apiUrlItem?.id,
      tempId: tempItem?.id,
      status: 'untested',
    };
  });
});

function getProviderIcon(pid: string) {
  const t = PROVIDER_TEMPLATES.find(p => p.id === pid);
  return t?.name === '自定义' ? LucideSettings2 : LucideBot;
}

// ─── Modal: 添加/编辑提供商 ───
const modalVisible = ref(false);
const modalMode = ref<'add' | 'edit'>('add');
const editingProviderId = ref('');
const formRef = ref();
const form = reactive({
  template: 'deepseek',
  apiKey: '',
  modelName: '',
  apiUrl: '',
  temperature: 0.7,
});

const selectedTemplate = computed(() =>
  PROVIDER_TEMPLATES.find(t => t.id === form.template) || PROVIDER_TEMPLATES[0]
);

function watchTemplate(tplId: string) {
  const tpl = PROVIDER_TEMPLATES.find(t => t.id === tplId);
  if (tpl && tpl.id !== 'custom') {
    form.modelName = tpl.model;
    form.apiUrl = tpl.apiUrl;
  }
}

function openAddModal() {
  modalMode.value = 'add';
  editingProviderId.value = '';
  form.template = 'deepseek';
  form.apiKey = '';
  form.modelName = '';
  form.apiUrl = '';
  form.temperature = 0.7;
  apiKeyEdited.value = true; // 新增模式必须输入密钥
  modalVisible.value = true;
  nextTick(() => formRef.value?.resetFields());
}

function openEditModal(provider: ProviderInfo) {
  modalMode.value = 'edit';
  editingProviderId.value = provider.id;
  const tplId = provider.template?.id || 'custom';
  // 回显后端返回的脱敏值（如 sk-1****abcd），未修改则不提交
  form.apiKey = provider.apiKey;
  form.modelName = provider.model || PROVIDER_TEMPLATES.find(t => t.id === tplId)?.model || '';
  form.apiUrl = provider.apiUrl || PROVIDER_TEMPLATES.find(t => t.id === tplId)?.apiUrl || '';
  form.temperature = parseFloat(provider.temperature) || 0.7;
  apiKeyEdited.value = false; // 编辑模式初始未修改
  modalVisible.value = true;
  nextTick(() => formRef.value?.resetFields());
}

function closeModal() {
  modalVisible.value = false;
}

async function handleSaveProvider() {
  // 编辑模式且未修改密钥时，跳过密钥校验（保留脱敏值不提交）
  if (modalMode.value === 'add' || apiKeyEdited.value) {
    if (!form.apiKey.trim()) { message.error('请填写API密钥'); return; }
  }
  if (!form.modelName.trim()) { message.error('请填写模型名称'); return; }
  if (!form.apiUrl.trim()) { message.error('请填写API地址'); return; }

  const prefix = form.template === 'custom' ? 'custom' : form.template;
  const displayName = selectedTemplate.value?.name || '自定义';

  try {
    if (modalMode.value === 'edit') {
      const existing = providers.value.find(p => p.id === editingProviderId.value);
      const updates = [];
      // 只有用户主动修改了密钥才提交更新，避免把脱敏值写回数据库
      if (apiKeyEdited.value && existing?.apiKeyId) {
        updates.push(updateAiConfigApi({ id: existing.apiKeyId, config_key: `ai_${prefix}_api_key`, config_name: `${displayName} API Key`, config_value: form.apiKey, config_type: 'N' }));
      }
      if (existing?.modelId) updates.push(updateAiConfigApi({ id: existing.modelId, config_key: `ai_${prefix}_model`, config_name: `${displayName}模型`, config_value: form.modelName, config_type: 'N' }));
      if (existing?.apiUrlId) updates.push(updateAiConfigApi({ id: existing.apiUrlId, config_key: `ai_${prefix}_api_url`, config_name: `${displayName} API地址`, config_value: form.apiUrl, config_type: 'N' }));
      if (existing?.tempId) updates.push(updateAiConfigApi({ id: existing.tempId, config_key: `ai_${prefix}_temperature`, config_name: `${displayName}生成温度`, config_value: String(form.temperature), config_type: 'N' }));
      await Promise.all(updates);
      message.success('提供商配置已更新');
    } else {
      await Promise.all([
        addAiConfigApi({ config_key: `ai_${prefix}_api_key`, config_name: `${displayName} API Key`, config_value: form.apiKey, config_type: 'N', remark: `${displayName}大模型API密钥` }),
        addAiConfigApi({ config_key: `ai_${prefix}_model`, config_name: `${displayName}模型`, config_value: form.modelName, config_type: 'N', remark: `${displayName}大模型名称` }),
        addAiConfigApi({ config_key: `ai_${prefix}_api_url`, config_name: `${displayName} API地址`, config_value: form.apiUrl, config_type: 'N', remark: `${displayName}大模型API接口地址` }),
        addAiConfigApi({ config_key: `ai_${prefix}_temperature`, config_name: `${displayName}生成温度`, config_value: String(form.temperature), config_type: 'N', remark: `${displayName}大模型生成温度，0-1之间` }),
      ]);
      message.success(`已添加上${displayName}模型提供商`);
    }
    closeModal();
    loadConfigs();
  } catch {
    message.error('操作失败');
  }
}

function handleDeleteProvider(provider: ProviderInfo) {
  Modal.confirm({
    title: `删除提供商「${provider.name}」`,
    content: `将删除该提供商所有配置项（共 ${provider.items.length} 项），确定继续吗？`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await Promise.all(provider.items.map((item: any) => deleteAiConfigApi(item.id)));
        message.success(`已删除提供商「${provider.name}」`);
        loadConfigs();
      } catch { message.error('删除失败'); }
    },
  });
}

// API 密钥是否已被用户修改（编辑模式下控制是否提交更新）
const apiKeyEdited = ref(false);

// 点击"修改密钥"按钮：清空当前值，进入可编辑状态
function enableApiKeyEdit() {
  form.apiKey = '';
  apiKeyEdited.value = true;
}

function getPromptType(key: string) {
  if (key.includes('background_check')) return { label: '企业背调', color: 'green' };
  if (key.includes('lead')) return { label: '线索分析', color: 'blue' };
  if (key.includes('customer')) return { label: '客户分析', color: 'purple' };
  if (key.includes('opportunity')) return { label: '商机评估', color: 'orange' };
  return { label: '自定义', color: 'default' };
}
</script>

<template>
  <Page auto-content-height>
    <!-- Header -->
    <div class="ai-header">
      <div class="ai-header-inner">
        <div class="ai-brand">
          <div class="ai-brand-icon">
            <LucideBot :size="20" />
          </div>
          <div>
            <h1 class="ai-title">AI 设置中心</h1>
            <p class="ai-subtitle">管理大模型提供商与提示词配置</p>
          </div>
        </div>
        <div class="ai-header-tabs">
          <button
            class="ai-tab"
            :class="{ active: activeTab === 'provider' }"
            @click="activeTab = 'provider'"
          >
            <LucideSettings2 :size="14" />
            模型提供商
          </button>
          <button
            class="ai-tab"
            :class="{ active: activeTab === 'prompt' }"
            @click="activeTab = 'prompt'"
          >
            <LucideFileText :size="14" />
            提示词管理
          </button>
        </div>
      </div>
    </div>

    <Spin :spinning="loading">
      <!-- ═══ 提供商 Tab ═══ -->
      <div v-show="activeTab === 'provider'">
        <div class="ai-toolbar">
          <span class="ai-toolbar-info">
            已接入 <strong>{{ providers.length }}</strong> 个模型提供商
          </span>
          <Button type="primary" class="ai-btn-primary" @click="openAddModal">
            <LucidePlus :size="14" /> 添加提供商
          </Button>
        </div>

        <div v-if="!providers.length" class="ai-empty">
          <div class="ai-empty-icon"><LucideBot :size="48" /></div>
          <p class="ai-empty-text">暂无模型提供商</p>
          <p class="ai-empty-desc">点击上方按钮添加你的第一个大模型提供商</p>
        </div>

        <div v-else class="ai-provider-grid">
          <div v-for="provider in providers" :key="provider.id" class="ai-provider-card">
            <!-- 卡片顶栏 -->
            <div class="ai-pcard-head" :style="{ borderLeftColor: provider.template?.color || '#6B7280' }">
              <div class="ai-pcard-head-left">
                <div
                  class="ai-pcard-icon"
                  :style="{ background: provider.template?.lightColor || '#F9FAFB', color: provider.template?.color || '#6B7280' }"
                >
                  <component :is="getProviderIcon(provider.id)" :size="18" />
                </div>
                <div>
                  <h3 class="ai-pcard-name">{{ provider.name }}</h3>
                  <span class="ai-pcard-badge">{{ provider.items.length }} 项配置</span>
                </div>
              </div>
              <div class="ai-pcard-head-right">
                <Tooltip title="编辑">
                  <button class="ai-pcard-action" @click="openEditModal(provider)">
                    <LucidePencil :size="14" />
                  </button>
                </Tooltip>
                <Tooltip title="删除">
                  <button class="ai-pcard-action danger" @click="handleDeleteProvider(provider)">
                    <LucideTrash2 :size="14" />
                  </button>
                </Tooltip>
              </div>
            </div>

            <!-- 关键信息 -->
            <div class="ai-pcard-body">
              <div class="ai-pcard-field">
                <span class="ai-pcard-label"><LucideKeyRound :size="12" /> API密钥</span>
                <div class="ai-pcard-value-row">
                  <code class="ai-pcard-code">{{ provider.apiKey || '-' }}</code>
                </div>
              </div>
              <div class="ai-pcard-field">
                <span class="ai-pcard-label"><LucideBot :size="12" /> 模型</span>
                <div class="ai-pcard-value-row">
                  <code class="ai-pcard-code">{{ provider.model || '-' }}</code>
                </div>
              </div>
              <div class="ai-pcard-field">
                <span class="ai-pcard-label"><LucideGlobe :size="12" /> API地址</span>
                <div class="ai-pcard-value-row">
                  <code class="ai-pcard-code ai-pcard-url">{{ provider.apiUrl || '-' }}</code>
                </div>
              </div>
              <div class="ai-pcard-field">
                <span class="ai-pcard-label"><LucideSettings2 :size="12" /> 温度</span>
                <div class="ai-pcard-value-row">
                  <span class="ai-pcard-plain">{{ provider.temperature || '0.7' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ═══ 提示词 Tab ═══ -->
      <div v-show="activeTab === 'prompt'" class="ai-prompts">
        <div class="ai-toolbar">
          <span class="ai-toolbar-info">
            共 <strong>{{ promptConfigs.length }}</strong> 个提示词模板
          </span>
          <Button type="primary" class="ai-btn-primary" disabled>
            <LucidePlus :size="14" /> 添加提示词
          </Button>
        </div>

        <div v-if="!promptConfigs.length" class="ai-empty">
          <div class="ai-empty-icon"><LucideFileText :size="48" /></div>
          <p class="ai-empty-text">暂无提示词配置</p>
        </div>

        <div v-else class="ai-prompt-grid">
          <div v-for="prompt in promptConfigs" :key="prompt.id" class="ai-prompt-card">
            <div class="ai-prompt-head">
              <div>
                <h4 class="ai-prompt-name">{{ prompt.configName }}</h4>
                <div class="ai-prompt-meta">
                  <Tag :color="getPromptType(prompt.configKey).color" size="small">
                    {{ getPromptType(prompt.configKey).label }}
                  </Tag>
                  <code class="ai-prompt-key">{{ prompt.configKey }}</code>
                </div>
              </div>
            </div>
            <div class="ai-prompt-body">
              <code class="ai-prompt-preview">{{ prompt.configValue?.slice(0, 180) }}{{ prompt.configValue?.length > 180 ? '...' : '' }}</code>
            </div>
            <div v-if="prompt.remark" class="ai-prompt-foot">{{ prompt.remark }}</div>
          </div>
        </div>
      </div>
    </Spin>

    <!-- ─── Modal: 添加/编辑提供商 ─── -->
    <Modal
      v-model:open="modalVisible"
      :title="null"
      :footer="null"
      :width="640"
      :destroyOnClose="true"
      class="ai-modal"
    >
      <div class="ai-modal-inner">
        <div class="ai-modal-head">
          <div class="ai-modal-icon">
            <LucideBot :size="22" />
          </div>
          <div>
            <h2 class="ai-modal-title">{{ modalMode === 'add' ? '添加模型提供商' : '编辑提供商配置' }}</h2>
            <p class="ai-modal-desc">快速接入主流大模型API，一键完成配置</p>
          </div>
        </div>

        <Form ref="formRef" layout="vertical" :model="form" class="ai-modal-form">
          <Form.Item label="选择提供商" required>
            <Select
              v-model:value="form.template"
              @change="watchTemplate"
              :disabled="modalMode === 'edit'"
              class="ai-tpl-select"
            >
              <Select.OptGroup label="主流提供商">
                <Select.Option v-for="t in PROVIDER_TEMPLATES.filter(p => p.id !== 'custom')" :key="t.id" :value="t.id">
                  <div class="ai-tpl-option">
                    <span
                      class="ai-tpl-dot"
                      :style="{ background: t.color }"
                    ></span>
                    <span>{{ t.name }}</span>
                    <span class="ai-tpl-model">{{ t.model }}</span>
                  </div>
                </Select.Option>
              </Select.OptGroup>
              <Select.OptGroup label="其他">
                <Select.Option value="custom">
                  <div class="ai-tpl-option">
                    <span class="ai-tpl-dot" style="background:#6B7280"></span>
                    <span>自定义接入</span>
                    <span class="ai-tpl-model">完全手动填写</span>
                  </div>
                </Select.Option>
              </Select.OptGroup>
            </Select>
          </Form.Item>

          <div class="ai-modal-divider">
            <span>连接配置</span>
          </div>

          <Form.Item label="API 密钥" :required="modalMode === 'add' || apiKeyEdited">
            <!-- 编辑模式且未修改：显示脱敏值 + 修改按钮 -->
            <div v-if="modalMode === 'edit' && !apiKeyEdited" class="ai-apikey-readonly">
              <Input
                :value="form.apiKey"
                readonly
                class="ai-input-mono"
              />
              <Button type="link" size="small" class="ai-apikey-edit-btn" @click="enableApiKeyEdit">
                <LucideKeyRound :size="12" /> 修改密钥
              </Button>
            </div>
            <!-- 新增模式或已点击修改：可输入，支持眼睛切换查看明文核对 -->
            <Input.Password
              v-else
              v-model:value="form.apiKey"
              :placeholder="modalMode === 'edit' ? '输入新的 API 密钥...' : '输入 API 密钥...'"
              visibility-toggle
              class="ai-input-mono"
            />
            <div class="ai-form-hint">
              <a v-if="selectedTemplate?.docUrl" :href="selectedTemplate.docUrl" target="_blank" class="ai-hint-link">
                如何获取 {{ selectedTemplate?.name }} API Key？
              </a>
            </div>
          </Form.Item>

          <Row :gutter="16">
            <Col :span="12">
              <Form.Item label="模型名称" required>
                <Input v-model:value="form.modelName" placeholder="例如: deepseek-chat" class="ai-input-mono" />
              </Form.Item>
            </Col>
            <Col :span="12">
              <Form.Item label="生成温度">
                <InputNumber v-model:value="form.temperature" :min="0" :max="2" :step="0.1" class="ai-input-full" />
              </Form.Item>
            </Col>
          </Row>

          <Form.Item label="API 地址" required>
            <Input v-model:value="form.apiUrl" placeholder="https://api.example.com/v1/chat/completions" class="ai-input-mono" />
          </Form.Item>

          <div class="ai-modal-actions">
            <Button @click="closeModal">取消</Button>
            <Button type="primary" class="ai-btn-primary" @click="handleSaveProvider">
              <LucideCheckCircle :size="14" />
              {{ modalMode === 'add' ? '确认添加' : '保存修改' }}
            </Button>
          </div>
        </Form>
      </div>
    </Modal>
  </Page>
</template>

<style scoped>
/* ─── Reset / Global ─── */
.ai-header {
  margin-bottom: 16px;
}
.ai-header-inner {
  background: #ffffff;
  border-radius: 12px;
  padding: 20px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.04);
  border: 1px solid #f0f0f0;
}
.ai-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}
.ai-brand-icon {
  width: 40px; height: 40px;
  border-radius: 10px;
  background: linear-gradient(135deg, #4F46E5, #7C3AED);
  display: flex; align-items: center; justify-content: center;
  color: #fff;
}
.ai-title {
  font-size: 17px; font-weight: 700; color: #111827; margin: 0; line-height: 1.3;
}
.ai-subtitle {
  font-size: 13px; color: #9CA3AF; margin: 0;
}
.ai-header-tabs {
  display: flex;
  background: #F3F4F6;
  border-radius: 8px;
  padding: 3px;
  gap: 2px;
}
.ai-tab {
  display: flex; align-items: center; gap: 6px;
  padding: 7px 16px; border: none; background: transparent;
  border-radius: 6px; font-size: 13px; font-weight: 500;
  color: #6B7280; cursor: pointer; transition: all 0.2s;
}
.ai-tab:hover { color: #374151; }
.ai-tab.active {
  background: #ffffff; color: #4F46E5; box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}

/* ─── Toolbar ─── */
.ai-toolbar {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 16px; flex-wrap: wrap; gap: 8px;
}
.ai-toolbar-info {
  font-size: 14px; color: #6B7280;
}
.ai-toolbar-info strong { color: #111827; }
.ai-btn-primary {
  display: inline-flex; align-items: center; gap: 6px;
}

/* ─── Empty ─── */
.ai-empty {
  text-align: center; padding: 64px 24px;
  background: #ffffff; border-radius: 12px; border: 1px dashed #E5E7EB;
}
.ai-empty-icon { color: #D1D5DB; margin-bottom: 12px; }
.ai-empty-text { font-size: 16px; font-weight: 600; color: #6B7280; margin: 0 0 4px; }
.ai-empty-desc { font-size: 13px; color: #9CA3AF; margin: 0; }

/* ─── Provider Grid ─── */
.ai-provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(420px, 1fr));
  gap: 16px;
}
.ai-provider-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #f0f0f0;
  overflow: hidden;
  transition: all 0.25s ease;
}
.ai-provider-card:hover {
  border-color: #E5E7EB;
  box-shadow: 0 4px 20px rgba(0,0,0,0.06);
}

/* Card Head */
.ai-pcard-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #F3F4F6;
  border-left: 4px solid transparent;
}
.ai-pcard-head-left {
  display: flex; align-items: center; gap: 12px;
}
.ai-pcard-icon {
  width: 40px; height: 40px; border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.ai-pcard-name {
  font-size: 15px; font-weight: 700; color: #111827; margin: 0; line-height: 1.3;
}
.ai-pcard-badge {
  font-size: 11px; color: #9CA3AF; background: #F3F4F6;
  padding: 2px 8px; border-radius: 10px;
}
.ai-pcard-head-right {
  display: flex; gap: 4px;
}
.ai-pcard-action {
  width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
  border: none; background: transparent; color: #9CA3AF; cursor: pointer;
  border-radius: 6px; transition: all 0.15s;
}
.ai-pcard-action:hover { background: #F3F4F6; color: #374151; }
.ai-pcard-action.danger:hover { background: #FEE2E2; color: #DC2626; }

/* Card Body */
.ai-pcard-body {
  padding: 16px 20px; display: flex; flex-direction: column; gap: 10px;
}
.ai-pcard-field {
  display: flex; align-items: baseline; gap: 8px;
}
.ai-pcard-label {
  font-size: 11px; font-weight: 600; color: #9CA3AF; text-transform: uppercase;
  letter-spacing: 0.3px; display: flex; align-items: center; gap: 4px;
  min-width: 60px; flex-shrink: 0;
}
.ai-pcard-value-row {
  display: flex; align-items: center; gap: 6px; flex: 1; min-width: 0;
}
.ai-pcard-code {
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 12px; color: #374151; background: #F9FAFB;
  padding: 4px 10px; border-radius: 4px; border: 1px solid #F3F4F6;
  flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.ai-pcard-url { font-size: 11px; }
.ai-pcard-plain {
  font-size: 13px; color: #374151; font-weight: 500;
}
.ai-pcard-inline-btn {
  font-size: 11px; color: #6366F1; background: none; border: none;
  cursor: pointer; padding: 2px 4px; white-space: nowrap; flex-shrink: 0;
}
.ai-pcard-inline-btn:hover { color: #4F46E5; text-decoration: underline; }

/* ─── Prompts Tab ─── */
.ai-prompts { }
.ai-prompt-grid {
  display: grid; grid-template-columns: repeat(auto-fill, minmax(360px, 1fr)); gap: 12px;
}
.ai-prompt-card {
  background: #ffffff; border-radius: 10px; border: 1px solid #f0f0f0;
  padding: 16px 20px; transition: all 0.2s;
}
.ai-prompt-card:hover {
  border-color: #E5E7EB; box-shadow: 0 2px 12px rgba(0,0,0,0.04);
}
.ai-prompt-head { margin-bottom: 12px; }
.ai-prompt-name { font-size: 14px; font-weight: 600; color: #111827; margin: 0 0 6px; }
.ai-prompt-meta { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.ai-prompt-key {
  font-family: 'JetBrains Mono', monospace; font-size: 11px; color: #9CA3AF;
}
.ai-prompt-body { margin-bottom: 8px; }
.ai-prompt-preview {
  font-family: 'JetBrains Mono', monospace; font-size: 11px; color: #6B7280;
  line-height: 1.6; background: #F9FAFB; padding: 10px 12px; border-radius: 6px;
  border-left: 3px solid #6366F1; display: block; white-space: pre-wrap; word-break: break-all;
}
.ai-prompt-foot { font-size: 12px; color: #9CA3AF; }

/* ─── Modal ─── */
.ai-modal-inner { padding: 0; }
.ai-modal-head {
  display: flex; align-items: center; gap: 14px;
  padding-bottom: 20px; margin-bottom: 4px;
}
.ai-modal-icon {
  width: 44px; height: 44px; border-radius: 12px;
  background: linear-gradient(135deg, #EEF2FF, #E0E7FF);
  display: flex; align-items: center; justify-content: center;
  color: #4F46E5; flex-shrink: 0;
}
.ai-modal-title { font-size: 17px; font-weight: 700; color: #111827; margin: 0 0 2px; }
.ai-modal-desc { font-size: 13px; color: #9CA3AF; margin: 0; }
.ai-modal-form { margin-top: 8px; }
.ai-tpl-select { width: 100%; }
.ai-tpl-option { display: flex; align-items: center; gap: 8px; }
.ai-tpl-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; flex-shrink: 0; }
.ai-tpl-model { font-size: 11px; color: #9CA3AF; margin-left: auto; }
.ai-modal-divider {
  text-align: center; margin: 16px 0; position: relative;
}
.ai-modal-divider::before {
  content: ''; position: absolute; left: 0; right: 0; top: 50%;
  height: 1px; background: #F3F4F6;
}
.ai-modal-divider span {
  position: relative; background: #fff; padding: 0 12px;
  font-size: 11px; font-weight: 600; color: #9CA3AF; text-transform: uppercase; letter-spacing: 0.5px;
}
.ai-input-mono :deep(input) {
  font-family: 'JetBrains Mono', 'Fira Code', monospace !important;
  font-size: 13px;
}
.ai-input-full { width: 100%; }
.ai-form-hint { margin-top: 4px; }
.ai-hint-link { font-size: 12px; color: #6366F1; }
.ai-apikey-readonly {
  display: flex;
  align-items: center;
  gap: 8px;
}
.ai-apikey-readonly :deep(input) {
  background: #F9FAFB;
  color: #6B7280;
  cursor: not-allowed;
}
.ai-apikey-edit-btn {
  white-space: nowrap;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.ai-hint-link:hover { color: #4F46E5; }
.ai-modal-actions {
  display: flex; justify-content: flex-end; gap: 8px; margin-top: 24px; padding-top: 16px;
  border-top: 1px solid #F3F4F6;
}
</style>