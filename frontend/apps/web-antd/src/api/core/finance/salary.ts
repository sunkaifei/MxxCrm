import { requestClient } from '#/api/request';

export const getSalaryListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/salary/list', { params });
};

export const getSalaryDetailApi = async (id: number) => {
  return requestClient.get('/api/system/finance/salary/detail', {
    params: { id },
  });
};

export const calculateSalaryApi = async (data: {
  year: number;
  month: number;
}) => {
  return requestClient.post('/api/system/finance/salary/calculate', data);
};

export const updateSalaryApi = async (data: any) => {
  return requestClient.post('/api/system/finance/salary/update', data);
};

export const approveSalaryApi = async (id: number) => {
  return requestClient.post('/api/system/finance/salary/approve', { id });
};

export const batchApproveSalaryApi = async (ids: number[]) => {
  return requestClient.post('/api/system/finance/salary/batch-approve', {
    ids,
  });
};

export const paySalaryApi = async (id: number) => {
  return requestClient.post('/api/system/finance/salary/pay', { id });
};

export const batchPaySalaryApi = async (ids: number[]) => {
  return requestClient.post('/api/system/finance/salary/batch-pay', { ids });
};

export const getSalarySummaryApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.get('/api/system/finance/salary/summary', { params });
};

// ===== 底薪配置 =====

export const getSalaryConfigListApi = async (params?: {
  employeeId?: number;
  year?: number;
}) => {
  return requestClient.get('/api/system/finance/salary/config/list', {
    params,
  });
};

export const upsertSalaryConfigApi = async (data: {
  employeeId: number;
  year: number;
  month?: number;
  baseSalary: number;
  positionAllowance?: number;
  performanceBase?: number;
  performanceCoefficient?: number;
}) => {
  return requestClient.post(
    '/api/system/finance/salary/config/upsert',
    data,
  );
};

export const deleteSalaryConfigApi = async (id: number) => {
  return requestClient.post('/api/system/finance/salary/config/delete', {
    id,
  });
};

// ===== 核算日志 =====

export const getSalaryCalcLogListApi = async (params?: {
  year?: number;
  month?: number;
  page?: number;
  pageSize?: number;
}) => {
  return requestClient.get('/api/system/finance/salary/calc-log/list', {
    params,
  });
};

// ===== 工资确认/申诉 =====

export const confirmSalaryApi = async (data: {
  salaryRecordId: number;
  action: number; // 1=确认, 2=申请重新核算
  reason?: string;
}) => {
  return requestClient.post('/api/system/finance/salary/confirm', data);
};

export const getMyConfirmsApi = async (params?: {
  page?: number;
  pageSize?: number;
}) => {
  return requestClient.get('/api/system/finance/salary/confirm/my-list', {
    params,
  });
};

export const getPendingConfirmsApi = async (params?: {
  page?: number;
  pageSize?: number;
  employeeId?: number;
  year?: number;
  month?: number;
  status?: number;
}) => {
  return requestClient.get(
    '/api/system/finance/salary/confirm/pending-list',
    {
      params,
    },
  );
};

// ===== 导出（V8-5）=====

export const exportSalaryApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.get('/api/system/finance/salary/export-salary', {
    params,
    responseType: 'blob',
  });
};

export const exportTaxApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.get('/api/system/finance/salary/export-tax', {
    params,
    responseType: 'blob',
  });
};

// V9: 真实 xlsx 导出（含表头样式与金额格式）
export const exportSalaryXlsxApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.get('/api/system/finance/salary/export-salary-xlsx', {
    params,
    responseType: 'blob',
  });
};

export const exportTaxXlsxApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.get('/api/system/finance/salary/export-tax-xlsx', {
    params,
    responseType: 'blob',
  });
};

// ===== V8-1: 工资单审批流对接 =====

export const submitSalaryApprovalApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.post(
    '/api/system/finance/salary/submit-approval',
    null,
    { params },
  );
};

export const syncSalaryApprovalApi = async (params: {
  year: number;
  month: number;
}) => {
  return requestClient.post(
    '/api/system/finance/salary/sync-approval',
    null,
    { params },
  );
};

export const handleConfirmApi = async (data: {
  confirmId: number;
  action: number; // 1=同意重算, 2=驳回
  remark?: string;
}) => {
  return requestClient.post('/api/system/finance/salary/confirm/handle', data);
};

// ===== P2-2: 工资历史趋势分析 =====

export interface SalaryTrendQuery {
  yearStart?: number;
  yearEnd?: number;
  monthStart?: number;
  monthEnd?: number;
  departmentName?: string;
  employeeId?: number;
  employeeName?: string;
}

export interface SalaryTrendMonthlyPoint {
  year: number;
  month: number;
  period: string;
  headcount: number;
  totalBase: number;
  totalCommission: number;
  totalPerformance: number;
  totalDeduction: number;
  totalTeamCommission: number;
  totalTax: number;
  totalGross: number;
  totalNet: number;
  avgNet: number;
}

export interface SalaryTrendDeptPoint {
  departmentName: string;
  headcount: number;
  totalBase: number;
  totalCommission: number;
  totalPerformance: number;
  totalGross: number;
  totalNet: number;
  avgNet: number;
}

export interface SalaryTrendEmployeePoint {
  employeeId: number;
  employeeName: string;
  departmentName?: string;
  totalBase: number;
  totalCommission: number;
  totalPerformance: number;
  totalGross: number;
  totalNet: number;
  months: number;
  avgMonthlyNet: number;
}

export interface SalaryTrendSummary {
  totalHeadcount: number;
  totalRecords: number;
  totalMonths: number;
  totalGross: number;
  totalNet: number;
  totalBase: number;
  totalCommission: number;
  totalPerformance: number;
  totalTeamCommission: number;
  totalTax: number;
  avgMonthlyNet: number;
}

export const getSalaryTrendMonthlyApi = async (
  params: SalaryTrendQuery,
): Promise<SalaryTrendMonthlyPoint[]> => {
  const res: any = await requestClient.get(
    '/api/system/finance/salary/trend/monthly',
    { params },
  );
  return res?.data ?? res ?? [];
};

export const getSalaryTrendDepartmentApi = async (
  params: SalaryTrendQuery,
): Promise<SalaryTrendDeptPoint[]> => {
  const res: any = await requestClient.get(
    '/api/system/finance/salary/trend/department',
    { params },
  );
  return res?.data ?? res ?? [];
};

export const getSalaryTrendEmployeeApi = async (
  params: SalaryTrendQuery & { limit?: number },
): Promise<SalaryTrendEmployeePoint[]> => {
  const res: any = await requestClient.get(
    '/api/system/finance/salary/trend/employee',
    { params },
  );
  return res?.data ?? res ?? [];
};

export const getSalaryTrendSummaryApi = async (
  params: SalaryTrendQuery,
): Promise<SalaryTrendSummary> => {
  const res: any = await requestClient.get(
    '/api/system/finance/salary/trend/summary',
    { params },
  );
  return res?.data ?? res ?? {};
};
