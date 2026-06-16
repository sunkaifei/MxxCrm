import { defineConfig } from '@vben/vite-config';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// 从后端 Config.ini 读取端口配置
function getBackendPort(): number {
  try {
    const configPath = resolve(
      __dirname,
      '../../../../backend/config/Config.ini',
    );
    const content = readFileSync(configPath, 'utf-8');
    const match = content.match(/^server_port\s*=\s*(\d+)/m);
    if (match) {
      return parseInt(match[1], 10);
    }
  } catch {
    // 读取失败时使用默认端口
  }
  return 8080;
}

const backendPort = getBackendPort();

export default defineConfig(async () => {
  return {
    application: {
      devtools: false,
    },
    vite: {
      server: {
        proxy: {
          '/api': {
            changeOrigin: true,
            // 自动读取后端 Config.ini 的 server_port
            target: `http://127.0.0.1:${backendPort}`,
            ws: true,
          },
        },
      },
    },
  };
});
