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
    /** 刷新凭据（64 字节随机数 hex，128 字符），每次登录/刷新重新签发 */
    refreshToken?: string;
  }

  /** 刷新接口返回值（旋转替换：新旧 refreshToken 均有效载荷） */
  export interface RefreshTokenResult {
    accessToken: string;
    refreshToken?: string;
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
 * 刷新 accessToken（登录认证整改 v1.0：双 Token 无感续期）
 * 路径与后端注册路由完全一致：POST /api/system/auth/refresh
 * 使用 baseRequestClient（不挂认证拦截器），刷新请求自身 401 不再递归刷新
 */
export async function refreshTokenApi(refreshToken: null | string) {
  return baseRequestClient.post<AuthApi.RefreshTokenResult>(
    '/api/system/auth/refresh',
    { refreshToken },
  );
}

/**
 * 退出登录（精确登出当前会话：优先携带 refreshToken 供后端定位会话行）
 */
export async function logoutApi(refreshToken?: null | string) {
  return baseRequestClient.delete('/api/auth/logout', {
    data: { refreshToken },
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
 * 查询注册开关状态（免鉴权）
 */
export async function getRegisterStatusApi() {
  return baseRequestClient.get<{ registerEnabled: boolean }>(
    '/api/system/auth/register-status',
  );
}

/**
 * 检查用户名是否已存在
 */
export async function checkUsernameApi(username: string) {
  return baseRequestClient.get<AuthApi.CheckUsernameResult>(
    '/api/system/auth/check-username',
    {
      params: { username },
    },
  );
}
