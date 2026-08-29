import { requestClient } from '#/api/request';

/**
 * 入职引导聚合配置（方案 4.3 / 13.2-1）
 * 仅需登录；管理员（system:onboarding:list）额外返回 admin 字段（version/steps 全量）
 */
export const getOnboardingConfigApi = async () => {
  return requestClient.get('/api/system/onboarding/config');
};

/**
 * 管理员保存步骤与配置（全量覆盖式，方案 4.3 / 13.3-3）
 * body: { steps: [...], config: {...}, version }；version 不匹配返回 409
 */
export const saveOnboardingConfigApi = async (data: {
  config: {
    announceEnabled: number;
    quickEnabled: number;
    quickPreset?: any;
    todoEnabled: number;
  };
  steps: any[];
  version: number;
}) => {
  return requestClient.post('/api/system/onboarding/save', data);
};
