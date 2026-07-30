import { requestClient } from '#/api/request';

// ============ 外勤拜访 API ============

/** 外勤拜访列表（筛选 activity_type=2 的跟进记录） */
export const getVisitListApi = async (params?: any) =>
  requestClient.get('/api/system/visit/list', { params });

/** 拜访详情 */
export const getVisitInfoApi = async (id: number) =>
  requestClient.get(`/api/system/visit/info/${id}`);

/** 签到（创建一条 activity_type=2 的跟进记录） */
export const checkInApi = async (data: any) =>
  requestClient.post('/api/system/visit/check-in', data);

/** 签退（更新 check_out_time） */
export const checkOutApi = async (id: number) =>
  requestClient.post(`/api/system/visit/check-out/${id}`);

/** 拜访统计（总次数/今日/本周/本月/拜访客户数） */
export const getVisitStatisticsApi = async () =>
  requestClient.get('/api/system/visit/statistics');
