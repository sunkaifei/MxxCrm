import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:contact',
      order: 10,
      title: $t('page.crm.title'),
    },
    name: 'Crm',
    path: '/crm',
    children: [
      {
        name: 'CrmLeadPool',
        path: '/crm/lead-pool',
        component: () => import('#/views/crm/lead-pool/index.vue'),
        meta: {
          title: $t('page.crm.leadPool.title'),
        },
      },
      {
        name: 'CrmLead',
        path: '/crm/lead',
        component: () => import('#/views/crm/lead/index.vue'),
        meta: {
          title: $t('page.crm.lead.title'),
        },
      },
      {
        name: 'CrmCustomer',
        path: '/crm/customer',
        component: () => import('#/views/crm/customer/index.vue'),
        meta: {
          title: $t('page.crm.customer.title'),
        },
      },
      {
        name: 'CrmContact',
        path: '/crm/contact',
        component: () => import('#/views/crm/contact/index.vue'),
        meta: {
          title: $t('page.crm.contact.title'),
        },
      },
      {
        name: 'CrmFollowup',
        path: '/crm/followup',
        component: () => import('#/views/crm/followup/index.vue'),
        meta: {
          title: $t('page.crm.followup.title'),
        },
      },
    ],
  },
];

export default routes;
