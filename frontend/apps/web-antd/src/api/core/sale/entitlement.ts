import { requestClient } from '#/api/request';

// 服务权益列表
export const getEntitlementListApi = async (params: any) => {
  return requestClient.get('/api/system/sale/entitlement/list', { params });
};

// 权益详情
export const getEntitlementInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/entitlement/info', {
    params: { id },
  });
};

// 新建权益
export const createEntitlementApi = async (data: any) => {
  return requestClient.post('/api/system/sale/entitlement/save', data);
};

// 修改权益状态
export const updateEntitlementApi = async (data: any) => {
  return requestClient.put('/api/system/sale/entitlement/update', data);
};

// 续约（P0.1：字段契约 oldEntitlementId + newOrderId）
export const renewEntitlementApi = async (data: {
  newOrderId: number;
  oldEntitlementId: number;
}) => {
  return requestClient.post('/api/system/sale/entitlement/renew', data);
};

// 批量删除权益
export const deleteEntitlementApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/sale/entitlement/delete', {
    data: ids,
  });
};

// 按客户查询
export const getEntitlementByCustomerApi = async (customerId: number) => {
  return requestClient.get('/api/system/sale/entitlement/by-customer', {
    params: { id: customerId },
  });
};

// P0.3 回收站还原
export const restoreEntitlementApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/entitlement/restore', ids);
};

// P0.7 操作日志
export const getEntitlementLogApi = async (entitlementId: number) => {
  return requestClient.get('/api/system/sale/entitlement/log/list', {
    params: { entitlementId },
  });
};

// P1.3 配额流水
export const getEntitlementUsageApi = async (entitlementId: number) => {
  return requestClient.get('/api/system/sale/entitlement/usage/list', {
    params: { entitlementId },
  });
};

// P1.3 配额扣减
export const consumeEntitlementQuotaApi = async (data: {
  amount: number;
  bizId?: number;
  bizType?: number;
  id: number;
  remark?: string;
}) => {
  return requestClient.post('/api/system/sale/entitlement/usage/consume', data);
};

// P1.3 配额返还
export const refundEntitlementQuotaApi = async (data: {
  amount: number;
  bizId?: number;
  bizType?: number;
  id: number;
  remark?: string;
}) => {
  return requestClient.post('/api/system/sale/entitlement/usage/refund', data);
};

// P1.7 月度统计
export const getEntitlementSummaryApi = async () => {
  return requestClient.get('/api/system/sale/entitlement/summary');
};
