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

// 获取启用的规则列表（下拉选项）
export const getCommissionRuleOptionsApi = async () => {
  return requestClient.get('/api/system/finance/commission-rule/options');
};

// 设置默认方案
export const setCommissionDefaultApi = async (id: number) => {
  return requestClient.post('/api/system/finance/commission-rule/set-default', {
    id,
  });
};

// 获取默认方案
export const getCommissionDefaultApi = async () => {
  return requestClient.get('/api/system/finance/commission-rule/default');
};

// 预览合同提成
// 注意：使用 /contract/commission/preview（只需 crm:contract:list 权限），
// 而非 /finance/commission/preview（需要 finance:commission:manage 权限）
export const previewCommissionApi = async (contractId: number) => {
  return requestClient.post('/api/system/contract/commission/preview', {
    id: contractId,
  });
};

// 月度结算
export const monthlySettleCommissionApi = async (
  year: number,
  month: number,
) => {
  return requestClient.post('/api/system/finance/commission/monthly-settle', {
    year,
    month,
  });
};
