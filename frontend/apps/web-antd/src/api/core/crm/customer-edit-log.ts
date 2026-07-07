import { requestClient } from '#/api/request';

/**
 * 客户修改日志查询参数
 */
export interface CustomerEditLogQuery {
  customerId?: number;
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
 * 客户修改日志 VO
 */
export interface CustomerEditLogVO {
  id?: number;
  customerId?: number;
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
 * 查询客户修改日志（分页）
 */
export async function getCustomerEditLogApi(params: CustomerEditLogQuery) {
  return requestClient.get<ResultPage<CustomerEditLogVO>>(
    '/api/system/customer/edit-log',
    { params },
  );
}