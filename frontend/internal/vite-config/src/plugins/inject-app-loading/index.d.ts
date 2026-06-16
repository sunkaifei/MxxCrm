import type { PluginOption } from 'vite';
declare function viteInjectAppLoadingPlugin(
  isBuild: boolean,
  env?: Record<string, any>,
  loadingTemplate?: string,
): Promise<PluginOption | undefined>;
export { viteInjectAppLoadingPlugin };
