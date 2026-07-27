import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:bar-chart-4',
      order: 25,
      title: $t('page.report.title'),
    },
    name: 'Report',
    path: '/report',
    children: [],
  },
];

export default routes;
