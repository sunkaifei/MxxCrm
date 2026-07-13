import { baseRequestClient, requestClient } from '#/api/request';

export namespace AuthApi {
  /** 登录接口参数 */
  export interface LoginParams {
    password?: string;
    username?: string;
  }

  /** 登录接口返回值 */
  export interface LoginResult {
    accessToken: string;
  }

  export interface RefreshTokenResult {
    data: string;
    status: number;
  }

  /** 注册接口参数 */
  export interface RegisterParams {
    username?: string;
    password?: string;
    confirmPassword?: string;
    email?: string;
    mobile?: string;
    agreePolicy?: boolean;
  }

  /** 用户名检查返回值 */
  export interface CheckUsernameResult {
    exists: boolean;
    message: string;
  }
}

/**
 * 登录
 */
export async function loginApi(data: AuthApi.LoginParams) {
  return requestClient.post<AuthApi.LoginResult>(
    '/api/system/auth/login',
    data,
  );
}

/**
 * 刷新accessToken
 */
export async function refreshTokenApi() {
  return baseRequestClient.post<AuthApi.RefreshTokenResult>(
    '/api/system/auth/refresh',
    {
      withCredentials: true,
    },
  );
}

/**
 * 退出登录
 */
export async function logoutApi() {
  return baseRequestClient.delete('/api/auth/logout', {
    withCredentials: true,
  });
}

/**
 * 获取用户权限码
 */
export async function getAccessCodesApi() {
  return requestClient.get<string[]>('/api/system/auth/codes');
}

/**
 * 注册
 */
export async function registerApi(data: AuthApi.RegisterParams) {
  return baseRequestClient.post('/api/system/auth/register', data);
}

/**
 * 检查用户名是否已存在
 */
export async function checkUsernameApi(username: string) {
  return baseRequestClient.get<AuthApi.CheckUsernameResult>('/api/system/auth/check-username', {
    params: { username },
  });
}
