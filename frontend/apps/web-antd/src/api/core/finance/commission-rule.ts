import { requestClient } from '#/api/request';

// 提成规则列表
export const getCommissionRuleListApi = async (params?: any) => {
  return requestClient.get('/api/system/finance/commission-rule/list', {
    params,
  });
};

// 提成规则详情
export const getCommissionRuleDetailApi = async (id: number) => {
  return requestClient.get('/api/system/finance/commission-rule/detail', {
    params: { id },
  });
};

// 保存提成规则（新建/编辑）
export const saveCommissionRuleApi = async (data: any) => {
  return requestClient.post('/api/system/finance/commission-rule/save', data);
};

// 删除提成规则
export const deleteCommissionRuleApi = async (id: number) => {
  return requestClient.post('/api/system/finance/commission-rule/delete', {
    id,
  });
};

// 启用/禁用
export const toggleCommissionRuleApi = async (id: number) => {
  return requestClient.post('/api/system/finance/commission-rule/toggle', {
    id,
  });
};
