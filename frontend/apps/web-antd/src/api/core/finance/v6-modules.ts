import { requestClient } from '#/api/request';

// ===== 个税管理 =====
export const getTaxRateListApi = async (params?: { taxType?: number }) =>
  requestClient.get('/api/system/finance/tax/rate/list', { params });
export const upsertTaxRateApi = async (data: any) =>
  requestClient.post('/api/system/finance/tax/rate/upsert', data);
export const deleteTaxRateApi = async (id: number) =>
  requestClient.post('/api/system/finance/tax/rate/delete', { id });
export const getEmployeeTaxConfigListApi = async (params?: {
  employeeId?: number;
  year?: number;
}) =>
  requestClient.get('/api/system/finance/tax/employee-config/list', {
    params,
  });
export const upsertEmployeeTaxConfigApi = async (data: any) =>
  requestClient.post('/api/system/finance/tax/employee-config/upsert', data);
export const getTaxDetailListApi = async (params: {
  employeeId: number;
  year: number;
}) => requestClient.get('/api/system/finance/tax/detail/list', { params });
export const calculateAnnualBonusTaxApi = async (data: {
  bonusAmount: number;
}) =>
  requestClient.post('/api/system/finance/tax/annual-bonus-calculate', data);

// ===== 社保公积金 =====
export const getInsurancePolicyListApi = async (params?: {
  cityCode?: string;
  year?: number;
}) =>
  requestClient.get('/api/system/finance/insurance/policy/list', { params });
export const upsertInsurancePolicyApi = async (data: any) =>
  requestClient.post('/api/system/finance/insurance/policy/upsert', data);
export const deleteInsurancePolicyApi = async (id: number) =>
  requestClient.post('/api/system/finance/insurance/policy/delete', { id });
export const getEmployeeInsuranceConfigListApi = async (params?: {
  employeeId?: number;
}) =>
  requestClient.get('/api/system/finance/insurance/employee-config/list', {
    params,
  });
export const upsertEmployeeInsuranceConfigApi = async (data: any) =>
  requestClient.post(
    '/api/system/finance/insurance/employee-config/upsert',
    data,
  );
export const previewInsuranceCalcApi = async (data: any) =>
  requestClient.post('/api/system/finance/insurance/preview-calc', data);

// ===== 考勤扣款 =====
export const getAttendanceListApi = async (params?: any) =>
  requestClient.get('/api/system/finance/attendance/list', { params });
export const getAttendanceDetailApi = async (params: {
  employeeId: number;
  month: number;
  year: number;
}) => requestClient.get('/api/system/finance/attendance/detail', { params });
export const upsertAttendanceApi = async (data: any) =>
  requestClient.post('/api/system/finance/attendance/upsert', data);
export const deleteAttendanceApi = async (id: number) =>
  requestClient.post('/api/system/finance/attendance/delete', { id });
export const batchImportAttendanceApi = async (data: any[]) =>
  requestClient.post('/api/system/finance/attendance/batch-import', data);
export const calculateAttendanceDeductionApi = async (params: {
  employeeId: number;
  month: number;
  year: number;
}) =>
  requestClient.get('/api/system/finance/attendance/calculate-deduction', {
    params,
  });

// ===== 调薪记录 =====
export const getSalaryAdjustmentListApi = async (params?: any) =>
  requestClient.get('/api/system/finance/salary-adjustment/list', { params });
export const getSalaryAdjustmentHistoryApi = async (employeeId: number) =>
  requestClient.get('/api/system/finance/salary-adjustment/history', {
    params: { employeeId },
  });
export const createSalaryAdjustmentApi = async (data: any) =>
  requestClient.post('/api/system/finance/salary-adjustment/create', data);
export const approveSalaryAdjustmentApi = async (id: number) =>
  requestClient.post('/api/system/finance/salary-adjustment/approve', { id });
export const rejectSalaryAdjustmentApi = async (data: {
  id: number;
  reason: string;
}) => requestClient.post('/api/system/finance/salary-adjustment/reject', data);
export const getSalaryAdjustmentComparisonApi = async (employeeId: number) =>
  requestClient.get('/api/system/finance/salary-adjustment/comparison', {
    params: { employeeId },
  });
