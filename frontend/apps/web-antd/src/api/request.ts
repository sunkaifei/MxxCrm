/**
 * 该文件可自行根据业务逻辑进行调整
 */
import type { RequestClientOptions } from '@vben/request';

import { useAppConfig } from '@vben/hooks';
import { preferences } from '@vben/preferences';
import {
  authenticateResponseInterceptor,
  defaultResponseInterceptor,
  errorMessageResponseInterceptor,
  RequestClient,
} from '@vben/request';
import { useAccessStore } from '@vben/stores';

import { decode } from '@msgpack/msgpack';
import { message } from 'ant-design-vue';

import { useAuthStore } from '#/store';

import { refreshTokenApi } from './core';

const { apiURL } = useAppConfig(import.meta.env, import.meta.env.PROD);

// 登录过期时页面刷新/轮询会并发多个 401，同一过期期间错误提示只弹一次；
// 会话恢复有效（任一请求成功返回）后自动复位，保证下次过期仍能正常提示
let authErrorShown = false;
function showAuthErrorOnce(errorMessage: string) {
  if (authErrorShown) return;
  authErrorShown = true;
  message.error(errorMessage);
}

// 重新认证单飞：并发 401 时只执行一次登出/弹出过期弹窗
let reAuthPromise: null | Promise<void> = null;

/**
 * 提取后端返回的原始错误信息
 * 用于 403 这类原本被本地文案整体覆盖的场景，便于定位真实原因
 * 优先级：msgpack/json 的 msg > 纯文本原文（actix ErrorForbidden 即纯文本）> ''
 */
function pickBackendErrorMsg(error: any): string {
  const data = error?.response?.data;
  if (data === null || data === undefined) return '';
  if (typeof data === 'string') return data.trim();
  if (!(data instanceof ArrayBuffer)) {
    const obj = data as Record<string, unknown>;
    return String(obj.msg ?? obj.message ?? obj.error ?? '').trim();
  }
  const bytes = new Uint8Array(data);
  if (bytes.length === 0) return '';
  const headers = (error?.response?.headers ?? {}) as Record<string, string>;
  const contentType = String(
    headers['content-type'] || headers['Content-Type'] || '',
  );
  const text = new TextDecoder('utf-8').decode(bytes).trim();
  if (contentType.includes('application/msgpack')) {
    try {
      const decoded = decode(bytes) as { msg?: string };
      const msg = String(decoded?.msg ?? '').trim();
      if (msg) return msg;
    } catch {
      // 忽略：回落到纯文本
    }
  }
  try {
    const obj = JSON.parse(text) as Record<string, unknown>;
    return String(obj.msg ?? obj.message ?? obj.error ?? '').trim() || text;
  } catch {
    // 非 JSON：直接使用纯文本
    return text;
  }
}

