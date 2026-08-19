import type {
  ComponentRecordType,
  GenerateMenuAndRoutesOptions,
} from '@vben/types';

import { generateAccessible } from '@vben/access';
import { preferences } from '@vben/preferences';
import { mapTree } from '@vben/utils';

import { message } from 'ant-design-vue';

import { getAllMenusApi } from '#/api';
import { BasicLayout, EmptyLayout, IFrameView } from '#/layouts';
import { $t } from '#/locales';

const forbiddenComponent = () => import('#/views/_core/fallback/forbidden.vue');

/**
 * 菜单名 i18n 翻译（带 .title fallback）。
 * 目录级 key（如 page.statistics.employee）在 locale 中是
 * { title, button } 对象，直接 $t 不命中；这里兜底 `${key}.title`。
 */
function translateMenuTitle(key?: null | string): string {
  if (!key) return '';
  const direct = $t(key);
  if (direct !== key && !direct.startsWith('[object ')) return direct;
  const withTitle = $t(`${key}.title`);
  if (withTitle !== `${key}.title`) return withTitle;
  return key;
}

async function generateAccess(options: GenerateMenuAndRoutesOptions) {
  const pageMap: ComponentRecordType = import.meta.glob('../views/**/*.vue');

  const layoutMap: ComponentRecordType = {
    BasicLayout,
    EmptyLayout,
    IFrameView,
  };

  return await generateAccessible(preferences.app.accessMode, {
    ...options,
    fetchMenuListAsync: async () => {
      message.loading({
        content: `${$t('common.loadingMenu')}...`,
        duration: 1.5,
      });
      const menus = await getAllMenusApi();
      // 从源头翻译：把 meta.title / meta.name 中的 i18n key 翻译成实际文本。
      // 这样后续 generateRoutesByBackend、generateMenus、breadcrumb、
      // 全局搜索、tabbar 等所有下游无需再处理 i18n 降级。
      return mapTree(menus, (node: any) => {
        if (node?.meta?.title) {
          node.meta.title = translateMenuTitle(node.meta.title);
        }
        if (node?.meta?.name) {
          node.meta.name = translateMenuTitle(node.meta.name);
        }
        return node;
      });
    },
    // 可以指定没有权限跳转403页面
    forbiddenComponent,
    // 如果 route.meta.menuVisibleWithForbidden = true
    layoutMap,
    pageMap,
  });
}

export { generateAccess };
