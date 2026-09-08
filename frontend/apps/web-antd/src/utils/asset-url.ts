/**
 * 统一附件/图片 URL 解析（附件URL与访问域名统一方案 v1.1 §6.1）
 *
 * - 已带协议（http/https/data:/协议相对 //）→ 原样返回
 * - assetDomain 非空 → 拼接为 {assetDomain}{path}（自动处理斜杠）
 * - 否则返回相对路径（同源访问，局域网直启场景）
 *
 * assetDomain 来源：/api/system/site/current 返回的 assetDomain（网站设置「资源访问域名」）。
 */
export function resolveAssetUrl(url?: string, assetDomain?: string): string {
  if (!url) return '';
  if (/^(https?:|data:)/i.test(url) || url.startsWith('//')) return url;
  const domain = (assetDomain || '').trim().replace(/\/+$/, '');
  if (domain) {
    const p = url.startsWith('/') ? url : `/${url}`;
    return `${domain}${p}`;
  }
  return url;
}

/**
 * 规范化用户输入的资源访问域名（设置页 onBlur 用）
 * 去首尾空格 + 去尾斜杠；空值返回空串（表示同源）
 */
export function normalizeAssetDomain(input?: string): string {
  return (input || '').trim().replace(/\/+$/, '');
}
