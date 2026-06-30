import { requestClient } from '#/api/request';

export const getQuotationListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/sale/quotation/list', { params });
};
export const getQuotationInfoApi = async (id: number) => {
  return requestClient.get('/api/system/sale/quotation/info', { params: { id } });
};
export const createQuotationApi = async (param: any) => {
  return requestClient.post('/api/system/sale/quotation/save', param);
};
export const updateQuotationApi = async (param: any) => {
  return requestClient.put('/api/system/sale/quotation/update', param);
};
export const deleteQuotationApi = async (ids: number[]) => {
  return requestClient.post('/api/system/sale/quotation/batch-delete', { ids });
};
export const submitQuotationApprovalApi = async (id: number, remark?: string) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/submit-approval`, { remark });
};
export const approveQuotationApi = async (id: number, remark?: string) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/approve`, { remark });
};
export const rejectQuotationApi = async (id: number, remark?: string) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/reject`, { remark });
};
export const convertToOrderApi = async (id: number) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/convert-order`);
};
