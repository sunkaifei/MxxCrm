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
    ],
  },
];

export default routes;
