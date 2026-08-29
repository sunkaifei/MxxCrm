<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import { Empty, Table, Tag, Tree } from 'ant-design-vue';

import {
  getDeptTreeApi,
  getMenuOptionsApi,
  getPermSetMenuIdsApi,
  getRoleInfoApi,
  getRoleMenuIdsApi,
  getUserListApi,
} from '#/api';
import { $t } from '#/locales';

// P3-2 权限预览面板：纯前端拼装现有接口，零后端改动
// mode=role：菜单权限（只读勾选态）+ 数据范围人话文案 + 成员列表 三区块
// mode=permSet：仅菜单权限区块（权限集无数据范围与成员概念）
type PreviewMode = 'permSet' | 'role';
const data = ref<{ mode: PreviewMode; row: any }>();

// 加载序号：快速切换预览对象时，上一次遗留的异步回调不得覆盖新数据（竞态防护）
let loadSeq = 0;

const isRoleMode = computed(() => (data.value?.mode ?? 'role') === 'role');

// ---------- 菜单权限（只读勾选态） ----------
const menuTreeData = ref<any[]>([]);
const menuCheckedKeys = ref<string[]>([]);
const menuExpandedKeys = ref<string[]>([]);
const fieldNames = {
  children: 'children',
  title: 'name',
  key: 'id',
};

// 超级管理员：与 set-auth.vue 口径一致，展示为全选
const isSuperAdmin = computed(() => {
  const row = data.value?.row;
  return (
    Number(row?.id) === 1 ||
    row?.roleKey === 'super_admin' ||
    row?.roleKey === 'admin'
  );
});

// 菜单名 i18n（与 set-auth.vue 保持一致：目录级 key 带 .title fallback）
const translateName = (key?: null | string): string => {
  if (!key) return '';
  // 已是翻译后的中文（如"概览"）直接返回，避免对非 i18n key 二次翻译触发 intlify 警告
  if (!/^[a-zA-Z][\w-]*(\.[a-zA-Z][\w-]*)+$/.test(key)) return key;
  const direct = $t(key);
  if (direct !== key && !direct.startsWith('[object ')) return direct;
  const withTitle = $t(`${key}.title`);
  return withTitle === `${key}.title` ? key : withTitle;
};

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

const collectIds = (nodes: any[]): string[] => {
  const keys: string[] = [];
  const walk = (list: any[]) => {
    list.forEach((node) => {
      if (node.id !== undefined && node.id !== null) {
        keys.push(String(node.id));
      }
      if (node.children?.length) {
        walk(node.children);
      }
    });
  };
  walk(nodes);
  return keys;
};

// 只读：所有节点置 disabled，勾选态仅作展示不可修改
const markDisabled = (nodes: any[]): any[] => {
  return nodes.map((node) => ({
    ...node,
    disabled: true,
    children: node.children?.length ? markDisabled(node.children) : undefined,
  }));
};

// ---------- 数据范围（人话文案 + 人数估算） ----------
const dataScopeValue = ref<number>(5);
const customDeptIds = ref<string[]>([]);
const allUsers = ref<any[]>([]);
const userTotal = ref(0);

// 五档文案与 set-auth.vue 的 dataScopeOptions 一一对应（验收要点）
const dataScopeOptions: Record<number, { desc: string; label: string }> = {
  1: { label: '全部数据', desc: '可查看系统中所有业务数据，不受部门和个人限制' },
  2: { label: '自定义数据', desc: '可查看指定部门及以下的业务数据' },
  3: { label: '本部门数据', desc: '只能查看所在部门所有成员负责的业务数据' },
  4: { label: '本部门及以下', desc: '可查看本部门及下属部门所有成员的业务数据' },
  5: { label: '仅本人数据', desc: '只能查看自己负责的业务数据' },
};
const scopeOption = computed(
  () => dataScopeOptions[dataScopeValue.value] ?? dataScopeOptions[5]!,
);

// ---------- 部门树索引（id↔name、父子关系，用于人数估算） ----------
const deptNameById = new Map<string, string>();
const deptChildIds = new Map<string, string[]>();
const deptIdsByName = new Map<string, string[]>();

