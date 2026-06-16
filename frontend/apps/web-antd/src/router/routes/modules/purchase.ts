import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:truck',
      order: 25,
      title: $t('page.purchase.title'),
    },
    name: 'Purchase',
    path: '/purchase',
    children: [
      {
        name: 'PurchaseSupplier',
        path: '/purchase/supplier',
        component: () => import('#/views/purchase/supplier/index.vue'),
        meta: {
          title: $t('page.purchase.supplier.title'),
        },
      },
      {
        name: 'PurchasePo',
        path: '/purchase/po',
        component: () => import('#/views/purchase/po/index.vue'),
        meta: {
          title: $t('page.purchase.po.title'),
        },
      },
      {
        name: 'PurchaseItem',
        path: '/purchase/item',
        component: () => import('#/views/purchase/item/index.vue'),
        meta: {
          title: $t('page.purchase.item.title'),
        },
      },
    ],
  },
];

export default routes;
