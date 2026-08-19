import { requestClient } from '#/api/request';

// ===== 退款记录 =====
export const getRefundRecordListApi = async (params?: {
  endTime?: string;
  page?: number;
  pageSize?: number;
  startTime?: string;
  status?: number;
  userId?: number;
}) => requestClient.get('/api/system/finance/refund-record/list', { params });

export const getRefundRecordDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/refund-record/detail/${id}`);

export const createRefundRecordApi = async (data: {
  amount: number;
  paymentRecordId: number;
  reason?: string;
  refundTime?: string;
  remark?: string;
  status?: number;
  transactionId?: string;
  userId: number;
}) => requestClient.post('/api/system/finance/refund-record/create', data);

export const updateRefundRecordApi = async (
  id: number,
  data: {
    amount: number;
    paymentRecordId: number;
    reason?: string;
    refundTime?: string;
    remark?: string;
    status?: number;
    transactionId?: string;
    userId: number;
  },
) => requestClient.put(`/api/system/finance/refund-record/update/${id}`, data);

export const deleteRefundRecordApi = async (id: number) =>
  requestClient.delete(`/api/system/finance/refund-record/delete/${id}`);
