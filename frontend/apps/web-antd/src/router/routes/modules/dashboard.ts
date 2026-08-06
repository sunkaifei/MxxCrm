import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:layout-dashboard',
      order: -1,
      title: $t('page.dashboard.title'),
    },
    name: 'Dashboard',
    path: '/dashboard',
    children: [
      {
        name: 'Workspace',
        path: '/workspace',
        component: () => import('#/views/dashboard/workspace/index.vue'),
        meta: {
          icon: 'carbon:workspace',
          title: $t('page.dashboard.workspace'),
        },
      },
      {
        name: 'statistics-performance',
        path: '/dashboard/performance',
        component: () => import('#/views/statistics/performance/index.vue'),
        meta: {
          icon: 'lucide:target',
          title: $t('page.statistics.performanceOverview'),
        },
      },
      {
        name: 'statistics-customer',
        path: '/dashboard/customer',
        component: () => import('#/views/statistics/customer/index.vue'),
        meta: {
          icon: 'lucide:users',
          title: $t('page.statistics.customerAnalysis'),
        },
      },
      {
        name: 'statistics-contract',
        path: '/dashboard/contract',
        component: () => import('#/views/statistics/contract/index.vue'),
        meta: {
          icon: 'lucide:file-text',
          title: $t('page.statistics.contract'),
        },
      },
      {
        name: 'statistics-payment',
        path: '/dashboard/payment',
        component: () => import('#/views/statistics/payment/index.vue'),
        meta: {
          icon: 'lucide:wallet',
          title: $t('page.statistics.payment'),
        },
      },
      {
        name: 'statistics-employee',
        path: '/dashboard/employee',
        component: () => import('#/views/statistics/employee/index.vue'),
        meta: {
          icon: 'lucide:user-check',
          title: $t('page.statistics.employeeComparison'),
        },
      },
    ],
  },
];

export default routes;
