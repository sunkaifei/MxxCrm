import { requestClient } from '#/api/request';

// ============== 类型定义 ==============

export interface SegmentConfig {
  type:
    | 'company'
    | 'biz_type'
    | 'year'
    | 'dept'
    | 'seq'
    | 'version'
    | 'fixed'
    | 'date';
  value?: string;
  format?: string; // type=year: yyyy/yy; type=date: yyyyMM/yyyydd
  source?: 'current' | 'business_date' | 'create_time'; // type=year 时使用
  length?: number; // type=seq 时使用
  sort: number;
}

export interface CodeRuleVO {
  id: string;
  moduleCode: string;
  moduleName: string;
  ruleName?: string;
  companyAbbr?: string;
  bizTypeCode?: string;
  separator?: string;
  segments: SegmentConfig[];
  seqLength?: number;
  enabled?: number;
  remark?: string;
  createTime?: string;
  updateTime?: string;
}

export interface CodeRuleSaveReq {
  id?: number;
  moduleCode: string;
  moduleName: string;
  ruleName?: string;
  bizTypeCode?: string;
  separator?: string;
  segments: SegmentConfig[];
  seqLength?: number;
  enabled?: number;
  remark?: string;
}

export interface PreviewCodeReq {
  moduleCode?: string;
  segments: SegmentConfig[];
  companyAbbr?: string;
  bizTypeCode?: string;
  separator?: string;
  seqLength?: number;
  deptCode?: string;
  businessDate?: string;
  mockSeq?: boolean;
}

export interface GenerateCodeReq {
  moduleCode: string;
  deptCode?: string;
  businessDate?: string;
}

export interface BatchRegenerateReq {
  moduleCodes: string[];
  years?: number[];
}

export interface BatchRegenerateProgressVO {
  total: number;
  done: number;
  currentModule?: string;
  status: 'running' | 'success' | 'failed' | '';
  message?: string;
}

// ============== API 接口 ==============

// 分页列表
export const getCodeRuleListApi = async (params: any) => {
  return requestClient.get('/api/system/code-rule/list', { params });
};

// 详情
export const getCodeRuleInfoApi = async (id: number) => {
  return requestClient.get(`/api/system/code-rule/info/${id}`);
};

// 新增
export const createCodeRuleApi = async (data: CodeRuleSaveReq) => {
  return requestClient.post('/api/system/code-rule', data);
};

// 修改
export const updateCodeRuleApi = async (id: number, data: CodeRuleSaveReq) => {
  return requestClient.put(`/api/system/code-rule/${id}`, data);
};

// 删除
export const deleteCodeRuleApi = async (id: number) => {
  return requestClient.delete(`/api/system/code-rule/${id}`);
};

// 启用/停用
export const toggleCodeRuleEnabledApi = async (
  id: number,
  enabled: number,
) => {
  return requestClient.put(
    `/api/system/code-rule/toggle/${id}/${enabled}`,
    {},
  );
};

// 预览编号
export const previewCodeApi = async (data: PreviewCodeReq) => {
  return requestClient.post('/api/system/code-rule/preview', data);
};

// 生成编号
export const generateCodeApi = async (data: GenerateCodeReq) => {
  return requestClient.post('/api/system/code-rule/generate', data);
};

// 一键更新
export const batchRegenerateApi = async (data: BatchRegenerateReq) => {
  return requestClient.post('/api/system/code-rule/batch-regenerate', data);
};

// 一键更新进度
export const getBatchRegenerateProgressApi = async () => {
  return requestClient.get('/api/system/code-rule/batch-regenerate/progress');
};
