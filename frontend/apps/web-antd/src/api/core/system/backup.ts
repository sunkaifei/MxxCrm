import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

// 备份记录列表（含恢复记录）
export const getBackupListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/backup/list', { params });
};

// 备份设置（任务状态/保留天数/目录）
export const getBackupConfigApi = async () => {
  return requestClient.get('/api/system/backup/config');
};

// 保存备份设置（保留天数/cron/启用）
export const updateBackupConfigApi = async (param: any) => {
  return requestClient.post('/api/system/backup/config/update', param);
};

// 立即备份（同步执行 pg_dump，远程库全量备份可能耗时数十秒，覆盖默认 10s 超时）
export const triggerBackupApi = async () => {
  return requestClient.post('/api/system/backup/trigger', {}, {
    timeout: 300_000,
  });
};

// 发送邮箱安全验证码（仅超管；action: delete/restore/download），返回脱敏邮箱
export const sendBackupOtpApi = async (action: string) => {
  return requestClient.post('/api/system/backup/otp/send', { action });
};

// 删除备份（危险操作：仅超管 + 登录密码 + 邮箱验证码；禁删最后一个成功备份）
export const deleteBackupApi = async (id: number, password: string, otp: string) => {
  return requestClient.post('/api/system/backup/delete', { id, password, otp });
};

// 数据恢复（危险操作：仅超管 + 登录密码 + 邮箱验证码；还原前自动备份当前数据）
export const restoreBackupApi = async (id: number, password: string, otp: string) => {
  return requestClient.post('/api/system/backup/restore', { id, password, otp }, {
    timeout: 600_000,
  });
};

// 下载备份文件（危险操作：仅超管 + 登录密码 + 邮箱验证码，返回 blob）
export const downloadBackupApi = (id: number, password: string, otp: string) => {
  return requestClient.post(
    '/api/system/backup/download',
    { id, password, otp },
    {
      timeout: 600_000,
      responseType: 'blob',
      responseReturn: 'body',
    },
  );
};

// 数据初始化预览（超管）：返回待清业务表清单 + 行数 + 一次性确认码（5 分钟有效）
export const cleanPreviewApi = async () => {
  return requestClient.get('/api/system/backup/clean/preview', {
    timeout: 120_000,
  });
};

// 数据初始化执行（超管 + 登录密码 + 确认码三重验证，执行前强制自动备份）
export const cleanExecuteApi = async (password: string, confirmCode: string) => {
  return requestClient.post(
    '/api/system/backup/clean/execute',
    { password, confirmCode },
    { timeout: 600_000 },
  );
};
