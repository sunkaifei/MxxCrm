import { requestClient } from '#/api/request';

export interface CartListParams {
  userId?: number;
}

export interface CartItemVO {
  id: number;
  userId?: number;
  productId?: number;
  skuId?: number;
  productName?: string;
  productImage?: string;
  skuCode?: string;
  skuSpecs?: string;
  price?: number;
  quantity?: number;
  checked?: number;
  createTime?: string;
}

export interface CartAddDTO {
  productId: number;
  skuId?: number;
  quantity?: number;
}

export interface CartUpdateDTO {
  id: number;
  quantity?: number;
  checked?: number;
}

export const cartApi = {
  /** 获取购物车列表 */
  list: (params?: CartListParams) =>
    requestClient.get('/api/user/cart/list', { params }),

  /** 添加商品到购物车 */
  add: (data: CartAddDTO) =>
    requestClient.post('/api/user/cart/add', data),

  /** 更新购物车项（数量/选中状态） */
  update: (id: number, data: CartUpdateDTO) =>
    requestClient.put(`/api/user/cart/update/${id}`, data),

  /** 批量删除购物车项 */
  batchDelete: (ids: number[]) =>
    requestClient.delete('/api/user/cart/batch_delete', { data: { ids } }),

  /** 清空购物车 */
  clear: (userId?: number) =>
    requestClient.delete('/api/user/cart/clear', { params: { userId } }),
};