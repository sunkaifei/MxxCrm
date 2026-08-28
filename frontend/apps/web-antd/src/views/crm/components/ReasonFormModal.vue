<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import {
  Button,
  message,
  Modal,
  Select,
  Textarea,
} from 'ant-design-vue';

/**
 * 通用原因表单弹窗组件
 *
 * 复用于三处场景（mode 区分）：
 * - mode='pool'：退回公海 / 退回线索池 —— 原因类型下拉（必选）+ 补充说明
 *   （类型为"其他"时补充说明必填，与后端 validate_release_reason 校验对齐）
 * - mode='void'：商机作废 —— 仅作废原因文本域（必填）
 *
 * 组件只负责收集与校验，提交 API 由父组件在 @confirm 回调中执行；
 * 父组件通过 :submitting 控制按钮 loading，成功后自行关闭弹窗。
 *
 * 用法：
 * <ReasonFormModal
 *   v-model:visible="reasonVisible"
 *   title="退回公海"
 *   mode="pool"
 *   ok-text="确认退回"
 *   :submitting="reasonSubmitting"
 *   @confirm="onReasonConfirm"
 * />
 */

interface Props {
  /** 弹窗是否可见 */
  visible: boolean;
  /** 弹窗标题（退回公海/退回线索池/作废商机） */
  title: string;
  /** 表单模式：pool=退回（原因类型+说明） void=作废（仅说明） */
  mode?: 'pool' | 'void';
  /** 确认按钮文案 */
  okText?: string;
  /** 父组件提交中（控制确认按钮 loading） */
  submitting?: boolean;
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
  (e: 'confirm', data: { reason: string; reasonType?: number }): void;
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'pool',
  okText: '确定',
  submitting: false,
});

const emit = defineEmits<Emits>();

const innerVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

// 退回原因类型（与后端 RELEASE_REASON_TYPES = [1,2,3,4,9] 对齐）
const REASON_TYPE_OTHER = 9;
const reasonTypeOptions = [
  { label: '跟进无回应', value: 1 },
  { label: '客户无意向', value: 2 },
  { label: '客户信息无效', value: 3 },
  { label: '换业务方向', value: 4 },
  { label: '其他', value: 9 },
];

const form = ref<{ reason: string; reasonType?: number }>({
  reason: '',
  reasonType: undefined,
});

// 是否选择了"其他"（补充说明变为必填）
const isOther = computed(
  () => props.mode === 'pool' && form.value.reasonType === REASON_TYPE_OTHER,
);

const reasonLabel = computed(() =>
  props.mode === 'void' ? '作废原因' : '补充说明',
);

const reasonPlaceholder = computed(() => {
  if (props.mode === 'void') {
    return '请填写作废原因（必填，最多 200 字）';
  }
  return isOther.value
    ? '退回原因为【其他】时，请补充说明（必填，最多 200 字）'
    : '可补充说明退回原因（选填，最多 200 字）';
});

const confirmDisabled = computed(() => {
  if (props.mode === 'void') {
    return !form.value.reason.trim();
  }
  if (form.value.reasonType === undefined) return true;
  if (isOther.value && !form.value.reason.trim()) return true;
  return false;
});

// 弹窗打开时重置表单
watch(
  () => props.visible,
  (val) => {
    if (val) {
      form.value = { reason: '', reasonType: undefined };
    }
  },
  { immediate: true },
);

function handleSubmit() {
  const reason = form.value.reason.trim();
  if (props.mode === 'void') {
    if (!reason) {
      message.warning('请填写作废原因');
      return;
    }
    emit('confirm', { reason });
    return;
  }
  if (form.value.reasonType === undefined) {
    message.warning('请选择退回原因类型');
    return;
  }
  if (form.value.reasonType === REASON_TYPE_OTHER && !reason) {
    message.warning('退回原因为【其他】时，必须填写补充说明');
    return;
  }
  emit('confirm', { reason, reasonType: form.value.reasonType });
}
</script>

<template>
  <Modal
    v-model:open="innerVisible"
    :title="title"
    :width="520"
    :destroy-on-close="true"
    :mask-closable="false"
    :footer="null"
    class="reason-form-modal"
  >
    <!-- 原因类型（退回场景） -->
    <div v-if="mode === 'pool'" class="reason-section">
      <div class="section-title">
        <span class="title-bar"></span>
        <span>退回原因类型</span>
        <span class="required-mark">*</span>
      </div>
      <Select
        v-model:value="form.reasonType"
        :options="reasonTypeOptions"
        placeholder="请选择退回原因类型"
        class="reason-select"
      />
    </div>

    <!-- 补充说明 / 作废原因 -->
    <div class="reason-section">
      <div class="section-title">
        <span class="title-bar"></span>
        <span>{{ reasonLabel }}</span>
        <span class="required-mark">*</span>
      </div>
      <Textarea
        v-model:value="form.reason"
        :rows="4"
        :maxlength="200"
        show-count
        :placeholder="reasonPlaceholder"
        class="reason-textarea"
      />
    </div>

    <!-- 底部操作按钮 -->
    <div class="footer-actions">
      <Button @click="innerVisible = false">取消</Button>
      <Button
        type="primary"
        danger
        :loading="submitting"
        :disabled="confirmDisabled"
        @click="handleSubmit"
      >
        {{ okText }}
      </Button>
    </div>
  </Modal>
</template>

<style scoped>
.reason-form-modal :deep(.ant-modal-body) {
  padding: 16px 24px 8px;
}

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
  display: inline-block;
  width: 3px;
  height: 14px;
  background: #1677ff;
  border-radius: 2px;
}

.required-mark {
  font-size: 14px;
  color: #ff4d4f;
}

.reason-select {
  width: 100%;
}

.footer-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 12px 0 16px;
  border-top: 1px solid #f0f0f0;
}
</style>
