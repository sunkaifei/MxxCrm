import { requestClient } from '#/api/request';

// ===== 付款记录管理 =====
export const getPaymentRecordListApi = async (params?: {
  userId?: number;
  paymentType?: number;
  status?: number;
  startTime?: string;
  endTime?: string;
  page?: number;
  pageSize?: number;
}) => requestClient.get('/api/system/finance/payment-record/list', { params });

export const getPaymentRecordDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/payment-record/detail/${id}`);

export const createPaymentRecordApi = async (data: {
  userId: number;
  memberProductId?: number;
  orderId?: string;
  paymentType?: number;
  amount: number;
  payMethod?: number;
  status?: number;
  transactionId?: string;
  payTime?: string;
  remark?: string;
}) => requestClient.post('/api/system/finance/payment-record/create', data);

export const updatePaymentRecordApi = async (
  id: number,
  data: {
    userId: number;
    memberProductId?: number;
    orderId?: string;
    paymentType?: number;
    amount: number;
    payMethod?: number;
    status?: number;
    transactionId?: string;
    payTime?: string;
    remark?: string;
  },
) => requestClient.put(`/api/system/finance/payment-record/update/${id}`, data);

export const deletePaymentRecordApi = async (id: number) =>
  requestClient.delete(`/api/system/finance/payment-record/delete/${id}`);
