import type { UserConfig } from 'vite';

/**
 * ⚠️ 改动本文件的任何配置前请先读这段说明
 *
 * 本包 package.json 的 `main` / `module` 指向 **`./dist/index.mjs`（预构建产物）**，
 * 而不是 src。也就是说 app 构建时加载的是 dist 下的旧产物，
 * **修改 src 源码后必须重新构建本包才会生效**：
 *
 *   pnpm --filter @vben/vite-config run build
 *
 * 由于 dist 产物是共享的、会影响所有 app，这里保持极简；
 * 针对单个 app 的优化请写在该 app 自己的 vite.config.ts 里。
 */
async function getCommonConfig(): Promise<UserConfig> {
  return {
    build: {
      chunkSizeWarningLimit: 2000,
      reportCompressedSize: false,
      sourcemap: false,
    },
  };
}

export { getCommonConfig };
