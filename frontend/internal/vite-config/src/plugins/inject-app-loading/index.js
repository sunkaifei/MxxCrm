import fs from 'node:fs';
import fsp from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { readPackageJSON } from '@vben/node-utils';
async function viteInjectAppLoadingPlugin(
  isBuild,
  env = {},
  loadingTemplate = 'loading.html',
) {
  const loadingHtml = await getLoadingRawByHtmlTemplate(loadingTemplate);
  const { version } = await readPackageJSON(process.cwd());
  const envRaw = isBuild ? 'prod' : 'dev';
  const cacheName = `'${env.VITE_APP_NAMESPACE}-${version}-${envRaw}-preferences-theme'`;
  const injectScript = `
  <script data-app-loading="inject-js">
  var theme = localStorage.getItem(${cacheName});
  document.documentElement.classList.toggle('dark', /dark/.test(theme));
</script>
`;
  if (!loadingHtml) {
    return;
  }
  return {
    enforce: 'pre',
    name: 'vite:inject-app-loading',
    transformIndexHtml: {
      handler(html) {
        const re = /<body\s*>/;
        html = html.replace(re, `<body>${injectScript}${loadingHtml}`);
        return html;
      },
      order: 'pre',
    },
  };
}
async function getLoadingRawByHtmlTemplate(loadingTemplate) {
  let appLoadingPath = join(process.cwd(), loadingTemplate);
  if (!fs.existsSync(appLoadingPath)) {
    const __dirname = fileURLToPath(new URL('.', import.meta.url));
    appLoadingPath = join(__dirname, './default-loading.html');
  }
  return await fsp.readFile(appLoadingPath, 'utf8');
}
export { viteInjectAppLoadingPlugin };
