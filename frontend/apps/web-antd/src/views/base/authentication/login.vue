<script lang="ts" setup>
import type { Component } from 'vue';

import type { VbenFormSchema } from '@vben/common-ui';

import type { SsoProviderStatus } from '#/api';

import { computed, markRaw, onMounted, ref } from 'vue';

import { AuthenticationLogin, z } from '@vben/common-ui';
import { LucideMessageSquare, LucideUsers, MdiWechat } from '@vben/icons';
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

// ==================== 测试账号速登（仅测试环境配置开启时渲染） ====================
// 开关来自 env：VITE_SHOW_TEST_ACCOUNTS=true（测试系统配置），生产配置不注入即隐藏
const showTestAccounts = (import.meta.env as any).VITE_SHOW_TEST_ACCOUNTS === 'true';

interface TestAccount {
  username: string;
  password: string;
  role: string;
  dept: string;
  perm: string;
}

// 演示账号清单（与 README 一致）
const TEST_ACCOUNTS: TestAccount[] = [
  { username: 'admin', password: 'admin123', role: '超级管理员', dept: '总经办', perm: '拥有系统所有权限' },
  { username: 'system', password: 'admin123', role: '系统管理员', dept: '总经办', perm: '系统配置管理' },
  { username: 'sales', password: 'admin123', role: '销售总监', dept: '销售部', perm: '销售管理、CRM客户管理' },
  { username: 'manager', password: 'admin123', role: '销售经理', dept: '销售一组', perm: '销售管理、CRM客户管理' },
  { username: 'rep', password: 'admin123', role: '业务员', dept: '销售一组', perm: '销售管理、CRM客户管理（数据范围受限）' },
  { username: 'zhangsan', password: 'admin123', role: '业务员', dept: '销售二组', perm: '销售管理、CRM客户管理（数据范围受限）' },
  { username: 'lisi', password: 'admin123', role: '业务员', dept: '销售一组', perm: '销售管理、CRM客户管理（数据范围受限）' },
  { username: 'ceo', password: 'admin123', role: 'CEO', dept: '总经办', perm: '销售总监权限、审批终审' },
  { username: 'purchase', password: 'admin123', role: '采购专员', dept: '采购部', perm: '采购管理' },
  { username: 'warehouse', password: 'admin123', role: '库管', dept: '采购部', perm: '仓储入库/出库审核、库存管理、库存调整' },
  { username: 'operator', password: 'admin123', role: '制单员', dept: '采购部', perm: '仓储入库/出库单据录入（制审分离：不可审核自己的单）' },
  { username: 'finance', password: 'admin123', role: '财务专员', dept: '财务部门', perm: '财务管理' },
  { username: 'fin_manager', password: 'admin123', role: '财务经理', dept: '财务部门', perm: '财务结算、工资审核' },
  { username: 'hr_manager', password: 'admin123', role: '人事经理', dept: '人事部', perm: '人事管理、入职/离职审核' },
];

// 部门分组色点：同一体系同一色相，快速辨识账号所属条线
const DEPT_COLORS: Record<string, string> = {
  总经办: '#8b5cf6',
  销售部: '#3b82f6',
  销售一组: '#3b82f6',
  销售二组: '#6366f1',
  采购部: '#f59e0b',
  财务部门: '#10b981',
  人事部: '#ec4899',
};

const testAccountsVisible = ref(false);

// 原生方式写值并派发 input 事件，确保 v-model / vee-validate 状态同步
function setNativeInputValue(el: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(
    HTMLInputElement.prototype,
    'value',
  )?.set;
  setter?.call(el, value);
  el.dispatchEvent(new Event('input', { bubbles: true }));
  el.dispatchEvent(new Event('change', { bubbles: true }));
}

