/**
 * 全局资源访问域名（附件URL统一方案 v1.1 §6.2）
 *
 * 模块级缓存：首次调用拉取 /api/system/site/current，之后所有页面共享；
 * 设置页保存 assetDomain 后调用 refreshAssetDomain() 刷新缓存。
 */
import { ref } from 'vue';

import { siteApi } from '#/api/core/website/site';

const assetDomain = ref('');
let loaded = false;
let loading: null | Promise<void> = null;

async function fetchOnce() {
  if (loaded) return;
  if (loading) return loading;
  loading = (async () => {
    try {
      const site = await siteApi.getCurrent();
      assetDomain.value = (site as any)?.assetDomain ?? '';
      loaded = true;
    } catch {
      // 静默失败：保持空串（同源相对路径），下次进入页面重试
    } finally {
      loading = null;
    }
  })();
  return loading;
}

export function useAssetDomain() {
  fetchOnce();
  return { assetDomain, refreshAssetDomain: refresh };
}

async function refresh() {
  loaded = false;
  await fetchOnce();
}
