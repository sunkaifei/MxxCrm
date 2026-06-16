const isDevelopment = process.env.NODE_ENV === 'development';
const getDefaultPwaOptions = (name) => ({
  manifest: {
    description:
      'Vben Admin is a modern admin dashboard template based on Vue 3. ',
    icons: [
      {
        sizes: '192x192',
        src: 'https://unpkg.com/@vbenjs/static-source@0.1.7/source/pwa-icon-192.png',
        type: 'image/png',
      },
      {
        sizes: '512x512',
        src: 'https://unpkg.com/@vbenjs/static-source@0.1.7/source/pwa-icon-512.png',
        type: 'image/png',
      },
    ],
    name: `${name}${isDevelopment ? ' dev' : ''}`,
    short_name: `${name}${isDevelopment ? ' dev' : ''}`,
  },
});
const defaultImportmapOptions = {
  defaultProvider: 'esm.sh',
  importmap: [
    { name: 'vue' },
    { name: 'pinia' },
    { name: 'vue-router' },
    { name: 'dayjs' },
    { name: 'vue-demi' },
  ],
};
export { defaultImportmapOptions, getDefaultPwaOptions };
