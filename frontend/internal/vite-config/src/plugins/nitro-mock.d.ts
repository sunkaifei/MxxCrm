import type { PluginOption } from 'vite';
import type { NitroMockPluginOptions } from '../typing';
export declare const viteNitroMockPlugin: ({
  mockServerPackage,
  port,
  verbose,
}?: NitroMockPluginOptions) => PluginOption;
