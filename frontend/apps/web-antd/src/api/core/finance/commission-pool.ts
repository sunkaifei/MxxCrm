import { requestClient } from '#/api/request';

// 资金池列表
export const getCommissionPoolListApi = async (params?: any) =>
  requestClient.get('/api/system/finance/commission-pool/list', { params });

// 资金池详情
export const getCommissionPoolDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/commission-pool/detail/${id}`);

// 保存资金池（新建/编辑）
export const saveCommissionPoolApi = async (data: any) =>
  requestClient.post('/api/system/finance/commission-pool/save', data);

// 删除资金池
export const deleteCommissionPoolApi = async (id: number) =>
  requestClient.post('/api/system/finance/commission-pool/save', {
    id,
    deleted: 1,
  });

// 支出登记
export const expenseCommissionPoolApi = async (data: {
  poolId: number;
  amount: number;
  usageDate?: string;
  usageDescription?: string;
}) =>
  requestClient.post('/api/system/finance/commission-pool/expense', data);

// 流水明细
export const getCommissionPoolLogApi = async (id: number, params?: any) =>
  requestClient.get(`/api/system/finance/commission-pool/log/${id}`, {
    params,
  });
