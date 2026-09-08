<script lang="ts" setup>
/**
 * 批2: 个人中心——安全设置 Tab
 * 卡片1：MFA 两步验证（TOTP / 邮箱验证码 绑定与解绑）
 * 卡片2：登录设备/会话管理（查看自己的在线会话并精确下线）
 * 卡片3：API 个人访问令牌（创建仅明文展示一次）
 */
import { onMounted, ref } from 'vue';

import { message } from 'ant-design-vue';

import {
  getUserInfoApi,
  mfaDisableApi,
  mfaSetupApi,
  mfaSetupConfirmApi,
  mySessionsApi,
  patCreateApi,
  patListApi,
  patRevokeApi,
  revokeMySessionApi,
} from '#/api';

const mfaType = ref(0); // 0未启用 1TOTP 2邮箱
const mfaBinding = ref(false);
const setupType = ref(1);
const setupSecret = ref('');
const setupUrl = ref('');
const setupEmail = ref('');
const setupCode = ref('');

const sessions = ref<any[]>([]);
const pats = ref<any[]>([]);
const patName = ref('');
const patDays = ref<number | null>(null);
const patNewToken = ref('');

async function loadAll() {
  const [info, sess, tokens] = await Promise.all([
    getUserInfoApi().catch(() => null),
    mySessionsApi().catch(() => []),
    patListApi().catch(() => []),
  ]);
  mfaType.value = Number((info as any)?.mfaType ?? 0);
  sessions.value = sess as any[];
  pats.value = tokens as any[];
}

onMounted(loadAll);

// ---- MFA ----
async function startSetup(type: number) {
  try {
    const res: any = await mfaSetupApi({ mfaType: type });
    mfaBinding.value = true;
    setupType.value = type;
    setupSecret.value = res?.secret ?? '';
    setupUrl.value = res?.otpauthUrl ?? '';
    setupEmail.value = res?.emailMasked ?? '';
  } catch (e: any) {
    message.error(e?.msg || e?.message || '发起绑定失败（需已绑定邮箱且 SMTP 已配置）');
  }
}

async function confirmSetup() {
  try {
    await mfaSetupConfirmApi({ mfaType: setupType.value, code: setupCode.value.trim() });
    message.success('MFA 绑定成功，下次登录需二次验证');
    mfaBinding.value = false;
    setupCode.value = '';
    await loadAll();
  } catch (e: any) {
    message.error(e?.msg || e?.message || '绑定失败');
  }
}

async function disableMfa() {
  if (!setupCode.value.trim()) {
    message.error('请先输入当前动态验证码');
    return;
  }
  try {
    await mfaDisableApi({ code: setupCode.value.trim() });
    message.success('MFA 已解绑');
    setupCode.value = '';
    await loadAll();
  } catch (e: any) {
    message.error(e?.msg || e?.message || '解绑失败');
  }
}

// ---- 会话 ----
async function revokeSession(token: string) {
  await revokeMySessionApi({ token });
  message.success('会话已下线');
  await loadAll();
}

// ---- PAT ----
async function createPat() {
  if (!patName.value.trim()) {
    message.error('请输入令牌名称');
    return;
  }
  try {
    const res: any = await patCreateApi({ name: patName.value.trim(), expireDays: patDays.value ?? undefined });
    patNewToken.value = res.token;
    patName.value = '';
    patDays.value = null;
    await loadAll();
  } catch (e: any) {
    message.error(e?.msg || e?.message || '创建失败');
  }
}

async function revokePat(id: number) {
  await patRevokeApi({ id });
  message.success('令牌已停用');
  await loadAll();
}

function copyToken() {
  navigator.clipboard?.writeText(patNewToken.value);
  message.success('已复制，请妥善保存（仅展示一次）');
}
</script>

