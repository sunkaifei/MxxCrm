import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'ant-design:account-book-outlined',
      order: 30,
      title: $t('page.finance.title'),
    },
    name: 'Finance',
    path: '/finance',
    children: [
      {
        name: 'CommissionRule',
        path: '/finance/commission-rule',
        component: () => import('#/views/finance/commission-rule/index.vue'),
        meta: {
          title: $t('page.finance.commissionRule.title'),
        },
      },
      {
        name: 'Salary',
        path: '/finance/salary',
        component: () => import('#/views/finance/salary/index.vue'),
        meta: {
          title: $t('page.finance.salary.title'),
        },
      },
      {
        name: 'SalaryDetail',
        path: '/finance/salary/detail/:id',
        component: () => import('#/views/finance/salary/detail.vue'),
        meta: {
          title: $t('page.finance.salary.detail'),
          hideInMenu: true,
          activePath: '/finance/salary',
        },
      },
      {
        name: 'FinancePayment',
        path: '/finance/payment',
        component: () => import('#/views/finance/payment/index.vue'),
        meta: {
          title: $t('page.finance.payment.title'),
        },
      },
    ],
  },
];

export default routes;
