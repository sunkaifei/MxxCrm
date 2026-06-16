import type { PluginOption } from 'vite';
interface PluginOptions {
  isBuild: boolean;
  root: string;
}
declare function viteExtraAppConfigPlugin({
  isBuild,
  root,
}: PluginOptions): Promise<PluginOption | undefined>;
export { viteExtraAppConfigPlugin };