<template>
  <div class="security-page">
    <div class="card">
      <h3>两步验证（MFA）</h3>
      <p v-if="mfaType === 0" class="muted">当前未启用。启用后登录需输入动态验证码，大幅提升账号安全性。</p>
      <p v-else class="ok">已启用（{{ mfaType === 1 ? 'TOTP 动态口令' : '邮箱验证码' }}）</p>

      <template v-if="!mfaBinding">
        <div class="row">
          <button class="btn" @click="startSetup(1)">{{ mfaType === 1 ? '重新绑定 TOTP' : '使用认证器 App（TOTP）' }}</button>
          <button class="btn" @click="startSetup(2)">{{ mfaType === 2 ? '重新绑定邮箱验证' : '使用邮箱验证码' }}</button>
        </div>
        <div v-if="mfaType !== 0" class="row">
          <input v-model="setupCode" class="ipt" placeholder="输入当前动态验证码以解绑" />
          <button class="btn danger" @click="disableMfa">解绑 MFA</button>
        </div>
      </template>
      <template v-else>
        <template v-if="setupType === 1">
          <p class="muted">在认证器 App（Google Authenticator 等）中"手动输入密钥"添加：</p>
          <code class="secret">{{ setupSecret }}</code>
          <p class="muted">或复制链接：<code class="secret">{{ setupUrl }}</code></p>
        </template>
        <p v-else class="muted">验证码已发送至 {{ setupEmail }}</p>
        <div class="row">
          <input v-model="setupCode" class="ipt" placeholder="输入 6 位动态验证码" />
          <button class="btn primary" @click="confirmSetup">确认绑定</button>
        </div>
      </template>
    </div>

    <div class="card">
      <h3>登录设备与会话</h3>
      <table class="tbl">
        <thead>
          <tr><th>会话</th><th>登录 IP</th><th>登录时间</th><th>到期时间</th><th>操作</th></tr>
        </thead>
        <tbody>
          <tr v-for="s in sessions" :key="s.token">
            <td>{{ s.tokenMasked }} <a-tag v-if="s.isCurrent" color="green">当前</a-tag></td>
            <td>{{ s.loginIp || '-' }}</td>
            <td>{{ s.loginTime || '-' }}</td>
            <td>{{ s.expireTime || '-' }}</td>
            <td>
              <button v-if="!s.isCurrent" class="btn danger" @click="revokeSession(s.token)">下线</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="card">
      <h3>API 个人访问令牌（PAT）</h3>
      <p class="muted">用于第三方程序以 `Authorization: Bearer mxxpat_xxx` 调用 API；每个令牌每小时限 1000 次请求。</p>
      <div class="row">
        <input v-model="patName" class="ipt" placeholder="令牌名称" />
        <input v-model.number="patDays" class="ipt narrow" placeholder="有效天数（空=永久）" type="number" />
        <button class="btn primary" @click="createPat">创建令牌</button>
      </div>
      <div v-if="patNewToken" class="token-new">
        <code>{{ patNewToken }}</code>
        <button class="btn" @click="copyToken">复制</button>
        <p class="muted">⚠️ 明文仅显示这一次，请立即保存。</p>
      </div>
      <table class="tbl">
        <thead>
          <tr><th>名称</th><th>前缀</th><th>过期时间</th><th>最近使用</th><th>状态</th><th>操作</th></tr>
        </thead>
        <tbody>
          <tr v-for="t in pats" :key="t.id">
            <td>{{ t.name }}</td>
            <td><code>{{ t.tokenPrefix }}...</code></td>
            <td>{{ t.expireTime || '永久' }}</td>
            <td>{{ t.lastUsedAt || '-' }}</td>
            <td>{{ t.status === 1 ? '启用' : '停用' }}</td>
            <td>
              <button v-if="t.status === 1" class="btn danger" @click="revokePat(t.id)">停用</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.btn {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  height: 34px;
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
.card {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  margin-bottom: 16px;
  padding: 16px;
}
.ipt {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  height: 34px;
  min-width: 200px;
  outline: none;
  padding: 0 10px;
}
.ipt.narrow {
  min-width: 0;
  width: 180px;
}
.muted {
  color: hsl(var(--foreground) / 55%);
  font-size: 13px;
}
.ok {
  color: #1a9;
  font-size: 13px;
}
.row {
  align-items: center;
  display: flex;
  gap: 8px;
  margin: 8px 0;
  flex-wrap: wrap;
}
.secret {
  background: hsl(var(--border) / 30%);
  border-radius: 4px;
  display: inline-block;
  padding: 2px 8px;
  word-break: break-all;
}
.security-page {
  padding: 4px;
}
.tbl {
  border-collapse: collapse;
  font-size: 13px;
  margin-top: 8px;
  width: 100%;
}
.tbl th,
.tbl td {
  border-bottom: 1px solid hsl(var(--border) / 60%);
  padding: 6px 8px;
  text-align: left;
}
.token-new {
  background: hsl(var(--border) / 25%);
  border-radius: 6px;
  margin: 8px 0;
  padding: 10px;
  word-break: break-all;
}
</style>
