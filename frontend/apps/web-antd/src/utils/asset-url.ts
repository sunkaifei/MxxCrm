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

/** 附件引用（上传返回的对象或编辑回显的相对路径） */
export type PublicFileRef = {
  /** 附件表主键（上传返回，字符串或数字） */
  id?: number | string | null;
  /** 相对路径（/upload/xxx/...）或绝对地址 */
  url?: string | null;
};

/** 后端静态直出目录（open_routes.rs：/upload/product/、/upload/avatar/） */
const STATIC_DIR_PREFIXES = ['/upload/product/', '/upload/avatar/'];

/**
 * 统一公开附件回显 URL 解析（附件URL统一方案 v1.1 §6.2）
 *
 * 解析优先级：
 * 1. 绝对地址（http/https/data:/协议相对 //）→ 原样返回（外部图或旧数据烘焙域名）
 * 2. 已是 /api/open/file/ 公开接口地址 → 原样返回
 * 3. 上传返回 id → 走 `/api/open/file/{id}`（is_public=1 闸门，三种部署形态都可用）
 * 4. 静态直出目录（product/avatar）→ 拼接资源域名（局域网同源时原样返回）
 * 5. 配置了资源域名 → 域名视为已按 nginx 方案托管 /upload/（含 common/banner）
 * 6. 其余相对路径（common/banner…）→ 走 `/api/open/file/by-path?path=` 反查
 */
export function toPublicFileUrl(
  input?: PublicFileRef | string | null,
  assetDomain?: string,
): string {
  if (!input) return '';
  const id =
    typeof input === 'string' ? '' : String((input as PublicFileRef).id ?? '');
  const url = (typeof input === 'string' ? input : (input as PublicFileRef).url) || '';
  if (!id && !url) return '';
  if (/^(https?:|data:)/i.test(url) || url.startsWith('//')) return url;
  if (url.startsWith('/api/open/file/')) return url;
  if (id) return `/api/open/file/${id}`;
  if (STATIC_DIR_PREFIXES.some((p) => url.startsWith(p))) {
    return resolveAssetUrl(url, assetDomain);
  }
  if (assetDomain) return resolveAssetUrl(url, assetDomain);
  return `/api/open/file/by-path?path=${encodeURIComponent(url)}`;
}
