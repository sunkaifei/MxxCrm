import { requestClient } from '#/api/request';

export interface RefundListParams {
  page?: number;
  pageSize?: number;
  refundNo?: string;
  orderNo?: string;
  status?: number;
  userId?: number;
}

export interface RefundVO {
  id: number;
  refundNo?: string;
  orderId?: number;
  orderNo?: string;
  userId?: number;
  orderItemId?: number;
  refundType?: number;
  refundReason?: string;
  refundAmount?: number;
  status?: number;
  refundWay?: number;
  transactionId?: string;
  handleRemark?: string;
  handleBy?: number;
  handleTime?: string;
  createTime?: string;
  updateTime?: string;
}

export interface RefundHandleParams {
  action: number;
  handleRemark?: string;
  refundWay?: number;
}

export interface MarkRefundedParams {
  transactionId?: string;
}

export const refundApi = {
  list: (params: RefundListParams) =>
    requestClient.get('/api/system/website_refund/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/website_refund/detail/${id}`),

  handle: (id: number, data: RefundHandleParams) =>
    requestClient.post(`/api/system/website_refund/handle/${id}`, data),

  markRefunded: (id: number, data?: MarkRefundedParams) =>
    requestClient.post(`/api/system/website_refund/mark_refunded/${id}`, data ?? {}),

  batchDelete: (ids: number[]) =>
    requestClient.delete('/api/system/website_refund/batch_delete', {
      data: { ids },
    }),
};
