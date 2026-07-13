import { defineConfig } from '@vben/vite-config';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

function getBackendPort(): number {
  try {
    const configPath = resolve(
      __dirname,
      '../../../../backend/config/config.ini',
    );
    const content = readFileSync(configPath, 'utf-8');
    const match = content.match(/^server_port\s*=\s*(\d+)/m);
    if (match) {
      return parseInt(match[1], 10);
    }
  } catch {
  }
  return 8080;
}

const backendPort = getBackendPort();

export default defineConfig(async () => {
  const vueI18nPath = resolve(
    __dirname,
    '../../node_modules/.pnpm/vue-i18n@11.4.2_vue@3.5.34_typescript@6.0.3_/node_modules/vue-i18n/dist/vue-i18n.esm-bundler.js',
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
      ],
      server: {
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
