import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:shopping-cart',
      order: 5,
      title: $t('page.shop.title'),
    },
    name: 'Shop',
    path: '/shop',
    children: [
      {
        name: 'ShopCategory',
        path: '/shop/category',
        component: () => import('#/views/shop/category/index.vue'),
        meta: {
          title: $t('page.shop.category'),
        },
      },
      {
        name: 'ShopSupplierAudit',
        path: '/shop/supplier-audit',
        component: () => import('#/views/shop/supplier-audit/index.vue'),
        meta: {
          title: $t('page.shop.supplierAudit'),
        },
      },
      {
        name: 'ShopGoodsAudit',
        path: '/shop/goods-audit',
        component: () => import('#/views/shop/goods-audit/index.vue'),
        meta: {
          title: $t('page.shop.goodsAudit'),
        },
      },
      {
        name: 'ShopGoods',
        path: '/shop/goods',
        component: () => import('#/views/shop/goods/index.vue'),
        meta: {
          title: $t('page.shop.goods'),
        },
      },
      {
        name: 'ShopList',
        path: '/shop/shop-list',
        component: () => import('#/views/shop/shop-list/index.vue'),
        meta: {
          title: $t('page.shop.shopList'),
        },
      },
      {
        name: 'ShopOrderList',
        path: '/shop/order-list',
        component: () => import('#/views/shop/order-list/index.vue'),
        meta: {
          title: $t('page.shop.orderList'),
        },
      },
      {
        name: 'ShopSettlement',
        path: '/shop/settlement',
        component: () => import('#/views/shop/settlement/index.vue'),
        meta: {
          title: $t('page.shop.settlement'),
        },
      },
      {
        name: 'ShopPromotion',
        path: '/shop/promotion',
        component: () => import('#/views/shop/promotion/index.vue'),
        meta: {
          title: $t('page.shop.promotion'),
        },
      },
      {
        name: 'ShopCommissionConfig',
        path: '/shop/commission-config',
        component: () => import('#/views/shop/commission-config/index.vue'),
        meta: {
          title: $t('page.shop.commissionConfig'),
        },
      },
    ],
  },
];

export default routes;
