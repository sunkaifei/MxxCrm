import { requestClient } from '#/api/request';

// 发货单列表
export const getShipmentListApi = async (params: any) => {
  return requestClient.get('/api/system/sale/shipment/list', { params });
};

// 发货单详情
export const getShipmentInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/shipment/info', {
    params: { id },
  });
};

// 创建发货单
export const createShipmentApi = async (data: any) => {
  return requestClient.post('/api/system/sale/shipment/save', data);
};

// 修改发货单
export const updateShipmentApi = async (data: any) => {
  return requestClient.put('/api/system/sale/shipment/update', data);
};

// 删除发货单
export const deleteShipmentApi = async (id: number) => {
  return requestClient.delete('/api/system/sale/shipment/delete', {
    params: { id },
  });
};

// 签收确认（后端使用 web::Query<InfoId>，需要以 query 参数传递 id）
export const signShipmentApi = async (id: number) => {
  return requestClient.post(
    '/api/system/sale/shipment/sign',
    {},
    { params: { id } },
  );
};
