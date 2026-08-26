import { requestClient } from '#/api/request';

// ============ Flow Management ============

export const getApprovalFlowListApi = async (params?: any) =>
  requestClient.get('/api/system/approval/flow/list', { params });

export const getApprovalFlowDetailApi = async (id: number) =>
  requestClient.get(`/api/system/approval/flow/detail/${id}`);

/** 按流程编码预览流程（B2：提交审核前展示流程图，仅登录鉴权） */
export const getApprovalFlowPreviewApi = async (code: string) =>
  requestClient.get(`/api/system/approval/flow/preview/${code}`);

export const saveApprovalFlowApi = async (data: any) =>
  requestClient.post('/api/system/approval/flow/save', data);

export const toggleApprovalFlowApi = async (id: number) =>
  requestClient.post(`/api/system/approval/flow/toggle/${id}`);

export const deleteApprovalFlowApi = async (id: number) =>
  requestClient.post(`/api/system/approval/flow/delete/${id}`);

// ============ Approval Instance ============

export const submitApprovalApi = async (data: any) =>
  requestClient.post('/api/system/approval/submit', data);

export const processApprovalApi = async (data: any) =>
  requestClient.post('/api/system/approval/process', data);

export const getApprovalDetailApi = async (id: number) =>
  requestClient.get(`/api/system/approval/detail/${id}`);

export const getApprovalListApi = async (params?: any) =>
  requestClient.get('/api/system/approval/list', { params });

// ============ 审批增强：取消/退回/转办/委派/加签/抄送 ============

/** 发起人撤回审批 */
export const cancelApprovalApi = async (data: {
  cancelReason?: string;
  instanceId: number;
}) => requestClient.post('/api/system/approval/cancel', data);

/** 退回（退回到发起人修改 或 指定节点） */
export const rejectToApprovalApi = async (data: {
  comment?: string;
  instanceId: number;
  rejectToNodeKey?: string;
}) => requestClient.post('/api/system/approval/reject-to', data);

/** 转办（当前审批人转给他人，责任转移） */
export const transferApprovalApi = async (data: {
  comment?: string;
  instanceId: number;
  targetUserId: number;
  targetUserName?: string;
}) => requestClient.post('/api/system/approval/transfer', data);

/** 委派（委托他人处理，责任仍归原审批人） */
export const delegateApprovalApi = async (data: {
  comment?: string;
  instanceId: number;
  targetUserId: number;
  targetUserName?: string;
}) => requestClient.post('/api/system/approval/delegate', data);

/** 加签（1=前加签,2=后加签,3=并加签） */
export const addSignApprovalApi = async (data: {
  addSignType: number;
  comment?: string;
  instanceId: number;
  targetUserIds: number[];
}) => requestClient.post('/api/system/approval/add-sign', data);

/** 添加抄送 */
export const addCcApprovalApi = async (data: {
  ccReason?: string;
  instanceId: number;
  userIds: number[];
}) => requestClient.post('/api/system/approval/cc/add', data);

/** 我的抄送列表 */
export const getCcListApi = async (params?: any) =>
  requestClient.get('/api/system/approval/cc/list', { params });

/** 标记抄送已读 */
export const markCcReadApi = async (id: number) =>
  requestClient.post(`/api/system/approval/cc/read/${id}`);

/**
 * 按「连线拓扑」排序流程节点（从开始节点沿边遍历），
 * 与审批引擎实际执行顺序 / 设计器画布连线保持一致。
 *
 * 背景：设计器插入新节点后 node_order 不会重排，导致按 nodeOrder 排序
 * 的展示与真实流程错位（如「部门经理审批」被排到最后）。
 *
 * 兼容两种数据结构：
 * - 流程预览接口（FlowDetailVO，camelCase：nodeKey/nodeType + edges.source/target）
 * - 审批实例快照（snake_case：node_key/node_type + edges.source_node_key/target_node_key）
 */
export function sortApprovalNodes(nodes: any[] = [], edges: any[] = []): any[] {
  if (!nodes.length) return [];
  const pick = (obj: any, camel: string, snake: string) =>
    obj?.[camel] ?? obj?.[snake];
  const nodeKey = (n: any) => pick(n, 'nodeKey', 'node_key');
  const nodeType = (n: any) => pick(n, 'nodeType', 'node_type');
  const sourceOf = (e: any) => pick(e, 'source', 'source_node_key');
  const targetOf = (e: any) => pick(e, 'target', 'target_node_key');

  const start = nodes.find((n) => nodeType(n) === 1);
  const sorted: any[] = [];
  const visited = new Set<string>();

  const walk = (n: any) => {
    const key = nodeKey(n);
    if (key === undefined || visited.has(key)) return;
    visited.add(key);
    sorted.push(n);
    for (const e of edges) {
      if (sourceOf(e) === key) {
        const next = nodes.find((nn) => nodeKey(nn) === targetOf(e));
        if (next) walk(next);
      }
    }
  };

  if (start) walk(start);
  // 兜底：游离/未连通节点按原顺序追加，避免丢失
  for (const n of nodes) {
    const key = nodeKey(n);
    if (key !== undefined && !visited.has(key)) walk(n);
  }
  return sorted;
}
