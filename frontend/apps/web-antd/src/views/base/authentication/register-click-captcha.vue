<script lang="ts" setup>
/**
 * A-2.10: 注册页点选验证码触发器（表单字段 captchaTicket）
 * 整行淡化提示"点击完成验证"，点击弹出气泡浮动窗口（锚定本字段附近，与登录页一致的
 * 文字点选验证）；验证通过后回填一次性 ticket。
 * 组件通过 onApiReady 回调向父级注册 open()，供"提交时未验证 → 自动弹出"使用。
 */
import { nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue';

import ClickCaptcha from './click-captcha.vue';

const ticket = defineModel<string>({ default: '' });

const props = defineProps<{
  onApiReady?: (api: { open: () => void } | null) => void;
}>();

const show = ref(false);
const arrowUp = ref(true);
const popupStyle = reactive({ top: 0, left: 0 });
const triggerEl = ref<HTMLDivElement | null>(null);
const popupEl = ref<HTMLDivElement | null>(null);

function layoutPopup() {
  if (!triggerEl.value || !popupEl.value) return;
  const rect = triggerEl.value.getBoundingClientRect();
  const pw = popupEl.value.offsetWidth;
  const ph = popupEl.value.offsetHeight;
  const gap = 10;
  // 固定显示在触发区域上方（下方会被表单遮挡）；仅当上方真放不下（会溢出视口顶部）时才兜底下方
  let top = rect.top - ph - gap;
  if (top < 8) {
    arrowUp.value = false;
    top = rect.bottom + gap;
  } else {
    arrowUp.value = true;
  }
  let left = rect.left + rect.width / 2 - pw / 2;
  left = Math.max(8, Math.min(left, window.innerWidth - pw - 8));
  popupStyle.top = top;
  popupStyle.left = left;
}

function open() {
  show.value = true;
}

function close() {
  show.value = false;
}

function toggle() {
  if (show.value) {
    close();
  } else {
    open();
  }
}

function onVerified(t: string) {
  ticket.value = t;
  close();
}

function onDocMousedown(e: MouseEvent) {
  if (!show.value) return;
  const target = e.target as Node;
  if (popupEl.value?.contains(target)) return;
  if (triggerEl.value?.contains(target)) return;
  close();
}

function onResize() {
  if (show.value) layoutPopup();
}

let ro: ResizeObserver | null = null;

// 打开时布局气泡；气泡尺寸变化（验证码图片异步加载导致高度变化）时重新定位，防止顶部溢出
watch(show, (val) => {
  if (val) {
    nextTick(() => {
      layoutPopup();
      ro?.disconnect();
      ro = new ResizeObserver(() => {
        if (show.value) layoutPopup();
      });
      if (popupEl.value) ro.observe(popupEl.value);
    });
  } else {
    ro?.disconnect();
    ro = null;
  }
});

onMounted(() => {
  document.addEventListener('mousedown', onDocMousedown);
  window.addEventListener('resize', onResize);
  props.onApiReady?.({ open });
});

onUnmounted(() => {
  document.removeEventListener('mousedown', onDocMousedown);
  window.removeEventListener('resize', onResize);
  ro?.disconnect();
  props.onApiReady?.(null);
});
</script>

<template>
  <div class="captcha-trigger">
    <div
      ref="triggerEl"
      class="captcha-field"
      :class="{ done: !!ticket }"
      role="button"
      tabindex="0"
      @click="toggle"
      @keydown.enter="toggle"
    >
      <span class="captcha-text" :class="{ done: !!ticket }">
        {{ ticket ? '✓ 验证通过' : '点击完成验证' }}
      </span>
    </div>
    <!-- 气泡浮动窗口: 锚定本字段附近浮动显示，点击外部/✕/取消关闭 -->
    <Teleport to="body">
      <div
        v-if="show"
        ref="popupEl"
        class="captcha-popup"
        :class="arrowUp ? 'arrow-up' : 'arrow-down'"
        :style="{ left: `${popupStyle.left}px`, top: `${popupStyle.top}px` }"
      >
        <div class="popup-head">
          <span class="popup-title">安全验证</span>
          <button
            aria-label="关闭"
            class="popup-close"
            type="button"
            @click="close"
          >
            ✕
          </button>
        </div>
        <ClickCaptcha @cancelled="close" @verified="onVerified" />
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.captcha-trigger {
  width: 100%;
}
.captcha-field {
  align-items: center;
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  height: 40px;
  justify-content: center;
  outline: none;
  transition: border-color 0.2s;
  width: 100%;
}
.captcha-field:hover,
.captcha-field:focus-visible {
  border-color: hsl(var(--primary));
}
.captcha-field.done {
  border-color: #1a9;
}
.captcha-text {
  color: hsl(var(--foreground) / 35%);
  font-size: 13px;
  user-select: none;
}
.captcha-text.done {
  color: #1a9;
}
.captcha-popup {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  box-shadow: 0 10px 30px rgb(0 0 0 / 15%);
  max-width: calc(100vw - 16px);
  padding: 14px;
  position: fixed;
  width: 360px;
  z-index: 1000;
}
.captcha-popup::before {
  background: hsl(var(--card));
  content: '';
  height: 10px;
  left: 50%;
  position: absolute;
  transform: translateX(-50%) rotate(45deg);
  width: 10px;
}
.captcha-popup.arrow-up::before {
  border-left: 1px solid hsl(var(--border));
  border-top: 1px solid hsl(var(--border));
  top: -6px;
}
.captcha-popup.arrow-down::before {
  border-bottom: 1px solid hsl(var(--border));
  border-right: 1px solid hsl(var(--border));
  bottom: -6px;
}
.popup-head {
  align-items: center;
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}
.popup-title {
  font-size: 15px;
  font-weight: 600;
}
.popup-close {
  background: transparent;
  border: none;
  color: hsl(var(--foreground) / 60%);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 4px;
}
.popup-close:hover {
  color: hsl(var(--foreground));
}
</style>
