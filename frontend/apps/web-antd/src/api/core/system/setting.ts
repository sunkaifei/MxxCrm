import { requestClient } from '#/api/request';

/** 系统设置返回结构 */
export interface SettingConfigVO {
  multiDevice: boolean;
  sessionTimeout: number;
  maxDevices: number;
  registerEnabled: boolean;
  inboundAuditEnabled: boolean;
  outboundAuditEnabled: boolean;
  inboundAuditMode: number;
  outboundAuditMode: number;
  /** 验证码设置: 登录页形态（image/click/none） */
  loginCaptchaType: 'click' | 'image' | 'none';
  /** 验证码设置: 注册页形态（image/click/none） */
  registerCaptchaType: 'click' | 'image' | 'none';
  /** 仓库数据互看开关 */
  warehouseCrossViewEnabled: boolean;
  /** 注册策略: 用户名最少字符 */
  usernameMinLength: number;
  /** 注册策略: 用户名最多字符 */
  usernameMaxLength: number;
  /** 注册策略: 用户名允许字符集（alnum_underscore/alnum/alpha/cjk_alnum_underscore） */
  usernameCharset: string;
  /** 注册策略: 用户名必须以字母开头 */
  usernameLetterStart: boolean;
  /** 注册策略: 禁止纯数字用户名 */
  usernameNotPureNumber: boolean;
  /** 注册策略: 禁止的保留/敏感关键字（英文逗号分隔） */
  usernameBannedKeywords: string;
}

/** 在线会话返回结构 */
export interface SessionVO {
  userId: number;
  userName: string;
  token: string;
  tokenExpire: number;
  current: boolean;
}

/** 读取系统设置 */
export const getSettingConfigApi = async () => {
  return requestClient.get<SettingConfigVO>('/api/system/setting/config');
};

/** 保存系统设置 */
export const updateSettingConfigApi = async (data: {
  inboundAuditEnabled?: boolean;
  warehouseCrossViewEnabled?: boolean;
  loginCaptchaType?: 'click' | 'image' | 'none';
  registerCaptchaType?: 'click' | 'image' | 'none';
  inboundAuditMode?: number;
  maxDevices?: number;
  multiDevice?: boolean;
  outboundAuditEnabled?: boolean;
  outboundAuditMode?: number;
  registerEnabled?: boolean;
  sessionTimeout?: number;
  usernameMinLength?: number;
  usernameMaxLength?: number;
  usernameCharset?: string;
  usernameLetterStart?: boolean;
  usernameNotPureNumber?: boolean;
  usernameBannedKeywords?: string;
}) => {
  return requestClient.put('/api/system/setting/config', data);
};

/** 在线会话列表 */
export const getOnlineSessionListApi = async () => {
  return requestClient.get<SessionVO[]>('/api/system/session/list');
};

/** 按会话踢下线 */
export const kickSessionApi = async (userId: number, token: string) => {
  return requestClient.post(`/api/system/session/kick/${userId}/${token}`);
};