const indexDeptTree = (nodes: any[]) => {
  deptNameById.clear();
  deptChildIds.clear();
  deptIdsByName.clear();
  const walk = (list: any[]) => {
    list.forEach((n) => {
      const id = String(n.value);
      const name = String(n.label || '');
      deptNameById.set(id, name);
      if (!deptIdsByName.has(name)) deptIdsByName.set(name, []);
      deptIdsByName.get(name)!.push(id);
      const children: string[] = [];
      deptChildIds.set(id, children);
      if (n.children?.length) {
        n.children.forEach((c: any) => children.push(String(c.value)));
        walk(n.children);
      }
    });
  };
  walk(nodes);
};

// 按部门 id 集合展开全部后代 id（"及以下"口径）
const expandWithDescendants = (ids: string[]): Set<string> => {
  const result = new Set<string>();
  const stack = [...ids];
  while (stack.length > 0) {
    const id = stack.pop()!;
    if (result.has(id)) continue;
    result.add(id);
    (deptChildIds.get(id) || []).forEach((child) => stack.push(child));
  }
  return result;
};

// 自定义数据权限：已选部门名称（仅展示用户勾选的部门本身，不含展开的下级）
const customDeptNames = computed(() =>
  customDeptIds.value
    .map((id) => deptNameById.get(id))
    .filter((n): n is string => !!n),
);

// ---------- 成员列表 ----------
// 用户部门名集合（depts 数组优先，deptName 逗号分隔串兜底）
const userDeptNames = (u: any): string[] => {
  if (Array.isArray(u?.depts) && u.depts.length > 0) {
    return u.depts.map((d: any) => d?.deptName).filter(Boolean);
  }
  return String(u?.deptName || '')
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
};

// 用户角色名集合（roles 数组优先，roleName 逗号分隔串兜底）
const userRoleNames = (u: any): string[] => {
  if (Array.isArray(u?.roles) && u.roles.length > 0) {
    return u.roles.map((r: any) => r?.roleName).filter(Boolean);
  }
  return String(u?.roleName || '')
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
};

const roleName = computed(() => data.value?.row?.roleName ?? '');

// 成员：按角色名称匹配用户列表（角色名全局唯一）
const members = computed(() => {
  const name = roleName.value;
  if (!name) return [];
  return allUsers.value.filter((u) => userRoleNames(u).includes(name));
});

const memberPreviewLimit = 20;
const memberPreview = computed(() => members.value.slice(0, memberPreviewLimit));

const memberColumns = [
  { dataIndex: 'nickName', key: 'nickName', title: '姓名', width: 100 },
  { dataIndex: 'userName', key: 'userName', title: '账号', width: 130 },
  { key: 'deptName', title: '部门' },
  { key: 'status', title: '状态', width: 70 },
];

// 部门名集合 → 员工数（按部门名匹配；列表未拉全时为近似值，故文案用"约"）
const countUsersInDeptNames = (names: Set<string>): number => {
  if (names.size === 0) return 0;
  return allUsers.value.filter((u) =>
    userDeptNames(u).some((n) => names.has(n)),
  ).length;
};

// 数据范围人话明细（区块二的补充行）
const scopeDetail = computed(() => {
  if (!isRoleMode.value) return '';
  const scope = dataScopeValue.value;
  if (scope === 1) {
    return userTotal.value > 0
      ? `全部 ${userTotal.value} 名员工均在可见范围内`
      : '';
  }
  if (scope === 2) {
    if (customDeptIds.value.length === 0) return '尚未选择可见部门';
    const names = [...expandWithDescendants(customDeptIds.value)]
      .map((id) => deptNameById.get(id))
      .filter((n): n is string => !!n);
    const count = countUsersInDeptNames(new Set(names));
    return `已选 ${customDeptIds.value.length} 个部门（含下级），约 ${count} 人可见`;
  }
  if (scope === 3 || scope === 4) {
    const memberDeptSet = new Set(
      members.value.flatMap((m) => userDeptNames(m)),
    );
    if (memberDeptSet.size === 0) {
      return '当前角色暂无成员，无法估算可见人数';
    }
    if (scope === 3) {
      const count = countUsersInDeptNames(memberDeptSet);
      return `约 ${count} 人可见（按成员所在部门估算）`;
    }
    // 本部门及以下：成员部门展开全部下级部门
    const ids = [...memberDeptSet].flatMap(
      (name) => deptIdsByName.get(name) || [],
    );
    const names = [...expandWithDescendants(ids)]
      .map((id) => deptNameById.get(id))
      .filter((n): n is string => !!n);
    const count = countUsersInDeptNames(new Set(names));
    return `约 ${count} 人可见（按成员所在部门及下级估算）`;
  }
  if (scope === 5) {
    return '每位成员只能看到自己负责的业务数据';
  }
  return '';
});

