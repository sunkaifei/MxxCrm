import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

export const getOrderListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/order/list', { params });
};
export const getOrderInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/order/info', { params: { id } });
};
export const createOrderApi = async (param: any) => {
  return requestClient.post('/api/system/sale/order/save', param);
};
export const updateOrderApi = async (param: any) => {
  return requestClient.put('/api/system/sale/order/update', param);
};
export const deleteOrderApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/order/batch-delete', { ids });
};
export const updateOrderStatusApi = async (param: {
  id: number | string;
  orderStatus: number;
  remark?: string;
  trackingNo?: string;
}) => {
  return requestClient.put('/api/system/sale/order/updateStatus', param);
};

// ========== 订单审批 ==========

export const submitOrderApi = async (data: {
  id: number;
  ccUserIds?: number[];
  ccReason?: string;
}) => {
  return requestClient.post('/api/system/sale/order/submit', data);
};

export const approveOrderApi = async (orderId: number, reason?: string) => {
  return requestClient.post('/api/system/sale/order/approve', {
    orderId,
    reason,
  });
};

export const rejectOrderApi = async (orderId: number, reason?: string) => {
  return requestClient.post('/api/system/sale/order/reject', {
    orderId,
    reason,
  });
};

export const getOrderApprovalDetailApi = async (orderId: number) => {
  return requestClient.get(`/api/system/sale/order/approval-detail/${orderId}`);
};

/** 订单审批流预览（提交审批前查看流程） */
export const getOrderApprovalFlowApi = async () => {
  return requestClient.get('/api/system/sale/order/approval-flow');
};

export const createContractFromOrderApi = async (orderId: number) => {
  return requestClient.post('/api/system/sale/order/create-contract', {
    id: orderId,
  });
};
