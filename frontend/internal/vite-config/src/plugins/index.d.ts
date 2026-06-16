import type { PluginOption } from 'vite';
import type { ApplicationPluginOptions, LibraryPluginOptions } from '../typing';
import { visualizer as viteVisualizerPlugin } from 'rollup-plugin-visualizer';
import viteDtsPlugin from 'unplugin-dts/vite';
import viteCompressPlugin from 'vite-plugin-compression';
import { viteArchiverPlugin } from './archiver';
import { viteHtmlPlugin } from './html';
import { viteVxeTableImportsPlugin } from './vxe-table';
declare function loadApplicationPlugins(
  options: ApplicationPluginOptions,
): Promise<PluginOption[]>;
declare function loadLibraryPlugins(
  options: LibraryPluginOptions,
): Promise<PluginOption[]>;
export {
  loadApplicationPlugins,
  loadLibraryPlugins,
  viteArchiverPlugin,
  viteCompressPlugin,
  viteDtsPlugin,
  viteHtmlPlugin,
  viteVisualizerPlugin,
  viteVxeTableImportsPlugin,
};
