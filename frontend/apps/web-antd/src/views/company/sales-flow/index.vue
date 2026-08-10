<script lang="ts" setup>
import { ref, computed, onMounted } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Card,
  message,
  Spin,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import {
  getSalesFlowModeApi,
  setSalesFlowModeApi,
  type SalesFlowMode,
} from '#/api';

const accessStore = useAccessStore();

// 通过权限码判断是否有编辑权限
const canEdit = computed(() =>
  accessStore.hasAccessCode('company:sales-flow:update'),
);

const loading = ref(false);
const saving = ref(false);
const currentMode = ref<SalesFlowMode>('both');
// 临时选中的模式（点击卡片时记录，未保存前不写 currentMode）
const selectedMode = ref<SalesFlowMode>('both');
// 是否已修改未保存
const isDirty = computed(() => selectedMode.value !== currentMode.value);

// 三种模式定义
const modeOptions: Array<{
  value: SalesFlowMode;
  title: string;
  description: string;
  flow: string;
  badge?: string;
  recommended?: boolean;
}> = [
  {
    value: 'A',
    title: '标准流程',
    description: '客户 → 商机 → 报价单 → 订单 → 合同',
    flow: '适用于大额订单、需要正式报价、需要比价/竞争的项目。所有订单必须经过报价单。',
    badge: '严格',
  },
  {
    value: 'B',
    title: '简易流程',
    description: '客户 → 商机 → 订单 → 合同',
    flow: '适用于老客户复购、小额订单、无需报价环节。商机可直接转订单，但必须关联商机。',
    badge: '快捷',
  },
  {
    value: 'both',
    title: '两种都允许',
    description: '商机处同时显示"转报价单"和"转订单"两个入口',
    flow: '默认推荐。业务员可根据实际情况自行选择走标准流程或简易流程，灵活性最高。',
    recommended: true,
    badge: '推荐',
  },
];

// 加载当前模式
const loadMode = async () => {
  loading.value = true;
  try {
    const mode = await getSalesFlowModeApi();
    // 后端可能返回空字符串，做兜底
    const safeMode: SalesFlowMode =
      mode === 'A' || mode === 'B' || mode === 'both' ? mode : 'both';
    currentMode.value = safeMode;
    selectedMode.value = safeMode;
  } catch (e) {
    console.error('加载销售流程模式失败', e);
    currentMode.value = 'both';
    selectedMode.value = 'both';
  } finally {
    loading.value = false;
  }
};

// 选择模式（仅记录，不立即保存）
const handleSelect = (mode: SalesFlowMode) => {
  if (!canEdit.value || saving.value) return;
  selectedMode.value = mode;
};

// 保存
const handleSave = async () => {
  if (!isDirty.value) return;
  saving.value = true;
  try {
    await setSalesFlowModeApi(selectedMode.value);
    currentMode.value = selectedMode.value;
    message.success('销售流程模式已更新');
  } catch (e) {
    console.error(e);
  } finally {
    saving.value = false;
  }
};

// 取消
const handleCancel = () => {
  selectedMode.value = currentMode.value;
};

onMounted(() => {
  loadMode();
});
</script>

<template>
  <Page :hide-footer="true">
    <Card :bordered="false" class="sales-flow-card">
      <template #title>
        <span>销售流程模式</span>
      </template>
      <template #extra>
        <span class="current-tip">
          当前模式：
          <Tag color="blue">
            {{ modeOptions.find((m) => m.value === currentMode)?.title || '-' }}
          </Tag>
        </span>
      </template>

      <Spin :spinning="loading">
        <div class="mode-grid">
          <div
            v-for="opt in modeOptions"
            :key="opt.value"
            class="mode-card"
            :class="{
              active: selectedMode === opt.value,
              disabled: !canEdit,
            }"
            @click="handleSelect(opt.value)"
          >
            <div class="mode-card-header">
              <div class="mode-title">
                {{ opt.title }}
                <Tag v-if="opt.badge" :color="opt.recommended ? 'green' : 'default'" class="ml-2">
                  {{ opt.badge }}
                </Tag>
              </div>
              <Tooltip :title="opt.description">
                <span class="mode-value">模式 {{ opt.value }}</span>
              </Tooltip>
            </div>

            <div class="mode-flow">{{ opt.description }}</div>
            <div class="mode-desc">{{ opt.flow }}</div>

            <div class="mode-check" v-if="selectedMode === opt.value">
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path
                  fill="currentColor"
                  d="M9 16.17 4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"
                />
              </svg>
            </div>
          </div>
        </div>

        <!-- 模式说明 -->
        <div class="mode-tips">
          <div class="tip-title">说明</div>
          <ul>
            <li><b>标准流程</b>：所有订单必须从报价单转化而来，商机列表"更多"操作只显示"转报价单"。</li>
            <li><b>简易流程</b>：商机可直接转订单，订单表单中"关联报价单"字段隐藏，"关联商机"改为必填。</li>
            <li><b>两种都允许</b>：商机列表"更多"操作同时显示"转报价单"和"转订单"，业务员自行选择。</li>
            <li>配置变更不影响已有数据，仅在新建/转单时按当前模式校验。</li>
          </ul>
        </div>

        <!-- 操作按钮 -->
        <div v-if="canEdit" class="mode-actions">
          <Button
            type="primary"
            :loading="saving"
            :disabled="!isDirty"
            @click="handleSave"
          >
            保存
          </Button>
          <Button :disabled="!isDirty" @click="handleCancel" class="ml-2">
            取消
          </Button>
          <span v-if="isDirty" class="dirty-tip">有未保存的修改</span>
        </div>
      </Spin>
    </Card>
  </Page>
</template>

<style scoped>
.sales-flow-card {
  margin-bottom: 16px;
}

.current-tip {
  font-size: 13px;
  color: #666;
}

.mode-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.mode-card {
  position: relative;
  padding: 20px;
  border: 2px solid #e8e8e8;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  background: #fff;
}

.mode-card:hover:not(.disabled) {
  border-color: #1677ff;
  box-shadow: 0 2px 8px rgba(22, 119, 255, 0.12);
}

.mode-card.active {
  border-color: #1677ff;
  background: #f0f7ff;
}

.mode-card.disabled {
  cursor: not-allowed;
  opacity: 0.85;
}

.mode-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.mode-title {
  font-size: 16px;
  font-weight: 600;
  color: #1f1f1f;
}

.mode-value {
  font-size: 12px;
  color: #999;
  cursor: help;
  border-bottom: 1px dashed #ccc;
}

.mode-flow {
  font-size: 14px;
  color: #1677ff;
  font-weight: 500;
  margin-bottom: 8px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.mode-desc {
  font-size: 13px;
  color: #666;
  line-height: 1.6;
}

.mode-check {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: #1677ff;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mode-tips {
  padding: 16px;
  background: #fafafa;
  border-radius: 6px;
  border-left: 4px solid #1677ff;
}

.tip-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #1f1f1f;
}

.mode-tips ul {
  margin: 0;
  padding-left: 20px;
}

.mode-tips li {
  font-size: 13px;
  color: #555;
  line-height: 1.8;
}

.mode-actions {
  margin-top: 20px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.dirty-tip {
  margin-left: 12px;
  color: #faad14;
  font-size: 13px;
}

.ml-2 {
  margin-left: 8px;
}

@media (max-width: 768px) {
  .mode-grid {
    grid-template-columns: 1fr;
  }
}
</style>
