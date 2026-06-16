import type { GeneratorOptions } from '@jspm/generator';
import type { Plugin } from 'vite';
type pluginOptions = GeneratorOptions & {
  debug?: boolean;
  defaultProvider?: 'esm.sh' | 'jsdelivr' | 'jspm.io';
  importmap?: Array<{
    name: string;
    range?: string;
  }>;
};
declare function viteImportMapPlugin(
  pluginOptions?: pluginOptions,
): Promise<Plugin[]>;
export { viteImportMapPlugin };
