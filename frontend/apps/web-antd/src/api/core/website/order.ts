import type { DeliveryListParams } from './delivery';

import { requestClient } from '#/api/request';

export interface OrderListParams {
  page?: number;
  pageSize?: number;
  orderNo?: string;
  status?: number;
  payStatus?: number;
  shipStatus?: number;
  userId?: number;
}

export interface OrderItemVO {
  id: number;
  orderId?: number;
  productId?: number;
  skuId?: number;
  productName?: string;
  productImage?: string;
  skuCode?: string;
  skuSpecs?: string;
  price?: number;
  quantity?: number;
  totalAmount?: number;
  refundStatus?: number;
  createTime?: string;
}

export interface OrderVO {
  id: number;
  orderNo?: string;
  userId?: number;
  websiteId?: number;
  totalAmount?: number;
  discountAmount?: number;
  shippingFee?: number;
  payAmount?: number;
  status?: number;
  payStatus?: number;
  shipStatus?: number;
  payType?: number;
  payTime?: string;
  shipTime?: string;
  finishTime?: string;
  cancelTime?: string;
  cancelReason?: string;
  consigneeName?: string;
  consigneePhone?: string;
  consigneeAddress?: string;
  consigneeProvince?: string;
  consigneeCity?: string;
  consigneeDistrict?: string;
  consigneeZipcode?: string;
  buyerRemark?: string;
  sellerRemark?: string;
  transactionId?: string;
  createTime?: string;
  updateTime?: string;
  items?: OrderItemVO[];
}

export interface OrderUpdateParams {
  sellerRemark?: string;
  status?: number;
}

export interface ShipParams {
  deliveryNo: string;
  deliveryCompany: string;
  deliveryType?: number;
  remark?: string;
}

export const orderApi = {
  list: (params: OrderListParams) =>
    requestClient.get('/api/system/website_order/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/website_order/detail/${id}`),

  update: (id: number, data: OrderUpdateParams) =>
    requestClient.put(`/api/system/website_order/update/${id}`, data),

  ship: (id: number, data: ShipParams) =>
    requestClient.post(`/api/system/website_order/ship/${id}`, data),

  batchDelete: (ids: number[]) =>
    requestClient.delete('/api/system/website_order/batch_delete', {
      data: { ids },
    }),

  deliveryList: (params: DeliveryListParams) =>
    requestClient.get('/api/system/website_order/delivery/list', { params }),

  deliveryByOrder: (orderId: number) =>
    requestClient.get(`/api/system/website_order/delivery/order/${orderId}`),
};
