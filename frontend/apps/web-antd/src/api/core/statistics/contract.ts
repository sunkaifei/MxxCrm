import { requestClient } from '#/api/request';

/** 统计时间筛选参数：start_date/end_date 优先，兼容 year/month */
export interface StatsTimeParams {
  start_date?: string;
  end_date?: string;
  year?: number;
  month?: number;
}

export const getContractRankingApi = async (
  params?: StatsTimeParams & {
    limit?: number;
    order_by?: string;
    order_type?: string;
  },
) => {
  return requestClient.get('/api/system/statistics/contract/ranking', {
    params,
  });
};

export const getContractTypeDistributionApi = async (
  params?: StatsTimeParams,
) => {
  return requestClient.get(
    '/api/system/statistics/contract/type-distribution',
    { params },
  );
};

export const getContractStatusAnalysisApi = async (
  params?: StatsTimeParams,
) => {
  return requestClient.get('/api/system/statistics/contract/status-analysis', {
    params,
  });
};
