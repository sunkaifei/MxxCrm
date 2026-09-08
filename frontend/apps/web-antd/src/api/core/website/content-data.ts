import { requestClient } from '#/api/request';

/**
 * 内容模型「动态表内容」通用 API（T-P1.3）
 * 所有模型共用；`code` 为模型编码（modelCode），对应后端 `mxx_model_{code}` 表。
 */
export interface ContentDataListParams {
  page?: number;
  pageSize?: number;
  categoryId?: number;
  keywords?: string;
}

export const contentDataApi = {
  /** 分页列表 */
  list: (code: string, params?: ContentDataListParams) =>
    requestClient.get(`/api/system/content/data/${code}/list`, { params }),

  /** 详情 */
  detail: (code: string, id: number) =>
    requestClient.get(`/api/system/content/data/${code}/detail/${id}`),

  /** 新增（原始 JSON 对象体） */
  add: (code: string, data: Record<string, any>) =>
    requestClient.post(`/api/system/content/data/${code}/add`, data),

  /** 修改 */
  update: (code: string, id: number, data: Record<string, any>) =>
    requestClient.put(`/api/system/content/data/${code}/update/${id}`, data),

  /** 批量软删除 */
  delete: (code: string, ids: number[]) =>
    requestClient.delete(`/api/system/content/data/${code}/batch_delete`, {
      data: { ids },
    }),
};

export default contentDataApi;
