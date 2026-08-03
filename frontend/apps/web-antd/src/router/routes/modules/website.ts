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
        component: () => import('#/views/website/site/settings.vue'),
        meta: {
          title: $t('page.website.list'),
        },
      },
      {
        name: 'WebsiteNavigation',
        path: '/website/navigation',
        component: () => import('#/views/website/navigation/index.vue'),
        meta: {
          title: $t('page.website.navigationTitle'),
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
        name: 'WebsiteArticleField',
        path: '/website/article-field',
        component: () => import('#/views/website/article-field/index.vue'),
        meta: {
          title: $t('page.website.articleFieldTitle'),
        },
      },
      {
        name: 'WebsiteMessage',
        path: '/website/message',
        component: () => import('#/views/website/message/index.vue'),
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
        name: 'WebsiteTemplate',
        path: '/website/template',
        component: () => import('#/views/website/template/index.vue'),
        meta: {
          title: $t('page.website.templateManageTitle'),
        },
      },
      {
        name: 'WebsiteContentModel',
        path: '/website/content-model',
        component: () => import('#/views/website/content-model/index.vue'),
        meta: {
          title: $t('page.website.contentModelTitle'),
        },
      },
      {
        name: 'WebsiteTemplateVar',
        path: '/website/template-var',
        component: () => import('#/views/website/template-var/index.vue'),
        meta: {
          title: $t('page.website.templateVarTitle'),
        },
      },
      {
        name: 'WebsiteArticleTag',
        path: '/website/article-tag',
        component: () => import('#/views/website/article-tag/index.vue'),
        meta: {
          title: '文章标签',
        },
      },
      {
        name: 'WebsiteMedia',
        path: '/website/media',
        component: () => import('#/views/website/media/index.vue'),
        meta: {
          title: $t('page.website.mediaTitle'),
        },
      },
      {
        name: 'WebsiteBanner',
        path: '/website/banner',
        component: () => import('#/views/website/banner/index.vue'),
        meta: {
          title: $t('page.website.bannerTitle'),
        },
      },
      {
        name: 'WebsiteBlock',
        path: '/website/block',
        component: () => import('#/views/website/block/index.vue'),
        meta: {
          title: $t('page.website.blockTitle'),
        },
      },
      {
        name: 'WebsitePage',
        path: '/website/page',
        component: () => import('#/views/website/page/index.vue'),
        meta: {
          title: $t('page.website.pageTitle'),
        },
      },
      {
        name: 'WebsiteOrder',
        path: '/website/order',
        component: () => import('#/views/website/order/index.vue'),
        meta: {
          title: $t('page.website.orderTitle'),
        },
      },
      {
        name: 'WebsiteRefund',
        path: '/website/refund',
        component: () => import('#/views/website/refund/index.vue'),
        meta: {
          title: $t('page.website.refundTitle'),
        },
      },
      {
        name: 'WebsiteUser',
        path: '/website/user',
        component: () => import('#/views/website/user/index.vue'),
        meta: {
          title: $t('page.website.userTitle'),
        },
      },
      {
        name: 'WebsiteStaticGenerate',
        path: '/website/static-generate',
        component: () => import('#/views/website/static-generate/index.vue'),
        meta: {
          title: '静态化生成',
        },
      },
      {
        name: 'WebsiteCollector',
        path: '/website/collector',
        component: () => import('#/views/website/collector/index.vue'),
        meta: {
          title: '内容采集器',
        },
      },
    ],
  },
];

export default routes;
