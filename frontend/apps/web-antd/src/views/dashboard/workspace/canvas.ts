/**
 * 工作台画布卡片注册表（方案 5.3-M1：卡片组件声明 min_h/max_h；M2 设计器复用）
 *
 * - code 与后端 mxx_system_dashboard_card.card_code 一致（d30 种子数据，page_key='default'）
 * - minW/maxW/minH/maxH 为 gridstack 栅格约束（单位 = 12 列列宽 / cellHeight 行高）
 * - 卡片渲染参数与事件由 index.vue 按 code 动态注入（v-bind / v-on），注册表保持纯净
 */
import type { Component } from 'vue';

import TodoOverviewCard from '../components/TodoOverviewCard.vue';
import AnnouncementCard from './components/AnnouncementCard.vue';
import CalendarCard from './components/CalendarCard.vue';
import HrTodoCard from './components/HrTodoCard.vue';
import OnboardingCard from './components/OnboardingCard.vue';
import PaymentReminderCard from './components/PaymentReminderCard.vue';
import PayslipStatCard from './components/PayslipStatCard.vue';
import PurchaseApprovalCard from './components/PurchaseApprovalCard.vue';
import SalesPerformanceCard from './components/SalesPerformanceCard.vue';
import SmartTodoCard from './components/SmartTodoCard.vue';
import StockAlertCard from './components/StockAlertCard.vue';
import StockDocTodoCard from './components/StockDocTodoCard.vue';
import WeekLoadCard from './components/WeekLoadCard.vue';

export interface WorkspaceCanvasCardDef {
  code: string;
  component: Component;
  maxH: number;
  maxW: number;
  minH: number;
  minW: number;
}

export const WORKSPACE_CANVAS_CARDS: WorkspaceCanvasCardDef[] = [
  {
    code: 'workspace_onboarding',
    component: OnboardingCard,
    maxH: 16,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_todo_overview',
    component: TodoOverviewCard,
    maxH: 14,
    maxW: 12,
    minH: 5,
    minW: 4,
  },
  {
    code: 'workspace_smart_todo',
    component: SmartTodoCard,
    maxH: 20,
    maxW: 12,
    minH: 6,
    minW: 3,
  },
  {
    code: 'workspace_week_load',
    component: WeekLoadCard,
    maxH: 20,
    maxW: 12,
    minH: 6,
    minW: 3,
  },
  {
    code: 'workspace_calendar',
    component: CalendarCard,
    maxH: 24,
    maxW: 12,
    // minH 6：卡内日历可滚动（组件 body overflowY auto），允许用户压低高度
    minH: 6,
    minW: 3,
  },
  // 三期 8 张岗位卡（d31 种子 sort 40-110，minH 按方案 9.1：公告 4、其余 6）
  {
    code: 'workspace_announcement',
    component: AnnouncementCard,
    maxH: 12,
    maxW: 12,
    minH: 4,
    minW: 4,
  },
  {
    code: 'workspace_stock_alert',
    component: StockAlertCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_stock_doc_todo',
    component: StockDocTodoCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_purchase_approval',
    component: PurchaseApprovalCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_payment_reminder',
    component: PaymentReminderCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_payslip_stat',
    component: PayslipStatCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_hr_todo',
    component: HrTodoCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
  {
    code: 'workspace_sales_performance',
    component: SalesPerformanceCard,
    maxH: 12,
    maxW: 12,
    minH: 6,
    minW: 4,
  },
];

export function getCanvasCardDef(
  code: string,
): WorkspaceCanvasCardDef | undefined {
  return WORKSPACE_CANVAS_CARDS.find((c) => c.code === code);
}

export type { Component };
