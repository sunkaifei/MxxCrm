<script lang="ts" setup>
/**
 * 轻量取色器（替代 ant-design-vue ColorPicker）
 *
 * 背景：本项目的 ant-design-vue 为 4.2.6，**尚无 ColorPicker 导出**（4.3+ 才引入）。
 * 为避免为一个色板升级整个组件库，这里用原生 `<input type="color">` + 十六进制
 * 文本框自建一个等价控件，值始终是 `#RRGGBB` 字符串。
 */
import { computed } from 'vue';

import { Input } from 'ant-design-vue';

const props = withDefaults(
  defineProps<{
    disabled?: boolean;
    /** 十六进制色值，如 #1f2937 */
    value?: string;
  }>(),
  { disabled: false, value: '#000000' },
);

const emit = defineEmits<{ (e: 'update:value', v: string): void }>();

/** 统一成 #RRGGBB，容错 #RGB / 无 # / 非法值 */
const hex = computed(() => {
  const raw = (props.value ?? '').trim();
  if (/^#[0-9a-f]{6}$/i.test(raw)) return raw.toLowerCase();
  if (/^#[0-9a-f]{3}$/i.test(raw)) {
    const [r, g, b] = [raw[1], raw[2], raw[3]];
    return `#${r}${r}${g}${g}${b}${b}`.toLowerCase();
  }
  if (/^[0-9a-f]{6}$/i.test(raw)) return `#${raw}`.toLowerCase();
  return '#000000';
});

function onPick(e: Event) {
  emit('update:value', (e.target as HTMLInputElement).value);
}

function onType(e: Event) {
  const v = (e.target as HTMLInputElement).value.trim();
  // 允许用户输入中途的不完整值，但不向下游抛出非法色值
  if (/^#?[0-9a-f]{3}$/i.test(v) || /^#?[0-9a-f]{6}$/i.test(v)) {
    emit('update:value', v.startsWith('#') ? v.toLowerCase() : `#${v.toLowerCase()}`);
  }
}
</script>

<template>
  <div class="color-field">
    <input
      class="cf-swatch"
      :disabled="disabled"
      :value="hex"
      type="color"
      @input="onPick"
    />
    <Input
      :disabled="disabled"
      :value="hex"
      class="cf-hex"
      size="small"
      @change="onType"
    />
  </div>
</template>

<style scoped>
.color-field {
  display: flex;
  gap: 6px;
  align-items: center;
  width: 100%;
}

.cf-swatch {
  width: 28px;
  height: 24px;
  padding: 0;
  cursor: pointer;
  background: transparent;
  border: 1px solid rgb(0 0 0 / 15%);
  border-radius: 4px;
}

.cf-hex {
  flex: 1;
  min-width: 0;
  font-family: ui-monospace, monospace;
  text-transform: lowercase;
}
</style>
