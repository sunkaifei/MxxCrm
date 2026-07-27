import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:globe',
      order: 5,
      title: $t('page.website.title'),
    },
    name: 'Website',
    path: '/website',
    children: [
      {
        name: 'WebsiteList',
        path: '/website/list',
        component: () => import('#/views/website/site/index.vue'),
        meta: {
          title: $t('page.website.list'),
        },
      },
      {
        name: 'WebsiteCategory',
        path: '/website/category',
        component: () => import('#/views/website/category/index.vue'),
        meta: {
          title: $t('page.website.categoryTitle'),
        },
      },
      {
        name: 'WebsiteArticle',
        path: '/website/article',
        component: () => import('#/views/website/article/index.vue'),
        meta: {
          title: $t('page.website.articleTitle'),
        },
      },
      {
        name: 'WebsiteMessage',
        path: '/website/message',
        component: () => import('#/views/shop/goods-audit/index.vue'),
        meta: {
          title: $t('page.website.messageTitle'),
        },
      },
      {
        name: 'WebsiteLinks',
        path: '/website/links',
        component: () => import('#/views/website/links/index.vue'),
        meta: {
          title: $t('page.website.linksTitle'),
        },
      },
      {
        name: 'WebsiteSupplierAudit',
        path: '/website/supplier-audit',
        component: () => import('#/views/shop/supplier-audit/index.vue'),
        meta: {
          title: $t('page.website.supplierAudit'),
        },
      },
      {
        name: 'WebsiteGoodsAudit',
        path: '/website/goods-audit',
        component: () => import('#/views/shop/goods-audit/index.vue'),
        meta: {
          title: $t('page.website.goodsAudit'),
        },
      },
      {
        name: 'WebsiteGoods',
        path: '/website/goods',
        component: () => import('#/views/shop/goods/index.vue'),
        meta: {
          title: $t('page.website.goods'),
        },
      },
      {
        name: 'WebsiteOrderList',
        path: '/website/order-list',
        component: () => import('#/views/shop/order-list/index.vue'),
        meta: {
          title: $t('page.website.orderList'),
        },
      },
      {
        name: 'WebsiteSettlement',
        path: '/website/settlement',
        component: () => import('#/views/shop/settlement/index.vue'),
        meta: {
          title: $t('page.website.settlement'),
        },
      },
      {
        name: 'WebsitePromotion',
        path: '/website/promotion',
        component: () => import('#/views/shop/promotion/index.vue'),
        meta: {
          title: $t('page.website.promotion'),
        },
      },
      {
        name: 'WebsiteCommissionConfig',
        path: '/website/commission-config',
        component: () => import('#/views/shop/commission-config/index.vue'),
        meta: {
          title: $t('page.website.commissionConfig'),
        },
      },
      {
        name: 'WebsiteTemplate',
        path: '/website/template',
        component: () => import('#/views/website/template/index.vue'),
        meta: {
          title: $t('page.website.templateTitle'),
        },
      },
      {
        name: 'WebsiteTemplateData',
        path: '/website/template-data',
        component: () => import('#/views/website/template-data/index.vue'),
        meta: {
          title: $t('page.website.templateDataTitle'),
        },
      },
    ],
  },
];

export default routes;
