import { requestClient } from '#/api/request';

/** 获取产品规格定义和SKU列表 */
export const getProductSpecsApi = async (productId: number) => {
  return requestClient.get('/api/system/product/spec/list', {
    params: { productId },
  });
};

/** 保存产品规格定义 */
export const saveProductSpecsApi = async (param: any) => {
  return requestClient.post('/api/system/product/spec/save', param);
};

/** 根据规格自动生成SKU组合 */
export const generateSkusApi = async (productId: number) => {
  return requestClient.get('/api/system/product/sku/generate', {
    params: { productId },
  });
};

/** 批量保存SKU */
export const batchSaveSkusApi = async (param: { productId: number; skus: any[] }) => {
  return requestClient.post('/api/system/product/sku/batchSave', param);
};
