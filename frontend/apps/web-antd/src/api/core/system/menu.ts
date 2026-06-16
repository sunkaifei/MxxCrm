import type { MenuType, Timestamp } from '#/store';

import { requestClient } from '#/api/request';
import { $t } from '#/locales';

export namespace MenuApi {
  /** 菜单查询参数 */
  export interface MenuQuery {
    /** 搜索关键字 */
    keywords?: string;
    /** 是否只获取父级 */
    onlyParent: boolean;
  }

  /** 菜单状态 */
  export enum MenuStatus {
    OFF = 'OFF',
    ON = 'ON',
  }

  /** 菜单表单对象 */
  export interface MenuForm {
    /** 菜单ID */
    id?: null | number | undefined;
    /** 菜单状态 */
    status?: MenuStatus | null | undefined;
    /** 菜单类型 */
    type?: MenuType | null | undefined;
    /** 路由路径 */
    path?: null | string | undefined;
    /** 重定向地址 */
    redirect?: null | string | undefined;
    /** 路由别名 */
    alias?: null | string | undefined;
    /** 路由命名 */
    name?: null | string | undefined;
    /** 指向的组件 */
    component?: null | string | undefined;
    /** 路由元信息 */
    meta?: null | RouteMeta | undefined;
    /** 父节点ID */
    parentId?: null | number | undefined;
    /** 子节点树 */
    children: MenuForm[];
    /** 创建时间 */
    createTime?: null | Timestamp | undefined;
    /** 更新时间 */
    updateTime?: null | Timestamp | undefined;
    /** 删除时间 */
    deleteTime?: null | Timestamp | undefined;
  }

  /** 路由元数据 */
  export interface RouteMeta {
    activeIcon?: null | string | undefined;
    activePath?: null | string | undefined;
    affixTab?: null | number | undefined;
    affixTabOrder?: null | number | undefined;
    authority: string[];
    badge?: null | string | undefined;
    badgeType?: null | string | undefined;
    badgeVariants?: null | string | undefined;
    hideChildrenInMenu?: null | number | undefined;
    hideInBreadcrumb?: null | number | undefined;
    hideInMenu?: null | number | undefined;
    hideInTab?: null | number | undefined;
    icon?: null | string | undefined;
    iframeSrc?: null | string | undefined;
    ignoreAccess?: boolean | null | undefined;
    keepAlive?: null | number | undefined;
    link?: null | string | undefined;
    loaded?: boolean | null | undefined;
    maxNumOfOpenTab?: null | number | undefined;
    menuVisibleWithForbidden?: boolean | null | undefined;
    openInNewWindow?: boolean | null | undefined;
    sort?: null | number | undefined;
    name?: null | string | undefined;
  }

  /** 查询菜单列表 - 响应 */
  export interface ListMenuResp {
    items: MenuForm[];
    total: number;
  }
}

/**
 * 获取用户所有菜单
 */
export const getMenusRouterApi = async (param: any) => {
  return await requestClient.get<MenuApi.ListMenuResp>(
    '/api/system/menu/getUserMenus',
    {
      params: param,
    },
  );
};

/**
 * 获取菜单树形列表
 */
export const getMenuTreeApi = async (param: any) => {
  return await requestClient.get<MenuApi.ListMenuResp>(
    '/api/system/menu/list',
    {
      params: param,
    },
  );
};

/**
 * 获取表单内的指定数据
 */
export const getFormMenuInfoApi = async (id: number) => {
  return await requestClient.get(`/api/system/menu/${id}`);
};

/**
 * 修改菜单信息
 */
export const updateMenuApi = async (id: number, param: any) => {
  return await requestClient.put(`/api/system/menu/update/${id}`, param);
};

/**
 * 新增菜单信息
 */
export const createMenuApi = async (param: any) => {
  return await requestClient.post('/api/system/menu/add', param);
};

/**
 * 删除菜单
 */
export const deleteMenuApi = async (id: number) => {
  return await requestClient.delete(`/api/system/menu/delete/${id}`);
};

export function buildMenuTree(menus: MenuApi.MenuForm[]): MenuApi.MenuForm[] {
  const tree: MenuApi.MenuForm[] = [];

  for (const menu of menus) {
    if (!menu) continue;
    if (menu.type === 'BUTTON') continue;
    const parentId = String(menu.parentId);
    if (parentId !== '0' && menu.parentId !== undefined) continue;
    if (menu?.name) menu.name = $t(menu?.name ?? '');
    if (menu?.meta?.name) menu.meta.name = $t(menu?.meta?.name ?? '');
    tree.push(menu);
  }

  for (const menu of menus) {
    if (!menu) continue;
    if (menu.type === 'BUTTON') continue;
    const parentId = String(menu.parentId);
    if (parentId === '0' || menu.parentId === undefined) continue;
    if (travelMenuChild(tree, menu)) continue;
    if (menu?.name) menu.name = $t(menu?.name ?? '');
    if (menu?.meta?.name) menu.meta.name = $t(menu?.meta?.name ?? '');
    tree.push(menu);
  }

  return tree;
}

function travelMenuChild(
  nodes: MenuApi.MenuForm[],
  parent: MenuApi.MenuForm,
): boolean {
  if (!Array.isArray(nodes)) return false;
  if (parent.type === 'BUTTON') return false;
  const parentId = String(parent.parentId);
  if (parentId === '0' || parent.parentId === undefined) {
    if (parent?.name) parent.name = $t(parent?.name ?? '');
    if (parent?.meta?.name) parent.meta.name = $t(parent?.meta?.name ?? '');
    nodes.push(parent);
    return true;
  }

  for (const node of nodes) {
    if (String(node.id) === String(parent.parentId)) {
      if (parent?.name) parent.name = $t(parent?.name ?? '');
      if (parent?.meta?.name) parent.meta.name = $t(parent?.meta?.name ?? '');
      node.children = node.children || [];
      node.children.push(parent);
      return true;
    }
    if (travelMenuChild(node.children, parent)) return true;
  }

  return false;
}
