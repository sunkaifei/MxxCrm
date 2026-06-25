<script lang="ts" setup>
import { useVbenDrawer } from '@vben/common-ui';
import { ref, computed, nextTick } from 'vue';
import {
  MenuApi,
  getMenuTreeApi,
  buildMenuTree,
  updateRoleAuthApi,
  updateRoleApi,
  getRoleInfoApi,
  getRoleMenuIdsApi,
  getRoleDeptIdsApi,
  updateRoleDeptApi,
  getDeptTreeApi,
} from '#/api';
import { $t } from '#/locales';
import { Tree, Button, Space, Divider, Radio, Empty, message } from 'ant-design-vue';

const data = ref();
const activeSection = ref<'menu' | 'dataScope'>('menu');

// ---------- 菜单权限 ----------
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

// ---------- 数据权限 ----------
const dataScopeValue = ref<number>(5); // 默认仅本人数据
const deptTreeData = ref<any[]>([]);
const deptCheckedKeys = ref<string[]>([]);
const deptExpandedKeys = ref<string[]>([]);
const deptTreeRef = ref<any>();

const dataScopeOptions = [
  { value: 1, label: '全部数据', desc: '可查看系统中所有业务数据，不受部门和个人限制' },
  { value: 2, label: '自定义数据', desc: '可查看指定部门及以下的业务数据' },
  { value: 3, label: '本部门数据', desc: '只能查看所在部门所有成员负责的业务数据' },
  { value: 4, label: '本部门及以下', desc: '可查看本部门及下属部门所有成员的业务数据' },
  { value: 5, label: '仅本人数据', desc: '只能查看自己负责的业务数据' },
];

const showDeptTree = computed(() => dataScopeValue.value === 2);

const getAllDeptKeys = (data: any[]): string[] => {
  const keys: string[] = [];
  const traverse = (nodes: any[]) => {
    nodes.forEach((node) => {
      if (node.key !== undefined && node.key !== null) {
        keys.push(String(node.key));
      }
      if (node.children?.length) {
        traverse(node.children);
      }
    });
  };
  traverse(data);
  return keys;
};

const buildDeptTreeData = (nodes: any[]): any[] => {
  return nodes.map((node) => {
    const children = node.children && node.children.length > 0
      ? buildDeptTreeData(node.children)
      : undefined;
    return {
      title: node.label,
      key: String(node.value),
      children,
    };
  });
};

