<script lang="ts" setup>
/**
 * 批2: SSO 回调页
 * - 登录场景：解析 IdP 回跳 fragment 中的双 Token 完成登录（#accessToken=..）
 * - 绑定场景：解析 query 参数（?bind_result=success / ?error=..）并返回个人中心
 */
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { useAccessStore, useUserStore } from '@vben/stores';

import { message } from 'ant-design-vue';

import { getAccessCodesApi, getUserInfoApi } from '#/api';

defineOptions({ name: 'SsoCallback' });

const router = useRouter();
const accessStore = useAccessStore();
const userStore = useUserStore();
const error = ref('');

onMounted(async () => {
  // 绑定场景：个人中心发起，回跳携带 query 参数
  const query = new URLSearchParams(window.location.search);
  const bindResult = query.get('bind_result');
  const queryError = query.get('error');

  if (bindResult === 'success') {
    message.success('第三方账号绑定成功');
    router.replace({ path: '/profile', query: { tab: 'account' } });
    return;
  }
  if (queryError) {
    error.value = decodeURIComponent(queryError);
    return;
  }

  // 登录场景：IdP 回跳携带 fragment 参数
  const hash = window.location.hash.replace(/^#/, '');
  const params = new URLSearchParams(hash);
  const hashError = params.get('error');
  const accessToken = params.get('accessToken');
  const refreshToken = params.get('refreshToken');

  if (hashError || !accessToken) {
    error.value = hashError ? decodeURIComponent(hashError) : 'SSO 登录失败：缺少凭据';
    return;
  }
  try {
    accessStore.setAccessToken(accessToken);
    if (refreshToken) accessStore.setRefreshToken(refreshToken);
    const userInfo = await getUserInfoApi();
    userStore.setUserInfo(userInfo);
    const codes = await getAccessCodesApi();
    accessStore.setAccessCodes(codes ?? []);
    await router.replace(userInfo.homePath || '/');
    message.success('SSO 登录成功');
  } catch (error_) {
    error.value = `SSO 登录失败: ${error_ instanceof Error ? error_.message : String(error_)}`;
    accessStore.setAccessToken(null);
    accessStore.setRefreshToken(null);
  }
});

function backToLogin() {
  router.replace('/auth/login');
}

function backToProfile() {
  router.replace({ path: '/profile', query: { tab: 'account' } });
}
</script>

<template>
  <div class="sso-callback">
    <template v-if="error">
      <p class="err">SSO 处理失败：{{ error }}</p>
      <div class="btns">
        <button class="btn" @click="backToLogin">返回登录页</button>
        <button class="btn" @click="backToProfile">返回个人中心</button>
      </div>
    </template>
    <p v-else>SSO 登录中，请稍候…</p>
  </div>
</template>

<style scoped>
.btn {
  border: 1px solid #ddd;
  border-radius: 6px;
  cursor: pointer;
  margin-top: 12px;
  padding: 8px 20px;
}
.btns {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}
.btns .btn {
  margin-top: 0;
}
.err {
  color: #d33;
}
.sso-callback {
  align-items: center;
  display: flex;
  flex-direction: column;
  height: 100vh;
  justify-content: center;
}
</style>
