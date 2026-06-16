import { requestClient } from '#/api/request';

export namespace ApiTreeApi {
  export interface ApiTreeNode {
    id: number;
    parentId: number;
    description: string;
    method: string;
    path: string;
    createdAt: number;
    children: ApiTreeNode[] | null;
  }

  export interface ListApiTreeResp {
    items: ApiTreeNode[];
    total: number;
  }
}

/**
 * 获取 API 树形列表
 */
export const getApiTreeApi = async () => {
  return await requestClient.get<ApiTreeApi.ListApiTreeResp>('/api/tree');
};

/**
 * 获取 API 列表
 */
export const getApiListApi = async (params?: Record<string, unknown>) => {
  return await requestClient.get('/api', { params });
};

/**
 * 创建 API
 */
export const createApiApi = async (data: Record<string, unknown>) => {
  return await requestClient.post('/api', data);
};

/**
 * 更新 API
 */
export const updateApiApi = async (
  id: number,
  data: Record<string, unknown>,
) => {
  return await requestClient.put(`/api/${id}`, data);
};

/**
 * 删除 API
 */
export const deleteApiApi = async (id: number) => {
  return await requestClient.delete(`/api/${id}`);
};

export function buildApiTree(
  apis: ApiTreeApi.ApiTreeNode[],
): ApiTreeApi.ApiTreeNode[] {
  const tree: ApiTreeApi.ApiTreeNode[] = [];

  for (const api of apis) {
    if (!api) continue;
    if (api.parentId !== 0) continue;
    api.children = api.children || [];
    tree.push(api);
  }

  for (const api of apis) {
    if (!api) continue;
    if (api.parentId === 0) continue;
    if (travelApiChild(tree, api)) continue;
    api.children = api.children || [];
    tree.push(api);
  }

  return tree;
}

function travelApiChild(
  nodes: ApiTreeApi.ApiTreeNode[],
  child: ApiTreeApi.ApiTreeNode,
): boolean {
  if (!Array.isArray(nodes)) return false;
  if (child.parentId === 0) {
    child.children = child.children || [];
    nodes.push(child);
    return true;
  }

  for (const node of nodes) {
    if (node.id === child.parentId) {
      node.children = node.children || [];
      child.children = child.children || [];
      node.children.push(child);
      return true;
    }
    if (node.children && travelApiChild(node.children, child)) return true;
  }

  return false;
}
