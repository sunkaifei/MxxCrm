<script lang="ts" setup>
import { nextTick, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Button,
  Empty,
  message,
  Modal,
  Space,
  Tree,
} from 'ant-design-vue';

import {
  getMenuOptionsApi,
  getPermSetMenuIdsApi,
  updatePermSetAuthApi,
} from '#/api';
import { $t } from '#/locales';

const data = ref();

// ---------- 菜单权限 ----------
const treeData = ref<any[]>([]);
const expandedKeys = ref<string[]>([]);
const checkedKeys = ref<string[]>([]);

const fieldNames = {
  children: 'children',
  title: 'name',
  key: 'id',
};

const getAllKeys = (list: any[]): string[] => {
  const keys: string[] = [];
  const traverse = (nodes: any[]) => {
    nodes.forEach((node: any) => {
      if (node.id !== undefined && node.id !== null) {
        keys.push(String(node.id));
      }
      if (node.children?.length) {
        traverse(node.children);
      }
    });
  };
  traverse(list);
  return keys;
};

const expandAll = () => {
  expandedKeys.value = getAllKeys(treeData.value);
};

const collapseAll = () => {
  expandedKeys.value = [];
};

const checkAll = () => {
  checkedKeys.value = getAllKeys(treeData.value);
};

const uncheckAll = () => {
  checkedKeys.value = [];
};

// ---------- 断链勾选检测 ----------
// 菜单权限需保持从根到叶的完整链路：勾选了子级但祖先未勾选时，子级保存后不生效
const brokenVisible = ref(false);
const brokenItems = ref<Array<{ name: string; missingParents: string[] }>>([]);
const brokenMissingIds = ref<string[]>([]);

const currentCheckedIds = (): string[] => {
  const checked = checkedKeys.value;
  if (Array.isArray(checked)) {
    return checked;
  }
  if (checked && typeof checked === 'object' && 'checked' in checked) {
    return (checked as { checked?: string[] }).checked ?? [];
  }
  return [];
};

// 检测勾选节点中祖先链路不完整的项
// 返回：断链节点明细（含缺失的父级名称，按树层级从外到内排序）+ 需补勾的父级 ID 集合
const detectBrokenChecks = (): {
  missingIds: Set<string>;
  brokenItems: Array<{ name: string; missingParents: string[] }>;
} => {
  const checkedSet = new Set(currentCheckedIds());
  // 节点 id -> 名称映射，用于把缺失父级 id 转成可读名称
  const nameMap = new Map<string, string>();
  const collectNames = (nodes: any[]) => {
    nodes.forEach((node: any) => {
      nameMap.set(String(node.id), node.name || String(node.id));
      if (node.children?.length) {
        collectNames(node.children);
      }
    });
  };
  collectNames(treeData.value);

  const items: Array<{ name: string; missingParents: string[] }> = [];
  const missingIds = new Set<string>();
  const traverse = (nodes: any[], uncheckedAncestors: string[]) => {
    nodes.forEach((node: any) => {
      const key = String(node.id);
      const isChecked = checkedSet.has(key);
      if (isChecked && uncheckedAncestors.length > 0) {
        items.push({
          name: node.name || key,
          missingParents: uncheckedAncestors.map(
            (id) => nameMap.get(id) || id,
          ),
        });
        uncheckedAncestors.forEach((id) => missingIds.add(id));
      }
      if (node.children?.length) {
        // 当前节点未勾选时，将其加入未勾选祖先集合向下传递
        const next = isChecked
          ? uncheckedAncestors
          : [...uncheckedAncestors, key];
        traverse(node.children, next);
      }
    });
  };
  traverse(treeData.value, []);
  return { missingIds, brokenItems: items };
};

// ---------- Drawer 生命周期 ----------
// 加载序号：快速切换权限集时，上一次打开遗留的异步回调不得覆盖新权限集的数据（竞态防护）
let loadSeq = 0;

