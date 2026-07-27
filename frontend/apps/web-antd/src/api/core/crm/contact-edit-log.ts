import { requestClient } from '#/api/request';

/**
 * 联系人修改日志查询参数
 */
export interface ContactEditLogQuery {
  contactId?: number;
  page?: number;
  pageSize?: number;
}

/**
 * 单个字段变更记录
 */
export interface EditLogItem {
  field: string;
  fieldLabel: string;
  old?: string;
  new?: string;
}

/**
 * 联系人修改日志 VO
 */
export interface ContactEditLogVO {
  id?: number;
  contactId?: number;
  editorId?: number;
  editorName?: string;
  content?: EditLogItem[];
  editTime?: string;
}

/**
 * 分页结果
 */
export interface ResultPage<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

/**
 * 查询联系人修改日志（分页）
 */
export async function getContactEditLogApi(params: ContactEditLogQuery) {
  return requestClient.get<ResultPage<ContactEditLogVO>>(
    '/api/system/contact/edit-log',
    { params },
  );
}
