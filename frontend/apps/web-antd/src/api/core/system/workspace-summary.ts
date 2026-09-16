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

/** 数据视角（与项目既有列表页 Tab 口径一致，禁止新造枚举） */
export type WorkspaceScope = 'all' | 'my' | 'subordinate';

/** 统计时间范围（工作台卡片配置项：month=本月 / quarter=本季 / year=本年） */
export type WorkspaceTimeRange = 'month' | 'quarter' | 'year';

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
  /** 本次实际生效的数据视角 */
  scope: WorkspaceScope;
  /** 请求视角是否被成功应用（false = 已静默降级为最大允许范围） */
  scopeApplied: boolean;
  /** 当前用户可用的视角列表（仅 1 项时前端不显示切换器） */
  scopeOptions: WorkspaceScope[];
}

/** 工作台聚合摘要（仅需登录；单卡数据异常时后端已降级为空摘要） */
export const getWorkspaceSummaryApi = async (
  scope?: WorkspaceScope,
  timeRange?: WorkspaceTimeRange,
): Promise<WorkspaceSummaryVO> => {
  const params: Record<string, string> = {};
  if (scope) params.scope = scope;
  if (timeRange) params.timeRange = timeRange;
  return requestClient.get('/api/system/dashboard/workspace/summary', {
    params: Object.keys(params).length > 0 ? params : undefined,
  });
};

// ===== 首屏请求合并（方案 11.4：首屏请求数不随卡片数线性增长） =====
// 8 张三期卡挂载时并发调用 getWorkspaceSummaryShared，TTL 内共享同一份在途请求，
// 仅发出一次 HTTP；reload 传 force=true 强制刷新，失败不缓存以便下次重试
//
// v2.1：合并键需带上 scope —— 不同视角必须发不同请求，否则切视角会读到旧缓存
//
// 卡片配置化：合并键同时带上 timeRange（切时间范围即拉对应统计数据）
const SUMMARY_TTL = 30_000;
const summaryCache = new Map<string, { at: number; promise: Promise<WorkspaceSummaryVO> }>();

export async function getWorkspaceSummaryShared(
  force = false,
  scope?: WorkspaceScope,
  timeRange?: WorkspaceTimeRange,
): Promise<WorkspaceSummaryVO> {
  const key = `${scope || 'default'}|${timeRange || 'month'}`;
  const now = Date.now();
  const hit = summaryCache.get(key);
  if (!force && hit && now - hit.at < SUMMARY_TTL) {
    return hit.promise;
  }
  const promise: Promise<WorkspaceSummaryVO> = getWorkspaceSummaryApi(scope, timeRange).catch(
    (error) => {
      // 失败不缓存，便于下次重试
      if (summaryCache.get(key)?.promise === promise) {
        summaryCache.delete(key);
      }
      throw error;
    },
  );
  summaryCache.set(key, { at: now, promise });
  return promise;
}

/** 视角切换后清空全部合并缓存（各卡下次挂载即拉新视角数据） */
export function clearWorkspaceSummaryCache() {
  summaryCache.clear();
}
