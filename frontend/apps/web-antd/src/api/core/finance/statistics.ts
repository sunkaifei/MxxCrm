import { requestClient } from '#/api/request';

// ===== 财务统计 =====
export const getFinanceStatisticsSummaryApi = async () =>
  requestClient.get('/api/system/finance/statistics/summary');

export const getFinanceStatisticsListApi = async (params?: {
  statType?: number;
  startDate?: string;
  endDate?: string;
}) => requestClient.get('/api/system/finance/statistics/list', { params });

export const generateDailyStatisticsApi = async () =>
  requestClient.post('/api/system/finance/statistics/generate-daily');
