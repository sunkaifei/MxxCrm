import { baseRequestClient, requestClient } from '#/api/request';

export namespace AuthApi {
  /** 登录接口参数 */
  export interface LoginParams {
    captchaCode?: string;
    captchaKey?: string;
    /** 点选验证码方案: click 模式下的一次性登录凭证 */
    captchaTicket?: string;
    password?: string;
    /** A-2.9: 记住我——refreshToken 走 30 天档 */
    rememberMe?: boolean;
    username?: string;
  }

  /** 登录接口返回值 */
  export interface LoginResult {
    /** 批2 MFA: 需二次验证时为 true，其余凭据字段为空 */
    mfaRequired?: boolean;
    mfaTicket?: string;
    mfaType?: number;
    mfaEmail?: string;
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
    deptName?: string;
    postName?: string;
    expectedSalary?: string;
    /** 验证码设置: 图形模式凭据（registerCaptchaType=image） */
    captchaCode?: string;
    captchaKey?: string;
    /** 验证码设置: 点选模式一次性 ticket（registerCaptchaType=click） */
    captchaTicket?: string;
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
/** 点选方案: 验证码形态配置（enabled / mode=image|click） */
export async function getCaptchaConfigApi() {
  return requestClient.get<{ enabled: boolean; mode: 'click' | 'image' }>('/api/open/captcha/config');
}

/** 点选方案: 生成文字点选图（背景 + 3 个汉字，坐标服务端留存） */
export async function getClickCaptchaApi() {
  return requestClient.get<{ captchaKey: string; hintChars: string[]; imageBase64: string; imageHeight: number; imageWidth: number; ttl: number; }>(
    '/api/open/captcha/click/get',
  );
}

/** 点选方案: 校验点选坐标，通过签发一次性登录 ticket */
export async function verifyClickCaptchaApi(data: {
  captchaKey: string;
  clicks: Array<{ x: number; y: number }>;
  displayHeight?: number;
  displayWidth?: number;
}) {
  // silentError: 失败消息由点选组件内嵌提示条展示，避免全局拦截器重复弹窗
  return requestClient.post<{ captchaTicket: string; expiresIn: number }>(
    '/api/open/captcha/click/verify',
    data,
    { silentError: true },
  );
}

/** A-0.2: 获取登录图形验证码（base64 图片 + 一次性 key），后端公开路由 */
export async function getCaptchaApi() {
  return requestClient.get<{ captchaBase64: string; captchaKey: string }>(
    '/api/open/captcha/get',
  );
}

/** 批2: MFA 登录二次验证 */
export async function mfaVerifyApi(data: { code: string; ticket: string; }) {
  return requestClient.post<AuthApi.LoginResult>('/api/system/auth/mfa/verify', data);
}

/** 批2: 发起 MFA 绑定（type=1 TOTP 返回密钥/二维码链接；type=2 发送邮箱验证码） */
export async function mfaSetupApi(data: { mfaType: number }) {
  return requestClient.post<{ emailMasked?: string; mfaType: number; otpauthUrl?: string; secret?: string; }>(
    '/api/system/auth/mfa/setup',
    data,
  );
}

/** 批2: 确认 MFA 绑定 */
export async function mfaSetupConfirmApi(data: { code: string; mfaType: number; }) {
  return requestClient.post('/api/system/auth/mfa/setup/confirm', data);
}

/** 批2: 解绑 MFA */
export async function mfaDisableApi(data: { code: string }) {
  return requestClient.post('/api/system/auth/mfa/disable', data);
}

/** 批2: 找回密码（发送邮箱验证码，返回票据与脱敏邮箱） */
export async function forgotPasswordApi(data: { email: string }) {
  return baseRequestClient.post<{ emailMasked: string; ticket: string; }>('/api/system/auth/forgot_password', data);
}

/** 批2: 重置密码 */
export async function resetPasswordApi(data: { code: string; password: string; ticket: string; }) {
  return baseRequestClient.post('/api/system/auth/reset_password', data);
}

/** 批2: SSO 状态（登录页判断是否显示企业登录按钮） */
export interface SsoProviderStatus {
  code: string;
  name: string;
  configured: boolean;
  enabled: boolean;
}

export interface SsoStatusResult {
  enabled: boolean;
  providers?: SsoProviderStatus[];
}

export async function ssoStatusApi() {
  return baseRequestClient.get<SsoStatusResult>('/api/system/auth/sso/status');
}

/** 批2: 个人中心——账号绑定：查询当前用户已绑定的第三方账号 */
export interface SsoBindItem {
  provider: string;
  provider_uid: string;
  created_at?: string;
}

export async function ssoBindListApi() {
  return requestClient.get<SsoBindItem[]>('/api/system/auth/sso/bind/list');
}

/** 批2: 个人中心——账号绑定：解绑指定 provider（软删） */
export async function ssoBindUnbindApi(provider: string) {
  return requestClient.post('/api/system/auth/sso/bind/unbind', { provider });
}

/** 批2: 发起第三方扫码登录（公开接口，直接跳转） */
export function ssoAuthorizeUrl(provider: string) {
  return `/api/system/auth/sso/authorize?provider=${provider}`;
}

/** 批2: 个人中心——发起第三方扫码绑定（登录态 XHR，返回授权 URL 由前端跳转） */
export async function ssoBindStartApi(provider: string) {
  return requestClient.get<{ url: string }>(
    '/api/system/auth/sso/bind/start',
    { params: { provider } },
  );
}

/** 批2: 我的在线会话 */
export async function mySessionsApi() {
  return requestClient.get<
    Array<{ expireTime?: string; isCurrent: boolean; loginIp?: string; loginTime?: string; token: string; tokenMasked: string; }>
  >('/api/system/auth/my-sessions');
}

/** 批2: 下线自己的指定会话 */
export async function revokeMySessionApi(data: { token: string }) {
  return requestClient.post('/api/system/auth/my-session/revoke', data);
}

/** 批2: API 令牌管理 */
export async function patListApi() {
  return requestClient.get<Array<{ createTime?: string; expireTime?: string; id: number; lastUsedAt?: string; name?: string; status?: number; tokenPrefix?: string; }>>(
    '/api/system/auth/api-token/list',
  );
}

export async function patCreateApi(data: { expireDays?: number; name: string; }) {
  return requestClient.post<{ id: number; token: string }>('/api/system/auth/api-token/create', data);
}

export async function patRevokeApi(data: { id: number }) {
  return requestClient.post('/api/system/auth/api-token/revoke', data);
}

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
 * 查询注册开关状态 + 用户名注册规则（免鉴权）
 */
export async function getRegisterStatusApi() {
  return baseRequestClient.get<{
    registerEnabled: boolean;
    usernameBannedKeywords: string;
    usernameCharset: string;
    usernameLetterStart: boolean;
    usernameMaxLength: number;
    usernameMinLength: number;
    usernameNotPureNumber: boolean;
  }>('/api/system/auth/register-status');
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
