import { minify } from 'html-minifier-terser';
const HTML_MINIFY_OPTIONS = {
  collapseWhitespace: true,
  minifyCSS: true,
  minifyJS: true,
  removeComments: true,
  removeRedundantAttributes: true,
  removeScriptTypeAttributes: true,
  removeStyleLinkTypeAttributes: true,
  useShortDoctype: true,
};
function viteHtmlPlugin(options = {}) {
  return {
    name: 'vben-native-html',
    transformIndexHtml: {
      order: 'post',
      async handler(html, ctx) {
        if (!ctx.bundle) {
          return html;
        }
        return await minify(html, {
          ...HTML_MINIFY_OPTIONS,
          ...options,
        });
      },
    },
  };
}
export { viteHtmlPlugin };
