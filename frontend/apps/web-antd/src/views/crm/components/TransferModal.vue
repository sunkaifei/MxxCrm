<script lang="ts" setup>
import type { DictDataOptionVO, TransferPreviewVO } from '#/api';

/**
 * 客户转移弹窗组件
 *
 * 功能：
 * - 选择新负责人（复用 UserSelectModal）
 * - 自动预览影响范围（商机/合同/订单/回款计划/回款/发票/报价单 数量统计）
 * - 交接原因 Radio Group（从字典 crm_transfer_reason 加载）
 * - 备注 TextArea（非必填，最多 200 字）
 * - 二次确认防误操作
 * - 成功后 emit success 通知父组件刷新
 *
 * 用法：
 * <TransferModal
 *   v-model:visible="visible"
 *   :customer-ids="selectedIds"
 *   @success="onTransferSuccess"
 * />
 */
import { computed, ref, watch } from 'vue';

import {
  LucideArrowRight,
  LucideFileText,
  LucideLoader2,
  LucidePackage,
  LucideReceipt,
  LucideScrollText,
  LucideSend,
  LucideUserCheck,
  LucideUsers,
  LucideWallet,
} from '@vben/icons';

import {
  Button,
  Empty,
  message,
  Modal,
  Radio,
  RadioGroup,
  Skeleton,
  Spin,
  Tag,
  Textarea,
} from 'ant-design-vue';

import {
  getDictOptionsApi,
  previewCustomerTransferApi,
  transferCustomerApi,
} from '#/api';

import UserSelectModal from './UserSelectModal.vue';

