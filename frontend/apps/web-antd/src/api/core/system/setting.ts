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
  inboundAuditMode?: number;
  maxDevices?: number;
  multiDevice?: boolean;
  outboundAuditEnabled?: boolean;
  outboundAuditMode?: number;
  registerEnabled?: boolean;
  sessionTimeout?: number;
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
