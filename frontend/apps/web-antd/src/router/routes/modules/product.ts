import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:warehouse',
      order: 20,
      title: $t('page.warehouse.title'),
    },
    name: 'Warehouse',
    path: '/warehouse',
    children: [
      {
        name: 'WarehouseList',
        path: '/warehouse/list',
        component: () => import('#/views/product/list/index.vue'),
        meta: {
          title: $t('page.product.list.title'),
        },
      },
      {
        name: 'WarehouseCategory',
        path: '/warehouse/category',
        component: () => import('#/views/product/category/index.vue'),
        meta: {
          title: $t('page.product.category.title'),
        },
      },
      {
        name: 'WarehouseInventory',
        path: '/warehouse/inventory',
        component: () => import('#/views/product/inventory/index.vue'),
        meta: {
          title: $t('page.product.inventory.title'),
        },
      },
      {
        name: 'WarehouseManage',
        path: '/warehouse/warehouse',
        component: () => import('#/views/product/warehouse/index.vue'),
        meta: {
          title: $t('page.product.warehouse.title'),
        },
      },
      {
        name: 'WarehouseSku',
        path: '/warehouse/sku',
        component: () => import('#/views/product/sku/index.vue'),
        meta: {
          title: $t('page.product.sku.title'),
        },
      },
    ],
  },
];

export default routes;