interface Props {
  /** 弹窗是否可见 */
  visible: boolean;
  /** 待转移的客户 ID 列表 */
  customerIds: number[];
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
  (
    e: 'success',
    data: { affectedTotal: number; transferredCount: number },
  ): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// ===== 弹窗可见性（v-model:visible）=====
const innerVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

// ===== 用户选择器 =====
const userSelectVisible = ref(false);
const selectedUser = ref<null | {
  id?: number;
  nickName?: string;
  userName?: string;
}>(null);

function openUserSelect() {
  userSelectVisible.value = true;
}

function onUserSelect(row: any) {
  selectedUser.value = {
    id: row.id,
    nickName: row.nickName || row.realName || row.name,
    userName: row.userName,
  };
  userSelectVisible.value = false;
  // 选择新负责人后自动预览
  loadPreview();
}

function clearSelectedUser() {
  selectedUser.value = null;
  previewData.value = null;
}

// ===== 字典数据（交接原因）=====
const reasonOptions = ref<DictDataOptionVO[]>([]);
const reasonLoading = ref(false);

async function loadReasonOptions() {
  if (reasonOptions.value.length > 0) return;
  reasonLoading.value = true;
  try {
    const data = await getDictOptionsApi('crm_transfer_reason');
    reasonOptions.value = Array.isArray(data) ? data : [];
    // 默认选中第一项（或 isDefault=1 的项）
    const defaultItem =
      reasonOptions.value.find((it) => it.isDefault === 1) ||
      reasonOptions.value[0];
    if (defaultItem && !form.value.transferReason) {
      form.value.transferReason = defaultItem.label;
    }
  } catch {
    // 字典加载失败时使用兜底选项
    reasonOptions.value = [
      { label: '员工离职交接', value: '1', isDefault: 1 },
      { label: '员工调岗', value: '2', isDefault: 0 },
      { label: '其他', value: '8', isDefault: 0 },
    ];
    form.value.transferReason = '员工离职交接';
  } finally {
    reasonLoading.value = false;
  }
}

// ===== 表单数据 =====
const form = ref({
  transferReason: '',
  remark: '',
});

// ===== 预览数据 =====
const previewData = ref<null | TransferPreviewVO>(null);
const previewLoading = ref(false);

async function loadPreview() {
  if (!selectedUser.value?.id || props.customerIds.length === 0) {
    previewData.value = null;
    return;
  }
  previewLoading.value = true;
  previewData.value = null;
  try {
    const data = await previewCustomerTransferApi({
      customerIds: props.customerIds,
      toUserId: selectedUser.value.id,
    });
    previewData.value = data;
  } catch {
    previewData.value = null;
  } finally {
    previewLoading.value = false;
  }
}

// ===== 影响范围统计项配置（图标 + 颜色 + 标签）=====
const statItems = computed(() => {
  if (!previewData.value) return [];
  const d = previewData.value;
  return [
    {
      key: 'customer',
      label: '客户',
      value: d.customerCount,
      icon: LucideUsers,
      color: '#1677ff',
      bg: '#e6f4ff',
    },
    {
      key: 'opportunity',
      label: '商机',
      value: d.opportunityCount,
      icon: LucideSend,
      color: '#722ed1',
      bg: '#f9f0ff',
    },
    {
      key: 'quotation',
      label: '报价单',
      value: d.quotationCount,
      icon: LucideFileText,
      color: '#13c2c2',
      bg: '#e6fffb',
    },
    {
      key: 'order',
      label: '订单',
      value: d.orderCount,
      icon: LucidePackage,
      color: '#fa8c16',
      bg: '#fff7e6',
    },
    {
      key: 'contract',
      label: '合同',
      value: d.contractCount,
      icon: LucideScrollText,
      color: '#eb2f96',
      bg: '#fff0f6',
    },
    {
      key: 'paymentPlan',
      label: '回款计划',
      value: d.paymentPlanCount,
      icon: LucideWallet,
      color: '#52c41a',
      bg: '#f6ffed',
    },
    {
      key: 'payment',
      label: '回款',
      value: d.paymentCount,
      icon: LucideWallet,
      color: '#a0d911',
      bg: '#f6ffed',
    },
    {
      key: 'invoice',
      label: '发票',
      value: d.invoiceCount,
      icon: LucideReceipt,
      color: '#fa541c',
      bg: '#fff2e8',
    },
  ];
});

// ===== 提交转移 =====
const submitting = ref(false);

function handleSubmit() {
  // 校验
  const user = selectedUser.value;
  const userId = user?.id;
  if (!userId) {
    message.warning('请先选择新负责人');
    return;
  }
  if (!form.value.transferReason) {
    message.warning('请选择交接原因');
    return;
  }

  // 二次确认
  const customerCount = props.customerIds.length;
  const affectedTotal = previewData.value?.affectedTotal ?? 0;
  Modal.confirm({
    title: '确认转移客户？',
    content: `即将把 ${customerCount} 个客户及其关联数据（共 ${affectedTotal} 条）转移给「${
      user.nickName || user.userName
    }」，此操作不可撤销。`,
    okText: '确认转移',
    cancelText: '再想想',
    okType: 'primary',
    okButtonProps: { danger: false },
    onOk: async () => {
      submitting.value = true;
      try {
        const result = await transferCustomerApi({
          customerIds: props.customerIds,
          toUserId: userId,
          transferReason: form.value.transferReason,
          remark: form.value.remark?.trim() || undefined,
        });
        message.success(
          `转移成功：已转移 ${result.transferredCount} 个客户，影响 ${result.affectedTotal} 条关联数据`,
        );
        emit('success', {
          transferredCount: result.transferredCount,
          affectedTotal: result.affectedTotal,
        });
        // 关闭弹窗
        innerVisible.value = false;
      } catch (error: any) {
        // requestClient 已有全局错误处理，这里兜底

        console.error('[TransferModal] 转移失败', error);
      } finally {
        submitting.value = false;
      }
    },
  });
}

// ===== 弹窗打开时初始化 =====
watch(
  () => props.visible,
  (val) => {
    if (val) {
      // 重置状态
      selectedUser.value = null;
      previewData.value = null;
      form.value = { transferReason: '', remark: '' };
      // 加载字典
      loadReasonOptions();
    }
  },
  { immediate: true },
);
</script>

<template>
  <Modal
    v-model:open="innerVisible"
    title="客户转移"
    :width="640"
    :destroy-on-close="true"
    :mask-closable="false"
    :footer="null"
    class="transfer-modal"
  >
    <!-- 顶部：客户数量 + 新负责人选择 -->
    <div class="transfer-header-card">
      <div class="header-item">
        <div class="header-item-label">
          <LucideUsers class="header-icon" />
          <span>转移客户</span>
        </div>
        <div class="header-item-value">
          <span class="value-num">{{ customerIds.length }}</span>
          <span class="value-unit">个</span>
        </div>
      </div>

