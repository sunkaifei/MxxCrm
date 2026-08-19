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

// 删除备份（文件+记录）
export const deleteBackupApi = async (id: number) => {
  return requestClient.delete('/api/system/backup/delete', {
    params: { id },
  });
};

// 数据恢复（危险操作，confirm 必须为 RESTORE）
export const restoreBackupApi = async (id: number, confirm: string) => {
  return requestClient.post('/api/system/backup/restore', { id, confirm });
};

// 下载备份文件（返回 blob，大文件下载放宽超时）
export const downloadBackupApi = (id: number) => {
  return requestClient.get(`/api/system/backup/download/${id}`, {
    timeout: 600_000,
    responseType: 'blob',
    responseReturn: 'body',
  });
};
