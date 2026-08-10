import { requestClient } from '#/api/request';

// 虚拟商品交付列表
export const getDeliveryListApi = async (params: any) => {
  return requestClient.get('/api/system/sale/delivery/list', { params });
};

// 交付详情（脱敏）
export const getDeliveryInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/delivery/info', { params: { id } });
};

// 查看完整内容（需 sale:delivery:view 权限）
export const viewFullDeliveryApi = async (id: number) => {
  return requestClient.get('/api/system/sale/delivery/view-full', {
    params: { id },
  });
};

// 手动录入交付记录
export const createDeliveryApi = async (data: any) => {
  return requestClient.post('/api/system/sale/delivery/save', data);
};

// 修改交付状态
export const updateDeliveryApi = async (data: any) => {
  return requestClient.put('/api/system/sale/delivery/update', data);
};

// 重发通知
export const resendDeliveryApi = async (id: number) => {
  return requestClient.post(
    '/api/system/sale/delivery/resend',
    {},
    { params: { id } },
  );
};