      <div class="header-arrow">
        <LucideArrowRight />
      </div>

      <div class="header-item">
        <div class="header-item-label">
          <LucideUserCheck class="header-icon" />
          <span>新负责人</span>
        </div>
        <div v-if="!selectedUser" class="header-user-empty">
          <Button type="primary" size="small" ghost @click="openUserSelect">
            选择员工
          </Button>
        </div>
        <div v-else class="header-user-selected">
          <Tag color="blue" class="user-tag">
            {{ selectedUser.nickName || selectedUser.userName }}
          </Tag>
          <a class="change-link" @click="openUserSelect">更换</a>
          <a class="clear-link" @click="clearSelectedUser">清除</a>
        </div>
      </div>
    </div>

    <!-- 影响范围预览 -->
    <div class="preview-section">
      <div class="section-title">
        <span class="title-bar"></span>
        <span>影响范围预览</span>
        <Tag v-if="previewData" color="blue" class="total-tag">
          共 {{ previewData.affectedTotal }} 条
        </Tag>
      </div>

      <!-- 加载中 -->
      <div v-if="previewLoading" class="preview-loading">
        <Skeleton active :paragraph="{ rows: 3 }" />
      </div>

      <!-- 未选择负责人 -->
      <div v-else-if="!selectedUser" class="preview-empty">
        <Empty
          description="请先选择新负责人"
          :image-style="{ height: '50px' }"
        />
      </div>

      <!-- 预览数据 -->
      <div v-else-if="previewData" class="preview-grid">
        <div
          v-for="item in statItems"
          :key="item.key"
          class="stat-card"
          :class="{ 'stat-zero': item.value === 0 }"
        >
          <div
            class="stat-icon-wrap"
            :style="{ backgroundColor: item.bg, color: item.color }"
          >
            <component :is="item.icon" />
          </div>
          <div class="stat-info">
            <div
              class="stat-value"
              :style="{ color: item.value > 0 ? item.color : '#bfbfbf' }"
            >
              {{ item.value }}
            </div>
            <div class="stat-label">{{ item.label }}</div>
          </div>
        </div>
      </div>

      <!-- 无数据 -->
      <div v-else class="preview-empty">
        <Empty description="暂无影响数据" :image-style="{ height: '50px' }" />
      </div>
    </div>

    <!-- 交接原因 + 备注 -->
    <div class="reason-section">
      <div class="section-title">
        <span class="title-bar"></span>
        <span>交接原因</span>
        <span class="required-mark">*</span>
      </div>
      <Spin :spinning="reasonLoading">
        <RadioGroup
          v-model:value="form.transferReason"
          class="reason-radio-group"
        >
          <Radio
            v-for="opt in reasonOptions"
            :key="opt.value"
            :value="opt.label"
            class="reason-radio-item"
          >
            {{ opt.label }}
          </Radio>
        </RadioGroup>
      </Spin>

      <div class="remark-wrap">
        <div class="remark-label">
          <span>备注</span>
          <span class="optional-tag">选填</span>
        </div>
        <Textarea
          v-model:value="form.remark"
          :rows="3"
          :maxlength="200"
          show-count
          placeholder="可填写本次转移的详细说明（非必填，最多 200 字）"
          class="remark-textarea"
        />
      </div>
    </div>

    <!-- 底部操作按钮 -->
    <div class="footer-actions">
      <Button @click="innerVisible = false">取消</Button>
      <Button
        type="primary"
        :loading="submitting"
        :disabled="!selectedUser || !form.transferReason"
        @click="handleSubmit"
      >
        <template #icon v-if="submitting"><LucideLoader2 /></template>
        确认转移
      </Button>
    </div>

