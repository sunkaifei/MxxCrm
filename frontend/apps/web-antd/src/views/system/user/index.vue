<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';

import type { VxeGridPropTypes } from '@vben/plugins/vxe-table';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Dropdown, Menu, Popconfirm, Progress, Tag, Tooltip } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteUserApi,
  getColumnsConfigApi,
  getUserListApi,
  kickOfflineApi,
  saveColumnsConfigApi,
  updateUserApi,
} from '#/api';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';
import { statusList } from '#/store';

import UserDetailDrawer from '../../crm/components/UserDetailDrawer.vue';
import ArchiveDrawer from './archive-drawer.vue';
import AuditDrawer from './audit-drawer.vue';
import ColumnsConfigDrawer from './columns-config-drawer.vue';
import SubmitAuditDrawer from './submit-audit-drawer.vue';
import UserDrawer from './drawer.vue';
import SendMailModal from './send-mail.vue';
import SendMessageModal from './send-message.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

const { isSuperAdmin } = useSuperAdminGuard();

// ---- 列显示权限级别 ----
// 判定顺序：admin > hr > manager > employee
// admin:   超管/系统管理员（super_admin / system_admin / user_type=1 / data_scope=1），全列
// hr:      有人事档案权限（system:hr-archive:view），全列
// manager: data_scope<=4（全部/自定义/本部门/本部门及以下），12列
// employee: data_scope=5（仅本人），8列
const viewerLevel = computed<'admin' | 'hr' | 'manager' | 'employee'>(() => {
  const roles = userStore.userInfo?.roles ?? [];
  if (
    isSuperAdmin.value ||
    roles.includes('super_admin') ||
    roles.includes('system_admin')
  ) {
    return 'admin';
  }
  if (accessStore.hasAccessCode('system:hr-archive:view')) {
    return 'hr';
  }
  const scope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  const dataScope = typeof scope === 'number' ? scope : 5;
  if (dataScope <= 4) {
    return 'manager';
  }
  return 'employee';
});

// ---- 列定义（key -> 列配置）----
const allColumns: Array<{
  key: string;
  column: NonNullable<VxeGridPropTypes.Columns>[number] & Record<string, any>;
}> = [
  { key: 'seq', column: { type: 'seq', width: 70 } },
  { key: 'userName', column: { field: 'userName', width: 140, slots: { default: 'userName' } } },
  { key: 'nickName', column: { field: 'nickName', width: 140, slots: { default: 'nickName' } } },
  { key: 'deptName', column: { field: 'deptName', width: 140 } },
  { key: 'postName', column: { field: 'postName', width: 140 } },
  { key: 'mobile', column: { field: 'mobile', width: 120 } },
  { key: 'email', column: { field: 'email', width: 180 } },
  { key: 'roleName', column: { field: 'roleName', width: 120, slots: { default: 'roleName' } } },
  { key: 'directManagerName', column: { field: 'directManagerName', width: 120 } },
  { key: 'status', column: { field: 'status', width: 80, slots: { default: 'status' } } },
  { key: 'online', column: { field: 'online', width: 80, slots: { default: 'online' } } },
  { key: 'auditStatus', column: { field: 'auditStatus', width: 90, slots: { default: 'auditStatus' } } },
  { key: 'lastLoginTime', column: { field: 'lastLoginTime', width: 140, slots: { default: 'lastLoginTime' } } },
  { key: 'lastLoginIp', column: { field: 'lastLoginIp', width: 120 } },
  { key: 'hireDate', column: { field: 'hireDate', width: 110 } },
  { key: 'filledDetail', column: { field: 'filledDetail', minWidth: 280, showOverflow: false, slots: { default: 'filledDetail' } } },
  { key: 'completeness', column: { field: 'completeness', width: 120, slots: { default: 'completeness' } } },
  { key: 'salaryEnabled', column: { field: 'salaryEnabled', width: 100, slots: { default: 'salaryEnabled' } } },
  { key: 'createTime', column: { field: 'createTime', width: 140, slots: { default: 'createdAt' } } },
  { key: 'action', column: { fixed: 'right', slots: { default: 'action' }, width: 220 } },
];

