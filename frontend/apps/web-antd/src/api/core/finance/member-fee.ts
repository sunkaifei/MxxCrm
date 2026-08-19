import { requestClient } from '#/api/request';

// ===== 会员费 =====
export const getMemberFeeListApi = async (params?: {
  memberType?: number;
  page?: number;
  pageSize?: number;
  status?: number;
  userId?: number;
}) => requestClient.get('/api/system/finance/member-fee/list', { params });

export const getMemberFeeDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/member-fee/detail/${id}`);

export const createMemberFeeApi = async (data: {
  amount: number;
  memberType?: number;
  paymentRecordId?: number;
  remark?: string;
  status?: number;
  userId: number;
  validEndTime?: string;
  validStartTime?: string;
}) => requestClient.post('/api/system/finance/member-fee/create', data);

export const updateMemberFeeApi = async (
  id: number,
  data: {
    amount: number;
    memberType?: number;
    paymentRecordId?: number;
    remark?: string;
    status?: number;
    userId: number;
    validEndTime?: string;
    validStartTime?: string;
  },
) => requestClient.put(`/api/system/finance/member-fee/update/${id}`, data);

export const deleteMemberFeeApi = async (id: number) =>
  requestClient.delete(`/api/system/finance/member-fee/delete/${id}`);
