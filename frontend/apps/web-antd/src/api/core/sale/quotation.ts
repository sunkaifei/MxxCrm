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
  return requestClient.delete('/api/system/sale/quotation/batch-delete', {
    data: { ids },
  });
};
export const sendQuotationApi = async (id: number) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/send`);
};
export const acceptQuotationApi = async (id: number) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/accept`);
};
export const rejectQuotationApi = async (id: number) => {
  return requestClient.post(`/api/system/sale/quotation/${id}/reject`);
};