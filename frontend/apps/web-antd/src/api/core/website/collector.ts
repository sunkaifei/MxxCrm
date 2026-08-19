import { requestClient } from '#/api/request';

export const collectorApi = {
  /** 执行所有启用的采集规则 */
  run: () => requestClient.post('/api/system/content_collector/run'),
};
