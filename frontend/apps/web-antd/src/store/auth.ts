import type { Recordable, UserInfo } from '@vben/types';

import { ref } from 'vue';
import { useRouter } from 'vue-router';

import { LOGIN_PATH } from '@vben/constants';
import { preferences } from '@vben/preferences';
import { resetAllStores, useAccessStore, useUserStore } from '@vben/stores';

import { notification } from 'ant-design-vue';
import { defineStore } from 'pinia';

/** A-2.4: 跨标签页登出同步标记 */
const FORCE_LOGOUT_KEY = 'mxx_force_logout_at';

import {
  getAccessCodesApi,
  getUserInfoApi,
  loginApi,
  logoutApi,
  registerApi,
  mfaVerifyApi,
} from '#/api';
import { $t } from '#/locales';

export const useAuthStore = defineStore('auth', () => {
  const accessStore = useAccessStore();
  const userStore = useUserStore();
  const router = useRouter();

  const loginLoading = ref(false);

  /**
   * 异步处理登录操作
   * Asynchronously handle the login process
   * @param params 登录表单数据
   */
  // 批2 MFA：登录二次验证待验证状态
  const mfaTicket = ref('');
  const mfaType = ref(0);
  const mfaEmail = ref('');

  /** 批2: MFA 二次验证通过后完成登录（与 authLogin 共用收尾逻辑） */
  async function verifyMfa(code: string) {
    if (!mfaTicket.value) return { userInfo: null };
    loginLoading.value = true;
    try {
      const res = await mfaVerifyApi({ ticket: mfaTicket.value, code });
      const userInfo = await finishLogin(res);
      mfaTicket.value = '';
      mfaType.value = 0;
      mfaEmail.value = '';
      return { userInfo };
    } finally {
      loginLoading.value = false;
    }
  }

  /** 登录凭据落库 + 用户信息/权限码 + 跳转 */
  async function finishLogin(res: {
    accessToken?: string;
    refreshToken?: string;
  }) {
    if (!res.accessToken) return null;
    accessStore.setAccessToken(res.accessToken);
    if (res.refreshToken) {
      accessStore.setRefreshToken(res.refreshToken);
    }
    const [userInfoRes, accessCodesData] = await Promise.all([fetchUserInfo(), getAccessCodesApi()]);
    const userInfo = userInfoRes;
    userStore.setUserInfo(userInfo);
    accessStore.setAccessCodes(accessCodesData ?? []);
    if (accessStore.loginExpired) {
      accessStore.setLoginExpired(false);
    } else {
      await router.push(userInfo.homePath || preferences.app.defaultHomePath);
    }
    if (userInfo?.realName) {
      notification.success({
        description: `${$t('authentication.loginSuccessDesc')}:${userInfo?.realName}`,
        duration: 3,
        message: $t('authentication.loginSuccess'),
      });
    }
    return userInfo;
  }

  async function authLogin(
    params: Recordable<any>,
    onSuccess?: () => Promise<void> | void,
  ) {
    // 异步处理用户登录操作并获取 accessToken
    let userInfo: null | UserInfo = null;
    try {
      loginLoading.value = true;
      const res = await loginApi(params);

      // 批2 MFA: 服务端要求二次验证——记录挑战，登录页展示动态码输入
      if (res?.mfaRequired && res.mfaTicket) {
        mfaTicket.value = res.mfaTicket;
        mfaType.value = res.mfaType ?? 1;
        mfaEmail.value = res.mfaEmail ?? '';
        return { userInfo: null };
      }
      const { accessToken, refreshToken } = res;

      // 如果成功获取到 accessToken
      if (accessToken) {
        userInfo = await finishLogin({ accessToken, refreshToken });
        if (accessStore.loginExpired) {
          // 过期重登场景由 LoginExpiredModal 关闭逻辑处理
        } else if (onSuccess) {
          await onSuccess?.();
        }
      }
    } finally {
      loginLoading.value = false;
    }

    return {
      userInfo,
    };
  }

  /**
   * 异步处理注册操作
   * Asynchronously handle the register process
   * @param params 注册表单数据
   */
  async function authRegister(params: Recordable<any>) {
    const userInfo: null | UserInfo = null;
    try {
      loginLoading.value = true;
      await registerApi(params);

      await authLogin({
        username: params.username,
        password: params.password,
      });
    } finally {
      loginLoading.value = false;
    }

    return {
      userInfo,
    };
  }

  async function logout(redirect: boolean = true) {
    try {
      // 携带 refreshToken 供后端精确删除当前会话（多设备互不影响）
      await logoutApi(accessStore.refreshToken);
    } catch {
      // 不做任何处理
    }
    // 清空本地双凭据（resetAllStores 将 accessToken/refreshToken 一并复位）
    accessStore.setAccessToken(null);
    accessStore.setRefreshToken(null);
    // A-2.4: 通知其他标签页同步登出（storage 事件跨页触发）
    try {
      localStorage.setItem(FORCE_LOGOUT_KEY, String(Date.now()));
    } catch {}

    resetAllStores();
    accessStore.setLoginExpired(false);

    // 回登录页带上当前路由地址
    await router.replace({
      path: LOGIN_PATH,
      query: redirect
        ? {
            redirect: encodeURIComponent(router.currentRoute.value.fullPath),
          }
        : {},
    });
  }

  async function fetchUserInfo() {
    const userInfo = await getUserInfoApi();
    userStore.setUserInfo(userInfo);
    return userInfo;
  }

  function $reset() {
    loginLoading.value = false;
  }

  return {
    $reset,
    authLogin,
    authRegister,
    fetchUserInfo,
    loginLoading,
    logout,
    verifyMfa,
    mfaTicket,
    mfaType,
    mfaEmail,
  };
});
