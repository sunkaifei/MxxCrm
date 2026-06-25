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

/** 获取可用规格值（淘宝式级联选择） */
export const getAvailableSpecValuesApi = async (param: { productId: number; selectedSpecs: Record<string, string> }) => {
  return requestClient.post('/api/system/product/spec/availableValues', param);
};

/** 根据规格组合获取对应的SKU */
export const getSkuBySpecsApi = async (param: { productId: number; specs: Record<string, string> }) => {
  return requestClient.post('/api/system/product/sku/getBySpecs', param);
};
