import type { ApplicationPluginOptions } from '../typing';
declare function loadEnv<T = Record<string, string>>(
  match?: string,
  confFiles?: string[],
): Promise<T>;
declare function loadAndConvertEnv(
  match?: string,
  confFiles?: string[],
): Promise<
  Partial<ApplicationPluginOptions> & {
    appTitle: string;
    base: string;
    port: number;
  }
>;
export { loadAndConvertEnv, loadEnv };
