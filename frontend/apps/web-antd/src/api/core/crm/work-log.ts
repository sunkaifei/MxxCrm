import { requestClient } from '#/api/request';

// ============ 工作日志 API ============

/** 今日工作日志列表 */
export const getTodayWorkLogApi = async () =>
  requestClient.get('/api/system/work-log/today');

/** 本周工作负载统计 */
export const getWeekWorkloadApi = async () =>
  requestClient.get('/api/system/work-log/week-workload');

/** 今日待办汇总（已处理数 + 剩余数 + 完成率，来自后端聚合接口） */
export const getTodaySummaryApi = async () =>
  requestClient.get('/api/system/work-log/today-summary');
