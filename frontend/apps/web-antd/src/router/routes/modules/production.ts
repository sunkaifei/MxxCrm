import type { RouteRecordRaw } from 'vue-router';
import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:factory',
      order: 27,
      title: $t('page.production.title'),
    },
    name: 'Production',
    path: '/production',
    children: [
      {
        name: 'ProductionPlan',
        path: '/production/plan',
        component: () => import('#/views/production/plan/index.vue'),
        meta: {
          title: $t('page.production.plan.title'),
        },
      },
      {
        name: 'ProductionOrder',
        path: '/production/order',
        component: () => import('#/views/production/order/index.vue'),
        meta: {
          title: $t('page.production.order.title'),
        },
      },
    ],
  },
];
export default routes;