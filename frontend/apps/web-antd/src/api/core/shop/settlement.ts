import { requestClient } from '#/api/request';

export interface SettlementListParams {
  page?: number;
  pageSize?: number;
  status?: number;
  shopId?: number;
}

export interface SettlementPayDTO {
  id: number;
}

export interface SettlementVO {
  id: number;
  shopId: number;
  shopName?: string;
  settlementNo: string;
  periodStart: string;
  periodEnd: string;
  orderCount: number;
  totalAmount: number;
  commissionAmount: number;
  settlementAmount: number;
  status: number;
  payTime?: string;
  remark?: string;
  createTime?: string;
  updateTime?: string;
}

export const settlementApi = {
  list: (params: SettlementListParams) =>
    requestClient.get('/api/system/settlement/list', { params }),
  detail: (id: number) =>
    requestClient.get(`/api/system/settlement/detail/${id}`),
  pay: (data: SettlementPayDTO) =>
    requestClient.put('/api/system/settlement/pay', data),
};
