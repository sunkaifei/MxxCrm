<script lang="ts" setup>
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { LockKeyhole } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Avatar, Button, InputPassword } from 'ant-design-vue';
import { useDateFormat, useNow } from '@vueuse/core';
import { useI18n } from 'vue-i18n';

const router = useRouter();
const accessStore = useAccessStore();

const { locale } = useI18n();

const now = useNow();
const meridiem = useDateFormat(now, 'A');
const hour = useDateFormat(now, 'HH');
const minute = useDateFormat(now, 'mm');
const date = useDateFormat(now, 'YYYY-MM-DD dddd', { locales: locale.value });

const submitting = ref(false);
const errorTip = ref('');
const password = ref('');
const ready = ref(false);  // hash 准备好后才能输入

// 标记页是否被劫持（防 F12 删除 isLockScreen 绕过）
const ACTIVE_KEY = '__lock_active__';

// 监听 hash 准备就绪：lockScreenPassword 是 64 位 SHA-256 字符串
watch(
  () => accessStore.lockScreenPassword,
  (val) => {
    if (val && val.length === 64) {
      ready.value = true;
    }
  },
  { immediate: true },
);

onMounted(() => {
  // 检查是否处于锁屏激活状态
  const active = sessionStorage.getItem(ACTIVE_KEY);
  if (active !== '1') {
    // 异常：没标记但访问了 /lock，跳回首页
    router.replace('/');
    return;
  }
  // 兜底：1.5 秒后无论 hash 是否准备好都强制 ready（防 user-dropdown 哈希失败时死锁）
  setTimeout(() => {
    if (!ready.value) {
      ready.value = true;
    }
  }, 1500);
});

const avatar = computed(
  () => accessStore.userInfo?.avatar ?? '/default-avatar.png',
);

/**
 * SHA-256 哈希
 */
async function sha256(text: string): Promise<string> {
  const buf = new TextEncoder().encode(text);
  const hashBuf = await crypto.subtle.digest('SHA-256', buf);
  return Array.from(new Uint8Array(hashBuf))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

async function handleUnlock() {
  errorTip.value = '';
  if (!password.value || password.value.length < 6) {
    errorTip.value = '请输入至少 6 位密码';
    return;
  }

  submitting.value = true;
  try {
    const inputHash = await sha256(password.value);
    const storedHash = accessStore.lockScreenPassword;

    // 始终校验密码哈希，不放行未设置密码的情况
    if (!storedHash || inputHash !== storedHash) {
      errorTip.value = storedHash ? '密码错误，请重试' : '系统异常，请稍后重试';
      password.value = '';
      return;
    }

    accessStore.unlockScreen();
    sessionStorage.removeItem(ACTIVE_KEY);
    await router.push('/');
  } catch {
    errorTip.value = '解锁失败，请重试';
  } finally {
    submitting.value = false;
  }
}

async function handleBackToLogin() {
  accessStore.unlockScreen();
  accessStore.setAccessToken(null);
  accessStore.setRefreshToken(null);
  sessionStorage.removeItem(ACTIVE_KEY);
  await router.push({
    path: '/auth/login',
    query: { redirect: encodeURIComponent(router.currentRoute.value.fullPath) },
  });
}

// 拦截键盘事件，防止 ESC 关闭
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    e.stopPropagation();
  }
}

// 阻止右键菜单
function handleContextMenu(e: MouseEvent) {
  e.preventDefault();
}
</script>

<template>
  <div
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-gradient-to-br from-slate-900 via-blue-950 to-slate-900"
    style="user-select: none; -webkit-user-select: none"
    role="dialog"
    aria-modal="true"
    @contextmenu="handleContextMenu"
    @keydown="handleKeydown"
  >
    <!-- 顶部时间和日期 -->
    <div class="absolute top-12 left-1/2 -translate-x-1/2 text-center">
      <div class="text-6xl font-bold text-white/90 tracking-wider">
        {{ hour }}:{{ minute }}
        <span class="text-2xl text-white/60 ml-2">{{ meridiem }}</span>
      </div>
      <div class="text-lg text-white/60 mt-2">{{ date }}</div>
    </div>

    <!-- 中间锁定图标 + 用户信息 -->
    <div class="flex flex-col items-center gap-6">
      <div class="relative">
        <div
          class="size-32 rounded-full bg-white/10 backdrop-blur flex items-center justify-center border-4 border-white/20 overflow-hidden"
        >
          <Avatar :src="avatar" :size="120" />
        </div>
        <div
          class="absolute -bottom-2 -right-2 size-12 rounded-full bg-blue-500 flex items-center justify-center border-4 border-slate-900"
        >
          <LockKeyhole class="size-6 text-white" />
        </div>
      </div>
      <div class="text-center">
        <div class="text-xl font-semibold text-white">
          {{ accessStore.userInfo?.realName || '已锁定' }}
        </div>
        <div class="text-sm text-white/60 mt-1">会话已锁定</div>
      </div>

      <!-- 解锁表单 -->
      <div class="w-[320px] mt-4">
        <div
          v-if="!ready"
          class="mb-3 px-3 py-2 text-sm text-white/60 text-center"
        >
          正在准备解锁...
        </div>
        <div
          v-else-if="errorTip"
          class="mb-3 px-3 py-2 text-sm text-red-300 bg-red-500/20 border border-red-500/30 rounded"
        >
          {{ errorTip }}
        </div>
        <InputPassword
          v-model:value="password"
          placeholder="请输入登录密码以解锁"
          size="large"
          autocomplete="current-password"
          :disabled="!ready"
          @keydown.enter="handleUnlock"
        />
        <Button
          type="primary"
          class="mt-3 w-full"
          size="large"
          :loading="submitting || !ready"
          :disabled="!ready"
          @click="handleUnlock"
        >
          解锁
        </Button>
        <Button
          class="mt-2 w-full"
          size="large"
          :disabled="!ready"
          @click="handleBackToLogin"
        >
          重新登录
        </Button>
      </div>
    </div>

    <!-- 底部提示 -->
    <div class="absolute bottom-8 left-1/2 -translate-x-1/2 text-center">
      <div class="text-xs text-white/40">
        出于安全考虑，已锁定此会话。请输入登录密码解锁。
      </div>
    </div>
  </div>
</template>
