import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

/**
 * 盘点类型映射：前端数字 → 后端字符串
 * 后端 stocktake_type 字段为字符串：full / partial / dynamic
 */
const CHECK_TYPE_MAP: Record<number, string> = {
  1: 'full',
  2: 'partial',
  3: 'dynamic',
};

const CHECK_TYPE_REVERSE_MAP: Record<string, number> = {
  full: 1,
  partial: 2,
  dynamic: 3,
};

/** 将前端盘点类型(数字)转为后端格式(字符串) */
export function toBackendType(type: number | undefined): string | undefined {
  if (type === undefined || type === null) return undefined;
  return CHECK_TYPE_MAP[type] ?? 'partial';
}

/** 将后端盘点类型(字符串)转为前端格式(数字) */
export function toFrontendType(type: string | undefined): number {
  if (!type) return 2;
  return CHECK_TYPE_REVERSE_MAP[type] ?? 2;
}

export const getCheckListApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/inventory/check/list', { params });
};

export const getCheckInfoApi = async (id: number) => {
  return requestClient.get('/api/system/inventory/check/info', {
    params: { id },
  });
};

export const createCheckApi = async (data: any) => {
  const { checkType, ...rest } = data;
  return requestClient.post('/api/system/inventory/check/save', {
    ...rest,
    stocktakeType: toBackendType(checkType),
  });
};

export const updateCheckApi = async (data: any) => {
  const { id, checkType, ...rest } = data;
  return requestClient.put(`/api/system/inventory/check/update/${id}`, {
    ...rest,
    stocktakeType: toBackendType(checkType),
  });
};

export const deleteCheckApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/inventory/check/batch_delete', {
    data: { ids },
  });
};

export const auditCheckApi = async (id: number) => {
  return requestClient.post('/api/system/inventory/check/audit', { id });
};

/** 提交盘点单（草稿→盘点中） */
export const submitCheckApi = async (id: number) => {
  return requestClient.put(`/api/system/inventory/check/submit/${id}`);
};

/** 录入实盘数量 */
export const inputCheckApi = async (id: number, items: any[]) => {
  return requestClient.put(`/api/system/inventory/check/input/${id}`, {
    items,
  });
};

/** 完成盘点（自动生成出入库单+调库存） */
export const completeCheckApi = async (id: number) => {
  return requestClient.put(`/api/system/inventory/check/complete/${id}`);
};

/** 取消盘点 */
export const cancelCheckApi = async (id: number) => {
  return requestClient.put(`/api/system/inventory/check/cancel/${id}`);
};

/** 获取盘点明细列表 */
export const getCheckItemsApi = async (stocktakeId: number) => {
  return requestClient.get('/api/system/inventory/check/items', {
    params: { stocktakeId },
  });
};
