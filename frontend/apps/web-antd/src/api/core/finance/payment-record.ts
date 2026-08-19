import { requestClient } from '#/api/request';

// ===== 付款记录管理 =====
export const getPaymentRecordListApi = async (params?: {
  endTime?: string;
  page?: number;
  pageSize?: number;
  paymentType?: number;
  startTime?: string;
  status?: number;
  userId?: number;
}) => requestClient.get('/api/system/finance/payment-record/list', { params });

export const getPaymentRecordDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/payment-record/detail/${id}`);

export const createPaymentRecordApi = async (data: {
  amount: number;
  memberProductId?: number;
  orderId?: string;
  paymentType?: number;
  payMethod?: number;
  payTime?: string;
  remark?: string;
  status?: number;
  transactionId?: string;
  userId: number;
}) => requestClient.post('/api/system/finance/payment-record/create', data);

export const updatePaymentRecordApi = async (
  id: number,
  data: {
    amount: number;
    memberProductId?: number;
    orderId?: string;
    paymentType?: number;
    payMethod?: number;
    payTime?: string;
    remark?: string;
    status?: number;
    transactionId?: string;
    userId: number;
  },
) => requestClient.put(`/api/system/finance/payment-record/update/${id}`, data);

export const deletePaymentRecordApi = async (id: number) =>
  requestClient.delete(`/api/system/finance/payment-record/delete/${id}`);
