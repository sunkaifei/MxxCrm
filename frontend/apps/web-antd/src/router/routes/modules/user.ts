import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:users',
      keepAlive: true,
      order: 30,
      title: '用户管理',
    },
    name: 'UserManage',
    path: '/user',
    children: [
      {
        meta: {
          title: '用户列表',
        },
        name: 'UserList',
        path: '/user/list',
        component: () => import('#/views/user/index.vue'),
      },
    ],
  },
];

export default routes;
