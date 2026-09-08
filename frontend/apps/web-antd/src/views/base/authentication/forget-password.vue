<script lang="ts" setup>
/**
 * 批2: 找回密码——两步流程
 * 第一步：输入绑定邮箱 → 后端发送验证码，返回一次性票据
 * 第二步：输入邮箱验证码 + 新密码 → 重置成功后回登录页
 */
import { ref } from 'vue';
import { useRouter } from 'vue-router';

import { message } from 'ant-design-vue';

import { forgotPasswordApi, resetPasswordApi } from '#/api';

defineOptions({ name: 'ForgetPassword' });

const router = useRouter();

const step = ref<1 | 2>(1);
const loading = ref(false);
const email = ref('');
const ticket = ref('');
const emailMasked = ref('');
const code = ref('');
const password = ref('');
const confirmPassword = ref('');

async function sendCode() {
  if (!email.value.trim() || !email.value.includes('@')) {
    message.error('请输入正确的邮箱地址');
    return;
  }
  loading.value = true;
  try {
    const res: any = await forgotPasswordApi({ email: email.value.trim() });
    ticket.value = res.ticket;
    emailMasked.value = res.emailMasked;
    step.value = 2;
    message.success(`验证码已发送至 ${res.emailMasked}`);
  } catch (e: any) {
    message.error(e?.msg || e?.message || '验证码发送失败，请确认邮箱已绑定账号');
  } finally {
    loading.value = false;
  }
}

async function resetPassword() {
  if (!code.value.trim()) {
    message.error('请输入邮箱验证码');
    return;
  }
  if (password.value.length < 8 || !/[A-Za-z]/.test(password.value) || !/\d/.test(password.value)) {
    message.error('新密码需 8-64 位且同时包含字母和数字');
    return;
  }
  if (password.value !== confirmPassword.value) {
    message.error('两次输入的新密码不一致');
    return;
  }
  loading.value = true;
  try {
    await resetPasswordApi({
      ticket: ticket.value,
      code: code.value.trim(),
      password: password.value,
    });
    message.success('密码重置成功，请使用新密码登录');
    router.replace('/auth/login');
  } catch (e: any) {
    message.error(e?.msg || e?.message || '密码重置失败');
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="forget-box">
    <template v-if="step === 1">
      <p class="tip">输入账号绑定的邮箱，我们将发送验证码</p>
      <input v-model="email" class="ipt" placeholder="example@example.com" type="email" />
      <button :disabled="loading" class="btn primary" @click="sendCode">发送验证码</button>
      <button class="btn" @click="router.replace('/auth/login')">返回登录</button>
    </template>
    <template v-else>
      <p class="tip">验证码已发送至 {{ emailMasked }}，5 分钟内有效</p>
      <input v-model="code" class="ipt" placeholder="邮箱验证码" />
      <input v-model="password" class="ipt" placeholder="新密码（8-64位，含字母和数字）" type="password" />
      <input v-model="confirmPassword" class="ipt" placeholder="确认新密码" type="password" />
      <button :disabled="loading" class="btn primary" @click="resetPassword">重置密码</button>
      <button class="btn" @click="step = 1">上一步</button>
    </template>
  </div>
</template>

<style scoped>
.btn {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  height: 40px;
  width: 100%;
}
.btn.primary {
  background: hsl(var(--primary));
  border: none;
  color: #fff;
}
.btn:disabled {
  opacity: 0.6;
}
.forget-box {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 320px;
}
.ipt {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  height: 40px;
  outline: none;
  padding: 0 10px;
}
.tip {
  color: hsl(var(--foreground) / 60%);
  font-size: 13px;
}
</style>