    <!-- 用户选择器弹窗 -->
    <UserSelectModal
      v-model:visible="userSelectVisible"
      :extra-params="{ status: 1 }"
      @select="onUserSelect"
    />
  </Modal>
</template>

<style scoped>
.transfer-modal :deep(.ant-modal-body) {
  padding: 16px 24px 8px;
}

/* ===== 顶部信息卡片 ===== */
.transfer-header-card {
  display: flex;
  gap: 12px;
  align-items: stretch;
  padding: 16px;
  margin-bottom: 20px;
  background: linear-gradient(135deg, #f5f9ff 0%, #f0f5ff 100%);
  border: 1px solid #d6e4ff;
  border-radius: 8px;
}

.header-item {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
  padding: 4px 0;
}

.header-item-label {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 13px;
  color: #595959;
}

.header-icon {
  width: 16px;
  height: 16px;
  color: #1677ff;
}

.header-item-value {
  display: flex;
  gap: 4px;
  align-items: baseline;
}

.value-num {
  font-size: 28px;
  font-weight: 600;
  line-height: 1;
  color: #1677ff;
}

.value-unit {
  font-size: 13px;
  color: #8c8c8c;
}

.header-arrow {
  display: flex;
  align-items: center;
  padding: 0 4px;
  color: #bfbfbf;
}

.header-arrow svg {
  width: 18px;
  height: 18px;
}

.header-user-empty {
  display: flex;
  align-items: center;
  height: 32px;
}

.header-user-selected {
  display: flex;
  gap: 8px;
  align-items: center;
}

.user-tag {
  padding: 2px 10px;
  margin: 0;
  font-size: 13px;
}

.change-link,
.clear-link {
  font-size: 12px;
  color: #1677ff;
  cursor: pointer;
}

.clear-link {
  color: #8c8c8c;
}

.change-link:hover,
.clear-link:hover {
  text-decoration: underline;
}

/* ===== 预览区 ===== */
.preview-section,
.reason-section {
  margin-bottom: 20px;
}

.section-title {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 12px;
  font-size: 14px;
  font-weight: 500;
  color: #262626;
}

.title-bar {
  width: 3px;
  height: 14px;
  background: #1677ff;
  border-radius: 2px;
}

.total-tag {
  margin-left: 4px;
  font-size: 12px;
}

.preview-loading,
.preview-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100px;
  padding: 16px;
  background: #fafafa;
  border-radius: 6px;
}

.preview-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.stat-card {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 10px 8px;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.stat-card:hover {
  background: #fff;
  border-color: #d9d9d9;
  box-shadow: 0 2px 6px rgb(0 0 0 / 4%);
  transform: translateY(-1px);
}

.stat-card.stat-zero {
  opacity: 0.55;
}

.stat-icon-wrap {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
}

.stat-icon-wrap svg {
  width: 14px;
  height: 14px;
}

.stat-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.stat-value {
  font-size: 16px;
  font-weight: 600;
  line-height: 1;
}

.stat-label {
  font-size: 11px;
  line-height: 1;
  color: #8c8c8c;
}

/* ===== 交接原因 + 备注 ===== */
.reason-radio-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 0;
}

.reason-radio-item {
  margin-right: 16px;
  margin-bottom: 4px;
}

.reason-radio-item:last-child {
  margin-right: 0;
}

.remark-wrap {
  margin-top: 16px;
}

.remark-label {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 6px;
  font-size: 13px;
  color: #595959;
}

.optional-tag {
  padding: 1px 6px;
  font-size: 11px;
  color: #8c8c8c;
  background: #f5f5f5;
  border-radius: 2px;
}

.required-mark {
  margin-left: 2px;
  font-size: 14px;
  color: #ff4d4f;
}

.remark-textarea {
  width: 100%;
}

/* ===== 底部操作 ===== */
.footer-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding-top: 12px;
  margin-top: 8px;
  border-top: 1px solid #f0f0f0;
}

/* ===== 响应式 ===== */
@media (max-width: 640px) {
  .preview-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .transfer-header-card {
    flex-direction: column;
    gap: 8px;
  }

  .header-arrow {
    padding: 0;
    transform: rotate(90deg);
  }
}
</style>
