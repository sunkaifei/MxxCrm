import { requestClient } from '#/api/request';

export interface SupplierAuditListParams {
  page?: number;
  pageSize?: number;
  status?: number;
}

export interface SupplierAuditDTO {
  id: number;
  status: number;
  auditRemark?: string;
}

export interface SupplierApplyVO {
  id: number;
  userId: number;
  shopName: string;
  contactName: string;
  contactPhone: string;
  shopLogo?: string;
  shopDesc?: string;
  businessLicense?: string;
  status: number;
  auditRemark?: string;
  auditTime?: string;
  createTime?: string;
  updateTime?: string;
}

export const supplierAuditApi = {
  list: (params: SupplierAuditListParams) =>
    requestClient.get('/api/system/supplier/list', { params }),
  audit: (data: SupplierAuditDTO) =>
    requestClient.put('/api/system/supplier/audit', data),
};
