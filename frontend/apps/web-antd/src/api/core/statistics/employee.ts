import type { StatsTimeParams } from './contract';

import { requestClient } from '#/api/request';

// 口径开关：true=仅统计「当年已通过年度销售计划」的员工（有计划 ⇒ 销售身份）
type EmployeeStatsParams = StatsTimeParams & {
  department_id?: number;
  only_sales?: boolean;
};

export const getEmployeeCustomerCountApi = async (
  params?: EmployeeStatsParams,
) => {
  return requestClient.get('/api/system/statistics/employee/customer-count', {
    params,
  });
};

export const getEmployeeFollowUpApi = async (
  params?: EmployeeStatsParams,
) => {
  return requestClient.get('/api/system/statistics/employee/follow-up', {
    params,
  });
};

export const getEmployeeConversionApi = async (
  params?: EmployeeStatsParams,
) => {
  return requestClient.get('/api/system/statistics/employee/conversion', {
    params,
  });
};
