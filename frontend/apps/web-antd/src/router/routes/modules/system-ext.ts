import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:building-2',
      keepAlive: true,
      order: 31,
      title: '系统管理',
    },
    name: 'SystemExt',
    path: '/system-ext',
    children: [
      {
        name: 'SystemDept',
        path: '/system-ext/dept',
        component: () => import('#/views/system/dept/index.vue'),
        meta: {
          icon: 'lucide:building-2',
          title: $t('page.system.dept.title'),
        },
      },
      {
        name: 'SystemPost',
        path: '/system-ext/post',
        component: () => import('#/views/system/post/index.vue'),
        meta: {
          icon: 'lucide:briefcase',
          title: $t('page.system.post.title'),
        },
      },
      {
        name: 'SystemConfig',
        path: '/system-ext/config',
        component: () => import('#/views/system/config/index.vue'),
        meta: {
          icon: 'lucide:settings',
          title: $t('page.system.config.title'),
        },
      },
      {
        name: 'SystemNotice',
        path: '/system-ext/notice',
        component: () => import('#/views/system/notice/index.vue'),
        meta: {
          icon: 'lucide:bell',
          title: $t('page.system.notice.title'),
        },
      },
      {
        name: 'SystemLog',
        path: '/system-ext/log',
        component: () => import('#/views/system/log/index.vue'),
        meta: {
          icon: 'lucide:scroll-text',
          title: $t('page.system.log.title'),
        },
      },
      {
        name: 'SystemTag',
        path: '/system-ext/tag',
        component: () => import('#/views/system/tag/index.vue'),
        meta: {
          icon: 'lucide:tag',
          title: $t('page.system.tag.title'),
        },
      },
    ],
  },
];

export default routes;
