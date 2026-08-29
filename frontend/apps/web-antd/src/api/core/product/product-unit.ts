import { requestClient } from '#/api/request';

export interface ProductUnitOption {
  id: string;
  name: string;
  /** 小数位精度：0=仅整数，2=最多两位小数 */
  decimalPlaces: number;
  isDefault: boolean;
}

export const getProductUnitOptionsApi = async () => {
  return requestClient.get('/api/system/product/unit/options');
};

export const saveProductUnitApi = async (param: {
  decimalPlaces?: number;
  name: string;
  remark?: string;
}) => {
  return requestClient.post('/api/system/product/unit/save', param);
};

export const updateProductUnitApi = async (param: {
  decimalPlaces: number;
  id: number;
}) => {
  return requestClient.put('/api/system/product/unit/update', param);
};

export const deleteProductUnitApi = async (id: number) => {
  return requestClient.delete('/api/system/product/unit/delete', {
    params: { id },
  });
};
