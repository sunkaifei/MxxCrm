import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:paperclip',
      order: 35,
      title: $t('page.attachment.title'),
    },
    name: 'Attachment',
    path: '/attachment',
    children: [
      {
        name: 'AttachmentFile',
        path: '/attachment/file',
        component: () => import('#/views/attachment/file/index.vue'),
        meta: {
          title: $t('page.attachment.file.title'),
        },
      },
    ],
  },
];

export default routes;
