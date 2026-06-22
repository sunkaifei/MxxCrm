import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:package',
      order: 20,
      title: $t('page.product.title'),
    },
    name: 'Product',
    path: '/product',
    children: [
      {
        name: 'ProductList',
        path: '/product/list',
        component: () => import('#/views/product/list/index.vue'),
        meta: {
          title: $t('page.product.list.title'),
        },
      },
      {
        name: 'ProductCategory',
        path: '/product/category',
        component: () => import('#/views/product/category/index.vue'),
        meta: {
          title: $t('page.product.category.title'),
        },
      },
      {
        name: 'ProductInventory',
        path: '/product/inventory',
        component: () => import('#/views/product/inventory/index.vue'),
        meta: {
          title: $t('page.product.inventory.title'),
        },
      },
      {
        name: 'ProductWarehouse',
        path: '/product/warehouse',
        component: () => import('#/views/product/warehouse/index.vue'),
        meta: {
          title: $t('page.product.warehouse.title'),
        },
      },
      {
        name: 'ProductSku',
        path: '/product/sku',
        component: () => import('#/views/product/sku/index.vue'),
        meta: {
          title: $t('page.product.sku.title'),
        },
      },
    ],
  },
];

export default routes;
