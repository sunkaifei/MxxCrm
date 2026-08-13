import { requestClient } from '#/api/request';

/** 审计事件查询参数 */
export interface AuditEventParams {
  user_id?: number;
  module?: string;
  action?: string;
  start_date?: string;
  end_date?: string;
  keyword?: string;
  page?: number;
  page_size?: number;
}

/** 审计事件列表 */
export const getAuditListApi = async (params?: AuditEventParams) => {
  return requestClient.get('/api/system/audit/list', { params });
};

/** 汇总批次查询参数 */
export interface AggBatchParams {
  topic?: string;
  page?: number;
  page_size?: number;
}

/** 汇总批次列表 */
export const getAggBatchesApi = async (params?: AggBatchParams) => {
  return requestClient.get('/api/system/statistics/agg/batches', { params });
};

/** 手动重算汇总 */
export const refreshAggApi = async (data: {
  end_date: string;
  start_date: string;
  topic: string;
}) => {
  return requestClient.post('/api/system/statistics/agg/refresh', data);
};
