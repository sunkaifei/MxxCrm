import { requestClient } from '#/api/request';

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
export const updateOrderStatusApi = async (param: { id: number | string; orderStatus: number; trackingNo?: string; remark?: string }) => {
  return requestClient.put('/api/system/sale/order/updateStatus', param);
};

// ========== 订单审批 ==========

export const submitOrderApi = async (orderId: number) => {
  return requestClient.post('/api/system/sale/order/submit', { id: orderId });
};

export const approveOrderApi = async (orderId: number, reason?: string) => {
  return requestClient.post('/api/system/sale/order/approve', { orderId, reason });
};

export const rejectOrderApi = async (orderId: number, reason?: string) => {
  return requestClient.post('/api/system/sale/order/reject', { orderId, reason });
};

export const getOrderApprovalDetailApi = async (orderId: number) => {
  return requestClient.get(`/api/system/sale/order/approval-detail/${orderId}`);
};

export const createContractFromOrderApi = async (orderId: number) => {
  return requestClient.post('/api/system/sale/order/create-contract', { id: orderId });
};
