import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { defineConfig } from '@vben/vite-config';

function getBackendPort(): number {
  try {
    const configPath = resolve(
      import.meta.dirname,
      '../../../../backend/config/config.ini',
    );
    const content = readFileSync(configPath, 'utf8');
    const match = content.match(/^server_port\s*=\s*(\d+)/m);
    if (match) {
      return Number.parseInt(match[1], 10);
    }
  } catch {}
  return 8080;
}

const backendPort = getBackendPort();

export default defineConfig(async () => {
  const vueI18nPath = resolve(
    import.meta.dirname,
    '../../node_modules/.pnpm/vue-i18n@11.4.2_vue@3.5.34_typescript@6.0.3_/node_modules/vue-i18n/dist/vue-i18n.esm-bundler.js',
  );
  // monaco-editor@0.56 的 exports 字段对深层 worker 子路径映射有误
  // （"./*.js": "./esm/vs/*.js" 会把 esm/vs/editor/editor.worker.js
  //  错误映射到 esm/vs/esm/vs/editor/editor.worker.js，路径重复）
  // 用 resolveId 钩子直接把 monaco-editor/esm/ 子路径映射到文件系统绝对路径，
  // 绕过 package.json exports 解析。比 resolve.alias 更可靠（alias 对 ?worker 后缀兼容性差）
  const monacoEsmDir = resolve(
    import.meta.dirname,
    'node_modules/monaco-editor/esm',
  );
  return {
    application: {
      devtools: false,
      i18n: true,
    },
    vite: {
      plugins: [
        {
          name: 'fix-vue-i18n-alias',
          enforce: 'pre',
          resolveId(id) {
            if (id === 'vue-i18n') {
              return vueI18nPath;
            }
          },
        },
        {
          name: 'fix-monaco-worker-path',
          enforce: 'pre',
          resolveId(id) {
            // 保留 query（如 ?worker），Vite worker 插件依赖它判断是否为 worker 模块
            const [cleanId, query] = id.split('?');
            if (cleanId.startsWith('monaco-editor/esm/')) {
              const subPath = cleanId.slice('monaco-editor/esm/'.length);
              let fullPath = resolve(monacoEsmDir, subPath);
              // worker 导入路径无扩展名（如 editor.worker），但实际文件是 .js
              // 必须返回带扩展名的真实路径，否则 Vite SPA fallback 会返回 index.html
              if (!existsSync(fullPath) && existsSync(`${fullPath}.js`)) {
                fullPath = `${fullPath}.js`;
              }
              // 若原始导入有 query，追加回去（如 ?worker → editor.worker.js?worker）
              if (query) {
                fullPath = `${fullPath}?${query}`;
              }
              return fullPath;
            }
          },
        },
      ],
      server: {
        host: '0.0.0.0',
        port: 5668,
        strictPort: true,
        proxy: {
          '/api': {
            changeOrigin: true,
            target: `http://127.0.0.1:${backendPort}`,
            ws: true,
          },
        },
      },
    },
  };
});
