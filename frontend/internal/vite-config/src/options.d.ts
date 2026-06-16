import type { Options as PwaPluginOptions } from 'vite-plugin-pwa';
import type { ImportmapPluginOptions } from './typing';
declare const getDefaultPwaOptions: (name: string) => Partial<PwaPluginOptions>;
declare const defaultImportmapOptions: ImportmapPluginOptions;
export { defaultImportmapOptions, getDefaultPwaOptions };
