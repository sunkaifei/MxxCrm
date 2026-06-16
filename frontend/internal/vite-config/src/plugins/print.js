import { colors } from '@vben/node-utils';
export const vitePrintPlugin = (options = {}) => {
  const { infoMap = {} } = options;
  return {
    configureServer(server) {
      const _printUrls = server.printUrls;
      server.printUrls = () => {
        _printUrls();
        for (const [key, value] of Object.entries(infoMap)) {
          console.log(
            `  ${colors.green('➜')}  ${colors.bold(key)}: ${colors.cyan(value)}`,
          );
        }
      };
    },
    enforce: 'pre',
    name: 'vite:print-info',
  };
};