const [Drawer, drawerApi] = useVbenDrawer({
  async onOpened() {
    const seq = ++loadSeq;
    data.value = drawerApi.getData<Record<string, any>>();
    // 每次打开都重置展示状态，避免残留上一个权限集的数据
    checkedKeys.value = [];
    expandedKeys.value = [];
    brokenVisible.value = false;
    treeData.value = [];

    // 加载当前用户可授权的菜单树（后端 /menu/options 已按用户权限过滤）
    const menuList = await getMenuOptionsApi();
    if (seq !== loadSeq) return;
    // 菜单名 i18n（目录级 key 带 .title fallback）
    const translateName = (key?: null | string): string => {
      if (!key) return '';
      // 已是翻译后的中文（如“概览”）直接返回，避免对非 i18n key 二次翻译触发 intlify 警告
      if (!/^[a-zA-Z][\w-]*(\.[a-zA-Z][\w-]*)+$/.test(key)) return key;
      const direct = $t(key);
      if (direct !== key && !direct.startsWith('[object ')) return direct;
      const withTitle = $t(`${key}.title`);
      return withTitle === `${key}.title` ? key : withTitle;
    };
    // 递归翻译菜单名称（包含 BUTTON 类型的权限按钮）
    const translateMenu = (items: any[]): any[] => {
      return items.map((item) => {
        if (item.name) item.name = translateName(item.name);
        if (item.meta?.name) item.meta.name = translateName(item.meta.name);
        if (item.children?.length) {
          item.children = translateMenu(item.children);
        }
        return item;
      });
    };
    treeData.value = translateMenu(menuList);

    if (data.value?.row?.id) {
      const permSetId = Number(data.value.row.id);

      try {
        // 加载已有的菜单权限
        const permSetMenuIds = await getPermSetMenuIdsApi(permSetId);
        if (seq !== loadSeq) return;
        await nextTick();

        if (permSetMenuIds) {
          const treeIds = getAllKeys(treeData.value);
          const validMenuIds = (Array.isArray(permSetMenuIds) ? permSetMenuIds : [])
            .map(String)
            .filter((id: string) => treeIds.includes(id));

          checkedKeys.value = validMenuIds;
          expandedKeys.value = treeIds;
        }
      } catch {
        // 权限集暂无菜单授权配置
        if (seq !== loadSeq) return;
      }
    }
  },

  async onConfirm() {
    if (!data.value?.row?.id) {
      message.error($t('page.system.permSet.notFound'));
      return;
    }
    // 断链检测：勾选了子级但父级未勾选时，弹窗由用户选择处理方式，不直接保存
    const { missingIds, brokenItems: items } = detectBrokenChecks();
    if (items.length > 0) {
      brokenItems.value = items;
      brokenMissingIds.value = [...missingIds];
      brokenVisible.value = true;
      return;
    }
    await doSaveAuth();
  },
});

// 执行权限保存
// * extraIds 需额外补充勾选的父级菜单 ID（"自动勾选父级"场景）
async function doSaveAuth(extraIds: string[] = []) {
  if (!data.value?.row?.id) {
    message.error($t('page.system.permSet.notFound'));
    return;
  }
  const permSetId = Number(data.value.row.id);
  setLoading(true);
  try {
    // 保存菜单权限
    // 后端会再次校验链路完整性：父级未勾选的菜单不保存
    const authId = [...new Set([...currentCheckedIds(), ...extraIds])];
    await updatePermSetAuthApi(permSetId, { authId });

    brokenVisible.value = false;
    message.success($t('ui.notification.update_success'));
    drawerApi.close();
  } catch {
    // 错误提示由 request.ts 拦截器统一处理，此处不再重复弹出
  } finally {
    setLoading(false);
  }
}

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="$t('page.system.permSet.button.auth')" width="720px">
    <div class="flex flex-col gap-4">
      <!-- 菜单权限区域 -->
      <div>
        <div class="mb-3 text-xs text-gray-400">
          {{ $t('page.system.permSet.authTip') }}
        </div>
        <Space class="mb-3">
          <Button size="small" @click="expandAll">
            {{ $t('ui.tree.expand_all') }}
          </Button>
          <Button size="small" @click="collapseAll">
            {{ $t('ui.tree.collapse_all') }}
          </Button>
          <Button size="small" type="primary" ghost @click="checkAll">
            {{ $t('ui.tree.select_all') }}
          </Button>
          <Button size="small" @click="uncheckAll">
            {{ $t('ui.tree.unselect_all') }}
          </Button>
        </Space>
        <div
          class="max-h-[420px] overflow-y-auto border border-gray-100 rounded-lg p-3"
        >
          <Tree
            v-model:expanded-keys="expandedKeys"
            v-model:checked-keys="checkedKeys"
            :tree-data="treeData"
            checkable
            :check-strictly="true"
            :field-names="fieldNames"
            class="w-full"
          >
            <template #title="{ data: item }">
              <span class="text-sm">{{
                $t(item.meta?.name || item.name)
              }}</span>
            </template>
          </Tree>
          <div
            v-if="treeData.length === 0"
            class="py-12 text-center text-gray-400"
          >
            <Empty :description="$t('page.system.permSet.noMenuData')" />
          </div>
        </div>
      </div>

      <!-- 断链勾选确认弹窗 -->
      <Modal
        v-model:open="brokenVisible"
        :title="$t('page.system.permSet.brokenTitle')"
        :footer="null"
        width="520px"
      >
        <div class="text-sm">
          {{ $t('page.system.permSet.brokenTip', { count: brokenItems.length }) }}
        </div>
        <ul
          class="mt-2 max-h-[220px] overflow-y-auto rounded-lg border border-gray-100 p-3 text-xs text-gray-500"
        >
          <li
            v-for="(item, index) in brokenItems"
            :key="index"
            class="py-1"
          >
            <span class="text-gray-800">{{ item.name }}</span>
            <span v-if="item.missingParents.length" class="text-red-500">
              （{{ $t('page.system.permSet.brokenParents') }}{{ item.missingParents.join('、') }}）
            </span>
          </li>
        </ul>
        <div class="mt-4 flex justify-end gap-2">
          <Button @click="brokenVisible = false">{{ $t('page.system.permSet.backToEdit') }}</Button>
          <Button @click="doSaveAuth()">{{ $t('page.system.permSet.saveAsIs') }}</Button>
          <Button type="primary" @click="doSaveAuth(brokenMissingIds)">
            {{ $t('page.system.permSet.autoCheckParents') }}
          </Button>
        </div>
      </Modal>
    </div>
  </Drawer>
</template>
