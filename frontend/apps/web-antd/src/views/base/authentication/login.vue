<script lang="ts" setup>
import type { Component } from 'vue';

import type { VbenFormSchema } from '@vben/common-ui';

import type { SsoProviderStatus } from '#/api';

import { computed, markRaw, onMounted, ref } from 'vue';

import { AuthenticationLogin, z } from '@vben/common-ui';
import { LucideMessageSquare, MdiWechat } from '@vben/icons';
import { $t } from '@vben/locales';

import {
  getCaptchaConfigApi,
  getRegisterStatusApi,
  ssoAuthorizeUrl,
  ssoStatusApi,
} from '#/api';
import { useAuthStore } from '#/store';

import ClickCaptcha from './click-captcha.vue';
import ImageCaptcha, { captchaKeyRef } from './image-captcha.vue';

defineOptions({ name: 'Login' });

const authStore = useAuthStore();

// 注册开关：默认关闭，查询后更新
const registerEnabled = ref(false);

// 点选方案: 验证码形态（image=图形字符 / click=文字点选浮层 / disabled=不启用），登录页按此分派
const captchaMode = ref<'click' | 'disabled' | 'image'>('image');
const showClickCaptcha = ref(false);
// 点选触发时暂存的登录参数（账号密码/记住我），验证通过后携 ticket 完成登录
const pendingLoginParams = ref<null | Record<string, any>>(null);

// 批2: SSO 第三方登录入口（仅渲染已配置且开启的 provider）
const ssoProviders = ref<SsoProviderStatus[]>([]);

// 图标映射: 企业微信/钉钉（钉钉无品牌图标，用通用消息图标）
const PROVIDER_ICONS: Record<string, Component> = {
  wecom: MdiWechat,
  dingtalk: LucideMessageSquare,
};

function providerIcon(code: string): Component {
  return PROVIDER_ICONS[code] ?? LucideMessageSquare;
}

function handleSsoLogin(provider: string) {
  window.location.href = ssoAuthorizeUrl(provider);
}

// 批2 MFA: 二次验证输入
const mfaCode = ref('');

// A-2.9: 登录页使用内置"记住账号"复选框（与"忘记密码"同行）——勾选 = 记住用户名（本地预填）+ 30 天内免登录
const REMEMBER_ME_KEY = `REMEMBER_ME_USERNAME_${location.hostname}`;
const rememberedUsername = localStorage.getItem(REMEMBER_ME_KEY) || '';

onMounted(async () => {
  try {
    const data = await getRegisterStatusApi();
    registerEnabled.value = data?.registerEnabled ?? false;
  } catch {
    // 查询失败默认不显示注册入口
  }
  try {
    const sso = await ssoStatusApi();
    ssoProviders.value = (sso?.providers ?? []).filter(
      (p) => p.configured && p.enabled,
    );
  } catch {
    ssoProviders.value = [];
  }
  try {
    const cfg: any = await getCaptchaConfigApi();
    if (cfg?.enabled) {
      captchaMode.value = cfg.mode === 'click' ? 'click' : 'image';
    } else {
      captchaMode.value = 'disabled';
    }
  } catch {
    captchaMode.value = 'image'; // 配置查询失败回退图形模式（保守）
  }
});

const formSchema = computed((): VbenFormSchema[] => {
  const list: VbenFormSchema[] = [
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: $t('authentication.usernameTip'),
      },
      defaultValue: rememberedUsername,
      fieldName: 'username',
      label: $t('authentication.username'),
      rules: z.string().min(1, { message: $t('authentication.usernameTip') }),
    },
    {
      component: 'VbenInputPassword',
      componentProps: {
        placeholder: $t('authentication.password'),
      },
      fieldName: 'password',
      label: $t('authentication.password'),
      rules: z.string().min(1, { message: $t('authentication.passwordTip') }),
    },
  ];
  // 点选方案: 图形模式才渲染图形验证码字段；click 模式由登录按钮触发弹窗
  if (captchaMode.value === 'image') {
    list.push({
      component: markRaw(ImageCaptcha),
      componentProps: {
        placeholder: $t('page.auth.loginSecurity.captchaTip'),
      },
      fieldName: 'captchaCode',
      rules: z
        .string()
        .min(1, { message: $t('page.auth.loginSecurity.captchaTip') }),
    });
  }
  // A-2.9: 记住账号由内置复选框承载（与"忘记密码"同行），不再作为表单项单独占一行
  return list;
});

function handleLogin(params: Record<string, any>) {
  // A-2.9: 记住账号（内置复选框）——用户名预填由内置组件写入 localStorage；rememberMe 随登录参数转发，后端据此签发 30 天 refreshToken
  if (captchaMode.value === 'disabled') {
    // 后端不校验验证码，直登
    return authStore.authLogin({ ...params });
  }
  if (captchaMode.value === 'click') {
    // 点选方案: 暂存参数并弹出点选浮层，验证通过后携 ticket 自动续登
    pendingLoginParams.value = { ...params };
    showClickCaptcha.value = true;
    return;
  }
  // 图形模式: 验证码 key 与输入值配套提交
  return authStore.authLogin({ ...params, captchaKey: captchaKeyRef.value });
}

