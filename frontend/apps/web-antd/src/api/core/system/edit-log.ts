import { requestClient } from '#/api/request';

/**
 * 编辑日志查询参数
 */
export interface EditLogQuery {
  businessType?: number;
  businessId?: number;
  editorId?: number;
  keyword?: string;
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
 * 编辑日志项
 */
export interface EditLogRecord {
  id: number;
  businessType?: number;
  businessId?: number;
  businessNo?: string;
  businessTitle?: string;
  editorId?: number;
  editorName?: string;
  content?: EditLogItem[];
  editTime?: string;
}

/**
 * 分页查询编辑日志
 */
export function getEditLogListApi(params: EditLogQuery) {
  return requestClient.get('/api/system/edit-log/list', { params });
}
