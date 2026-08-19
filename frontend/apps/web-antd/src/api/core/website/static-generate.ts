import { requestClient } from '#/api/request';

export const staticGenerateApi = {
  /** 生成所有静态页面（首页 + 栏目页 + 文章页） */
  generateAll: () => requestClient.post('/api/system/static_generate/all'),

  /** 生成首页静态文件 */
  generateIndex: () => requestClient.post('/api/system/static_generate/index'),

  /** 生成所有栏目页 */
  generateCategories: () =>
    requestClient.post('/api/system/static_generate/categories'),

  /** 生成所有文章页 */
  generateArticles: () =>
    requestClient.post('/api/system/static_generate/articles'),

  /** 清空静态化输出目录 */
  clearOutput: () => requestClient.delete('/api/system/static_generate/clear'),
};
