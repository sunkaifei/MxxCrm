import type { Options as HtmlMinifierOptions } from 'html-minifier-terser';
import type { PluginVisualizerOptions } from 'rollup-plugin-visualizer';
import type { PluginOptions } from 'unplugin-dts';
import type {
  ConfigEnv,
  PluginOption,
  UserConfig,
  UserConfigFnPromise,
} from 'vite';
import type { Options as PwaPluginOptions } from 'vite-plugin-pwa';
interface IImportMap {
  imports?: Record<string, string>;
  scopes?: {
    [scope: string]: Record<string, string>;
  };
}
interface PrintPluginOptions {
  infoMap?: Record<string, string | undefined>;
}
interface NitroMockPluginOptions {
  mockServerPackage?: string;
  port?: number;
  verbose?: boolean;
}
interface ArchiverPluginOptions {
  name?: string;
  outputDir?: string;
}
type HtmlPluginOptions = HtmlMinifierOptions;
interface ImportmapPluginOptions {
  defaultProvider?: 'esm.sh' | 'jspm.io';
  importmap?: Array<{
    name: string;
    range?: string;
  }>;
  inputMap?: IImportMap;
}
interface ConditionPlugin {
  condition?: boolean;
  plugins: () => PluginOption[] | PromiseLike<PluginOption[]>;
}
interface CommonPluginOptions {
  devtools?: boolean;
  env?: Record<string, any>;
  injectMetadata?: boolean;
  isBuild?: boolean;
  mode?: string;
  visualizer?: boolean | PluginVisualizerOptions;
}
interface ApplicationPluginOptions extends CommonPluginOptions {
  archiver?: boolean;
  archiverPluginOptions?: ArchiverPluginOptions;
  compress?: boolean;
  compressTypes?: ('brotli' | 'gzip')[];
  extraAppConfig?: boolean;
  html?: boolean | HtmlPluginOptions;
  i18n?: boolean;
  importmap?: boolean;
  importmapOptions?: ImportmapPluginOptions;
  injectAppLoading?: boolean;
  injectGlobalScss?: boolean;
  license?: boolean;
  nitroMock?: boolean;
  nitroMockOptions?: NitroMockPluginOptions;
  print?: boolean;
  printInfoMap?: PrintPluginOptions['infoMap'];
  pwa?: boolean;
  pwaOptions?: Partial<PwaPluginOptions>;
  vxeTableLazyImport?: boolean;
}
interface LibraryPluginOptions extends CommonPluginOptions {
  dts?: boolean | PluginOptions;
}
type ApplicationOptions = ApplicationPluginOptions;
type LibraryOptions = LibraryPluginOptions;
type DefineApplicationOptions = (config?: ConfigEnv) => Promise<{
  application?: ApplicationOptions;
  vite?: UserConfig;
}>;
type DefineLibraryOptions = (config?: ConfigEnv) => Promise<{
  library?: LibraryOptions;
  vite?: UserConfig;
}>;
type DefineConfig = DefineApplicationOptions | DefineLibraryOptions;
type VbenViteConfig = Promise<UserConfig> | UserConfig | UserConfigFnPromise;
export type {
  ApplicationPluginOptions,
  ArchiverPluginOptions,
  CommonPluginOptions,
  ConditionPlugin,
  DefineApplicationOptions,
  DefineConfig,
  DefineLibraryOptions,
  HtmlPluginOptions,
  IImportMap,
  ImportmapPluginOptions,
  LibraryPluginOptions,
  NitroMockPluginOptions,
  PrintPluginOptions,
  VbenViteConfig,
};