// 各角色默认可见列（与数据库初始化配置保持一致）
const defaultVisibleColumns: Record<string, string[]> = {
  admin: allColumns.map((c) => c.key),
  hr: allColumns.map((c) => c.key),
  manager: ['seq', 'userName', 'nickName', 'deptName', 'postName', 'mobile', 'email', 'roleName', 'directManagerName', 'status', 'online', 'auditStatus', 'action'],
  employee: ['seq', 'userName', 'nickName', 'deptName', 'postName', 'status', 'online', 'action'],
};

// 列 key -> 标题 i18n key 映射
const columnTitleMap: Record<string, string> = {
  seq: 'ui.table.seq',
  userName: 'page.system.user.username',
  nickName: 'page.system.user.nickName',
  deptName: 'page.system.user.dept',
  postName: 'page.system.user.post',
  mobile: 'page.system.user.mobile',
  email: 'page.system.user.email',
  roleName: 'page.system.user.role',
  directManagerName: 'page.system.user.directManager',
  status: 'ui.table.status',
  online: 'page.system.setting.online',
  auditStatus: 'page.system.user.auditStatus.label',
  lastLoginTime: 'page.system.user.lastLoginTime',
  lastLoginIp: 'page.system.user.lastLoginIp',
  hireDate: 'page.system.user.hireDate',
  filledDetail: 'page.system.hrArchive.filledDetail',
  completeness: 'page.system.hrArchive.completeness',
  salaryEnabled: 'page.system.user.salaryEnabled',
  createTime: 'ui.table.createTime',
  action: 'ui.table.action',
};

// 后台配置的列显示（四个角色的列数组）
const columnsConfig = ref<Record<string, string[]>>({});

// 当前角色可见的列 key 集合
const visibleKeys = computed(() => {
  const level = viewerLevel.value;
  const configured = columnsConfig.value[level];
  return configured && configured.length > 0 ? configured : defaultVisibleColumns[level];
});

// 动态列定义（含标题）
const visibleColumns = computed(() => {
  const visibleKeySet = new Set(visibleKeys.value);
  return allColumns
    .filter((c) => visibleKeySet.has(c.key))
    .map((c) => ({
      title: $t(columnTitleMap[c.key] ?? ''),
      ...c.column,
    }));
});

// 加载列显示配置
async function loadColumnsConfig() {
  try {
    const data = await getColumnsConfigApi();
    if (data && typeof data === 'object') {
      columnsConfig.value = data as Record<string, string[]>;
    }
  } catch {
    // 无配置或加载失败时使用默认配置
  }
}

// 保存列显示配置
async function handleSaveColumnsConfig(config: Record<string, string[]>) {
  await saveColumnsConfigApi(config);
  columnsConfig.value = config;
  window.$message.success($t('page.system.user.columnsConfig.saveSuccess'));
}

// 列配置抽屉
const [ColumnsConfigDrawerRef, columnsConfigApi] = useVbenDrawer({
  connectedComponent: ColumnsConfigDrawer,
  onClosed() {
    const data = columnsConfigApi.getData();
    if (data && data.savedConfig) {
      handleSaveColumnsConfig(data.savedConfig);
    }
  },
});

function openColumnsConfig() {
  columnsConfigApi.setData({
    config: columnsConfig.value,
    defaultConfig: defaultVisibleColumns,
    allColumns: allColumns.map((c) => ({
      key: c.key,
      title: $t(columnTitleMap[c.key] ?? ''),
    })),
    levels: [
      { key: 'admin', label: $t('page.system.user.columnsConfig.levelAdmin') },
      { key: 'hr', label: $t('page.system.user.columnsConfig.levelHr') },
      { key: 'manager', label: $t('page.system.user.columnsConfig.levelManager') },
      { key: 'employee', label: $t('page.system.user.columnsConfig.levelEmployee') },
    ],
  });
  columnsConfigApi.open();
}

const detailVisible = ref(false);
const detailUserId = ref<null | number | string>(null);

function openDetail(row: any) {
  detailUserId.value = row.id;
  detailVisible.value = true;
}

