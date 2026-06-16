import { lazyImport, VxeResolver } from 'vite-plugin-lazy-import';
async function viteVxeTableImportsPlugin() {
  return [
    lazyImport({
      resolvers: [
        VxeResolver({
          libraryName: 'vxe-table',
        }),
        VxeResolver({
          libraryName: 'vxe-pc-ui',
        }),
      ],
    }),
  ];
}
export { viteVxeTableImportsPlugin };