// ---------- Drawer 生命周期 ----------
const [Drawer, drawerApi] = useVbenDrawer({
  // 预览仅展示，隐藏底部确认/取消
  footer: false,
  async onOpened() {
    const seq = ++loadSeq;
    data.value = drawerApi.getData<{ mode: PreviewMode; row: any }>();
    // 每次打开都重置展示状态，避免残留上一次的数据
    menuTreeData.value = [];
    menuCheckedKeys.value = [];
    menuExpandedKeys.value = [];
    customDeptIds.value = [];
    allUsers.value = [];
    userTotal.value = 0;
    dataScopeValue.value = 5;
    drawerApi.setState({ loading: true });
    try {
      const mode = data.value?.mode ?? 'role';
      const id = Number(data.value?.row?.id);

      // 区块一：菜单树（当前用户可授权的菜单，后端已按权限过滤）
      const menuList = await getMenuOptionsApi();
      if (seq !== loadSeq) return;
      menuTreeData.value = markDisabled(translateMenu(menuList || []));
      menuExpandedKeys.value = collectIds(menuTreeData.value);

      if (Number.isFinite(id) && id > 0) {
        if (isSuperAdmin.value) {
          menuCheckedKeys.value = collectIds(menuTreeData.value);
        } else {
          try {
            const ids =
              mode === 'permSet'
                ? await getPermSetMenuIdsApi(id)
                : await getRoleMenuIdsApi(id);
            if (seq !== loadSeq) return;
            const valid = new Set(collectIds(menuTreeData.value));
            menuCheckedKeys.value = (Array.isArray(ids) ? ids : [])
              .map(String)
              .filter((x) => valid.has(x));
          } catch {
            // 角色无权限配置时忽略
          }
        }
      }

      // 区块二/三（仅角色模式）：数据范围 + 成员列表
      if (mode === 'role' && Number.isFinite(id) && id > 0) {
        try {
          const detail = await getRoleInfoApi(id);
          if (seq !== loadSeq) return;
          if (detail) {
            dataScopeValue.value = detail.dataScope ?? (isSuperAdmin.value ? 1 : 5);
            if (Array.isArray(detail.deptIds)) {
              customDeptIds.value = detail.deptIds.map(String);
            }
          }
        } catch {
          // 详情获取失败时按默认口径展示
        }

        try {
          const deptResult = await getDeptTreeApi();
          if (seq !== loadSeq) return;
          const deptList = Array.isArray(deptResult)
            ? deptResult
            : deptResult?.data || [];
          indexDeptTree(deptList);
        } catch {
          // 部门树获取失败时人数估算展示为空
        }

        try {
          const userResult = await getUserListApi({ page: 1, pageSize: 999 });
          if (seq !== loadSeq) return;
          allUsers.value = userResult?.items ?? [];
          userTotal.value = Number(userResult?.total ?? allUsers.value.length);
        } catch {
          // 用户列表获取失败时成员列表展示为空
        }
      }
    } finally {
      if (seq === loadSeq) {
        drawerApi.setState({ loading: false });
      }
    }
  },
});
</script>

