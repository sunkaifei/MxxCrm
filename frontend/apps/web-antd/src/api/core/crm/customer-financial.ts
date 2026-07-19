import { requestClient } from '#/api/request';

export interface BankAccount {
  accountName?: string;
  bankName: string;
  accountNumber: string;
  isDefault: boolean;
}

export interface CustomerFinancialVO {
  id?: number;
  customerId?: number;
  taxId?: string;
  invoiceTitle?: string;
  registeredAddress?: string;
  registeredPhone?: string;
  financePhone?: string;
  bankAccounts?: BankAccount[];
  createdBy?: number;
  createTime?: string;
  updatedBy?: number;
  updateTime?: string;
}

export const getCustomerFinancialApi = async (customerId: number) => {
  return requestClient.get(`/api/system/customer/financial/${customerId}`);
};

export const updateCustomerFinancialApi = async (params: CustomerFinancialVO & { customerId: number }) => {
  return requestClient.put('/api/system/customer/financial/update', params);
};
