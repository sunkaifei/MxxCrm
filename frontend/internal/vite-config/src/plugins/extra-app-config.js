import {
  colors,
  generatorContentHash,
  readPackageJSON,
} from '@vben/node-utils';
import { loadEnv } from '../utils/env';
const GLOBAL_CONFIG_FILE_NAME = '_app.config.js';
const VBEN_ADMIN_PRO_APP_CONF = '_VBEN_ADMIN_PRO_APP_CONF_';
async function viteExtraAppConfigPlugin({ isBuild, root }) {
  let publicPath;
  let source;
  if (!isBuild) {
    return;
  }
  const { version = '' } = await readPackageJSON(root);
  return {
    async configResolved(config) {
      publicPath = ensureTrailingSlash(config.base);
      source = await getConfigSource();
    },
    async generateBundle() {
      try {
        this.emitFile({
          fileName: GLOBAL_CONFIG_FILE_NAME,
          source,
          type: 'asset',
        });
        console.log(colors.cyan(`✨configuration file is build successfully!`));
      } catch (error) {
        console.log(
          colors.red(
            `configuration file configuration file failed to package:\n${error}`,
          ),
        );
      }
    },
    name: 'vite:extra-app-config',
    async transformIndexHtml(html) {
      const hash = `v=${version}-${generatorContentHash(source, 8)}`;
      const appConfigSrc = `${publicPath}${GLOBAL_CONFIG_FILE_NAME}?${hash}`;
      return {
        html,
        tags: [{ attrs: { src: appConfigSrc }, tag: 'script' }],
      };
    },
  };
}
async function getConfigSource() {
  const config = await loadEnv();
  const windowVariable = `window.${VBEN_ADMIN_PRO_APP_CONF}`;
  let source = `${windowVariable}=${JSON.stringify(config)};`;
  source += `
    Object.freeze(${windowVariable});
    Object.defineProperty(window, "${VBEN_ADMIN_PRO_APP_CONF}", {
      configurable: false,
      writable: false,
    });
  `.replaceAll(/\s/g, '');
  return source;
}
function ensureTrailingSlash(path) {
  return path.endsWith('/') ? path : `${path}/`;
}
export { viteExtraAppConfigPlugin };
