import { requestClient } from '#/api/request';

/** 线索转移预览请求 */
export interface LeadTransferPreviewRequest {
  leadIds: number[];
  toUserId: number;
}

/** 线索转移预览响应 */
export interface LeadTransferPreviewVO {
  leadCount: number;
  followupCount: number;
  opportunityCount: number;
  affectedTotal: number;
}

/** 线索转移执行请求 */
export interface LeadTransferRequest {
  leadIds: number[];
  toUserId: number;
  /** 交接原因（字典 dict_label 文本） */
  transferReason: string;
  /** 备注（非必填） */
  remark?: string;
}

/** 线索转移执行响应 */
export interface LeadTransferResult {
  transferredCount: number;
  affectedTotal: number;
}

/** 预览线索转移影响范围 */
export const previewLeadTransferApi = async (data: LeadTransferPreviewRequest) => {
  return requestClient.post<LeadTransferPreviewVO>(
    '/api/system/lead/transfer/preview',
    data,
  );
};

/** 执行线索转移 */
export const transferLeadApi = async (data: LeadTransferRequest) => {
  return requestClient.post<LeadTransferResult>(
    '/api/system/lead/transfer',
    data,
  );
};