// 档案抽屉（HR 视角）
const archiveVisible = ref(false);
const archiveAdminId = ref<null | number>(null);

function openArchive(row: any) {
  archiveAdminId.value = row.id;
  archiveVisible.value = true;
}

// 六项完善明细展示配置（字段 -> i18n key）
const filledItems = [
  { field: 'idFilled', key: 'page.system.hrArchive.idCard' },
  { field: 'bankFilled', key: 'page.system.hrArchive.bankCard' },
  { field: 'emailFilled', key: 'page.system.profile.email' },
  { field: 'hireFilled', key: 'page.system.hrArchive.hireDate' },
  { field: 'resumeFilled', key: 'page.system.hrArchive.tabResume' },
  { field: 'contactFilled', key: 'page.system.hrArchive.tabContacts' },
] as const;

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'userName',
      label: $t('page.system.user.username'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('ui.table.status'),
      componentProps: {
        options: statusList,
        placeholder: $t('ui.placeholder.select'),
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  // height: 'auto' 让表格高度跟随内容自适应，CSS min-height: 200px 保证最少200px
  height: 'auto',
  scrollX: { enabled: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' as any },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getUserListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          userName: formValues.userName,
          status: formValues.status,
        });
        // DOM 更新后再同步一次固定列行高（覆盖字体/标签换行后高度计算延迟）
        requestAnimationFrame(() => {
          syncFixedRowHeights();
          setTimeout(syncFixedRowHeights, 150);
          setTimeout(syncFixedRowHeights, 400);
        });
        return result;
      },
    },
  },

  // 列由 visibleColumns computed 动态注入（见下方 watch + setGridOptions）
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 配置变化时同步列
watch(
  visibleColumns,
  (cols) => {
    gridApi.setGridOptions({ columns: cols });
  },
  { immediate: false },
);

// ---- 固定列行高自适应同步 ----
// vxe-table 未开启虚拟滚动时，固定列（右侧操作列）的行高不会跟随主表内容自适应。
// "完善情况"列标签换行导致主表行变高后，通过 ResizeObserver/MutationObserver
// 实时把主表行高同步到固定列（覆盖数据刷新、窗口/列宽变化、字体加载等场景）。
let rowResizeObserver: null | ResizeObserver = null;
let rowMutationObserver: MutationObserver | null = null;
let resizeHandler: (() => void) | null = null;

function syncFixedRowHeights() {
  const $el = gridApi.grid?.$el as HTMLElement | undefined;
  if (!$el) return;
  const mainBody = $el.querySelector('.vxe-table--body-wrapper tbody');
  if (!mainBody) return;
  const fixedBodies = $el.querySelectorAll(
    '.vxe-table--fixed-left-wrapper tbody, .vxe-table--fixed-right-wrapper tbody',
  );
  if (fixedBodies.length === 0) return;
  mainBody.querySelectorAll('tr.vxe-body--row').forEach((mainTr, i) => {
    const h = (mainTr as HTMLElement).offsetHeight;
    if (!h) return;
    const rowid = mainTr.getAttribute('rowid');
    fixedBodies.forEach((fixedBody) => {
      const fixedTr = rowid
        ? fixedBody.querySelector(`tr.vxe-body--row[rowid="${rowid}"]`)
        : fixedBody.querySelectorAll('tr.vxe-body--row')[i];
      if (!fixedTr) return;
      (fixedTr as HTMLElement).style.height = `${h}px`;
      fixedTr.querySelectorAll('td .vxe-cell').forEach((cell) => {
        const cellEl = cell as HTMLElement;
        cellEl.style.display = 'flex';
        cellEl.style.alignItems = 'center';
        cellEl.style.justifyContent = 'center';
        cellEl.style.height = `${h}px`;
      });
    });
  });
}

