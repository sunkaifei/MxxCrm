import type { DefineConfig, VbenViteConfig } from '../typing';
export * from './application';
export * from './library';
declare function defineConfig(
  userConfigPromise?: DefineConfig,
  type?: 'application' | 'auto' | 'library',
): VbenViteConfig;
export { defineConfig };
