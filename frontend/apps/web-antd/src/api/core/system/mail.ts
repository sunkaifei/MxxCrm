import { requestClient } from '#/api/request';

// 邮箱配置
export const getMailConfigListApi = async (params?: any) =>
  requestClient.get('/api/system/mail/config/list', { params });
export const getMailConfigInfoApi = async (id: number) =>
  requestClient.get('/api/system/mail/config/info', { params: { id } });
export const createMailConfigApi = async (data: any) =>
  requestClient.post('/api/system/mail/config/save', data);
export const updateMailConfigApi = async (data: any) =>
  requestClient.put('/api/system/mail/config/update', data);
export const deleteMailConfigApi = async (ids: number[]) =>
  requestClient.delete('/api/system/mail/config/bath_delete', { data: { ids } });
export const setDefaultMailConfigApi = async (id: number) =>
  requestClient.put('/api/system/mail/config/set_default', { id });

// 邮件模板
export const getMailTemplateListApi = async (params?: any) =>
  requestClient.get('/api/system/mail/template/list', { params });
export const getMailTemplateInfoApi = async (id: number) =>
  requestClient.get('/api/system/mail/template/info', { params: { id } });
export const getMailTemplateOptionsApi = async () =>
  requestClient.get('/api/system/mail/template/options');
export const createMailTemplateApi = async (data: any) =>
  requestClient.post('/api/system/mail/template/save', data);
export const updateMailTemplateApi = async (data: any) =>
  requestClient.put('/api/system/mail/template/update', data);
export const deleteMailTemplateApi = async (ids: number[]) =>
  requestClient.delete('/api/system/mail/template/bath_delete', { data: { ids } });

// 发送邮件
export const sendMailApi = async (data: any) =>
  requestClient.post('/api/system/mail/send', data);

// 邮件日志
export const getMailLogListApi = async (params?: any) =>
  requestClient.get('/api/system/mail/log/list', { params });
export const getCustomerMailLogApi = async (customerId: number) =>
  requestClient.get('/api/system/mail/log/by_customer', { params: { customerId } });
