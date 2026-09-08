<script lang="ts" setup>
/**
 * A-2.10: 注册页协议勾选字段组件
 * 默认不勾选；勾选状态通过 change 事件同步到父级，父级据此控制提交按钮禁用/启用。
 * 作为表单字段（fieldName: agreePolicy）使用，defineModel 绑定表单值。
 */
import { $t } from '@vben/locales';

const modelValue = defineModel<boolean>({ default: false });

const emit = defineEmits<{
  change: [boolean];
}>();

function onToggle(e: Event) {
  const next = (e.target as HTMLInputElement).checked;
  modelValue.value = next;
  emit('change', next);
}
</script>

<template>
  <label class="agree-box">
    <input
      :checked="modelValue"
      class="agree-input"
      type="checkbox"
      @change="onToggle"
    />
    <span class="agree-text">
      {{ $t('authentication.agree') }}
      <a class="vben-link ml-1" href="">
        {{ $t('authentication.privacyPolicy') }} & {{ $t('authentication.terms') }}
      </a>
    </span>
  </label>
</template>

<style scoped>
.agree-box {
  align-items: center;
  cursor: pointer;
  display: flex;
  font-size: 13px;
  gap: 6px;
  line-height: 1.5;
  user-select: none;
}
</style>
