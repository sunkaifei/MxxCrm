<script lang="ts" setup>
import { computed, nextTick, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Button,
  Divider,
  Empty,
  message,
  Modal,
  Radio,
  Space,
  Tree,
} from 'ant-design-vue';

import {
  getDeptTreeApi,
  getMenuOptionsApi,
  getRoleDeptIdsApi,
  getRoleInfoApi,
  getRoleMenuIdsApi,
  updateRoleApi,
  updateRoleAuthApi,
  updateRoleDeptApi,
} from '#/api';
import { $t } from '#/locales';

const data = ref();
const activeSection = ref<'dataScope' | 'menu'>('menu');

// ---------- 菜单权限 ----------
const treeData = ref<any[]>([]);
const expandedKeys = ref<string[]>([]);
const checkedKeys = ref<string[]>([]);

const fieldNames = {
  children: 'children',
  title: 'name',
  key: 'id',
};

const getAllKeys = (data: any[]): string[] => {
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

  const brokenItems: Array<{ name: string; missingParents: string[] }> = [];
  const missingIds = new Set<string>();
  const traverse = (nodes: any[], uncheckedAncestors: string[]) => {
    nodes.forEach((node: any) => {
      const key = String(node.id);
      const isChecked = checkedSet.has(key);
      if (isChecked && uncheckedAncestors.length > 0) {
        brokenItems.push({
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
  return { missingIds, brokenItems };
};

// ---------- 数据权限 ----------
const dataScopeValue = ref<number>(5); // 默认仅本人数据
const deptTreeData = ref<any[]>([]);
const deptCheckedKeys = ref<string[]>([]);
const deptExpandedKeys = ref<string[]>([]);

const dataScopeOptions = [
  {
    value: 1,
    label: '全部数据',
    desc: '可查看系统中所有业务数据，不受部门和个人限制',
  },
  { value: 2, label: '自定义数据', desc: '可查看指定部门及以下的业务数据' },
  {
    value: 3,
    label: '本部门数据',
    desc: '只能查看所在部门所有成员负责的业务数据',
  },
  {
    value: 4,
    label: '本部门及以下',
    desc: '可查看本部门及下属部门所有成员的业务数据',
  },
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
    const children =
      node.children && node.children.length > 0
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
// 加载序号：快速切换角色时，上一次打开遗留的异步回调不得覆盖新角色的数据（竞态防护）
let loadSeq = 0;

const [Drawer, drawerApi] = useVbenDrawer({
  async onOpened() {
    const seq = ++loadSeq;
    data.value = drawerApi.getData<Record<string, any>>();
    activeSection.value = 'menu';
    // 每次打开都重置展示状态，避免残留上一个角色的数据
    checkedKeys.value = [];
    expandedKeys.value = [];
    brokenVisible.value = false;
    deptCheckedKeys.value = [];
    dataScopeValue.value = 5;
    treeData.value = [];
    deptTreeData.value = [];

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

    // 加载部门树
    const deptResult = await getDeptTreeApi();
    if (seq !== loadSeq) return;
    const deptList = Array.isArray(deptResult)
      ? deptResult
      : deptResult?.data || [];
    deptTreeData.value = buildDeptTreeData(deptList);
    deptExpandedKeys.value = getAllDeptKeys(deptTreeData.value);

    if (data.value?.row?.id) {
      const roleId = Number(data.value.row.id);
      const isSuperAdmin =
        roleId === 1 ||
        data.value.row.roleKey === 'super_admin' ||
        data.value.row.roleKey === 'admin';

      // 从API加载角色详情获取最新的dataScope
      try {
        const roleDetail = await getRoleInfoApi(roleId);
        if (seq !== loadSeq) return;
        if (roleDetail) {
          dataScopeValue.value = roleDetail.dataScope ?? (isSuperAdmin ? 1 : 5);
          if (roleDetail.deptIds && roleDetail.deptIds.length > 0) {
            deptCheckedKeys.value = roleDetail.deptIds.map(String);
          }
        }
      } catch {
        if (seq !== loadSeq) return;
        dataScopeValue.value = isSuperAdmin
          ? 1
          : (data.value.row.dataScope ?? 5);
      }

      // 超级管理员默认全部数据权限
      if (isSuperAdmin && !dataScopeValue.value) {
        dataScopeValue.value = 1;
      }

      // 如果是自定义数据权限，加载角色关联的部门ID
      if (dataScopeValue.value === 2 && !isSuperAdmin) {
        try {
          const roleDeptIds = await getRoleDeptIdsApi(roleId);
          if (seq !== loadSeq) return;
          if (roleDeptIds) {
            deptCheckedKeys.value = (
              Array.isArray(roleDeptIds) ? roleDeptIds : []
            ).map(String);
          }
        } catch {
          // ignore
        }
      }

      try {
        // 加载已有的菜单权限
        const roleMenuIds = await getRoleMenuIdsApi(roleId);
        if (seq !== loadSeq) return;
        await nextTick();

        if (roleMenuIds) {
          const treeIds = getAllKeys(treeData.value);
          const validMenuIds = (Array.isArray(roleMenuIds) ? roleMenuIds : [])
            .map(String)
            .filter((id: string) => treeIds.includes(id));

          checkedKeys.value = validMenuIds;
          expandedKeys.value = treeIds;
        }

        // 超级管理员：默认全选所有菜单
        if (isSuperAdmin) {
          expandedKeys.value = getAllKeys(treeData.value);
          checkedKeys.value = getAllKeys(treeData.value);
        }
      } catch {
        // 角色无权限配置或超级管理员
        if (seq !== loadSeq) return;
        if (isSuperAdmin) {
          expandedKeys.value = getAllKeys(treeData.value);
          checkedKeys.value = getAllKeys(treeData.value);
        }
      }
    }
  },

  async onConfirm() {
    if (!data.value?.row?.id) {
      message.error('角色信息不存在');
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
    message.error('角色信息不存在');
    return;
  }
  const roleId = Number(data.value.row.id);
  const isSuperAdmin =
    roleId === 1 ||
    data.value.row.roleKey === 'super_admin' ||
    data.value.row.roleKey === 'admin';
  setLoading(true);
  try {
    // 保存菜单权限（超级管理员后端会直接返回成功，不做实际修改）
    // 后端会再次校验链路完整性：父级未勾选的菜单不保存
    const authId = [...new Set([...currentCheckedIds(), ...extraIds])];
    await updateRoleAuthApi(roleId, { authId });

    // 保存数据权限（通过角色更新接口）
    await updateRoleApi(roleId, {
      dataScope: dataScopeValue.value,
    });

    // 保存自定义数据权限的部门关联
    if (dataScopeValue.value === 2 && !isSuperAdmin) {
      await updateRoleDeptApi(roleId, deptCheckedKeys.value);
    }

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
  <Drawer :title="$t('page.system.role.button.auth')" width="720px">
    <div class="flex flex-col gap-4">
      <!-- 权限切换标签 -->
      <div class="flex gap-4 border-b border-gray-200 pb-3">
        <button
          class="relative px-4 py-2 text-sm font-medium transition-colors"
          :class="[
            activeSection === 'menu'
              ? 'text-primary after:absolute after:bottom-[-3px] after:left-0 after:h-[2px] after:w-full after:bg-primary'
              : 'text-gray-500 hover:text-gray-700',
          ]"
          @click="activeSection = 'menu'"
        >
          <span class="mr-1.5">🔐</span> 菜单权限
        </button>
        <button
          class="relative px-4 py-2 text-sm font-medium transition-colors"
          :class="[
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
            <Empty description="暂无菜单数据" />
          </div>
        </div>
      </div>

      <!-- 断链勾选确认弹窗 -->
      <Modal
        v-model:open="brokenVisible"
        title="存在父级未勾选的菜单"
        :footer="null"
        width="520px"
      >
        <div class="text-sm">
          检测到以下 {{ brokenItems.length }} 个菜单的父级未勾选，菜单权限需保持从根到叶的完整链路，直接保存后这些菜单将不可见：
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
              （父级未勾选：{{ item.missingParents.join('、') }}）
            </span>
          </li>
        </ul>
        <div class="mt-4 flex justify-end gap-2">
          <Button @click="brokenVisible = false">返回修改</Button>
          <Button @click="doSaveAuth()">仍按当前勾选保存</Button>
          <Button type="primary" @click="doSaveAuth(brokenMissingIds)">
            自动勾选父级并保存
          </Button>
        </div>
      </Modal>

      <!-- 数据权限区域 -->
      <div v-show="activeSection === 'dataScope'">
        <div class="mb-3 text-xs text-gray-400">
          配置角色可查看的业务数据范围，数据权限与菜单权限共同决定用户的最终可见范围
        </div>

        <Radio.Group v-model:value="dataScopeValue" class="w-full">
          <div class="space-y-3">
            <div
              v-for="opt in dataScopeOptions"
              :key="opt.value"
              class="relative flex items-start gap-3 rounded-lg border p-4 cursor-pointer transition-all"
              :class="[
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
          <div class="mb-2 text-sm font-medium text-gray-700">选择可见部门</div>
          <div
            class="max-h-[280px] overflow-y-auto border border-gray-100 rounded-lg p-3"
          >
            <Tree
              v-model:expanded-keys="deptExpandedKeys"
              v-model:checked-keys="deptCheckedKeys"
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
              <Empty description="暂无部门数据" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Drawer>
</template>
