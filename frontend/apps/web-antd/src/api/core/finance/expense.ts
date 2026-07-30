import { requestClient } from '#/api/request';

// ==================== 费用申请基础 CRUD ====================

export const getExpenseListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/expense/list', { params });
};

export const getExpenseDetailApi = async (id: number) => {
  return requestClient.get(`/api/system/finance/expense/info/${id}`);
};

export const createExpenseApi = async (data: any) => {
  return requestClient.post('/api/system/finance/expense/save', data);
};

export const updateExpenseApi = async (data: any) => {
  return requestClient.put('/api/system/finance/expense/save', data);
};

export const batchDeleteExpenseApi = async (ids: number[]) => {
  return requestClient.post('/api/system/finance/expense/batch-delete', { ids });
};

// ==================== 费用申请流程操作 ====================

// 提交审批
export const submitExpenseApi = async (id: number) => {
  return requestClient.post('/api/system/finance/expense/submit', { id });
};

// 审批通过
export const approveExpenseApi = async (id: number, reason?: string) => {
  return requestClient.post('/api/system/finance/expense/approve', { id, reason });
};

// 审批驳回
export const rejectExpenseApi = async (id: number, reason?: string) => {
  return requestClient.post('/api/system/finance/expense/reject', { id, reason });
};

// 打款
export const paymentExpenseApi = async (data: {
  id: number;
  paymentAmount?: number;
  paymentDate?: string;
  paymentAccount?: string;
  transactionNo?: string;
  remark?: string;
}) => {
  return requestClient.post('/api/system/finance/expense/payment', data);
};

// ==================== 费用类型管理 ====================

export const getExpenseTypeListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/expense/type/list', { params });
};

export const saveExpenseTypeApi = async (data: any) => {
  return requestClient.post('/api/system/finance/expense/type/save', data);
};

export const deleteExpenseTypeApi = async (ids: number[]) => {
  return requestClient.post('/api/system/finance/expense/type/batch-delete', { ids });
};
