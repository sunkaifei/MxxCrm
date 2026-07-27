import { requestClient } from '#/api/request';

/** 转移预览请求 */
export interface TransferPreviewRequest {
  customerIds: number[];
  toUserId: number;
}

/** 转移预览响应 */
export interface TransferPreviewVO {
  customerCount: number;
  opportunityCount: number;
  quotationCount: number;
  orderCount: number;
  contractCount: number;
  paymentPlanCount: number;
  paymentCount: number;
  invoiceCount: number;
  affectedTotal: number;
}

/** 转移执行请求 */
export interface TransferRequest {
  customerIds: number[];
  toUserId: number;
  /** 交接原因（字典 dict_label 文本） */
  transferReason: string;
  /** 备注（非必填） */
  remark?: string;
}

/** 转移执行响应 */
export interface TransferResult {
  transferredCount: number;
  affectedTotal: number;
}

/** 字典数据项（来自 /api/system/dict/data/{dict_code}/options） */
export interface DictDataOptionVO {
  label: string;
  value: string;
  tagType?: string;
  isDefault?: number;
}

/** 预览转移影响范围 */
export const previewCustomerTransferApi = async (data: TransferPreviewRequest) => {
  return requestClient.post<TransferPreviewVO>(
    '/api/system/customer/transfer/preview',
    data,
  );
};

/** 执行客户转移 */
export const transferCustomerApi = async (data: TransferRequest) => {
  return requestClient.post<TransferResult>(
    '/api/system/customer/transfer',
    data,
  );
};

/** 按 dict_code 获取字典数据选项（通用工具函数） */
export const getDictOptionsApi = async (dictCode: string) => {
  return requestClient.get<DictDataOptionVO[]>(
    `/api/system/dict/data/${dictCode}/options`,
  );
};
