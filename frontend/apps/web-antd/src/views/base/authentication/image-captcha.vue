<script lang="ts">
/**
 * A-0.2: 登录图形验证码组件
 * 展示后端 /api/open/captcha/get 生成的验证码图片（base64），点击图片刷新；
 * captchaKey 通过共享 ref 提交给登录请求（与验证码输入值配套校验）。
 * 注意：<script setup> 不允许运行时导出，共享 ref 由本普通 <script> 块导出。
 */
import { ref } from 'vue';

// 共享给登录提交处使用（schema 组件与提交逻辑解耦）
export const captchaKeyRef = ref('');
</script>

<script lang="ts" setup>
import { onMounted } from 'vue';

import { getCaptchaApi } from '#/api';

defineProps<{
  placeholder?: string;
}>();

const modelValue = defineModel<string>({ default: '' });

const captchaBase64 = ref('');

async function refresh() {
  try {
    const data = await getCaptchaApi();
    captchaBase64.value = data?.captchaBase64 ?? '';
    captchaKeyRef.value = data?.captchaKey ?? '';
  } catch {
    // 加载失败保持空值，后端会以"验证码不能为空"拒绝
  }
}

onMounted(() => {
  refresh();
});
</script>

<template>
  <div class="captcha-row">
    <input
      v-model="modelValue"
      :placeholder="placeholder"
      autocomplete="off"
      class="captcha-input"
      type="text"
    />
    <img
      v-if="captchaBase64"
      :src="`data:image/png;base64,${captchaBase64}`"
      alt="captcha"
      class="captcha-img"
      title="点击刷新"
      @click="refresh"
    />
    <div v-else class="captcha-img captcha-loading" @click="refresh">加载中</div>
  </div>
</template>

<style scoped>
.captcha-row {
  display: flex;
  gap: 8px;
  width: 100%;
}

.captcha-input {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  flex: 1;
  height: 40px;
  min-width: 0;
  outline: none;
  padding: 0 10px;
}

.captcha-input:focus {
  border-color: hsl(var(--primary));
}

.captcha-img {
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  height: 40px;
  width: 110px;
}

.captcha-loading {
  align-items: center;
  display: flex;
  font-size: 12px;
  justify-content: center;
}
</style>
