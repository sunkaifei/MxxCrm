import { requestClient } from '#/api/request';

// ===== 工资条下发 =====
export const getPayslipListApi = async (params?: any) =>
  requestClient.get('/api/system/finance/payslip/list', { params });
export const generatePayslipsApi = async (data: {
  month: number;
  year: number;
}) => requestClient.post('/api/system/finance/payslip/generate', data);
export const sendPayslipApi = async (data: {
  channels: string[];
  id: number;
}) => requestClient.post('/api/system/finance/payslip/send', data);
export const batchSendPayslipsApi = async (data: {
  channels: string[];
  ids: number[];
}) => requestClient.post('/api/system/finance/payslip/batch-send', data);
export const markPayslipReadApi = async (id: number) =>
  requestClient.post('/api/system/finance/payslip/mark-read', { id });
export const getPayslipStatisticsApi = async (params: {
  month: number;
  year: number;
}) => requestClient.get('/api/system/finance/payslip/statistics', { params });

// V8-4: 工资条密码与撤回
export const setPayslipPasswordApi = async (data: {
  password: string;
  payslipId: number;
}) => requestClient.post('/api/system/finance/payslip/set-password', data);

export const clearPayslipPasswordApi = async (id: number) =>
  requestClient.post('/api/system/finance/payslip/clear-password', { id });

export const verifyPayslipPasswordApi = async (data: {
  password: string;
  payslipId: number;
}) => requestClient.post('/api/system/finance/payslip/verify-password', data);

export const withdrawPayslipApi = async (data: {
  payslipId: number;
  reason: string;
}) => requestClient.post('/api/system/finance/payslip/withdraw', data);

// 工资条详情（含提成明细）
export const getPayslipDetailApi = async (id: number) =>
  requestClient.get('/api/system/finance/payslip/detail', { params: { id } });

// 员工确认工资条
export const confirmPayslipApi = async (id: number) =>
  requestClient.post('/api/system/finance/payslip/confirm', { id });

// ===== 银行代发 =====
export const getBankExportListApi = async (params?: any) =>
  requestClient.get('/api/system/finance/bank-export/list', { params });
export const generateBankFileApi = async (data: {
  bankType: string;
  month: number;
  year: number;
}) => requestClient.post('/api/system/finance/bank-export/generate', data);
export const downloadBankFileApi = (id: number) =>
  `/api/system/finance/bank-export/download?id=${id}`;

// V9: 生成 Excel 格式代发文件（xlsx，含合计行）
export const generateBankExcelFileApi = async (data: {
  bankType: string;
  month: number;
  year: number;
}) =>
  requestClient.post('/api/system/finance/bank-export/generate-excel', data, {
    responseType: 'blob',
  });

// ===== 工资项目 =====
export const getSalaryItemListApi = async () =>
  requestClient.get('/api/system/finance/salary-item/list');
export const upsertSalaryItemApi = async (data: any) =>
  requestClient.post('/api/system/finance/salary-item/upsert', data);
export const deleteSalaryItemApi = async (id: number) =>
  requestClient.post('/api/system/finance/salary-item/delete', { id });
export const getSalaryItemValuesApi = async (salaryRecordId: number) =>
  requestClient.get('/api/system/finance/salary-item/values', {
    params: { salaryRecordId },
  });
export const saveSalaryItemValuesApi = async (data: {
  salaryRecordId: number;
  values: any[];
}) => requestClient.post('/api/system/finance/salary-item/values/save', data);

// ===== 团队提成 =====
export const getTeamCommissionListApi = async (params?: any) =>
  requestClient.get('/api/system/finance/team-commission/list', { params });
export const calculateTeamCommissionApi = async (data: {
  month: number;
  year: number;
}) => requestClient.post('/api/system/finance/team-commission/calculate', data);
export const getTeamCommissionSummaryApi = async (params: {
  month: number;
  year: number;
}) =>
  requestClient.get('/api/system/finance/team-commission/summary', { params });

// ===== 团队提成 v2：待分配 + 分配 + 分配记录 =====
// 待分配列表（category=5 归集的待分配提成）
export const getPendingCommissionApi = async (params?: any) =>
  requestClient.get('/api/system/finance/team-commission/pending', { params });

// 提交分配
export const allocateCommissionApi = async (data: {
  allocateMethod: number;
  commissionResultId: number;
  members: Array<{
    amount?: number;
    employeeId: number;
    employeeName?: string;
    employeePayment?: number;
  }>;
  remark?: string;
}) => requestClient.post('/api/system/finance/team-commission/allocate', data);

// 分配记录
export const getAllocationLogApi = async (params?: any) =>
  requestClient.get('/api/system/finance/team-commission/allocation-log', {
    params,
  });
