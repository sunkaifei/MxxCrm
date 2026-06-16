import type { Options as HtmlMinifierOptions } from 'html-minifier-terser';
import type { PluginOption } from 'vite';
declare function viteHtmlPlugin(options?: HtmlMinifierOptions): PluginOption;
export { viteHtmlPlugin };