// ---------- Drawer 生命周期 ----------
const [Drawer, drawerApi] = useVbenDrawer({
  async onOpened() {
    data.value = drawerApi.getData<Record<string, any>>();
    activeSection.value = 'menu';
    checkedKeys.value = [];
    deptCheckedKeys.value = [];
    dataScopeValue.value = 5;

    // 加载菜单树
    const menuResult = await getMenuTreeApi(null);
    const menuList = Array.isArray(menuResult)
      ? menuResult
      : menuResult?.items || [];
    treeData.value = buildMenuTree(menuList);

    // 加载部门树
    const deptResult = await getDeptTreeApi();
    const deptList = Array.isArray(deptResult) ? deptResult : deptResult?.data || [];
    deptTreeData.value = buildDeptTreeData(deptList);
    deptExpandedKeys.value = getAllDeptKeys(deptTreeData.value);

    if (data.value?.row?.id) {
      const roleId = Number(data.value.row.id);
      const isSuperAdmin = roleId === 1 || data.value.row.roleKey === 'super_admin' || data.value.row.roleKey === 'admin';

      // 从API加载角色详情获取最新的dataScope
      try {
        const roleDetail = await getRoleInfoApi(roleId);
        if (roleDetail) {
          dataScopeValue.value = roleDetail.dataScope ?? (isSuperAdmin ? 1 : 5);
          if (roleDetail.deptIds && roleDetail.deptIds.length > 0) {
            deptCheckedKeys.value = roleDetail.deptIds.map(String);
          }
        }
      } catch {
        dataScopeValue.value = isSuperAdmin ? 1 : (data.value.row.dataScope ?? 5);
      }

      // 超级管理员默认全部数据权限
      if (isSuperAdmin && !dataScopeValue.value) {
        dataScopeValue.value = 1;
      }

      // 如果是自定义数据权限，加载角色关联的部门ID
      if (dataScopeValue.value === 2 && !isSuperAdmin) {
        try {
          const roleDeptIds = await getRoleDeptIdsApi(roleId);
          if (roleDeptIds) {
            const deptKeys = (Array.isArray(roleDeptIds) ? roleDeptIds : []).map(String);
            setTimeout(() => {
              deptCheckedKeys.value = deptKeys;
            }, 100);
          }
        } catch {
          // ignore
        }
      }

      try {
        // 加载已有的菜单权限
        const roleMenuIds = await getRoleMenuIdsApi(roleId);
        await nextTick();

        if (roleMenuIds) {
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

        // 超级管理员：默认全选所有菜单
        if (isSuperAdmin) {
          expandedKeys.value = getAllKeys(treeData.value);
          setTimeout(() => {
            checkedKeys.value = getAllKeys(treeData.value);
          }, 100);
        }
      } catch {
        // 角色无权限配置或超级管理员
        if (isSuperAdmin) {
          expandedKeys.value = getAllKeys(treeData.value);
          setTimeout(() => {
            checkedKeys.value = getAllKeys(treeData.value);
          }, 100);
        }
      }
    }
  },

  async onConfirm() {
    if (!data.value?.row?.id) {
      message.error('角色信息不存在');
      return;
    }
    const roleId = Number(data.value.row.id);
    const isSuperAdmin = roleId === 1 || data.value.row.roleKey === 'super_admin' || data.value.row.roleKey === 'admin';
    setLoading(true);
    try {
      // 保存菜单权限（超级管理员后端会直接返回成功，不做实际修改）
      let authId: string[] = [...checkedKeys.value];
      const halfChecked = treeRef.value?.getHalfCheckedKeys?.();
      if (halfChecked?.length) {
        authId = [...authId, ...halfChecked];
      }
      await updateRoleAuthApi(roleId, { authId });

      // 保存数据权限（通过角色更新接口）
      await updateRoleApi(roleId, {
        dataScope: dataScopeValue.value,
      });

      // 保存自定义数据权限的部门关联
      if (dataScopeValue.value === 2 && !isSuperAdmin) {
        await updateRoleDeptApi(roleId, deptCheckedKeys.value);
      }

      message.success($t('ui.notification.update_success'));
      drawerApi.close();
    } catch (err: any) {
      const errMsg = err?.response?.data?.msg || err?.message || $t('ui.notification.update_error');
      message.error(errMsg);
    } finally {
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="$t('page.system.role.button.auth')" width="720px">
    <div class="flex flex-col gap-4">
      <!-- 权限切换标签 -->
      <div class="flex gap-4 border-b border-gray-200 pb-3">
        <button
          :class="[
            'relative px-4 py-2 text-sm font-medium transition-colors',
            activeSection === 'menu'
              ? 'text-primary after:absolute after:bottom-[-3px] after:left-0 after:h-[2px] after:w-full after:bg-primary'
              : 'text-gray-500 hover:text-gray-700',
          ]"
          @click="activeSection = 'menu'"
        >
          <span class="mr-1.5">🔐</span> 菜单权限
        </button>
        <button
          :class="[
            'relative px-4 py-2 text-sm font-medium transition-colors',
            activeSection === 'dataScope'
              ? 'text-primary after:absolute after:bottom-[-3px] after:left-0 after:h-[2px] after:w-full after:bg-primary'
              : 'text-gray-500 hover:text-gray-700',
          ]"
          @click="activeSection = 'dataScope'"
        >
          <span class="mr-1.5">📊</span> 数据权限
        </button>
      </div>

      <!-- 菜单权限区域 -->
      <div v-show="activeSection === 'menu'">
        <div class="mb-3 text-xs text-gray-400">
          配置角色可访问的功能菜单，勾选的菜单及其子菜单将被授权
        </div>
        <Space class="mb-3">
          <Button size="small" @click="expandAll">{{ $t('ui.tree.expand_all') }}</Button>
          <Button size="small" @click="collapseAll">{{ $t('ui.tree.collapse_all') }}</Button>
          <Button size="small" type="primary" ghost @click="checkAll">{{ $t('ui.tree.select_all') }}</Button>
          <Button size="small" @click="uncheckAll">{{ $t('ui.tree.unselect_all') }}</Button>
        </Space>
        <div class="max-h-[420px] overflow-y-auto border border-gray-100 rounded-lg p-3">
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
            <template #title="{ data: item }">
              <span class="text-sm">{{ $t(item.meta?.name || item.name) }}</span>
            </template>
          </Tree>
          <div
            v-if="treeData.length === 0"
            class="py-12 text-center text-gray-400"
          >
            <Empty :description="'暂无菜单数据'" />
          </div>
        </div>
      </div>

      <!-- 数据权限区域 -->
      <div v-show="activeSection === 'dataScope'">
        <div class="mb-3 text-xs text-gray-400">
          配置角色可查看的业务数据范围，数据权限与菜单权限共同决定用户的最终可见范围
        </div>

        <Radio.Group
          v-model:value="dataScopeValue"
          class="w-full"
        >
          <div class="space-y-3">
            <div
              v-for="opt in dataScopeOptions"
              :key="opt.value"
              :class="[
                'relative flex items-start gap-3 rounded-lg border p-4 cursor-pointer transition-all',
                dataScopeValue === opt.value
                  ? 'border-primary bg-primary/5 shadow-sm'
                  : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50',
              ]"
              @click="dataScopeValue = opt.value"
            >
              <Radio :value="opt.value" class="mt-0.5" />
              <div class="flex-1">
                <div class="font-medium text-gray-800">{{ opt.label }}</div>
                <div class="mt-0.5 text-xs text-gray-500">{{ opt.desc }}</div>
              </div>
            </div>
          </div>
        </Radio.Group>

        <!-- 自定义数据权限：部门选择 -->
        <div v-if="showDeptTree" class="mt-4">
          <Divider class="my-3" />
          <div class="mb-2 text-sm font-medium text-gray-700">
            选择可见部门
          </div>
          <div class="max-h-[280px] overflow-y-auto border border-gray-100 rounded-lg p-3">
            <Tree
              ref="deptTreeRef"
              v-model:expandedKeys="deptExpandedKeys"
              v-model:checkedKeys="deptCheckedKeys"
              :tree-data="deptTreeData"
              checkable
              :check-strictly="false"
              default-expand-all
              class="w-full"
            />
            <div
              v-if="deptTreeData.length === 0"
              class="py-8 text-center text-gray-400"
            >
              <Empty :description="'暂无部门数据'" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Drawer>
</template>
