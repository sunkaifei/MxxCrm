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

function createRequestClient(baseURL: string, options?: RequestClientOptions) {
  const client = new RequestClient({
    ...options,
    baseURL,
    responseType: 'arraybuffer',
  });

  /**
   * 重新认证逻辑
   */
  async function doReAuthenticate() {
    console.warn('Access token or refresh token is invalid or expired. ');
    const accessStore = useAccessStore();
    const authStore = useAuthStore();
    accessStore.setAccessToken(null);
    if (
      preferences.app.loginExpiredMode === 'modal' &&
      accessStore.isAccessChecked
    ) {
      accessStore.setLoginExpired(true);
    } else {
      await authStore.logout();
    }
  }

  /**
   * 刷新token逻辑
   */
  async function doRefreshToken() {
    const accessStore = useAccessStore();
    const resp = await refreshTokenApi();
    const newToken = resp.data;
    accessStore.setAccessToken(newToken);
    return newToken;
  }

  function formatToken(token: null | string) {
    return token ? `Bearer ${token}` : null;
  }

  // 请求头处理
  client.addRequestInterceptor({
    fulfilled: async (config) => {
      const accessStore = useAccessStore();

      config.headers.Authorization = formatToken(accessStore.accessToken);
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
          msg: '权限不足，请联系管理员',
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

  // token过期的处理
  client.addResponseInterceptor(
    authenticateResponseInterceptor({
      client,
      doReAuthenticate,
      doRefreshToken,
      enableRefreshToken: preferences.app.enableRefreshToken,
      formatToken,
    }),
  );

  // 通用的错误处理,如果没有进入上面的错误处理逻辑，就会进入这里
  client.addResponseInterceptor(
    errorMessageResponseInterceptor((msg: string, error) => {
      const responseData = error?.response?.data ?? {};
      const errorMessage =
        responseData?.error ?? responseData?.message ?? responseData?.msg ?? '';
      message.error(errorMessage || msg);
    }),
  );

  return client;
}

export const requestClient = createRequestClient(apiURL, {
  responseReturn: 'data',
});

export const baseRequestClient = createRequestClient(apiURL, {
  responseReturn: 'data',
});
