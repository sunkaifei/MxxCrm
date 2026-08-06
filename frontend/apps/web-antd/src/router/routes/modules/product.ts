import type { RouteRecordRaw } from 'vue-router';
import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:package',
      order: 26,
      title: $t('page.product.title'),
    },
    name: 'Product',
    path: '/product',
    children: [
      {
        name: 'ProductBrand',
        path: '/product/brand',
        component: () => import('#/views/product/brand/index.vue'),
        meta: {
          title: $t('page.product.brand.title'),
        },
      },
      {
        name: 'InboundList',
        path: '/inbound',
        component: () => import('#/views/product/inbound/index.vue'),
        meta: {
          title: $t('page.inventory.inbound.title'),
        },
      },
      {
        name: 'OutboundList',
        path: '/outbound',
        component: () => import('#/views/product/outbound/index.vue'),
        meta: {
          title: $t('page.inventory.outbound.title'),
        },
      },
      {
        name: 'InventoryList',
        path: '/inventory',
        component: () => import('#/views/product/inventory/index.vue'),
        meta: {
          title: $t('page.inventory.title'),
        },
      },
      {
        name: 'InventoryCheckList',
        path: '/inventory-check',
        component: () => import('#/views/product/inventory-check/index.vue'),
        meta: {
          title: $t('page.inventory.check.title'),
        },
      },
      {
        name: 'TransferList',
        path: '/transfer',
        component: () => import('#/views/product/transfer/index.vue'),
        meta: {
          title: $t('page.inventory.transfer.title'),
        },
      },
      {
        name: 'InventoryAlert',
        path: '/inventory-alert',
        component: () => import('#/views/product/inventory-alert/index.vue'),
        meta: {
          title: $t('page.inventory.alert.title'),
        },
      },
      {
        name: 'InventoryReport',
        path: '/inventory-report',
        component: () => import('#/views/product/inventory-report/index.vue'),
        meta: {
          title: $t('page.inventory.report.title'),
        },
      },
      {
        name: 'StockLog',
        path: '/stock-log',
        component: () => import('#/views/product/inventory/log.vue'),
        meta: {
          title: $t('page.inventory.stockLog.title'),
        },
      },
      {
        name: 'AlertRule',
        path: '/alert-rule',
        component: () => import('#/views/product/inventory-alert/rule.vue'),
        meta: {
          title: $t('page.inventory.alert.title') + ' - ' + $t('page.inventory.alert.action.viewRule'),
        },
      },
    ],
  },
];
export default routes;