import type { DefineLibraryOptions } from '../typing';
declare function defineLibraryConfig(
  userConfigPromise?: DefineLibraryOptions,
): import('vite').UserConfigFnPromise;
export { defineLibraryConfig };
