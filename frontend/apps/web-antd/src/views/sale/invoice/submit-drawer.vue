<script lang="ts" setup>
// 提交审核页：提交前确认页面（不直接提交）
// 结构：票据摘要 → 审批流程预览（发起人 → 节点 → 结束）→ 底部确认提交
// 提交成功后由父组件刷新列表，操作列原位置显示"审核中"
import { computed, ref, watch } from 'vue';

import { Button, Drawer, message, Spin, Tag } from 'ant-design-vue';

import {
  getInvoiceApprovalPreviewApi,
  getInvoiceInfoApi,
  submitInvoiceApi,
} from '#/api';

const props = defineProps<{
  invoiceId: null | number;
  visible: boolean;
}>();
const emit = defineEmits<{
  success: [];
  'update:visible': [val: boolean];
}>();

const loading = ref(false);
const submitting = ref(false);
const detail = ref<any>(null);
const previewNodes = ref<any[]>([]);

const typeLabelMap: Record<number, string> = {
  1: '增值税专用发票',
  2: '增值税普通发票',
  3: '形式发票(PI)',
  4: '商业发票(CI)',
};
const approveModeMap: Record<number, { color: string; label: string }> = {
  1: { label: '或签', color: 'blue' },
  2: { label: '会签', color: 'purple' },
  3: { label: '依次审批', color: 'orange' },
};

const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};

// 价税合计
const totalAmount = computed(() => {
  const amount = Number(detail.value?.amount ?? 0);
  const tax = Number(detail.value?.taxAmount ?? 0);
  return amount + tax;
});

const drawerOpen = ref(false);
watch(
  () => props.visible,
  (val) => {
    drawerOpen.value = val;
  },
  { immediate: true },
);
watch(drawerOpen, (val) => {
  if (!val) emit('update:visible', false);
});

async function loadDetail() {
  if (!props.invoiceId) return;
  loading.value = true;
  try {
    const [info, preview] = await Promise.all([
      getInvoiceInfoApi(props.invoiceId),
      getInvoiceApprovalPreviewApi().catch(() => []),
    ]);
    detail.value = info ?? {};
    previewNodes.value = Array.isArray(preview) ? preview : [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val && props.invoiceId) {
      loadDetail();
    }
  },
);

