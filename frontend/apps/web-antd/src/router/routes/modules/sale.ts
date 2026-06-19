import type { RouteRecordRaw } from 'vue-router';

import { $t } from '#/locales';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:shopping-bag',
      order: 15,
      title: $t('page.sale.title'),
    },
    name: 'Sale',
    path: '/sale',
    children: [
      {
        name: 'SaleOrder',
        path: '/sale/order',
        component: () => import('#/views/sale/order/index.vue'),
        meta: {
          title: $t('page.sale.order.title'),
        },
      },
      {
        name: 'SaleOrderItem',
        path: '/sale/order-item',
        component: () => import('#/views/sale/orderItem/index.vue'),
        meta: {
          title: $t('page.sale.orderItem.title'),
        },
      },
      {
        name: 'SalePayment',
        path: '/sale/payment',
        component: () => import('#/views/sale/payment/index.vue'),
        meta: {
          title: $t('page.sale.payment.title'),
        },
      },
    ],
  },
];

export default routes;
