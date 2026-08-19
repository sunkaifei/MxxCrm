import { requestClient } from '#/api/request';

export interface DeliveryListParams {
  page?: number;
  pageSize?: number;
  orderNo?: string;
  deliveryNo?: string;
  status?: number;
}

export interface DeliveryVO {
  id: number;
  orderId?: number;
  orderNo?: string;
  deliveryNo?: string;
  deliveryCompany?: string;
  deliveryType?: number;
  consigneeName?: string;
  consigneePhone?: string;
  consigneeAddress?: string;
  status?: number;
  remark?: string;
  createTime?: string;
  updateTime?: string;
}

export interface DeliverySaveDTO {
  orderId: number;
  deliveryNo: string;
  deliveryCompany: string;
  deliveryType?: number;
  remark?: string;
}

export const deliveryApi = {
  list: (params: DeliveryListParams) =>
    requestClient.get('/api/system/website_order/delivery/list', { params }),

  /** 按订单ID查询发货记录 */
  detail: (orderId: number) =>
    requestClient.get(`/api/system/website_order/delivery/order/${orderId}`),
};