function setupRowHeightSync(attempt = 0) {
  const $el = gridApi.grid?.$el as HTMLElement | undefined;
  if (!$el) return;
  const mainBody = $el.querySelector(
    '.vxe-table--body-wrapper tbody',
  ) as HTMLElement | null;
  const fixedBody = $el.querySelector('.vxe-table--fixed-right-wrapper tbody');
  // scrollY 开启后，vxe 内部 body-wrapper 会独立滚动，需要在这里监听它
  const bodyWrapper = $el.querySelector(
    '.vxe-table--body-wrapper',
  ) as HTMLElement | null;
  if (!mainBody || !fixedBody) {
    if (attempt < 30) requestAnimationFrame(() => setupRowHeightSync(attempt + 1));
    return;
  }
  rowResizeObserver?.disconnect();
  rowMutationObserver?.disconnect();
  if (resizeHandler) {
    window.removeEventListener('resize', resizeHandler, true);
  }
  rowResizeObserver = new ResizeObserver(() => syncFixedRowHeights());
  const observeRows = () => {
    mainBody.querySelectorAll('tr.vxe-body--row').forEach((tr) => {
      rowResizeObserver?.observe(tr);
    });
  };
  observeRows();
  rowMutationObserver = new MutationObserver(() => {
    observeRows();
    syncFixedRowHeights();
  });
  rowMutationObserver.observe(mainBody, { childList: true });
  // 监听 body-wrapper / window 尺寸变化，覆盖 scrollY 开启后列对不齐
  resizeHandler = () => syncFixedRowHeights();
  window.addEventListener('resize', resizeHandler, true);
  if (bodyWrapper) {
    rowResizeObserver.observe(bodyWrapper);
    bodyWrapper.addEventListener('scroll', resizeHandler as any, {
      passive: true,
    });
  }
  requestAnimationFrame(() => syncFixedRowHeights());
}

onMounted(async () => {
  setupRowHeightSync();
  // 加载列显示配置并同步动态列
  await loadColumnsConfig();
  gridApi.setGridOptions({ columns: visibleColumns.value });
});

onBeforeUnmount(() => {
  rowResizeObserver?.disconnect();
  rowMutationObserver?.disconnect();
  if (resizeHandler) {
    window.removeEventListener('resize', resizeHandler, true);
    const bodyWrapper = gridApi.grid?.$el?.querySelector(
      '.vxe-table--body-wrapper',
    ) as HTMLElement | null | undefined;
    bodyWrapper?.removeEventListener('scroll', resizeHandler as any);
  }
  rowResizeObserver = null;
  rowMutationObserver = null;
  resizeHandler = null;
});

