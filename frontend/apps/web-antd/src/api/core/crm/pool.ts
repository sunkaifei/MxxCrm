import { requestClient } from '#/api/request';

type PageParams = Record<string, any>;

/** 公海池配置 VO（v3.0 公海优化） */
export interface PoolConfig {
  /** 自动分配开关（1=开 2=关） */
  autoAssignEnabled?: number;
  /** 自动分配模式（1=轮询 2=负载均衡/最少持有） */
  autoAssignMode?: number;
  /** 领取冷却天数 */
  coolDownDays?: number;
  /** 领取模式（1=自主领取 2=申领需审批 3=停用领取仅分配） */
  claimMode?: number;
  /** 单人每日领取上限 */
  claimDailyLimit?: number;
  /** 单人最大保有量 */
  holdLimit?: number;
  /** 脱敏字段数组（空=不脱敏） */
  maskFields?: string[];
  /** 池ID */
  poolId?: string;
  /** 回收规则（1=按未跟进天数 2=按创建天数） */
  recycleDays?: number;
  /** 到期前提醒天数 */
  reminderDays?: number;
  updateTime?: string;
}

/** 公海池列表 VO */
export interface PoolListVO {
  config?: null | PoolConfig;
  createTime?: string;
  description?: string;
  id?: string;
  /** 默认池标识（1=默认池） */
  isDefault?: number;
  /** 在池线索数（未分配） */
  leadCount?: number;
  /** 成员数 */
  memberCount?: number;
  name?: string;
  sort?: number;
  /** 状态（1=启用 2=停用） */
  status?: number;
  updateTime?: string;
}

/** 池基础信息保存请求（新建/更新共用） */
export interface PoolSaveRequest {
  description?: string;
  /** 更新时必传 */
  id?: number | string;
  name?: string;
  sort?: number;
}

/** 池配置保存请求 */
export interface PoolConfigSaveRequest {
  autoAssignEnabled?: number;
  autoAssignMode?: number;
  claimDailyLimit?: number;
  claimMode?: number;
  coolDownDays?: number;
  holdLimit?: number;
  maskFields?: string[];
  poolId?: number | string;
  recycleDays?: number;
  reminderDays?: number;
}

/** 池成员 VO */
export interface PoolMemberVO {
  createTime?: string;
  id?: string;
  /** 成员类型（1=池管理员 2=普通成员） */
  memberType?: number;
  poolId?: string;
  userId?: string;
  userName?: string;
}

/** 可选成员简要 VO（指派/自动分配候选） */
export interface PoolMemberSimpleVO {
  /** 当前私海线索数（负载均衡/保有量展示） */
  holdingCount?: number;
  memberType?: number;
  userId?: string;
  userName?: string;
}

/** 我的可见池 VO（公海工作台 Tab 用） */
export interface MyPoolVO {
  autoAssignEnabled?: number;
  claimMode?: number;
  id?: string;
  isDefault?: number;
  isManager?: boolean;
  leadCount?: number;
  maskFields?: string[];
  name?: string;
  status?: number;
}

/** 分页查询池列表 */
export const getPoolPageApi = async (params?: PageParams) => {
  return requestClient.get('/api/system/pool/page', { params });
};

/** 池详情（含配置） */
export const getPoolInfoApi = async (poolId: string | number) => {
  return requestClient.get('/api/system/pool/info', {
    params: { poolId },
  });
};

/** 新建池 */
export const createPoolApi = async (data: PoolSaveRequest) => {
  return requestClient.post('/api/system/pool/save', data);
};

/** 更新池基础信息 */
export const updatePoolApi = async (data: PoolSaveRequest) => {
  return requestClient.post('/api/system/pool/update', data);
};

/** 删除池（批量） */
export const deletePoolApi = async (ids: (number | string)[]) => {
  return requestClient.post('/api/system/pool/delete', { ids });
};

/** 启用/停用池 */
export const updatePoolStatusApi = async (
  id: string | number,
  status: number,
) => {
  return requestClient.post('/api/system/pool/status', null, {
    params: { id, status },
  });
};

/** 我的可见池（我加入的 ∪ 我管理的） */
export const getMyPoolsApi = async () => {
  return requestClient.get('/api/system/pool/my');
};

/** 池配置详情 */
export const getPoolConfigApi = async (poolId: string | number) => {
  return requestClient.get('/api/system/pool/config', {
    params: { poolId },
  });
};

/** 保存池配置 */
export const savePoolConfigApi = async (data: PoolConfigSaveRequest) => {
  return requestClient.post('/api/system/pool/config/save', data);
};

/** 池成员列表 */
export const getPoolMemberListApi = async (poolId: string | number) => {
  return requestClient.get('/api/system/pool/member/list', {
    params: { poolId },
  });
};

/** 批量添加池成员 */
export const savePoolMembersApi = async (
  poolId: string | number,
  members: { memberType: number; userId: number | string }[],
) => {
  return requestClient.post('/api/system/pool/member/save', {
    members,
    poolId,
  });
};

/** 批量移除池成员 */
export const deletePoolMembersApi = async (
  poolId: string | number,
  userIds: (number | string)[],
) => {
  return requestClient.post('/api/system/pool/member/delete', {
    poolId,
    userIds,
  });
};

/** 指派/自动分配候选成员（该池全部成员） */
export const getPoolMemberCandidatesApi = async (poolId: string | number) => {
  return requestClient.get('/api/system/pool/member/candidates', {
    params: { poolId },
  });
};
