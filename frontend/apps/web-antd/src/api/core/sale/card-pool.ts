import { requestClient } from '#/api/request';

// 卡密列表
export const getCardPoolListApi = async (params: any) => {
  return requestClient.get('/api/system/sale/card-pool/list', { params });
};

// 新增卡密
export const createCardPoolApi = async (data: any) => {
  return requestClient.post('/api/system/sale/card-pool/save', data);
};

// 批量导入
export const importCardPoolApi = async (data: any) => {
  return requestClient.post('/api/system/sale/card-pool/import', data);
};

// 删除/作废
export const deleteCardPoolApi = async (id: number) => {
  return requestClient.delete('/api/system/sale/card-pool/delete', {
    params: { id },
  });
};

// 查询可用卡密数量
export const countUnsoldCardPoolApi = async (productId: number) => {
  return requestClient.get('/api/system/sale/card-pool/count', {
    params: { id: productId },
  });
};
