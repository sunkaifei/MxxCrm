import type { DefineApplicationOptions } from '../typing';
declare function defineApplicationConfig(
  userConfigPromise?: DefineApplicationOptions,
): import('vite').UserConfigFnPromise;
export { defineApplicationConfig };
