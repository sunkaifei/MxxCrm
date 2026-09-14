import { requestClient } from '#/api/request';

/**
 * 网站展示产品（上架清单）API
 * 单站模式：后端自动定位默认站点，前端无需传 websiteId
 */

// 展示产品清单（分页，含产品信息富化）
export const getWebsiteProductListApi = async (params?: any) => {
  return requestClient.get('/api/system/website/product/list', { params });
};

// 从产品库选择产品加入展示清单（自动上架、自动去重，数量钳制为当前库存）
// 相关产品会一并加入清单，相关关系写入主产品（第一个添加项）
export const addWebsiteProductApi = async (
  items: {
    productId: number;
    quantity: number;
    skuIds?: number[];
  }[],
  relatedProductIds: number[] = [],
  relatedArticleIds: number[] = [],
) => {
  return requestClient.post('/api/system/website/product/add', {
    items,
    relatedProductIds,
    relatedArticleIds,
  });
};

// 批量设置/取消推荐
export const recommendWebsiteProductApi = async (ids: number[], isRecommend: number) => {
  return requestClient.put('/api/system/website/product/recommend', {
    ids,
    isRecommend,
  });
};

// 保存相关产品/相关文章
export const updateWebsiteProductRelatedApi = async (
  id: number,
  relatedProductIds: number[],
  relatedArticleIds: number[],
) => {
  return requestClient.put('/api/system/website/product/related', {
    id,
    relatedProductIds,
    relatedArticleIds,
  });
};

// 保存 SEO 设置（仅作用于前台详情页 head）
export const updateWebsiteProductSeoApi = async (
  id: number,
  seoTitle: string | undefined,
  seoKeywords: string | undefined,
  seoDescription: string | undefined,
) => {
  return requestClient.put('/api/system/website/product/seo', {
    id,
    seoTitle,
    seoKeywords,
    seoDescription,
  });
};

// 批量上架/下架（status: 1=上架 0=下架）
export const shelfWebsiteProductApi = async (ids: number[], status: number) => {
  return requestClient.put('/api/system/website/product/shelf', {
    ids,
    status,
  });
};

// 调整清单排序（数值越小前台展示越靠前）
export const sortWebsiteProductApi = async (id: number, sort: number) => {
  return requestClient.put('/api/system/website/product/sort', { id, sort });
};

// 调整展示数量（防超卖，后端钳制为不超过当前仓储库存）
export const quantityWebsiteProductApi = async (id: number, quantity: number) => {
  return requestClient.put('/api/system/website/product/quantity', {
    id,
    quantity,
  });
};

// 归属栏目分类（栏目管理中内容类型=产品的栏目；0=清除）
export const categoryWebsiteProductApi = async (id: number, categoryId: number) => {
  return requestClient.put('/api/system/website/product/category', {
    id,
    categoryId,
  });
};

// 栏目分类选项（内容类型=产品的栏目）
export const getWebsiteProductCategoriesApi = async () => {
  return requestClient.get('/api/system/website/product/categories');
};

// 产品的 SKU 列表（选品抽屉展开用）
export const getWebsiteProductSkusApi = async (productId: number) => {
  return requestClient.get('/api/system/website/product/skus', {
    params: { productId },
  });
};

// 更新清单行展示的 SKU（空 = 全部 SKU）
export const updateWebsiteProductSkusApi = async (id: number, skuIds: number[]) => {
  return requestClient.put('/api/system/website/product/update_skus', {
    id,
    skuIds,
  });
};

// 保存 SKU 前台零售价（prices: { [skuId: string]: number }，仅作用于前台在线销售）
export const saveWebsiteProductSkuPricesApi = async (
  id: number,
  prices: Record<string, number>,
) => {
  return requestClient.put('/api/system/website/product/sku_prices', {
    id,
    prices,
  });
};

// 保存 SKU 前台销售库存（quantities: { [skuId: string]: number }，仅作用于前台在线销售，后端钳制为不超过该 SKU 仓储库存）
export const saveWebsiteProductSkuQuantitiesApi = async (
  id: number,
  quantities: Record<string, number>,
) => {
  return requestClient.put('/api/system/website/product/sku_quantities', {
    id,
    quantities,
  });
};

// 批量移除出展示清单
export const deleteWebsiteProductApi = async (ids: number[]) => {
  return requestClient.delete('/api/system/website/product/delete', {
    data: { ids },
  });
};

// 产品库产品列表（选品弹窗数据源，复用产品库接口）
export const getProductLibraryListApi = async (params?: any) => {
  return requestClient.get('/api/system/product/product/list', { params });
};
