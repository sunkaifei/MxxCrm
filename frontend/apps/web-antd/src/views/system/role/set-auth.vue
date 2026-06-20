<script lang="ts" setup>
import { useVbenDrawer } from '@vben/common-ui';
import { ref } from 'vue';
import {
  MenuApi,
  getMenuTreeApi,
  buildMenuTree,
  updateRoleAuthApi,
  getRoleMenuIdsApi,
} from '#/api';
import { $t } from '#/locales';
import { nextTick } from 'vue';
import { Tree, Button, Space } from 'ant-design-vue';

const data = ref();

const [Drawer, drawerApi] = useVbenDrawer({
  async onOpened() {
    data.value = drawerApi.getData<Record<string, any>>();

    const menuResult = await getMenuTreeApi(null);
    const menuList = Array.isArray(menuResult)
      ? menuResult
      : menuResult?.items || [];
    treeData.value = buildMenuTree(menuList);

    if (data.value?.row?.id) {
      try {
        const roleMenuIds = await getRoleMenuIdsApi(data.value.row.id);
        await nextTick();

        if (roleMenuIds) {
          // 只保留树中存在的节点 ID（buildMenuTree 过滤了按钮类型）
          const treeIds = getAllKeys(treeData.value);
          const validMenuIds = (Array.isArray(roleMenuIds) ? roleMenuIds : [])
            .map(String)
            .filter((id: string) => treeIds.includes(id));

          checkedKeys.value = [];
          expandedKeys.value = treeIds;
          setTimeout(() => {
            checkedKeys.value = validMenuIds;
          }, 100);
        }
      } catch {
        // 角色无可授权限
      }
    }
  },

  async onConfirm() {
    let authId: string[] = [...checkedKeys.value];

    // 获取半选状态（父节点部分选中）
    const halfChecked = treeRef.value?.getHalfCheckedKeys?.();
    if (halfChecked?.length) {
      authId = [...authId, ...halfChecked];
    }

    if (authId.length <= 0) {
      window.$message.error($t('ui.notification.no_auth'));
      return;
    }

    setLoading(true);
    try {
      await updateRoleAuthApi(data.value.row.id, { authId });
      window.$message.success($t('ui.notification.update_success'));
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },
});

const treeData = ref<MenuApi.MenuForm[]>([]);
const treeRef = ref<any>();
const expandedKeys = ref<string[]>([]);
const checkedKeys = ref<string[]>([]);

const defaultProps = {
  children: 'children',
  title: 'name',
  key: 'id',
};

const getAllKeys = (data: MenuApi.MenuForm[]): string[] => {
  const keys: string[] = [];
  const traverse = (nodes: MenuApi.MenuForm[]) => {
    nodes.forEach((node) => {
      if (node.id !== undefined && node.id !== null) {
        keys.push(String(node.id));
      }
      if (node.children?.length) {
        traverse(node.children);
      }
    });
  };
  traverse(data);
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

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="$t('page.system.role.button.auth')">
    <Space direction="vertical" class="w-full">
      <Space>
        <Button @click="expandAll">{{ $t('ui.tree.expand_all') }}</Button>
        <Button @click="collapseAll">{{ $t('ui.tree.collapse_all') }}</Button>
        <Button @click="checkAll">{{ $t('ui.tree.select_all') }}</Button>
        <Button @click="uncheckAll">{{ $t('ui.tree.unselect_all') }}</Button>
      </Space>

      <Tree
        ref="treeRef"
        v-model:expandedKeys="expandedKeys"
        v-model:checkedKeys="checkedKeys"
        :tree-data="treeData"
        checkable
        :check-strictly="false"
        :replace-fields="defaultProps"
        class="w-full"
      >
        <template #title="{ data: treeData }">
          <span>{{ $t(treeData.meta?.name || treeData.name) }}</span>
        </template>
      </Tree>
    </Space>
  </Drawer>
</template>
