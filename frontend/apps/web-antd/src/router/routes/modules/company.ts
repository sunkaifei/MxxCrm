import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:building-2',
      keepAlive: true,
      order: 35,
      title: '公司',
    },
    name: 'Company',
    path: '/company',
    children: [
      {
        meta: {
          icon: 'lucide:info',
          title: '企业信息',
        },
        name: 'CompanyInfo',
        path: '/company/info',
        component: () => import('#/views/company/info/index.vue'),
      },
      {
        meta: {
          icon: 'lucide:hash',
          title: '编号规则配置',
        },
        name: 'CompanyCodeRule',
        path: '/company/code-rule',
        component: () => import('#/views/company/code-rule/index.vue'),
      },
      {
        meta: {
          icon: 'lucide:bot',
          title: 'AI设置',
        },
        name: 'CompanyAiSettings',
        path: '/company/ai-settings',
        component: () => import('#/views/crm/ai-settings/index.vue'),
      },
      {
        meta: {
          icon: 'lucide:workflow',
          title: '销售流程',
        },
        name: 'CompanySalesFlow',
        path: '/company/sales-flow',
        component: () => import('#/views/company/sales-flow/index.vue'),
      },
    ],
  },
];

export default routes;