function createRequestClient(
  baseURL: string,
  options?: RequestClientOptions,
  authenticate = true,
) {
  const client = new RequestClient({
    ...options,
    baseURL,
    responseType: 'arraybuffer',
  });

  /**
   * 重新认证逻辑
   */
  async function doReAuthenticate() {
    if (reAuthPromise) return reAuthPromise;
    reAuthPromise = (async () => {
      console.warn('Access token or refresh token is invalid or expired. ');
      const accessStore = useAccessStore();
      const authStore = useAuthStore();
      accessStore.setAccessToken(null);
      // A-2.4: 同步清除 refreshToken，避免残留凭据在下次 401 时静默换新 token
      accessStore.setRefreshToken(null);
      if (
        preferences.app.loginExpiredMode === 'modal' &&
        accessStore.isAccessChecked
      ) {
        accessStore.setLoginExpired(true);
      } else {
        await authStore.logout();
      }
    })();
    try {
      return await reAuthPromise;
    } finally {
      reAuthPromise = null;
    }
  }

  /**
   * 刷新token逻辑（登录认证整改 v1.0）
   * 携带 refreshToken 调用后端旋转刷新，成功后同步更新本地双凭据
   */
  async function doRefreshToken() {
    const accessStore = useAccessStore();
    const resp = await refreshTokenApi(accessStore.refreshToken);
    const newToken = resp.accessToken;
    accessStore.setAccessToken(newToken);
    // 旋转替换：后端每次刷新签发新 refreshToken，旧的已作废，必须同步覆盖
    if (resp.refreshToken) {
      accessStore.setRefreshToken(resp.refreshToken);
    }
    return newToken;
  }

  function formatToken(token: null | string) {
    return token ? `Bearer ${token}` : null;
  }

  // 请求头处理
  client.addRequestInterceptor({
    fulfilled: async (config) => {
      const accessStore = useAccessStore();

      // 只有在有 token 时才设置 Authorization 头
      const token = formatToken(accessStore.accessToken);
      if (token) {
        config.headers.Authorization = token;
      }
      config.headers['Accept-Language'] = preferences.app.locale;
      return config;
    },
  });

  // MessagePack 二进制数据解析拦截器
  client.addResponseInterceptor({
    fulfilled: async (response) => {
      const headers = response.headers as Record<string, string>;
      const contentType = String(
        headers?.['content-type'] || headers?.['Content-Type'] || '',
      );

      if (response.data instanceof ArrayBuffer) {
        if (contentType.includes('application/msgpack')) {
          try {
            const decoded = decode(new Uint8Array(response.data)) as {
              code: number;
              data: unknown;
              meta?: unknown;
              msg: string;
            };
            response.data = decoded;
          } catch (error) {
            console.error('MessagePack decode error:', error);
            const text = new TextDecoder('utf-8').decode(
              new Uint8Array(response.data),
            );
            try {
              response.data = JSON.parse(text);
            } catch {
              response.data = { code: -1, msg: 'Decode failed', data: text };
            }
          }
        } else if (
          contentType.includes('application/json') ||
          contentType.includes('text/plain')
        ) {
          const text = new TextDecoder('utf-8').decode(
            new Uint8Array(response.data),
          );
          try {
            response.data = JSON.parse(text);
          } catch {
            response.data = text;
          }
        } else if (
          contentType.includes('application/pdf') ||
          contentType.includes('application/octet-stream') ||
          contentType.includes('image/') ||
          contentType.startsWith('application/vnd') ||
          contentType.startsWith('application/zip')
        ) {
          // 二进制文件流（PDF/图片/Office 等），保持 ArrayBuffer 原样返回，避免被误解码
          response.data = new Blob([response.data], { type: contentType });
        } else {
          try {
            const decoded = decode(new Uint8Array(response.data)) as {
              code: number;
              data: unknown;
              meta?: unknown;
              msg: string;
            };
            response.data = decoded;
          } catch (error) {
            console.error('Binary decode error:', error);
            const text = new TextDecoder('utf-8').decode(
              new Uint8Array(response.data),
            );
            try {
              response.data = JSON.parse(text);
            } catch {
              response.data = {
                code: -1,
                msg: 'Unknown content type',
                data: text,
              };
            }
          }
        }
      }
      return response;
    },
    rejected: (error) => {
      const status = error?.response?.status;
      if (status === 403) {
        error.response.data = {
          code: 403,
          // 优先展示后端原始信息（如「缺少权限: website:notification:view」），
          // 便于定位是哪个权限码缺失；后端无信息时兜底为通用文案
          msg: pickBackendErrorMsg(error) || '权限不足，请联系管理员',
        };
      } else if (status === 401) {
        error.response.data = {
          code: 401,
          msg: '登录已过期，请重新登录',
        };
      } else if (error?.response?.data instanceof ArrayBuffer) {
        const raw = new Uint8Array(error.response.data);
        try {
          const decoded = decode(raw) as {
            code: number;
            data: unknown;
            meta?: unknown;
            msg: string;
          };
          error.response.data = decoded;
        } catch {
          try {
            const text = new TextDecoder('utf-8').decode(raw);
            error.response.data = JSON.parse(text);
          } catch {
            error.response.data = {
              code: -1,
              msg: 'Error response decode failed',
            };
          }
        }
      } else if (error?.code === 'ECONNREFUSED') {
        error.response = {
          data: {
            code: -1,
            msg: '服务连接失败，请检查后端服务是否正常运行',
          },
        };
      }
      return Promise.reject(error);
    },
  });

  // 处理返回的响应数据格式
  client.addResponseInterceptor(
    defaultResponseInterceptor({
      codeField: 'code',
      dataField: 'data',
      successCode: 200,
    }),
  );

  // token过期的处理（baseRequestClient 不挂载：刷新/登录等公共请求自身 401 不递归刷新）
  if (authenticate) {
    client.addResponseInterceptor(
      authenticateResponseInterceptor({
        client,
        doReAuthenticate,
        doRefreshToken,
        enableRefreshToken: preferences.app.enableRefreshToken,
        formatToken,
      }),
    );
  }

  // 通用的错误处理,如果没有进入上面的错误处理逻辑，就会进入这里
  client.addResponseInterceptor(
    errorMessageResponseInterceptor((msg: string, error) => {
      if (error?.config?.silentError) {
        return;
      }
      const responseData = error?.response?.data ?? {};
      const errorMessage =
        responseData?.error ?? responseData?.message ?? responseData?.msg ?? '';
      // 登录过期只提示一次，避免刷新页面时并发请求各弹一个错
      if (error?.response?.status === 401) {
        showAuthErrorOnce(errorMessage || msg);
        return;
      }
      message.error(errorMessage || msg);
    }),
  );

  // 任一请求成功返回即视为会话已恢复有效，复位登录过期的"只提示一次"状态
  client.addResponseInterceptor({
    fulfilled: (value) => {
      authErrorShown = false;
      return value;
    },
  });

  return client;
}

export const requestClient = createRequestClient(apiURL, {
  responseReturn: 'data',
});

// 基础客户端：不做 401 静默刷新（用于登录/注册/刷新/登出等认证公共请求）
export const baseRequestClient = createRequestClient(
  apiURL,
  {
    responseReturn: 'data',
  },
  false,
);
