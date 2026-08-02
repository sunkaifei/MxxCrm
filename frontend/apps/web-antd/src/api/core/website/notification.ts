import { requestClient } from '#/api/request';

export interface NotificationConfigListParams {
  page?: number;
  pageSize?: number;
  websiteId?: number;
  sceneCode?: string;
  enabled?: number;
}

export interface NotificationConfigSaveDTO {
  id?: number;
  websiteId?: number;
  /** 场景编码，如 order_paid / lead_created 等 */
  sceneCode: string;
  /** 场景名称 */
  sceneName?: string;
  /** 通知渠道，多个以英文逗号分隔：email,sms */
  channels?: string;
  /** 收件人邮箱，多个以英文逗号分隔 */
  recipientEmails?: string;
  /** 邮件主题模板 */
  emailSubject?: string;
  /** 邮件正文模板 */
  emailBody?: string;
  /** 是否启用：1启用 0停用 */
  enabled?: number;
}

export interface NotificationConfigVO {
  id: number;
  websiteId?: number;
  sceneCode?: string;
  sceneName?: string;
  channels?: string;
  recipientEmails?: string;
  emailSubject?: string;
  emailBody?: string;
  enabled?: number;
  createTime?: string;
  updateTime?: string;
}

export const notificationApi = {
  list: (params: NotificationConfigListParams) =>
    requestClient.get('/api/system/website_notification/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/website_notification/detail/${id}`),

  add: (data: NotificationConfigSaveDTO) =>
    requestClient.post('/api/system/website_notification/create', data),

  update: (id: number, data: NotificationConfigSaveDTO) =>
    requestClient.put(`/api/system/website_notification/update/${id}`, data),

  toggle: (id: number, enabled: number) =>
    requestClient.put(`/api/system/website_notification/toggle/${id}`, {
      enabled,
    }),

  delete: (ids: number[]) =>
    requestClient.delete('/api/system/website_notification/batch_delete', {
      data: { ids },
    }),

  /** 获取当前（默认）站点全部通知配置 —— 单站模式专用 */
  getCurrent: () =>
    requestClient.get('/api/system/website_notification/current'),

  /** 批量保存当前站点通知配置 —— 单站模式专用，按 sceneCode 走 upsert */
  updateCurrent: (configs: NotificationConfigSaveDTO[]) =>
    requestClient.put('/api/system/website_notification/current', {
      configs,
    }),
};
