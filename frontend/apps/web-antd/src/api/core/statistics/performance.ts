import { requestClient } from '#/api/request';

/** 业绩目标（个人/部门/全公司） */
export const getPerformanceTargetApi = async (params?: {
  employee_id?: number;
  month?: number;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/target', {
    params,
  });
};

/** 保存业绩目标 */
export const savePerformanceTargetApi = async (param: { targets: any[] }) => {
  return requestClient.post(
    '/api/system/statistics/performance/target/save',
    param,
  );
};

/** 月度业绩趋势 */
export const getMonthlyPerformanceApi = async (params?: {
  department_id?: number;
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/monthly', {
    params,
  });
};

/** 业绩排名（部门/销售员） */
export const getPerformanceRankingApi = async (params?: {
  department_id?: number;
  month?: number;
  order_by?: string;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/ranking', {
    params,
  });
};

/** 业绩对比（同比/环比） */
export const getPerformanceComparisonApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/comparison', {
    params,
  });
};

/** 业绩预测（含缺口/Pipeline覆盖率） */
export const getPerformanceForecastApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/forecast', {
    params,
  });
};

/** 销售漏斗（5阶段转化率+平均周期+赢单率） */
export const getSalesFunnelApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/funnel', {
    params,
  });
};

/** 客户维度拆解（新老/ABC/Top10） */
export const getCustomerBreakdownApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get(
    '/api/system/statistics/performance/customer-breakdown',
    { params },
  );
};

/** 产品维度拆解（产品排行+品类占比） */
export const getProductBreakdownApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get(
    '/api/system/statistics/performance/product-breakdown',
    { params },
  );
};

/** 行为指标（拜访/电话/跟进/转化率+趋势） */
export const getBehaviorMetricsApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/behavior', {
    params,
  });
};

/** 区域维度拆解（省份业绩分布） */
export const getRegionBreakdownApi = async (params?: {
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.get(
    '/api/system/statistics/performance/region-breakdown',
    { params },
  );
};

/** 个人成长档案（入职以来业绩曲线+累计+最佳月份） */
export const getPersonalGrowthApi = async (params?: {
  employee_id?: number;
}) => {
  return requestClient.get(
    '/api/system/statistics/performance/personal-growth',
    { params },
  );
};

/** 里程碑（当前已达+下一档+所有档位） */
export const getPerformanceMilestoneApi = async (params?: {
  employee_id?: number;
  year?: number;
}) => {
  return requestClient.get('/api/system/statistics/performance/milestone', {
    params,
  });
};

/** 业绩报表导出（Excel/PDF） */
export const exportPerformanceApi = async (params: {
  format: 'excel' | 'pdf';
  month?: number;
  time_dimension?: string;
  year?: number;
}) => {
  return requestClient.post(
    '/api/system/statistics/performance/export',
    params,
    { responseType: 'blob' },
  );
};

/** 业绩页面配置-获取 */
export const getPerformanceConfigApi = async () => {
  return requestClient.get('/api/system/performance-config/get');
};

/** 业绩页面配置-保存 */
export const savePerformanceConfigApi = async (param: any) => {
  return requestClient.post('/api/system/performance-config/save', param);
};