<template>
  <Drawer title="权限预览" width="640px">
    <div class="flex flex-col gap-5">
      <!-- 基本信息 -->
      <div class="flex flex-wrap items-center gap-2">
        <span class="text-base font-medium">
          {{ data?.row?.roleName || data?.row?.permSetName || '' }}
        </span>
        <Tag>{{ data?.row?.roleKey || data?.row?.permSetKey || '' }}</Tag>
        <Tag :color="Number(data?.row?.status) === 1 ? 'success' : 'default'">
          {{ Number(data?.row?.status) === 1 ? $t('enum.status.ON') : $t('enum.status.OFF') }}
        </Tag>
      </div>

      <!-- 区块一：菜单权限（只读勾选态） -->
      <section>
        <div class="mb-2 flex items-baseline justify-between">
          <h3 class="text-sm font-medium">菜单权限</h3>
          <span class="text-xs text-gray-400">
            共勾选 {{ menuCheckedKeys.length }} 项（只读）
          </span>
        </div>
        <div v-if="isSuperAdmin" class="mb-2 text-xs text-gray-500">
          超级管理员拥有全部菜单权限
        </div>
        <div
          class="max-h-[300px] overflow-y-auto rounded-lg border border-gray-100 p-3"
        >
          <Tree
            v-model:expanded-keys="menuExpandedKeys"
            :checked-keys="menuCheckedKeys"
            :tree-data="menuTreeData"
            checkable
            :check-strictly="true"
            :field-names="fieldNames"
            :selectable="false"
            class="w-full"
          >
            <template #title="{ data: item }">
              <span class="text-sm">{{ item.name }}</span>
            </template>
          </Tree>
          <div
            v-if="menuTreeData.length === 0"
            class="py-8 text-center text-gray-400"
          >
            <Empty description="暂无菜单数据" />
          </div>
        </div>
      </section>

      <!-- 区块二：数据范围（仅角色模式） -->
      <section v-if="isRoleMode">
        <h3 class="mb-2 text-sm font-medium">数据范围</h3>
        <div class="rounded-lg border border-gray-100 p-3">
          <div class="font-medium text-gray-800">{{ scopeOption.label }}</div>
          <div class="mt-0.5 text-xs text-gray-500">{{ scopeOption.desc }}</div>
          <div v-if="scopeDetail" class="mt-1.5 text-xs text-primary">
            {{ scopeDetail }}
          </div>
          <div
            v-if="Number(dataScopeValue) === 2 && customDeptNames.length"
            class="mt-2 flex flex-wrap gap-1"
          >
            <Tag v-for="name in customDeptNames.slice(0, 10)" :key="name">
              {{ name }}
            </Tag>
            <Tag v-if="customDeptNames.length > 10">
              +{{ customDeptNames.length - 10 }}
            </Tag>
          </div>
        </div>
      </section>

      <!-- 区块三：成员列表（仅角色模式） -->
      <section v-if="isRoleMode">
        <div class="mb-2 flex items-baseline justify-between">
          <h3 class="text-sm font-medium">成员列表</h3>
          <span class="text-xs text-gray-400">共 {{ members.length }} 名成员</span>
        </div>
        <div
          v-if="memberPreview.length > 0"
          class="max-h-[260px] overflow-y-auto rounded-lg border border-gray-100"
        >
          <Table
            :columns="memberColumns"
            :data-source="memberPreview"
            :pagination="false"
            size="small"
            row-key="id"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'deptName'">
                {{ userDeptNames(record).join('、') || '-' }}
              </template>
              <template v-else-if="column.key === 'status'">
                <Tag :color="Number(record.status) === 1 ? 'success' : 'default'">
                  {{ Number(record.status) === 1 ? $t('enum.status.ON') : $t('enum.status.OFF') }}
                </Tag>
              </template>
            </template>
          </Table>
        </div>
        <div
          v-else
          class="rounded-lg border border-gray-100 py-8 text-center text-gray-400"
        >
          <Empty description="暂无成员" />
        </div>
        <div v-if="members.length > memberPreviewLimit" class="mt-1 text-xs text-gray-400">
          仅显示前 {{ memberPreviewLimit }} 名
        </div>
      </section>
    </div>
  </Drawer>
</template>
