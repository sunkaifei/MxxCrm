import { requestClient } from '#/api/request';

// ===== 会员费 =====
export const getMemberFeeListApi = async (params?: {
  userId?: number;
  memberType?: number;
  status?: number;
  page?: number;
  pageSize?: number;
}) => requestClient.get('/api/system/finance/member-fee/list', { params });

export const getMemberFeeDetailApi = async (id: number) =>
  requestClient.get(`/api/system/finance/member-fee/detail/${id}`);

export const createMemberFeeApi = async (data: {
  userId: number;
  memberType?: number;
  amount: number;
  validStartTime?: string;
  validEndTime?: string;
  status?: number;
  paymentRecordId?: number;
  remark?: string;
}) => requestClient.post('/api/system/finance/member-fee/create', data);

export const updateMemberFeeApi = async (
  id: number,
  data: {
    userId: number;
    memberType?: number;
    amount: number;
    validStartTime?: string;
    validEndTime?: string;
    status?: number;
    paymentRecordId?: number;
    remark?: string;
  },
) => requestClient.put(`/api/system/finance/member-fee/update/${id}`, data);

export const deleteMemberFeeApi = async (id: number) =>
  requestClient.delete(`/api/system/finance/member-fee/delete/${id}`);
