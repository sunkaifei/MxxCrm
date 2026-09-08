<script lang="ts" setup>
/**
 * 批2: 个人中心——账号绑定 Tab
 * 展示当前用户已绑定的第三方账号（企业微信/钉钉），支持发起绑定与解绑
 */
import type { Component } from 'vue';

import type { SsoBindItem, SsoProviderStatus } from '#/api';

import { computed, onMounted, ref } from 'vue';

import { LucideMessageSquare, MdiWechat } from '@vben/icons';

import { message, Modal } from 'ant-design-vue';

import {
  ssoBindListApi,
  ssoBindStartApi,
  ssoBindUnbindApi,
  ssoStatusApi,
} from '#/api';

const PROVIDER_NAMES: Record<string, string> = {
  wecom: '企业微信',
  dingtalk: '钉钉',
};

const PROVIDER_ICONS: Record<string, Component> = {
  wecom: MdiWechat,
  dingtalk: LucideMessageSquare,
};

function providerName(code: string): string {
  return PROVIDER_NAMES[code] ?? code;
}

function providerIcon(code: string): Component {
  return PROVIDER_ICONS[code] ?? LucideMessageSquare;
}

const loading = ref(false);
const providers = ref<SsoProviderStatus[]>([]);
const bindMap = ref<Record<string, SsoBindItem>>({});

const bindCount = computed(
  () => Object.values(bindMap.value).filter((b) => !!b.provider).length,
);

function bindItem(code: string): SsoBindItem | undefined {
  return bindMap.value[code];
}

async function loadAll() {
  loading.value = true;
  try {
    const [status, binds] = await Promise.all([
      ssoStatusApi().catch(() => null),
      ssoBindListApi().catch(() => []),
    ]);
    // 展示已配置渠道；排除 oidc（OIDC 不支持账号绑定），未配置的不展示
    providers.value = (status?.providers ?? []).filter(
      (p) => p.configured && p.code !== 'oidc',
    );
    const map: Record<string, SsoBindItem> = {};
    for (const b of binds ?? []) {
      if (b?.provider) map[b.provider] = b;
    }
    bindMap.value = map;
  } finally {
    loading.value = false;
  }
}

onMounted(loadAll);

const binding = ref(false);

async function startBind(provider: string) {
  // 登录态 XHR 取授权 URL 后跳转：浏览器直跳无法携带 Authorization 头
  binding.value = true;
  try {
    const res = await ssoBindStartApi(provider);
    const url = (res as any)?.url;
    if (url) {
      window.location.href = url;
    } else {
      message.error('未获取到授权地址');
    }
  } catch {
    // 全局拦截器已提示业务错误（离职/未配置等）
  } finally {
    binding.value = false;
  }
}

function unbind(provider: string) {
  Modal.confirm({
    title: '解绑第三方账号',
    content: `确定解除与「${providerName(provider)}」的绑定？解绑后该第三方账号将无法直接登录。`,
    okText: '解绑',
    okButtonProps: { danger: true },
    cancelText: '取消',
    onOk: async () => {
      try {
        await ssoBindUnbindApi(provider);
        message.success('解绑成功');
        await loadAll();
      } catch (error: any) {
        message.error(error?.msg || error?.message || '解绑失败');
      }
    },
  });
}
</script>

<template>
  <div class="bind-page">
    <div class="card">
      <h3>第三方账号绑定</h3>
      <p class="muted">
        绑定后可用第三方应用扫码直接登录；解绑后该第三方账号将无法登录。
      </p>
      <div v-if="loading" class="muted">加载中…</div>
      <div v-else-if="providers.length === 0" class="muted">
        暂无可用的第三方登录渠道，请联系管理员在「系统设置 → 集成配置 → 第三方登录」中启用。
      </div>
      <template v-else>
        <div v-for="p in providers" :key="p.code" class="row">
          <span class="p-icon">
            <component :is="providerIcon(p.code)" />
          </span>
          <span class="p-name">{{ providerName(p.code) }}</span>
          <template v-if="bindItem(p.code)">
            <span class="ok">已绑定（{{ bindItem(p.code)!.provider_uid }}）</span>
            <span v-if="bindItem(p.code)!.created_at" class="muted">
              {{ bindItem(p.code)!.created_at }}
            </span>
            <button class="btn danger" @click="unbind(p.code)">解绑</button>
          </template>
          <template v-else-if="!p.enabled">
            <span class="muted">已被管理员停用</span>
          </template>
          <template v-else>
            <span class="muted">未绑定</span>
            <button
              class="btn primary"
              :disabled="binding"
              @click="startBind(p.code)"
            >
              {{ binding ? '跳转中…' : '绑定' }}
            </button>
          </template>
        </div>
        <p v-if="bindCount" class="muted bind-tip">
          已绑定 {{ bindCount }} 个第三方账号，登录时优先按绑定关系识别。
        </p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.btn {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  height: 32px;
  margin-left: auto;
  padding: 0 14px;
  white-space: nowrap;
}
.btn.danger {
  color: #d33;
}
.btn.primary {
  background: hsl(var(--primary));
  border: none;
  color: #fff;
}
.btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
.bind-page {
  padding: 4px;
}
.bind-tip {
  margin-top: 10px;
}
.card {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  margin-bottom: 16px;
  padding: 16px;
}
.muted {
  color: hsl(var(--foreground) / 55%);
  font-size: 13px;
}
.ok {
  color: #1a9;
  font-size: 13px;
}
.p-icon {
  align-items: center;
  border: 1px solid hsl(var(--border));
  border-radius: 50%;
  display: flex;
  height: 30px;
  justify-content: center;
  width: 30px;
}
.p-icon :deep(svg) {
  height: 16px;
  width: 16px;
}
.p-name {
  font-size: 14px;
  font-weight: 500;
}
.row {
  align-items: center;
  border-bottom: 1px solid hsl(var(--border) / 60%);
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  padding: 10px 2px;
}
.row:last-of-type {
  border-bottom: none;
}
</style>
