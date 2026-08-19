import type { PluginOption } from 'vite';

import type { ApplicationPluginOptions, LibraryPluginOptions } from '../typing';

declare function loadApplicationPlugins(
  options: ApplicationPluginOptions,
): Promise<PluginOption[]>;
declare function loadLibraryPlugins(
  options: LibraryPluginOptions,
): Promise<PluginOption[]>;
export { loadApplicationPlugins, loadLibraryPlugins };

export { viteArchiverPlugin } from './archiver';
export { viteHtmlPlugin } from './html';
export { viteVxeTableImportsPlugin } from './vxe-table';
export { visualizer as viteVisualizerPlugin } from 'rollup-plugin-visualizer';
export { default as viteDtsPlugin } from 'unplugin-dts/vite';
export { default as viteCompressPlugin } from 'vite-plugin-compression';
