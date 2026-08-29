import { requestClient } from '#/api/request';

/**
 * 工作台聚合摘要 API（方案 6 节 #4 / 9.5 camelCase 契约）
 *
 * GET /api/system/dashboard/workspace/summary 一次返回 8 张三期卡片摘要
 * （total + top 6 items），首屏请求数不随卡片数线性增长；仅需登录，
 * 不可见卡片由后端返回空摘要兜底（手册硬性规则：数据兜底，禁止报错抛出）。
 * 金额/数量字段后端 Decimal 序列化为字符串，前端统一 Number() 后展示。
 */

export interface AnnouncementItem {
  id?: null | number;
  isRead?: null | number;
  level?: null | string;
  publishName?: null | string;
  publishTime?: null | string;
  title?: null | string;
}

export interface AnnouncementSummary {
  items: AnnouncementItem[];
  total: number;
  unread: number;
}

export interface HrTodoItem {
  businessTitle?: null | string;
  businessType: string;
  id: number;
  submitterName?: null | string;
  submittedAt?: null | string;
}

export interface HrTodoSummary {
  items: HrTodoItem[];
  onboarding: number;
  resign: number;
  total: number;
}

export interface PaymentReminderItem {
  contractId?: null | number;
  id?: null | number;
  planAmount?: null | number | string;
  planDate?: null | string;
  receivedAmount?: null | number | string;
  stageName?: null | string;
  status?: null | number;
}

export interface PaymentReminderSummary {
  items: PaymentReminderItem[];
  total: number;
}

export interface PayslipStatSummary {
  readCount: number;
  sentCount: number;
  totalCount: number;
  unreadCount: number;
}

export interface PurchaseApprovalItem {
  createTime?: null | string;
  id?: null | number;
  prNo?: null | string;
  status?: null | number;
  title?: null | string;
  totalAmount?: null | number | string;
  urgency?: null | string;
}

export interface PurchaseApprovalSummary {
  items: PurchaseApprovalItem[];
  total: number;
}

export interface SalesPerformanceSummary {
  dealAmount: number | string;
  followUps: number;
  newCustomers: number;
  newOpportunities: number;
}

export interface StockAlertItem {
  alertMaxQuantity?: null | number | string;
  alertMinQuantity?: null | number | string;
  alertType?: null | string;
  productName?: null | string;
  quantity?: null | number | string;
  warehouseName?: null | string;
}

export interface StockAlertSummary {
  items: StockAlertItem[];
  total: number;
}

export interface StockDocTodoItem {
  bizType?: null | string;
  createdByName?: null | string;
  createTime?: null | string;
  id?: null | number;
  orderNo?: null | string;
  status?: null | number;
  totalAmount?: null | number | string;
  totalQuantity?: null | number | string;
}

export interface StockDocTodoSummary {
  inboundTotal: number;
  items: StockDocTodoItem[];
  outboundTotal: number;
}

/** 工作台聚合摘要（对齐后端 WorkspaceSummaryVO） */
export interface WorkspaceSummaryVO {
  announcement: AnnouncementSummary;
  hrTodo: HrTodoSummary;
  paymentReminder: PaymentReminderSummary;
  payslipStat: PayslipStatSummary;
  purchaseApproval: PurchaseApprovalSummary;
  salesPerformance: SalesPerformanceSummary;
  stockAlert: StockAlertSummary;
  stockDocTodo: StockDocTodoSummary;
}

/** 工作台聚合摘要（仅需登录；单卡数据异常时后端已降级为空摘要） */
export const getWorkspaceSummaryApi = async (): Promise<WorkspaceSummaryVO> => {
  return requestClient.get('/api/system/dashboard/workspace/summary');
};

// ===== 首屏请求合并（方案 11.4：首屏请求数不随卡片数线性增长） =====
// 8 张三期卡挂载时并发调用 getWorkspaceSummaryShared，TTL 内共享同一份在途请求，
// 仅发出一次 HTTP；reload 传 force=true 强制刷新，失败不缓存以便下次重试
const SUMMARY_TTL = 30_000;
let summaryAt = 0;
let summaryPromise: null | Promise<WorkspaceSummaryVO> = null;

export async function getWorkspaceSummaryShared(
  force = false,
): Promise<WorkspaceSummaryVO> {
  const now = Date.now();
  if (!force && summaryPromise && now - summaryAt < SUMMARY_TTL) {
    return summaryPromise;
  }
  summaryAt = now;
  const promise: Promise<WorkspaceSummaryVO> = getWorkspaceSummaryApi().catch(
    (error) => {
      if (summaryPromise === promise) {
        summaryAt = 0;
        summaryPromise = null;
      }
      throw error;
    },
  );
  summaryPromise = promise;
  return promise;
}
