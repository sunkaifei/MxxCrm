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