async function fillAccount(acc: TestAccount) {
  // 直接写 DOM 输入框并派发 input 事件（v-model/vee-validate 均能同步）。
  // 不走 AuthenticationLogin 的 formApi.setFieldValue——实测其 Promise 会悬挂导致弹窗无法关闭。
  const inputs = Array.from(
    document.querySelectorAll<HTMLInputElement>('form input'),
  );
  const userInput = inputs.find((i) => i.type !== 'password');
  const pwdInput = inputs.find((i) => i.type === 'password');
  if (userInput) setNativeInputValue(userInput, acc.username);
  if (pwdInput) setNativeInputValue(pwdInput, acc.password);
  testAccountsVisible.value = false;
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

        <!-- 测试账号速登入口（仅测试系统配置显示） -->
        <button
          v-if="showTestAccounts"
          class="ta-trigger"
          type="button"
          @click="testAccountsVisible = true"
        >
          <LucideUsers class="ta-trigger-ic" />
          <span>{{ $t('page.auth.testAccounts.trigger') }}</span>
          <span class="ta-trigger-count">{{ TEST_ACCOUNTS.length }}</span>
        </button>
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

    <!-- 测试账号一览弹窗（Teleport 避开登录卡片层叠上下文） -->
    <Teleport to="body">
      <Transition name="ta-fade">
        <div
          v-if="testAccountsVisible"
          class="ta-mask"
          @click.self="testAccountsVisible = false"
        >
          <div class="ta-modal" role="dialog" aria-modal="true">
            <div class="ta-accent"></div>
            <div class="ta-head">
              <div class="ta-head-title">
                {{ $t('page.auth.testAccounts.title') }}
                <span class="ta-badge">{{ TEST_ACCOUNTS.length }}</span>
              </div>
              <button
                aria-label="close"
                class="ta-close"
                type="button"
                @click="testAccountsVisible = false"
              >
                ✕
              </button>
            </div>
            <p class="ta-hint">{{ $t('page.auth.testAccounts.hint') }}</p>

            <div class="ta-cols">
              <span>{{ $t('page.auth.testAccounts.colAccount') }}</span>
              <span>{{ $t('page.auth.testAccounts.colRole') }}</span>
              <span>{{ $t('page.auth.testAccounts.colDept') }}</span>
              <span>{{ $t('page.auth.testAccounts.colPerm') }}</span>
            </div>

            <div class="ta-rows">
              <button
                v-for="(acc, idx) in TEST_ACCOUNTS"
                :key="acc.username"
                class="ta-row"
                :style="{ '--i': idx }"
                type="button"
                @click="fillAccount(acc)"
              >
                <span class="ta-acc">{{ acc.username }}</span>
                <span class="ta-role">{{ acc.role }}</span>
                <span class="ta-dept">
                  <i
                    class="ta-dot"
                    :style="{ background: DEPT_COLORS[acc.dept] || 'var(--muted-foreground, #999)' }"
                  ></i>
                  {{ acc.dept }}
                </span>
                <span class="ta-perm" :title="acc.perm">{{ acc.perm }}</span>
              </button>
            </div>

            <div class="ta-foot">{{ $t('page.auth.testAccounts.passwordNote') }}</div>
          </div>
        </div>
      </Transition>
    </Teleport>
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

<style scoped>
/* ===== 测试账号速登：触发按钮 ===== */
.ta-trigger {
  align-items: center;
  background: transparent;
  border: 1px dashed hsl(var(--border));
  border-radius: 8px;
  color: hsl(var(--foreground) / 60%);
  cursor: pointer;
  display: flex;
  font-size: 12.5px;
  gap: 7px;
  height: 38px;
  justify-content: center;
  margin-top: 16px;
  transition:
    border-color 0.2s,
    color 0.2s,
    transform 0.2s;
  width: 100%;
}
.ta-trigger:hover {
  border-color: hsl(var(--primary) / 70%);
  color: hsl(var(--primary));
  transform: translateY(-1px);
}
.ta-trigger:hover .ta-trigger-ic {
  transform: rotate(-8deg) scale(1.1);
}
.ta-trigger-ic {
  height: 15px;
  transition: transform 0.25s;
  width: 15px;
}
.ta-trigger-count {
  border: 1px solid currentcolor;
  border-radius: 999px;
  font-family: ui-monospace, monospace;
  font-size: 10.5px;
  line-height: 1;
  opacity: 0.75;
  padding: 2px 7px;
}

/* ===== 弹窗 ===== */
.ta-mask {
  align-items: center;
  background: rgb(0 0 0 / 45%);
  backdrop-filter: blur(3px);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 20px;
  position: fixed;
  z-index: 2000;
}
.ta-modal {
  animation: ta-pop 0.22s ease-out;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 14px;
  box-shadow:
    0 24px 60px rgb(0 0 0 / 25%),
    0 4px 16px rgb(0 0 0 / 10%);
  display: flex;
  flex-direction: column;
  max-height: 82vh;
  overflow: hidden;
  width: min(680px, 94vw);
}
/* 顶部渐变饰线：弹窗唯一的主色宣言 */
.ta-accent {
  background: linear-gradient(90deg, hsl(var(--primary)) 0%, hsl(var(--primary) / 0%) 70%);
  height: 3px;
  flex-shrink: 0;
}
.ta-head {
  align-items: center;
  display: flex;
  justify-content: space-between;
  padding: 16px 18px 0;
}
.ta-head-title {
  align-items: center;
  display: flex;
  font-size: 15px;
  font-weight: 600;
  gap: 8px;
}
.ta-badge {
  background: hsl(var(--primary) / 12%);
  border-radius: 999px;
  color: hsl(var(--primary));
  font-family: ui-monospace, monospace;
  font-size: 11px;
  line-height: 1;
  padding: 3px 8px;
}
.ta-close {
  background: transparent;
  border: none;
  border-radius: 50%;
  color: hsl(var(--foreground) / 50%);
  cursor: pointer;
  font-size: 13px;
  height: 28px;
  line-height: 1;
  transition:
    background 0.15s,
    color 0.15s;
  width: 28px;
}
.ta-close:hover {
  background: hsl(var(--muted));
  color: hsl(var(--foreground));
}
.ta-hint {
  color: hsl(var(--foreground) / 50%);
  font-size: 12px;
  margin: 4px 0 0;
  padding: 0 18px;
}

/* ===== 表头 + 行 ===== */
.ta-cols,
.ta-row {
  align-items: center;
  display: grid;
  gap: 10px;
  grid-template-columns: 118px 100px 112px 1fr;
}
.ta-cols {
  color: hsl(var(--foreground) / 45%);
  font-size: 11px;
  padding: 14px 18px 6px;
}
.ta-rows {
  min-height: 0;
  overflow-y: auto;
  padding: 0 10px 8px;
}
.ta-row {
  animation: ta-row-in 0.25s both;
  animation-delay: calc(var(--i) * 18ms);
  background: transparent;
  border: none;
  border-radius: 8px;
  color: hsl(var(--foreground));
  cursor: pointer;
  font-size: 12.5px;
  padding: 8px 10px;
  text-align: left;
  transition: background 0.15s;
}
.ta-row:hover {
  background: hsl(var(--primary) / 8%);
}
.ta-row:active {
  background: hsl(var(--primary) / 15%);
}
.ta-acc {
  font-family: ui-monospace, monospace;
  font-size: 12.5px;
  font-weight: 600;
}
.ta-dept {
  align-items: center;
  display: flex;
  gap: 5px;
}
.ta-dot {
  border-radius: 50%;
  flex-shrink: 0;
  height: 6px;
  width: 6px;
}
.ta-perm {
  color: hsl(var(--foreground) / 55%);
  font-size: 11.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ta-foot {
  border-top: 1px dashed hsl(var(--border));
  color: hsl(var(--foreground) / 50%);
  font-size: 11.5px;
  padding: 10px 18px;
}

/* ===== 动效 ===== */
@keyframes ta-pop {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.96);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
@keyframes ta-row-in {
  from {
    opacity: 0;
    transform: translateX(-6px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
.ta-fade-enter-active,
.ta-fade-leave-active {
  transition: opacity 0.18s ease;
}
.ta-fade-enter-from,
.ta-fade-leave-to {
  opacity: 0;
}
</style>