function handleClickVerified(ticket: string) {
  const params = pendingLoginParams.value ?? {};
  showClickCaptcha.value = false;
  pendingLoginParams.value = null;
  return authStore.authLogin({ ...params, captchaTicket: ticket });
}

function handleClickCancelled() {
  showClickCaptcha.value = false;
  pendingLoginParams.value = null;
}

// 批2 MFA: 二次验证提交
async function handleVerifyMfa() {
  if (!mfaCode.value.trim()) return;
  await authStore.verifyMfa(mfaCode.value.trim());
}
</script>

<template>
  <div class="login-wrap">
    <AuthenticationLogin
      v-if="!showClickCaptcha"
      :form-schema="formSchema"
      :loading="authStore.loginLoading"
      :show-code-login="false"
      :show-forget-password="true"
      :show-qrcode-login="false"
      :show-register="registerEnabled"
      :show-remember-me="true"
      :show-third-party-login="false"
      @submit="handleLogin"
    >
      <template v-if="captchaMode === 'click'" #to>
        <p class="click-hint">登录需完成文字点选验证</p>
      </template>
      <!-- 批2: 第三方登录（企业微信/钉钉，仅展示已配置且开启的 provider） -->
      <template #third-party-login>
        <div v-if="ssoProviders.length > 0" class="sso-panel">
          <div class="sso-divider">
            <span class="sso-divider-line"></span>
            <span class="sso-divider-text">其他登录方式</span>
            <span class="sso-divider-line"></span>
          </div>
          <div class="sso-icons">
            <a
              v-for="p in ssoProviders"
              :key="p.code"
              class="sso-icon"
              :title="p.name"
              @click="handleSsoLogin(p.code)"
            >
              <component :is="providerIcon(p.code)" class="sso-icon-ic" />
            </a>
          </div>
        </div>
      </template>
    </AuthenticationLogin>

    <!-- 点选方案: 文字点选浮层 -->
    <div v-if="showClickCaptcha" class="click-layer">
      <p class="click-title">安全验证</p>
      <ClickCaptcha @cancelled="handleClickCancelled" @verified="handleClickVerified" />
    </div>

    <!-- 批2 MFA: 登录二次验证输入 -->
    <div v-if="authStore.mfaTicket" class="click-layer">
      <p class="click-title">两步验证</p>
      <p v-if="authStore.mfaEmail" class="click-hint">验证码已发送至 {{ authStore.mfaEmail }}</p>
      <p v-else class="click-hint">请输入认证器中的 6 位动态码</p>
      <input
        v-model="mfaCode"
        :disabled="authStore.loginLoading"
        class="mfa-ipt"
        maxlength="6"
        placeholder="动态验证码"
        @keyup.enter="handleVerifyMfa"
      />
      <button :disabled="authStore.loginLoading" class="click-btn" @click="handleVerifyMfa">验证并登录</button>
      <button class="click-btn ghost" @click="authStore.mfaTicket = ''">返回重新登录</button>
    </div>
  </div>
</template>

<style scoped>
.click-btn {
  background: hsl(var(--primary));
  border: none;
  border-radius: 6px;
  color: #fff;
  cursor: pointer;
  height: 38px;
  width: 100%;
}
.click-btn.ghost {
  background: transparent;
  border: 1px solid hsl(var(--border));
  color: hsl(var(--foreground));
}
.click-btn:disabled {
  opacity: 0.6;
}
.mfa-ipt {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  height: 40px;
  letter-spacing: 6px;
  outline: none;
  padding: 0 10px;
  text-align: center;
}
.mfa-ipt + .click-btn,
.mfa-ipt + .click-btn + .click-btn {
  margin-top: 10px;
}
</style>

<style scoped>
.click-hint {
  color: hsl(var(--foreground) / 55%);
  font-size: 12px;
  margin: 0;
}
.click-layer {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  box-shadow: 0 8px 30px hsl(var(--border) / 60%);
  margin: 0 auto;
  max-width: 380px;
  padding: 18px;
}
.click-title {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 12px;
}
.login-wrap {
  width: 100%;
}
.sso-panel {
  margin-top: 18px;
}
.sso-divider {
  align-items: center;
  display: flex;
  gap: 12px;
}
.sso-divider-line {
  border-bottom: 1px solid hsl(var(--border));
  flex: 1;
}
.sso-divider-text {
  color: hsl(var(--foreground) / 55%);
  font-size: 12px;
  white-space: nowrap;
}
.sso-icons {
  display: flex;
  gap: 16px;
  justify-content: center;
  margin-top: 14px;
}
.sso-icon {
  align-items: center;
  border: 1px solid hsl(var(--border));
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  height: 40px;
  justify-content: center;
  transition: border-color 0.2s, transform 0.2s;
  width: 40px;
}
.sso-icon:hover {
  border-color: hsl(var(--primary));
  transform: translateY(-2px);
}
.sso-icon-ic {
  height: 20px;
  width: 20px;
}
</style>
