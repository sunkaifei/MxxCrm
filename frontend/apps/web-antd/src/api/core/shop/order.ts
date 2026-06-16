import { requestClient } from '#/api/request';

export interface OrderListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  shopId?: number;
  orderNo?: string;
  startTime?: string;
  endTime?: string;
}

export interface OrderVO {
  id: number;
  orderNo: string;
  userId: number;
  shopId: number;
  shopName?: string;
  totalAmount: number;
  freightAmount: number;
  commissionAmount: number;
  settlementAmount: number;
  goodsCount: number;
  status: number;
  receiverName: string;
  receiverPhone: string;
  receiverAddress: string;
  payTime?: string;
  deliveryTime?: string;
  receiveTime?: string;
  finishTime?: string;
  cancelReason?: string;
  createTime?: string;
  updateTime?: string;
}

export const orderApi = {
  list: (params: OrderListParams) =>
    requestClient.get('/api/system/order/list', { params }),
  detail: (id: number) => requestClient.get(`/api/system/order/detail/${id}`),
};