// 提交审核：走系统审批流引擎（部门主管 → 财务审核）
async function handleSubmit() {
  if (!props.invoiceId) return;
  submitting.value = true;
  try {
    await submitInvoiceApi(props.invoiceId);
    message.success('已提交审核');
    emit('success');
    drawerOpen.value = false;
  } catch (error: any) {
    message.error(error?.message || '提交失败');
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Drawer
    v-model:open="drawerOpen"
    title="提交审核"
    placement="right"
    width="75%"
    :body-style="{ padding: '0' }"
    :header-style="{ borderBottom: '1px solid #f0f0f0' }"
  >
    <Spin :spinning="loading">
      <div class="submit-page">
        <!-- ===== 票据摘要 ===== -->
        <div class="summary-card">
          <div class="summary-head">
            <span class="summary-type-badge">{{
              typeLabelMap[detail?.invoiceType]?.charAt(0) || '票'
            }}</span>
            <div class="summary-head-text">
              <div class="summary-title">
                {{ detail?.title || '未命名发票' }}
              </div>
              <div class="summary-no">
                NO. {{ detail?.invoiceNo || '尚未分配票号' }}
              </div>
            </div>
            <div class="summary-amount">
              <div class="summary-amount-label">价税合计</div>
              <div class="summary-amount-value">
                {{ currencySymbolMap[detail?.currency] || '¥'
                }}{{
                  totalAmount.toLocaleString('zh-CN', {
                    minimumFractionDigits: 2,
                  })
                }}
              </div>
            </div>
          </div>
          <div class="summary-grid">
            <div class="summary-item">
              <span class="summary-label">发票类型</span>
              <span class="summary-value">{{
                typeLabelMap[detail?.invoiceType] || '—'
              }}</span>
            </div>
            <div class="summary-item">
              <span class="summary-label">客户</span>
              <span class="summary-value">{{
                detail?.customerName || '—'
              }}</span>
            </div>
            <div class="summary-item">
              <span class="summary-label">购买方</span>
              <span class="summary-value">{{
                detail?.buyerName || detail?.customerName || '—'
              }}</span>
            </div>
            <div class="summary-item">
              <span class="summary-label">开票日期</span>
              <span class="summary-value">{{
                detail?.invoiceDate || '—'
              }}</span>
            </div>
          </div>
        </div>

        <!-- ===== 审批流程预览 ===== -->
        <div class="flow-section">
          <div class="flow-title">审批流程</div>
          <div class="flow-desc">提交后将按以下环节依次审批</div>

          <div class="flow-line">
            <!-- 发起人 -->
            <div class="flow-step">
              <div class="flow-dot flow-dot--submitter">
                <svg
                  class="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width="2.5"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                  />
                </svg>
              </div>
              <div class="flow-step-text">
                <div class="flow-step-name">发起人</div>
                <div class="flow-step-sub">提交发票</div>
              </div>
            </div>

            <div class="flow-arrow"></div>

            <!-- 审批节点 -->
            <div
              v-for="node in previewNodes"
              :key="node.nodeKey"
              class="flow-step-group"
            >
              <div class="flow-step">
                <div class="flow-dot">
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                </div>
                <div class="flow-step-text">
                  <div class="flow-step-name">{{ node.nodeName }}</div>
                  <div class="flow-step-sub">
                    {{ node.approverRoleName || node.approverDesc }}
                  </div>
                  <Tag
                    v-if="approveModeMap[node.approveMode]"
                    :color="approveModeMap[node.approveMode]?.color"
                    class="flow-mode-tag"
                  >
                    {{ approveModeMap[node.approveMode]?.label }}
                  </Tag>
                </div>
              </div>
              <div class="flow-arrow"></div>
            </div>

            <!-- 结束 -->
            <div class="flow-step">
              <div class="flow-dot flow-dot--end">
                <svg
                  class="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width="2.5"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <div class="flow-step-text">
                <div class="flow-step-name">结束</div>
                <div class="flow-step-sub">审核通过，发票生效</div>
              </div>
            </div>
          </div>
        </div>

        <!-- ===== 提示 ===== -->
        <div class="tips">
          <div class="tips-title">提交须知</div>
          <ul class="tips-list">
            <li>提交后发票进入审批流程，期间不可编辑或删除</li>
            <li>任一环节驳回后可修改重新提交</li>
            <li>财务审核通过后可上传税控发票文件</li>
          </ul>
        </div>
      </div>
    </Spin>

    <template #footer>
      <div class="footer-bar">
        <Button @click="drawerOpen = false">取消</Button>
        <Button type="primary" :loading="submitting" @click="handleSubmit">
          确认提交
        </Button>
      </div>
    </template>
  </Drawer>
</template>

<style scoped>
.submit-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

/* ===== 票据摘要 ===== */
.summary-card {
  padding: 16px;
  background: linear-gradient(165deg, #eef4ff 0%, #f8fafc 55%);
  border: 1px solid hsl(214 30% 90%);
  border-radius: 8px;
}

.summary-head {
  display: flex;
  gap: 12px;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px dashed hsl(214 20% 85%);
}

.summary-type-badge {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  font-family: 'Songti SC', 'STSong', SimSun, serif;
  font-size: 16px;
  font-weight: 700;
  color: hsl(212 100% 40%);
  background: hsl(212 100% 40% / 0.09);
  border: 1px solid hsl(212 100% 40% / 0.3);
  border-radius: 6px;
}

.summary-head-text {
  flex: 1;
  min-width: 0;
}

.summary-title {
  overflow: hidden;
  font-size: 15px;
  font-weight: 600;
  color: hsl(215 25% 15%);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.summary-no {
  margin-top: 2px;
  font-family: ui-monospace, Consolas, monospace;
  font-size: 11.5px;
  color: hsl(215 15% 50%);
}

.summary-amount {
  flex-shrink: 0;
  text-align: right;
}

.summary-amount-label {
  font-size: 10px;
  color: hsl(215 15% 50%);
}

.summary-amount-value {
  font-family: ui-monospace, Consolas, monospace;
  font-size: 17px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: hsl(212 100% 35%);
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px 16px;
  padding-top: 12px;
}

.summary-item {
  display: flex;
  gap: 6px;
  font-size: 12.5px;
}

.summary-label {
  flex-shrink: 0;
  color: hsl(215 15% 50%);
}

.summary-value {
  overflow: hidden;
  min-width: 0;
  color: hsl(215 25% 20%);
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ===== 审批流程预览 ===== */
.flow-section {
  padding: 14px 16px;
  background: #fff;
  border: 1px solid hsl(214 20% 91%);
  border-radius: 8px;
}

.flow-title {
  font-size: 13.5px;
  font-weight: 600;
  color: hsl(215 25% 15%);
}

.flow-desc {
  margin-top: 2px;
  margin-bottom: 14px;
  font-size: 12px;
  color: hsl(215 12% 55%);
}

.flow-line {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.flow-step-group {
  display: flex;
  flex-direction: column;
}

.flow-step {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.flow-dot {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  color: hsl(212 100% 40%);
  background: hsl(212 100% 40% / 0.08);
  border-radius: 50%;
}

.flow-dot--submitter {
  color: hsl(152 60% 35%);
  background: hsl(152 60% 35% / 0.08);
}

.flow-dot--end {
  color: hsl(152 60% 32%);
  background: hsl(152 60% 32% / 0.1);
}

.flow-step-text {
  padding-top: 4px;
}

.flow-step-name {
  font-size: 13px;
  font-weight: 600;
  color: hsl(215 25% 15%);
}

.flow-step-sub {
  margin-top: 1px;
  font-size: 11.5px;
  color: hsl(215 12% 55%);
}

.flow-mode-tag {
  margin-top: 3px;
  margin-left: 0;
  font-size: 10.5px;
  line-height: 16px;
}

.flow-arrow {
  width: 32px;
  height: 18px;
  border-left: 1.5px dashed hsl(214 15% 78%);
}

/* ===== 提示 ===== */
.tips {
  padding: 12px 16px;
  background: hsl(45 96% 95%);
  border: 1px solid hsl(45 60% 88%);
  border-radius: 8px;
}

.tips-title {
  margin-bottom: 6px;
  font-size: 12.5px;
  font-weight: 600;
  color: hsl(35 80% 35%);
}

.tips-list {
  padding-left: 18px;
  margin: 0;
  font-size: 12px;
  line-height: 1.9;
  color: hsl(35 30% 40%);
}

.footer-bar {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
