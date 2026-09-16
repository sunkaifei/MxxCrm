import { addCollection } from '@vben/icons';

/**
 * 图标集预加载
 *
 * 这里加载的是由 `scripts/gen-lucide-subset.mjs` 从源码中扫描出的**图标子集**
 * （约 42 KB），而不是完整的 lucide 图标集（约 516 KB）。
 *
 * 原因：完整集会被打成 545 KB 的独立 chunk，并位于 createApp 之前被 await，
 * 直接占用首屏关键路径。子集化后该开销降为原来的 ~8%，几乎无感，
 * 同时仍保持「纯本地、不依赖远程 iconify API」的原意。
 *
 * 新增图标后需要重新生成子集：`pnpm gen:icons`（构建时会自动执行）。
 */
async function registerLucideIcons(): Promise<void> {
  try {
    const mod = await import('./lucide-subset.json');
    const data = (mod as any).default || mod;
    addCollection(data);
  } catch (error) {
    console.warn('Failed to preload lucide icons:', error);
  }
}

export { registerLucideIcons };
