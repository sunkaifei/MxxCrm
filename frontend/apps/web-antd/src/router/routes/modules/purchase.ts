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
          icon: 'lucide:building-2',
          title: $t('page.purchase.supplier.title'),
        },
      },
      {
        name: 'PurchasePo',
        path: '/purchase/po',
        component: () => import('#/views/purchase/po/index.vue'),
        meta: {
          icon: 'lucide:file-text',
          title: $t('page.purchase.po.title'),
        },
      },
      {
        name: 'PurchaseRequisition',
        path: '/purchase/requisition',
        component: () => import('#/views/purchase/requisition/index.vue'),
        meta: {
          icon: 'lucide:clipboard-list',
          title: $t('page.purchase.requisition.title'),
        },
      },
      {
        name: 'PurchaseReceipt',
        path: '/purchase/receipt',
        component: () => import('#/views/purchase/receipt/index.vue'),
        meta: {
          icon: 'lucide:package-check',
          title: $t('page.purchase.receipt.title'),
        },
      },
      {
        name: 'PurchaseReturn',
        path: '/purchase/return',
        component: () => import('#/views/purchase/return/index.vue'),
        meta: {
          icon: 'lucide:undo-2',
          title: $t('page.purchase.return.title'),
        },
      },
      {
        name: 'PurchaseStockPlan',
        path: '/purchase/stock-plan',
        component: () => import('#/views/purchase/stock-plan/index.vue'),
        meta: {
          icon: 'lucide:warehouse',
          title: $t('page.purchase.stockPlan.title'),
        },
      },
      {
        name: 'PurchaseReport',
        path: '/purchase/report',
        component: () => import('#/views/purchase/report/index.vue'),
        meta: {
          icon: 'lucide:bar-chart-3',
          title: $t('page.purchase.report.title'),
        },
      },
    ],
  },
];

export default routes;