async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 0;
  try {
    await updateUserApi({ id: row.id, status: row.status });
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// 提交审核：打开「提交审核确认抽屉」（F2），流程预览 + 确认后提交
// 走系统审批流引擎（business_type=user），审批通过后自动启用用户
const submitDrawerVisible = ref(false);
const submitRow = ref<any>(undefined);

function handleSubmitAudit(row: any) {
  submitRow.value = row;
  submitDrawerVisible.value = true;
}

// 审核按钮：needAudit 由后端计算（当前登录人为当前节点审批人），点击打开审核抽屉（F3）
const auditDrawerVisible = ref(false);
const auditRow = ref<any>(undefined);
const auditInstanceId = ref<number | null>(null);

function handleAudit(row: any) {
  auditRow.value = row;
  auditInstanceId.value = row.approvalInstanceId ?? null;
  auditDrawerVisible.value = true;
}

// 重新提交：与首次提交共用提交抽屉（展示上次驳回/撤回/退回意见）
function handleResubmit(row: any) {
  submitRow.value = row;
  submitDrawerVisible.value = true;
}

// 离职申请（F4）：HR/管理员代发起离职审批（system:resign:save）
const resignApplyVisible = ref(false);
const resignApplyRow = ref<any>(undefined);

function handleResignApply(row: any) {
  resignApplyRow.value = row;
  resignApplyVisible.value = true;
}

// 交接单详情（F5）：按员工查看交接单（system:resign:view）
const resignDetailVisible = ref(false);
const resignDetailAdminId = ref<number | undefined>(undefined);

function handleResignDetail(row: any) {
  resignDetailAdminId.value = row.id;
  resignDetailVisible.value = true;
}

function handleResignChanged() {
  gridApi.query();
}

// 编辑/新建抽屉：与点击姓名打开的 UserDetailDrawer 一致，用 antd Drawer + v-model 控制（宽度 75% 由 drawer.vue 内 :width 控制）
const editVisible = ref(false);
const editCreate = ref(false);
const editRow = ref<any>(undefined);

function handleCreate() {
  editCreate.value = true;
  editRow.value = undefined;
  editVisible.value = true;
}

function handleEdit(row: any) {
  editCreate.value = false;
  editRow.value = row;
  editVisible.value = true;
}

function handleEdited() {
  gridApi.query();
}

// 发送邮件 / 站内消息：所有登录用户可用（不校验权限码）
const sendMailVisible = ref(false);
const sendMessageVisible = ref(false);
const sendTargetRow = ref<any>(undefined);

function handleSendMail(row: any) {
  sendTargetRow.value = row;
  sendMailVisible.value = true;
}

function handleSendMessage(row: any) {
  sendTargetRow.value = row;
  sendMessageVisible.value = true;
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteUserApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleKickOffline(row: any) {
  row.pending = true;
  try {
    await kickOfflineApi(row.id);
    window.$message.success($t('page.system.setting.kickSuccess'));
  } finally {
    row.pending = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid
      :table-title="$t('page.system.user.title')"
    >
      <template #toolbar-tools>
        <Button
          class="mr-2"
          v-access:code="['system:admin:save']"
          type="primary"
          @click="handleCreate"
        >
          {{ $t('page.system.user.button.create') }}
        </Button>
        <Button
          v-access:code="['system:admin:columns']"
          @click="openColumnsConfig"
        >
          {{ $t('page.system.user.button.columnsConfig') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #userName="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openDetail(row)"
          >{{ row.userName }}</a
        >
      </template>

      <template #nickName="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openDetail(row)"
          >{{ row.nickName }}</a
        >
      </template>

      <template #roleName="{ row }">
        <Tag color="success">
          {{ row.roleName }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="row.status === 1 ? 'success' : 'default'">
          {{ row.status === 1 ? $t('enum.status.ON') : $t('enum.status.OFF') }}
        </Tag>
      </template>

      <template #online="{ row }">
        <Tag :color="row.online ? 'success' : 'default'">
          {{
            row.online
              ? $t('page.system.setting.online')
              : $t('page.system.setting.offline')
          }}
        </Tag>
      </template>

      <template #salaryEnabled="{ row }">
        <Tag :color="row.salaryEnabled === 1 ? 'success' : 'default'">
          {{
            row.salaryEnabled === 1
              ? $t('page.system.user.salaryYes')
              : $t('page.system.user.salaryNo')
          }}
        </Tag>
      </template>

      <template #bizEnabled="{ row }">
        <Tag :color="row.bizEnabled === 1 ? 'success' : 'default'">
          {{
            row.bizEnabled === 1
              ? $t('page.system.user.bizYes')
              : $t('page.system.user.bizNo')
          }}
        </Tag>
      </template>

      <template #auditStatus="{ row }">
        <Tag v-if="row.auditStatus === 0" color="warning">
          {{ $t('page.system.user.auditStatus.pending') }}
        </Tag>
        <Tag v-else color="success">
          {{ $t('page.system.user.auditStatus.approved') }}
        </Tag>
      </template>

      <template #lastLoginTime="{ row }">
        {{ formatDateTime(row.lastLoginTime) }}
      </template>

      <template #filledDetail="{ row }">
        <div class="filled-badges">
          <Tooltip
            v-for="item in filledItems"
            :key="item.field"
            :title="$t(item.key) + (row[item.field] ? ` · ${$t('page.system.hrArchive.filled')}` : ` · ${$t('page.system.hrArchive.missing')}`)"
          >
            <span class="filled-badge" :class="[row[item.field] ? 'is-filled' : 'is-missing']">
              {{ $t(item.key) }}
            </span>
          </Tooltip>
        </div>
      </template>

      <template #completeness="{ row }">
        <Progress
          :percent="row.completeness"
          :status="row.completeness >= 80 ? 'success' : row.completeness >= 50 ? 'active' : 'exception'"
          size="small"
        />
      </template>

      <template #action="{ row }">
        <!-- 审核中且当前登录人为当前节点审批人：审核按钮（needAudit 由后端计算，数据驱动，不依赖 system:admin:audit） -->
        <Button
          v-if="row.needAudit"
          type="link"
          class="!px-0 mr-2.5"
          :loading="row.pending"
          @click="() => handleAudit(row)"
        >
          {{ $t('page.system.user.button.audit') }}
        </Button>
        <!-- 未提交（auditStatus=0 且无任何审批实例）：提交审核 -->
        <Button
          v-else-if="
            row.auditStatus === 0 &&
            row.userType !== 1 &&
            row.approvalStatus == null &&
            accessStore.hasAccessCode('system:admin:audit')
          "
          type="link"
          class="!px-0 mr-2.5"
          :loading="row.pending"
          @click="() => handleSubmitAudit(row)"
        >
          {{ $t('page.system.user.button.submitAudit') }}
        </Button>
        <!-- 已驳回/已撤回/待修改（auditStatus=0 且最近实例∈{4,5,6}）：重新提交 -->
        <Button
          v-else-if="
            row.auditStatus === 0 &&
            row.userType !== 1 &&
            [4, 5, 6].includes(row.approvalStatus) &&
            accessStore.hasAccessCode('system:admin:audit')
          "
          type="link"
          class="!px-0 mr-2.5"
          :loading="row.pending"
          @click="() => handleResubmit(row)"
        >
          {{ $t('page.system.user.button.resubmitAudit') }}
        </Button>
        <!-- 档案（HR 视角：完善度/解锁/代改留痕） -->
        <Button
          v-if="accessStore.hasAccessCode('system:hr-archive:view')"
          type="link"
          class="!px-0 mr-2.5"
          @click="() => openArchive(row)"
        >
          {{ $t('page.system.hrArchive.archiveBtn') }}
        </Button>

        <!-- 更多操作下拉菜单：停用/启用、下线、编辑、删除 -->
        <Dropdown placement="bottomRight">
          <Button type="link">{{ $t('page.system.user.button.more') }}</Button>
          <template #overlay>
            <Menu>
              <Menu.Item
                v-if="
                  row.userType !== 1 &&
                  accessStore.hasAccessCode('system:admin:update')
                "
                key="toggleStatus"
                @click="() => handleStatusChanged(row, row.status !== 1)"
              >
                {{
                  row.status === 1
                    ? $t('page.system.user.button.disable')
                    : $t('page.system.user.button.enable')
                }}
              </Menu.Item>
              <Popconfirm
                :title="$t('page.system.setting.kickConfirm')"
                :ok-text="$t('ui.button.ok')"
                :cancel-text="$t('ui.button.cancel')"
                @confirm="() => handleKickOffline(row)"
              >
                <Menu.Item
                  v-if="accessStore.hasAccessCode('system:admin:kick')"
                  key="kick"
                  :disabled="row.userType === 1"
                >
                  {{ $t('page.system.user.button.kickOffline') }}
                </Menu.Item>
              </Popconfirm>
              <Menu.Item
                v-if="accessStore.hasAccessCode('system:admin:update')"
                key="edit"
                :disabled="[1, 2].includes(row.approvalStatus)"
                :title="
                  [1, 2].includes(row.approvalStatus)
                    ? '审核进行中，档案已锁定'
                    : undefined
                "
                @click="() => handleEdit(row)"
              >
                {{ $t('page.system.user.button.editAction') }}
              </Menu.Item>
              <!-- 离职流程：申请发起（system:resign:save）/ 交接单查看（system:resign:view） -->
              <Menu.Item
                v-if="accessStore.hasAccessCode('system:resign:save')"
                key="resignApply"
                @click="() => handleResignApply(row)"
              >
                离职申请
              </Menu.Item>
              <Menu.Item
                v-if="accessStore.hasAccessCode('system:resign:view')"
                key="resignDetail"
                @click="() => handleResignDetail(row)"
              >
                交接单
              </Menu.Item>
              <!-- 发送邮件 / 站内消息：所有登录用户可用 -->
              <Menu.Item key="sendMail" @click="() => handleSendMail(row)">
                {{ $t('page.system.user.button.sendMail') }}
              </Menu.Item>
              <Menu.Item
                key="sendMessage"
                @click="() => handleSendMessage(row)"
              >
                {{ $t('page.system.user.button.sendMessage') }}
              </Menu.Item>
              <Popconfirm
                :title="
                  $t('ui.text.do_you_want_delete', {
                    moduleName: $t('page.system.user.module'),
                  })
                "
                :ok-text="$t('ui.button.ok')"
                :cancel-text="$t('ui.button.cancel')"
                @confirm="() => handleDelete(row)"
              >
                <Menu.Item
                  v-if="
                    row.userType !== 1 &&
                    accessStore.hasAccessCode('system:admin:delete')
                  "
                  key="delete"
                  danger
                >
                  {{ $t('page.system.user.button.deleteAction') }}
                </Menu.Item>
              </Popconfirm>
            </Menu>
          </template>
        </Dropdown>
      </template>
    </Grid>
    <UserDrawer
      v-model:open="editVisible"
      :create="editCreate"
      :row="editRow"
      @saved="handleEdited"
    />
    <SendMailModal v-model:open="sendMailVisible" :row="sendTargetRow" />
    <SendMessageModal
      v-model:open="sendMessageVisible"
      :row="sendTargetRow"
    />
    <ColumnsConfigDrawerRef />
    <UserDetailDrawer
      v-model:visible="detailVisible"
      :id="detailUserId ?? undefined"
    />
    <ArchiveDrawer v-model:open="archiveVisible" :admin-id="archiveAdminId" />
    <SubmitAuditDrawer
      v-model:visible="submitDrawerVisible"
      :row="submitRow"
      @success="handleEdited"
    />
    <AuditDrawer
      v-model:visible="auditDrawerVisible"
      :row="auditRow"
      :instance-id="auditInstanceId"
      @success="handleEdited"
    />
    <ResignDrawer
      v-model:visible="resignApplyVisible"
      :row="resignApplyRow"
      @success="handleResignChanged"
    />
    <ResignDetailDrawer
      v-model:visible="resignDetailVisible"
      :admin-id="resignDetailAdminId"
      @success="handleResignChanged"
    />
  </Page>
</template>

<style scoped>
/* 表格最少200px高度，记录少时显示200px，记录多时跟随内容撑高 */
:deep(.vxe-grid) {
  min-height: 200px;
}

/* 行高自适应：解除 vxe-table 对 body 单元格的固定高度约束，
   让 "完善情况" 列的标签换行后，行高可以真实地跟随内容变为两行甚至多行。 */
:deep(.vxe-table--main-wrapper .vxe-table--body-wrapper .vxe-body--column .vxe-cell),
:deep(.vxe-table--body-wrapper .vxe-body--column .vxe-cell) {
  height: auto !important;
  min-height: 44px;
  padding-top: 6px !important;
  padding-bottom: 6px !important;
  line-height: 20px;
  box-sizing: border-box;
  display: block;
  align-items: normal;
  justify-content: normal;
}

/* 主体单元格默认垂直居中，但允许内容自然撑开（比如两行标签） */
:deep(.vxe-table--body-wrapper td.vxe-body--column) {
  vertical-align: middle;
}

/* 固定列内容垂直居中，并跟随同步后的行高 */
:deep(.vxe-table--fixed-right-wrapper .vxe-body--column .vxe-cell) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  height: 100% !important;
  min-height: 0;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

:deep(.vxe-table--empty-block) {
  min-height: 150px;
}

.filled-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 6px;
  align-content: center;
  line-height: 1;
}

.filled-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  font-size: 12px;
  line-height: 18px;
  height: 22px;
  cursor: default;
  border-radius: 3px;
  box-sizing: border-box;
  white-space: nowrap;
}

.filled-badge.is-filled {
  color: #389e0d;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
}

.filled-badge.is-missing {
  color: rgb(0 0 0 / 35%);
  background: rgb(0 0 0 / 4%);
  border: 1px dashed rgb(0 0 0 / 15%);
  text-decoration: line-through;
}
</style>
