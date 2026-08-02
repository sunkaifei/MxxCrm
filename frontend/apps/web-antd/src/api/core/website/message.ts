import { requestClient } from '#/api/request';

export interface MessageListParams {
  page?: number;
  pageSize?: number;
  websiteId?: number;
  status?: number;
}

export interface MessageVO {
  id: number;
  websiteId?: number;
  contactName?: string;
  contactPhone?: string;
  contactEmail?: string;
  content?: string;
  status?: number;
  productId?: number;
  sourceUrl?: string;
  source?: string;
  leadId?: number;
  convertedToLead?: number;
  remark?: string;
  createTime?: string;
}

export interface ConvertLeadParams {
  assignedTo: number;
}

export const messageApi = {
  list: (params: MessageListParams) =>
    requestClient.get('/api/system/message/list', { params }),

  detail: (id: number) =>
    requestClient.get(`/api/system/message/detail/${id}`),

  convertLead: (id: number, data: ConvertLeadParams) =>
    requestClient.post(`/api/system/message/convert_lead/${id}`, data),

  updateStatus: (id: number, status: number) =>
    requestClient.put(`/api/system/message/status/${id}`, { status }),

  delete: (ids: number[]) =>
    requestClient.delete('/api/system/message/batch_delete', {
      data: { ids },
    }),
};
