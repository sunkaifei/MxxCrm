import { requestClient } from '#/api/request';

// ===== 退款记录 =====
export const getRefundRecordListApi = async (params?: {
  userId?: number;
  status?: number;
  startTime?: string;
  endTime?: string;
  page?: number;
  pageSize?: number;
}) => requestClient.get('/api/system/finance/refund-record/list', { params });

export const getRefundRecordDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/refund-record/detail/${id}`);

export const createRefundRecordApi = async (data: {
  userId: number;
  paymentRecordId: number;
  amount: number;
  status?: number;
  transactionId?: string;
  refundTime?: string;
  reason?: string;
  remark?: string;
}) => requestClient.post('/api/system/finance/refund-record/create', data);

export const updateRefundRecordApi = async (
  id: number,
  data: {
    userId: number;
    paymentRecordId: number;
    amount: number;
    status?: number;
    transactionId?: string;
    refundTime?: string;
    reason?: string;
    remark?: string;
  },
) => requestClient.put(`/api/system/finance/refund-record/update/${id}`, data);

export const deleteRefundRecordApi = async (id: number) =>
  requestClient.delete(`/api/system/finance/refund-record/delete/${id}`);
