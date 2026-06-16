import { requestClient } from '#/api/request';

export interface CommissionConfigDTO {
  defaultRate?: number;
  shopRates?: Array<{
    rate: number;
    shopId: number;
  }>;
}

export interface CommissionConfigVO {
  defaultRate: number;
  shopRates: Array<{
    rate: number;
    shopId: number;
    shopName: string;
  }>;
}

export const commissionApi = {
  getConfig: () => requestClient.get('/api/system/config/commission'),
  saveConfig: (data: CommissionConfigDTO) =>
    requestClient.put('/api/system/config/commission', data),
};
