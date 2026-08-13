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
          title: $t('page.finance.salary.detailTitle'),
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
      {
        name: 'FinanceExpense',
        path: '/finance/expense',
        component: () => import('#/views/finance/expense/index.vue'),
        meta: {
          title: $t('page.finance.expense.title'),
        },
      },
      {
        name: 'FinanceExpenseType',
        path: '/finance/expense-type',
        component: () => import('#/views/finance/expense-type/index.vue'),
        meta: {
          title: $t('page.finance.expenseType.title'),
        },
      },
      {
        name: 'FinanceTax',
        path: '/finance/tax',
        component: () => import('#/views/finance/tax/index.vue'),
        meta: {
          title: $t('page.finance.tax.title'),
        },
      },
      {
        name: 'FinanceSocialInsurance',
        path: '/finance/social-insurance',
        component: () => import('#/views/finance/social-insurance/index.vue'),
        meta: {
          title: $t('page.finance.insurance.title'),
        },
      },
      {
        name: 'FinancePayslip',
        path: '/finance/payslip',
        component: () => import('#/views/finance/payslip/index.vue'),
        meta: {
          title: $t('page.finance.payslip.title'),
        },
      },
      {
        name: 'FinanceBankExport',
        path: '/finance/bank-export',
        component: () => import('#/views/finance/bank-export/index.vue'),
        meta: {
          title: $t('page.finance.bankExport.title'),
        },
      },
      {
        name: 'FinanceAttendance',
        path: '/finance/attendance',
        component: () => import('#/views/finance/attendance/index.vue'),
        meta: {
          title: $t('page.finance.attendance.title'),
        },
      },
      {
        name: 'FinanceSalaryItem',
        path: '/finance/salary-item',
        component: () => import('#/views/finance/salary-item/index.vue'),
        meta: {
          title: $t('page.finance.salaryItem.title'),
        },
      },
      {
        name: 'FinanceSalaryAdjustment',
        path: '/finance/salary-adjustment',
        component: () => import('#/views/finance/salary-adjustment/index.vue'),
        meta: {
          title: $t('page.finance.adjustment.title'),
        },
      },
      {
        name: 'FinanceTeamCommission',
        path: '/finance/team-commission',
        component: () => import('#/views/finance/team-commission/index.vue'),
        meta: {
          title: $t('page.finance.teamCommission.title'),
        },
      },
      {
        name: 'FinanceCommissionPool',
        path: '/finance/commission-pool',
        component: () => import('#/views/finance/commission-pool/index.vue'),
        meta: {
          title: $t('page.finance.pool.title'),
        },
      },
    ],
  },
];

export default routes;
