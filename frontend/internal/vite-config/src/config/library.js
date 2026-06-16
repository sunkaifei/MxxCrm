import { readPackageJSON } from '@vben/node-utils';
import { defineConfig, mergeConfig } from 'vite';
import { loadLibraryPlugins } from '../plugins';
import { getCommonConfig } from './common';
function defineLibraryConfig(userConfigPromise) {
  return defineConfig(async (config) => {
    const options = await userConfigPromise?.(config);
    const { command, mode } = config;
    const { library = {}, vite = {} } = options || {};
    const root = process.cwd();
    const isBuild = command === 'build';
    const plugins = await loadLibraryPlugins({
      dts: false,
      injectMetadata: true,
      isBuild,
      mode,
      ...library,
    });
    const { dependencies = {}, peerDependencies = {} } =
      await readPackageJSON(root);
    const externalPackages = [
      ...Object.keys(dependencies),
      ...Object.keys(peerDependencies),
    ];
    const packageConfig = {
      build: {
        lib: {
          entry: 'src/index.ts',
          fileName: () => 'index.mjs',
          formats: ['es'],
        },
        rolldownOptions: {
          external: (id) => {
            return externalPackages.some(
              (pkg) => id === pkg || id.startsWith(`${pkg}/`),
            );
          },
        },
      },
      plugins,
    };
    const commonConfig = await getCommonConfig();
    const mergedConmonConfig = mergeConfig(commonConfig, packageConfig);
    return mergeConfig(mergedConmonConfig, vite);
  });
}
export { defineLibraryConfig };
