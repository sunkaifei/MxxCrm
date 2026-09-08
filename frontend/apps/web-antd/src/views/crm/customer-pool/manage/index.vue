<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Drawer,
  Form,
  Input,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Select,
  Switch,
  TabPane,
  Tabs,
  Table,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  addPoolMembersApi,
  auditCustomerPoolApplyApi,
  deleteCustomerPoolAssignRuleApi,
  deleteCustomerPoolDupRuleApi,
  auditCustomerPoolRetainApi,
  createPoolApi,
  deletePoolApi,
  getCustomerPoolApplyAuditPageApi,
  getCustomerPoolAssignRulePageApi,
  getCustomerPoolDupRulePageApi,
  getCustomerPoolStatsAdminApi,
  getFrozenPageApi,
  getPoolConfigApi,
  getPoolMembersApi,
  getPoolPageApi,
  getCustomerPoolRetainAuditPageApi,
  removePoolMembersApi,
  restoreFrozenApi,
  saveCustomerPoolAssignRuleApi,
  saveCustomerPoolDupRuleApi,
  savePoolConfigApi,
  updatePoolApi,
  updatePoolStatusApi,
} from '#/api';
import { UserPickerModal } from '#/components/UserPickerModal';
import { $t } from '#/locales';

const accessStore = useAccessStore();
const can = (code: string) => accessStore.hasAccessCode(code);

// ==================== 池列表 ====================
const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: {},
  rowConfig: { height: 'auto' as any },
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        return await getPoolPageApi({
          page: page.currentPage,
          pageSize: page.pageSize,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '池名称', field: 'name', minWidth: 160, slots: { default: 'name' } },
    { title: '描述', field: 'description', minWidth: 180 },
    {
      title: '状态',
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    { title: '池管理员', field: 'poolAdminName', width: 110 },
    { title: '成员数', field: 'memberCount', width: 80, align: 'right' },
    {
      title: '在池客户数',
      field: 'customerCount',
      width: 100,
      align: 'right',
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      formatter: ({ cellValue }: any) => formatDateTime(cellValue),
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      width: 260,
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

// ==================== Tab 切换与审批/冻结 ====================
const activeTab = ref('pools');

function handleTabChange(key: number | string) {
  if (key === 'applyAudit') loadApplyAudit();
  if (key === 'retainAudit') loadRetainAudit();
  if (key === 'freeze') loadFrozen();
  if (key === 'dupRules') loadDupRules();
  if (key === 'assignRules') loadAssignRules();
  if (key === 'stats') loadStatsAdmin();
}

function normalizePage(r: any) {
  const data = r?.data ?? r;
  return {
    list: data?.list ?? data?.items ?? [],
    total: data?.total ?? 0,
  };
}

// ---- 申领审批 ----
const applyAudit = reactive({ loading: false, list: [] as any[], total: 0, page: 1, pageSize: 10 });
const auditModal = reactive({
  visible: false,
  type: 'apply' as 'apply' | 'retain',
  id: 0,
  pass: true,
  remark: '',
  saving: false,
});

function loadApplyAudit() {
  applyAudit.loading = true;
  getCustomerPoolApplyAuditPageApi({ page: applyAudit.page, pageSize: applyAudit.pageSize })
    .then((r: any) => {
      const p = normalizePage(r);
      applyAudit.list = p.list;
      applyAudit.total = p.total;
    })
    .finally(() => (applyAudit.loading = false));
}

const applyColumns = [
  { title: '客户', dataIndex: 'customerName', width: 160 },
  { title: '公海池', dataIndex: 'poolName', width: 140 },
  { title: '申请人', dataIndex: 'userName', width: 100 },
  { title: '申领理由', dataIndex: 'reason', minWidth: 160 },
  { title: '状态', dataIndex: 'status', width: 90, slots: { customRender: 'status' } },
  { title: '申请时间', dataIndex: 'createTime', width: 160, slots: { customRender: 'createTime' } },
  { title: '操作', dataIndex: 'action', width: 150,  },
];

// ---- 延期审批 ----
const retainAudit = reactive({ loading: false, list: [] as any[], total: 0, page: 1, pageSize: 10 });
const retainColumns = [
  { title: '客户', dataIndex: 'customerName', width: 160 },
  { title: '公海池', dataIndex: 'poolName', width: 140 },
  { title: '申请人', dataIndex: 'userName', width: 100 },
  { title: '延期天数', dataIndex: 'extendDays', width: 90 },
  { title: '申请理由', dataIndex: 'reason', minWidth: 160 },
  { title: '状态', dataIndex: 'status', width: 90, slots: { customRender: 'status' } },
  { title: '申请时间', dataIndex: 'createTime', width: 160, slots: { customRender: 'createTime' } },
  { title: '操作', dataIndex: 'action', width: 150,  },
];

function loadRetainAudit() {
  retainAudit.loading = true;
  getCustomerPoolRetainAuditPageApi({ page: retainAudit.page, pageSize: retainAudit.pageSize })
    .then((r: any) => {
      const p = normalizePage(r);
      retainAudit.list = p.list;
      retainAudit.total = p.total;
    })
    .finally(() => (retainAudit.loading = false));
}

function openAudit(type: 'apply' | 'retain', id: number, pass: boolean) {
  auditModal.type = type;
  auditModal.id = id;
  auditModal.pass = pass;
  auditModal.remark = '';
  auditModal.visible = true;
}

async function submitAudit() {
  auditModal.saving = true;
  try {
    const api = auditModal.type === 'apply' ? auditCustomerPoolApplyApi : auditCustomerPoolRetainApi;
    await api({ id: auditModal.id, pass: auditModal.pass, remark: auditModal.remark || undefined });
    message.success(auditModal.pass ? '已通过' : '已拒绝');
    auditModal.visible = false;
    if (auditModal.type === 'apply') loadApplyAudit();
    else loadRetainAudit();
  } catch {
    // 错误提示由拦截器处理
  } finally {
    auditModal.saving = false;
  }
}

const statusTextMap: Record<number, string> = { 1: '待审批', 2: '已通过', 3: '已拒绝' };

// ---- 冻结列表 ----
const frozen = reactive({ loading: false, list: [] as any[], total: 0, page: 1, pageSize: 10 });
const frozenColumns = [
  { title: '客户', dataIndex: 'companyName', width: 200 },
  { title: '联系人', dataIndex: 'personName', width: 110 },
  { title: '累计被动退回/回收', dataIndex: 'releaseCount', width: 150 },
  { title: '进入公海时间', dataIndex: 'enteredPoolAt', width: 170 },
  { title: '操作', dataIndex: 'action', width: 100,  },
];

function loadFrozen() {
  frozen.loading = true;
  getFrozenPageApi({ page: frozen.page, pageSize: frozen.pageSize })
    .then((r: any) => {
      const p = normalizePage(r);
      frozen.list = p.list;
      frozen.total = p.total;
    })
    .finally(() => (frozen.loading = false));
}

async function handleRestore(row: any) {
  try {
    await restoreFrozenApi(Number(row.id));
    message.success('已恢复');
    loadFrozen();
  } catch {
    // 错误提示由拦截器处理
  }
}


// ==================== 查重规则 ====================
const dupRules = reactive({ loading: false, list: [] as any[], total: 0, page: 1, pageSize: 10 });
const dupModal = reactive({ visible: false, saving: false, id: undefined as number | undefined, name: '', fields: [] as string[], strength: 1, validDays: 0, matchRatio: 80, enabled: 1 });
const dupFieldOptions = [
  { label: '客户名称', value: 'name' },
  { label: '手机', value: 'mobile' },
  { label: '电话', value: 'phone' },
  { label: '微信', value: 'wechat' },
  { label: '邮箱', value: 'email' },
];
function loadDupRules() {
  dupRules.loading = true;
  getCustomerPoolDupRulePageApi({ page: dupRules.page, pageSize: dupRules.pageSize })
    .then((r: any) => {
      const d = r?.data ?? r;
      dupRules.list = d?.items ?? [];
      dupRules.total = d?.total ?? 0;
    })
    .finally(() => (dupRules.loading = false));
}
function openDupModal(row?: any) {
  dupModal.id = row ? Number(row.id) : undefined;
  dupModal.name = row?.name || '';
  dupModal.fields = row?.fields ? (typeof row.fields === 'string' ? JSON.parse(row.fields) : row.fields) : ['name', 'mobile'];
  dupModal.strength = Number(row?.strength ?? 1);
  dupModal.validDays = Number(row?.validDays ?? 0);
  dupModal.matchRatio = Number(row?.matchRatio ?? 80);
  dupModal.enabled = Number(row?.enabled ?? 1);
  dupModal.visible = true;
}
async function submitDup() {
  if (!dupModal.name) { message.warning('请输入规则名称'); return; }
  if (dupModal.fields.length === 0) { message.warning('请选择查重字段'); return; }
  dupModal.saving = true;
  try {
    await saveCustomerPoolDupRuleApi({ id: dupModal.id, name: dupModal.name, fields: dupModal.fields, strength: dupModal.strength, validDays: dupModal.validDays, matchRatio: dupModal.matchRatio, enabled: dupModal.enabled });
    message.success('已保存');
    dupModal.visible = false;
    loadDupRules();
  } catch { /* 拦截器处理 */ } finally { dupModal.saving = false; }
}
async function handleDupDelete(row: any) {
  try { await deleteCustomerPoolDupRuleApi([Number(row.id)]); message.success('已删除'); loadDupRules(); } catch { /* 拦截器 */ }
}

// ==================== 自动分配规则 ====================
const assignRules = reactive({ loading: false, list: [] as any[] });
const poolOptions = ref<{ label: string; value: number }[]>([]);
const assignModal = reactive({ visible: false, saving: false, id: undefined as number | undefined, poolId: undefined as number | undefined, name: '', priority: 100, enabled: 1, entriesJson: '[]' });
function loadAssignRules() {
  assignRules.loading = true;
  getPoolPageApi({ page: 1, pageSize: 100 }).then((r: any) => {
    const d = r?.data ?? r;
    poolOptions.value = (d?.list ?? d?.items ?? []).map((x: any) => ({ label: x.name, value: Number(x.id) }));
  }).catch(() => {});
  getCustomerPoolAssignRulePageApi({})
    .then((r: any) => {
      const d = r?.data ?? r;
      assignRules.list = Array.isArray(d) ? d : d?.items ?? [];
    })
    .finally(() => (assignRules.loading = false));
}
function openAssignModal(row?: any) {
  assignModal.id = row ? Number(row.id) : undefined;
  assignModal.poolId = row ? Number(row.poolId) : (poolOptions.value[0] ? poolOptions.value[0].value : undefined);
  assignModal.name = row?.name || '';
  assignModal.priority = Number(row?.priority ?? 100);
  assignModal.enabled = Number(row?.enabled ?? 1);
  assignModal.entriesJson = row && row.entries && row.entries.length > 0 ? JSON.stringify(row.entries, null, 2) : '[]';
  assignModal.visible = true;
}
async function submitAssign() {
  if (!assignModal.name) { message.warning('请输入规则名称'); return; }
  if (!assignModal.poolId) { message.warning('请选择所属公海池'); return; }
  let entries: any;
  try { entries = JSON.parse(assignModal.entriesJson || '[]'); } catch { message.warning('条目 JSON 格式不正确'); return; }
  assignModal.saving = true;
  try {
    await saveCustomerPoolAssignRuleApi({ id: assignModal.id, poolId: assignModal.poolId, name: assignModal.name, triggerEvent: 1, priority: assignModal.priority, enabled: assignModal.enabled, entries });
    message.success('已保存');
    assignModal.visible = false;
    loadAssignRules();
  } catch { /* 拦截器处理 */ } finally { assignModal.saving = false; }
}
async function handleAssignDelete(row: any) {
  try { await deleteCustomerPoolAssignRuleApi([Number(row.id)]); message.success('已删除'); loadAssignRules(); } catch { /* 拦截器 */ }
}

// ==================== 考核统计 ====================
const statsAdmin = reactive({ loading: false, data: null as any });
function loadStatsAdmin() {
  statsAdmin.loading = true;
  getCustomerPoolStatsAdminApi({})
    .then((r: any) => { statsAdmin.data = r?.data ?? r; })
          .catch(() => {})
    .finally(() => (statsAdmin.loading = false));
}

// ==================== 新建/编辑抽屉 ====================
const editVisible = ref(false);
const editSaving = ref(false);
const editForm = reactive({
  description: '',
  id: undefined as number | undefined,
  name: '',
  poolAdminId: undefined as number | undefined,
  sort: 0,
});

function openCreate() {
  editForm.id = undefined;
  editForm.name = '';
  editForm.description = '';
  editForm.sort = 0;
  editForm.poolAdminId = undefined;
  editVisible.value = true;
}

function openEdit(row: any) {
  editForm.id = Number(row.id);
  editForm.name = row.name || '';
  editForm.description = row.description || '';
  editForm.sort = row.sort ?? 0;
  editForm.poolAdminId = row.poolAdminId
    ? Number(row.poolAdminId)
    : undefined;
  editVisible.value = true;
}

async function submitEdit() {
  if (!editForm.name) {
    message.warning('请输入池名称');
    return;
  }
  if (!editForm.poolAdminId) {
    message.warning('请选择池管理员');
    return;
  }
  editSaving.value = true;
  try {
    const payload = {
      description: editForm.description || undefined,
      id: editForm.id,
      name: editForm.name,
      poolAdminId: editForm.poolAdminId,
      sort: editForm.sort ?? 0,
    };
    if (editForm.id) {
      await updatePoolApi(payload);
      message.success('更新成功');
    } else {
      await createPoolApi(payload);
      message.success('创建成功');
    }
    editVisible.value = false;
    gridApi.query();
  } catch {
    // 错误提示由拦截器处理
  } finally {
    editSaving.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deletePoolApi([Number(row.id)]);
    message.success('删除成功');
    gridApi.query();
  } catch {
    // 错误提示由拦截器处理
  }
}

async function handleStatusChange(row: any, checked: boolean) {
  try {
    await updatePoolStatusApi({
      id: Number(row.id),
      status: checked ? 1 : 2,
    });
    message.success(checked ? '已启用' : '已停用');
    gridApi.query();
  } catch {
    // 失败回退开关状态由列表刷新处理
    gridApi.query();
  }
}

// ==================== 成员管理抽屉 ====================
const memberVisible = ref(false);
const memberLoading = ref(false);
const memberPoolId = ref<number | undefined>(undefined);
const memberPoolName = ref('');
const memberList = ref<any[]>([]);
const memberAddForm = reactive({
  memberType: 2,
  pickedUserId: undefined as number | undefined,
});

function openMembers(row: any) {
  memberPoolId.value = Number(row.id);
  memberPoolName.value = row.name || '';
  memberVisible.value = true;
  loadMembers();
}

async function loadMembers() {
  if (!memberPoolId.value) return;
  memberLoading.value = true;
  try {
    const list = (await getPoolMembersApi(memberPoolId.value)) as any[];
    memberList.value = Array.isArray(list) ? list : [];
  } catch {
    memberList.value = [];
  } finally {
    memberLoading.value = false;
  }
}

async function submitAddMember() {
  if (!memberAddForm.pickedUserId) {
    message.warning('请选择要添加的用户');
    return;
  }
  try {
    await addPoolMembersApi({
      members: [
        {
          userId: memberAddForm.pickedUserId,
          memberType: memberAddForm.memberType,
        },
      ],
      poolId: memberPoolId.value!,
    });
    message.success('已添加');
    memberAddForm.pickedUserId = undefined;
    loadMembers();
    gridApi.query();
  } catch {
    // 错误提示由拦截器处理
  }
}

async function handleRemoveMember(row: any) {
  try {
    await removePoolMembersApi({
      poolId: memberPoolId.value!,
      userIds: [Number(row.userId ?? row.id)],
    });
    message.success('已移除');
    loadMembers();
  } catch {
    // 错误提示由拦截器处理
  }
}

// ==================== 池配置抽屉 ====================
const configVisible = ref(false);
const configSaving = ref(false);
const configForm = reactive<Record<string, any>>({
  allowDetail: 0,
  autoAssignEnabled: 0,
  autoAssignMode: 1,
  claimDailyLimit: 0,
  claimMode: 1,
  coolDownDays: 7,
  dealProtectEnabled: 1,
  dealStatus: [2, 3, 4],
  firstTouchHours: 48,
  freezeReleaseCount: 3,
  hideClaimed: 0,
  hideConverted: 0,
  holdLimit: 0,
  includeSelfBuilt: 0,
  maskFields: [],
  maxRecycleDays: undefined as number | undefined,
  noContractDays: undefined as number | undefined,
  noOpportunityDays: undefined as number | undefined,
  opportunityProtectEnabled: 1,
  recycleDays: 30,
  releaseBackTo: 1,
  reminderDays: 3,
  selfBuiltRecycleEnabled: 0,
});
const maskFieldOptions = [
  { label: '手机', value: 'mobile' },
  { label: '电话', value: 'phone' },
  { label: '邮箱', value: 'email' },
  { label: '微信', value: 'wechat' },
];
const dealStatusOptions = [
  { label: '草拟', value: 1 },
  { label: '已审批', value: 2 },
  { label: '履行中', value: 3 },
  { label: '已完成', value: 4 },
];
const claimModeOptions = [
  { label: '自主领取', value: 1 },
  { label: '申领（需审批）', value: 2 },
  { label: '停用领取（仅分配）', value: 3 },
];
const releaseBackToOptions = [
  { label: '退回原池', value: 1 },
  { label: '退回时选择分组', value: 2 },
  { label: '退回指定分组', value: 3 },
];

function openConfig(row: any) {
  memberPoolId.value = Number(row.id);
  memberPoolName.value = row.name || '';
  configVisible.value = true;
  loadConfig();
}

async function loadConfig() {
  if (!memberPoolId.value) return;
  try {
    const vo = (await getPoolConfigApi(memberPoolId.value)) as any;
    for (const key of Object.keys(configForm)) {
      if (vo && vo[key] !== undefined && vo[key] !== null) {
        configForm[key] = vo[key];
      }
    }
  } catch {
    // 保持默认值
  }
}

async function submitConfig() {
  configSaving.value = true;
  try {
    await savePoolConfigApi({
      ...configForm,
      poolId: memberPoolId.value,
    });
    message.success('配置已保存');
    configVisible.value = false;
  } catch {
    // 错误提示由拦截器处理
  } finally {
    configSaving.value = false;
  }
}
</script>

<template>
  <Page>
    <Tabs v-model:active-key="activeTab" type="card" size="small" @change="handleTabChange">
      <TabPane key="pools" tab="池管理">
        <div class="mb-3 flex items-center justify-between">
          <div class="text-base font-medium">客户公海管理</div>
          <Button
            v-if="can('crm:customer-pool:save')"
            type="primary"
            @click="openCreate"
          >
            新建公海池
          </Button>
        </div>

        <Grid class="mt-2">
      <template #name="{ row }">
        <span>{{ row.name }}</span>
        <Tag v-if="row.isDefault === 1" class="ml-1" color="blue">默认池</Tag>
      </template>

      <template #status="{ row }">
        <Switch
          :checked="Number(row.status) === 1"
          :disabled="
            row.isDefault === 1 || !can('crm:customer-pool:update')
          "
          checked-children="启用"
          un-checked-children="停用"
          @change="(checked: any) => handleStatusChange(row, !!checked)"
        />
      </template>

      <template #action="{ row }">
        <span class="action-btns">
          <a
            v-if="can('crm:customer-pool:update')"
            class="action-btn"
            @click="() => openEdit(row)"
          >
            编辑
          </a>
          <a
            v-if="can('crm:customer-pool:member')"
            class="action-btn"
            @click="() => openMembers(row)"
          >
            成员
          </a>
          <a
            v-if="can('crm:customer-pool:config')"
            class="action-btn"
            @click="() => openConfig(row)"
          >
            规则配置
          </a>
          <Popconfirm
            v-if="can('crm:customer-pool:delete') && row.isDefault !== 1"
            :title="`确认删除公海池「${row.name}」？需池内无客户`"
            @confirm="() => handleDelete(row)"
          >
            <a class="action-btn danger">删除</a>
          </Popconfirm>
        </span>
      </template>
    </Grid>
      </TabPane>

      <TabPane key="applyAudit">
        <template #tab>申领审批</template>
        <Table
          :columns="applyColumns"
          :data-source="applyAudit.list"
          :loading="applyAudit.loading"
          :pagination="{ current: applyAudit.page, pageSize: applyAudit.pageSize, total: applyAudit.total, showSizeChanger: true, onChange: (p: number, s: number) => { applyAudit.page = p; applyAudit.pageSize = s; loadApplyAudit(); } }"
          row-key="id"
          size="middle"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'status'">
            <Tag :color="Number(record.status) === 1 ? 'orange' : Number(record.status) === 2 ? 'green' : 'red'">
              {{ statusTextMap[Number(record.status)] || record.status }}
            </Tag>
          </template>
            <template v-else-if="column.dataIndex === 'createTime'">
            {{ formatDateTime(record.createTime) }}
          </template>
            <template v-else-if="column.dataIndex === 'action'">
            <span v-if="Number(record.status) === 1 && can('crm:customer-pool:apply-audit')" class="action-btns">
              <a class="action-btn" @click="() => openAudit('apply', Number(record.id), true)">通过</a>
              <a class="action-btn danger" @click="() => openAudit('apply', Number(record.id), false)">拒绝</a>
            </span>
            <span v-else class="text-xs text-gray-400">已处理</span>
          </template>
          </template>
        </Table>
      </TabPane>

      <TabPane key="retainAudit">
        <template #tab>延期审批</template>
        <Table
          :columns="retainColumns"
          :data-source="retainAudit.list"
          :loading="retainAudit.loading"
          :pagination="{ current: retainAudit.page, pageSize: retainAudit.pageSize, total: retainAudit.total, showSizeChanger: true, onChange: (p: number, s: number) => { retainAudit.page = p; retainAudit.pageSize = s; loadRetainAudit(); } }"
          row-key="id"
          size="middle"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'status'">
              <Tag :color="Number(record.status) === 1 ? 'orange' : Number(record.status) === 2 ? 'green' : 'red'">
                {{ statusTextMap[Number(record.status)] || record.status }}
              </Tag>
            </template>
            <template v-else-if="column.dataIndex === 'createTime'">
              {{ formatDateTime(record.createTime) }}
            </template>
            <template v-else-if="column.dataIndex === 'action'">
              <span v-if="Number(record.status) === 1 && can('crm:customer-pool:retain-audit')" class="action-btns">
                <a class="action-btn" @click="() => openAudit('retain', Number(record.id), true)">通过</a>
                <a class="action-btn danger" @click="() => openAudit('retain', Number(record.id), false)">拒绝</a>
              </span>
              <span v-else class="text-xs text-gray-400">已处理</span>
            </template>
          </template>
        </Table>
      </TabPane>

      <TabPane key="freeze">
        <template #tab>冻结列表</template>
        <Table
          :columns="frozenColumns"
          :data-source="frozen.list"
          :loading="frozen.loading"
          :pagination="{ current: frozen.page, pageSize: frozen.pageSize, total: frozen.total, showSizeChanger: true, onChange: (p: number, s: number) => { frozen.page = p; frozen.pageSize = s; loadFrozen(); } }"
          row-key="id"
          size="middle"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'enteredPoolAt'">
              {{ formatDateTime(record.enteredPoolAt) }}
            </template>
            <template v-else-if="column.dataIndex === 'action'">
              <Popconfirm
                v-if="can('crm:customer-pool:freeze')"
                title="确认恢复该冻结客户？恢复后重新可被领取"
                @confirm="() => handleRestore(record)"
              >
                <a class="action-btn">恢复</a>
              </Popconfirm>
              <span v-else class="text-xs text-gray-400">-</span>
            </template>
          </template>
        </Table>
      </TabPane>

      <TabPane key="dupRules">
        <template #tab>查重规则</template>
        <div class="mb-3">
          <Button v-if="can('crm:customer-pool:dup-rule')" type="primary" @click="() => openDupModal()">新增查重规则</Button>
        </div>
        <Table :loading="dupRules.loading" :data-source="dupRules.list" :pagination="{ current: dupRules.page, pageSize: dupRules.pageSize, total: dupRules.total, onChange: (p: number, ps: number) => { dupRules.page = p; dupRules.pageSize = ps; loadDupRules(); } }" row-key="id" size="middle">
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'fields'">
              {{ Array.isArray(record.fields) ? JSON.parse(JSON.stringify(record.fields)).join('、') : '-' }}
            </template>
            <template v-else-if="column.dataIndex === 'strength'">
              <Tag :color="Number(record.strength) === 2 ? 'red' : 'blue'">{{ Number(record.strength) === 2 ? '阻断' : '提示' }}</Tag>
            </template>
            <template v-else-if="column.dataIndex === 'enabled'">
              {{ Number(record.enabled) === 1 ? '启用' : '停用' }}
            </template>
            <template v-else-if="column.dataIndex === 'action'">
              <span v-if="can('crm:customer-pool:dup-rule')" class="action-btns">
                <a class="action-btn" @click="() => openDupModal(record)">编辑</a>
                <Popconfirm title="确认删除该规则？" @confirm="() => handleDupDelete(record)"><a class="action-btn danger">删除</a></Popconfirm>
              </span>
              <span v-else class="text-xs text-gray-400">-</span>
            </template>
          </template>
        </Table>
      </TabPane>

      <TabPane key="assignRules">
        <template #tab>自动分配规则</template>
        <div class="mb-3">
          <Button v-if="can('crm:customer-pool:assign-rule')" type="primary" @click="() => openAssignModal()">新增分配规则</Button>
        </div>
        <Table :loading="assignRules.loading" :data-source="assignRules.list" :pagination="false" row-key="id" size="middle">
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'poolName'">{{ record.poolName || '-' }}</template>
            <template v-else-if="column.dataIndex === 'triggerEvent'">{{ Number(record.triggerEvent) === 1 ? '进池' : '其他' }}</template>
            <template v-else-if="column.dataIndex === 'entryCount'">{{ (record.entries || []).length }}</template>
            <template v-else-if="column.dataIndex === 'enabled'">{{ Number(record.enabled) === 1 ? '启用' : '停用' }}</template>
            <template v-else-if="column.dataIndex === 'action'">
              <span v-if="can('crm:customer-pool:assign-rule')" class="action-btns">
                <a class="action-btn" @click="() => openAssignModal(record)">编辑</a>
                <Popconfirm title="确认删除该规则？" @confirm="() => handleAssignDelete(record)"><a class="action-btn danger">删除</a></Popconfirm>
              </span>
              <span v-else class="text-xs text-gray-400">-</span>
            </template>
          </template>
        </Table>
      </TabPane>

      <TabPane key="stats">
        <template #tab>考核统计</template>
        <Spin :spinning="statsAdmin.loading">
          <Row :gutter="16" class="mb-4">
            <Col :span="8"><Card><div class="text-xs text-gray-400">全局在池客户</div><div class="text-2xl font-semibold">{{ statsAdmin.data?.totalInPool ?? 0 }}</div></Card></Col>
            <Col :span="8"><Card><div class="text-xs text-gray-400">全局冻结客户</div><div class="text-2xl font-semibold">{{ statsAdmin.data?.totalFrozen ?? 0 }}</div></Card></Col>
            <Col :span="8"><Card><div class="text-xs text-gray-400">公海池数量</div><div class="text-2xl font-semibold">{{ (statsAdmin.data?.pools || []).length }}</div></Card></Col>
          </Row>
          <Table
            :columns="[
              { title: '公海池', dataIndex: 'poolName' },
              { title: '在池客户数', dataIndex: 'poolCustomerCount', width: 110, align: 'right' },
              { title: '冻结客户数', dataIndex: 'frozenCount', width: 110, align: 'right' },
              { title: '本月流入', dataIndex: 'monthInflow', width: 100, align: 'right' },
              { title: '本月流出', dataIndex: 'monthOutflow', width: 100, align: 'right' },
            ]"
            :data-source="statsAdmin.data?.pools || []"
            :pagination="false"
            row-key="poolId"
            size="middle"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.dataIndex === 'poolName'">{{ record.poolName || ('池#' + record.poolId) }}</template>
            </template>
          </Table>
        </Spin>
      </TabPane>
    </Tabs>

    <!-- 新建/编辑抽屉 -->
    <Modal
      v-model:visible="editVisible"
      :confirm-loading="editSaving"
      :title="editForm.id ? '编辑公海池' : '新建公海池'"
      @ok="submitEdit"
    >
      <Form layout="vertical">
        <Form.Item label="池名称" required>
          <Input
            v-model:value="editForm.name"
            :maxlength="64"
            placeholder="如：华南区公海"
          />
        </Form.Item>
        <Form.Item label="描述">
          <Input
            v-model:value="editForm.description"
            :maxlength="255"
            placeholder="池用途说明（可选）"
          />
        </Form.Item>
        <Form.Item label="排序">
          <InputNumber v-model:value="editForm.sort" :min="0" class="w-full" />
        </Form.Item>
        <Form.Item label="池管理员" required>
          <UserPickerModal v-model:value="editForm.poolAdminId" />
          <div class="mt-1 text-xs text-gray-400">
            池管理员可看全池（含已分配客户），并可指派/收回/审批
          </div>
        </Form.Item>
      </Form>
    </Modal>

    <!-- 成员管理抽屉 -->
    <Drawer
      v-model:visible="memberVisible"
      :title="`池成员 - ${memberPoolName}`"
      width="min(560px, 92vw)"
    >
      <div class="mb-3 flex items-end gap-2">
        <Form layout="vertical" class="flex-1">
          <Form.Item label="添加成员" class="mb-0">
            <UserPickerModal v-model:value="memberAddForm.pickedUserId" />
          </Form.Item>
        </Form>
        <Form layout="vertical">
          <Form.Item label="身份" class="mb-0">
            <Select v-model:value="memberAddForm.memberType" style="width: 130px">
              <Select.Option :value="1">池管理员</Select.Option>
              <Select.Option :value="2">普通成员</Select.Option>
            </Select>
          </Form.Item>
        </Form>
        <Button
          v-if="can('crm:customer-pool:member')"
          class="mb-0.5"
          type="primary"
          @click="submitAddMember"
        >
          添加
        </Button>
      </div>
      <div
        v-for="m in memberList"
        :key="m.userId ?? m.id"
        class="flex items-center justify-between border-b border-gray-100 py-2"
      >
        <div>
          <span class="font-medium">{{ m.userName || m.nickName || m.userId }}</span>
          <Tag
            class="ml-2"
            :color="Number(m.memberType) === 1 ? 'blue' : 'default'"
          >
            {{ Number(m.memberType) === 1 ? '池管理员' : '普通成员' }}
          </Tag>
        </div>
        <Popconfirm
          v-if="can('crm:customer-pool:member')"
          title="确认移除该成员？"
          @confirm="() => handleRemoveMember(m)"
        >
          <a class="text-red-500">移除</a>
        </Popconfirm>
      </div>
      <div
        v-if="!memberLoading && memberList.length === 0"
        class="py-8 text-center text-gray-400"
      >
        暂无成员
      </div>
    </Drawer>

    <!-- 池配置抽屉 -->
    <Drawer
      v-model:visible="configVisible"
      :title="`规则配置 - ${memberPoolName}`"
      width="min(680px, 94vw)"
    >
      <Form layout="vertical">
        <div class="config-group-title">保护期与回收</div>
        <div class="config-grid">
          <Form.Item label="默认保护期（天）" extra="未跟进超期自动回收，0=关闭">
            <InputNumber v-model:value="configForm.recycleDays" :min="0" class="w-full" />
          </Form.Item>
          <Form.Item label="最长保护期（天）" extra="绝对到期硬上限，留空=无上限">
            <InputNumber
              v-model:value="configForm.maxRecycleDays"
              :min="1"
              class="w-full"
            />
          </Form.Item>
          <Form.Item label="未新增商机回收（天）" extra="留空=关闭该规则">
            <InputNumber
              v-model:value="configForm.noOpportunityDays"
              :min="1"
              class="w-full"
            />
          </Form.Item>
          <Form.Item label="未签合同回收（天）" extra="留空=关闭该规则">
            <InputNumber
              v-model:value="configForm.noContractDays"
              :min="1"
              class="w-full"
            />
          </Form.Item>
          <Form.Item label="回收前提醒（天）" extra="0=不提醒">
            <InputNumber v-model:value="configForm.reminderDays" :min="0" class="w-full" />
          </Form.Item>
        </div>

        <div class="config-group-title">领取规则</div>
        <div class="config-grid">
          <Form.Item label="领取模式">
            <Select v-model:value="configForm.claimMode" class="w-full" :options="claimModeOptions" />
          </Form.Item>
          <Form.Item label="冷却期（天）" extra="退回/回收后原负责人 N 天内不可再领，0=不启用">
            <InputNumber v-model:value="configForm.coolDownDays" :min="0" class="w-full" />
          </Form.Item>
          <Form.Item label="每日领取上限" extra="0=不限制">
            <InputNumber
              v-model:value="configForm.claimDailyLimit"
              :min="0"
              class="w-full"
            />
          </Form.Item>
          <Form.Item label="私海保有量上限" extra="0=不限制">
            <InputNumber v-model:value="configForm.holdLimit" :min="0" class="w-full" />
          </Form.Item>
        </div>

        <div class="config-group-title">成单保护与豁免</div>
        <div class="config-grid">
          <Form.Item label="成单保护">
            <Switch
              v-model:checked="configForm.dealProtectEnabled"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="开"
              un-checked-children="关"
            />
          </Form.Item>
          <Form.Item label="有效合同状态" extra="名下存在所选状态合同即豁免回收">
            <Select
              v-model:value="configForm.dealStatus"
              class="w-full"
              mode="multiple"
              :options="dealStatusOptions"
            />
          </Form.Item>
          <Form.Item label="进行中商机豁免">
            <Switch
              v-model:checked="configForm.opportunityProtectEnabled"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="开"
              un-checked-children="关"
            />
          </Form.Item>
          <Form.Item label="自建客户占保有量">
            <Switch
              v-model:checked="configForm.includeSelfBuilt"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="占"
              un-checked-children="不占"
            />
          </Form.Item>
          <Form.Item label="自建客户按池规则回收">
            <Switch
              v-model:checked="configForm.selfBuiltRecycleEnabled"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="回收"
              un-checked-children="仅统计"
            />
          </Form.Item>
        </div>

        <div class="config-group-title">自动分配</div>
        <div class="config-grid">
          <Form.Item label="入池自动分配">
            <Switch
              v-model:checked="configForm.autoAssignEnabled"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="开"
              un-checked-children="关"
            />
          </Form.Item>
          <Form.Item label="分配模式">
            <Select
              v-model:value="configForm.autoAssignMode"
              class="w-full"
              :disabled="Number(configForm.autoAssignEnabled) !== 1"
              :options="[
                { label: '轮询（Round-Robin）', value: 1 },
                { label: '权重（Weighted）', value: 2 },
              ]"
            />
          </Form.Item>
        </div>

        <div class="config-group-title">工作台可见性</div>
        <div class="config-grid">
          <Form.Item label="隐藏已领取客户">
            <Switch
              v-model:checked="configForm.hideClaimed"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="隐藏"
              un-checked-children="显示"
            />
          </Form.Item>
          <Form.Item label="隐藏已转化客户">
            <Switch
              v-model:checked="configForm.hideConverted"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="隐藏"
              un-checked-children="显示"
            />
          </Form.Item>
          <Form.Item label="普通成员可看详情">
            <Switch
              v-model:checked="configForm.allowDetail"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="可"
              un-checked-children="不可"
            />
          </Form.Item>
          <Form.Item label="脱敏字段" extra="普通成员视角打码，池管理员/超管明文">
            <Select
              v-model:value="configForm.maskFields"
              class="w-full"
              mode="multiple"
              :options="maskFieldOptions"
              placeholder="不选择=不脱敏"
            />
          </Form.Item>
        </div>

        <div class="config-group-title">退回与冻结</div>
        <div class="config-grid">
          <Form.Item label="退回去向">
            <Select
              v-model:value="configForm.releaseBackTo"
              class="w-full"
              :options="releaseBackToOptions"
            />
          </Form.Item>
          <Form.Item label="冻结阈值（次）" extra="连续被动退回/回收达该次数冻结，0=不冻结">
            <InputNumber
              v-model:value="configForm.freezeReleaseCount"
              :min="0"
              class="w-full"
            />
          </Form.Item>
          <Form.Item label="首次触达 SLA（小时）" extra="领取后 N 小时内需产生首次跟进">
            <InputNumber
              v-model:value="configForm.firstTouchHours"
              :min="1"
              class="w-full"
            />
          </Form.Item>
        </div>
      </Form>

      <div class="mt-4 flex justify-end gap-2">
        <Button @click="configVisible = false">取消</Button>
        <Button
          v-if="can('crm:customer-pool:config')"
          :loading="configSaving"
          type="primary"
          @click="submitConfig"
        >
          保存配置
        </Button>
      </div>
    </Drawer>
    <!-- 查重规则弹窗 -->
    <Modal
      v-model:visible="dupModal.visible"
      :confirm-loading="dupModal.saving"
      :title="dupModal.id ? '编辑查重规则' : '新增查重规则'"
      @ok="submitDup"
    >
      <Form layout="vertical">
        <Form.Item label="规则名称" required>
          <Input v-model:value="dupModal.name" :maxlength="64" placeholder="如：手机号撞单阻断" />
        </Form.Item>
        <Form.Item label="查重字段" required>
          <Select v-model:value="dupModal.fields" mode="multiple" class="w-full" :options="dupFieldOptions" />
        </Form.Item>
        <Form.Item label="控制强度">
          <Select v-model:value="dupModal.strength" class="w-full">
            <Select.Option :value="1">提示（可继续保存）</Select.Option>
            <Select.Option :value="2">阻断（命中禁止保存）</Select.Option>
          </Select>
        </Form.Item>
        <div class="config-grid">
          <Form.Item label="有效期（天）" extra="按最近跟进时间判定，0=永久有效">
            <InputNumber v-model:value="dupModal.validDays" :min="0" class="w-full" />
          </Form.Item>
          <Form.Item label="名称匹配度阈值（%）" extra="名称连续字符匹配率，80=默认">
            <InputNumber v-model:value="dupModal.matchRatio" :min="10" :max="100" class="w-full" />
          </Form.Item>
        </div>
        <Form.Item label="启用状态">
          <Switch v-model:checked="dupModal.enabled" :checked-value="1" :un-checked-value="0" checked-children="启用" un-checked-children="停用" />
        </Form.Item>
      </Form>
    </Modal>

    <!-- 自动分配规则弹窗 -->
    <Modal
      v-model:visible="assignModal.visible"
      :confirm-loading="assignModal.saving"
      :title="assignModal.id ? '编辑分配规则' : '新增分配规则'"
      width="min(680px, 94vw)"
      @ok="submitAssign"
    >
      <Form layout="vertical">
        <div class="config-grid">
          <Form.Item label="所属公海池" required>
            <Select v-model:value="assignModal.poolId" class="w-full" :options="poolOptions" placeholder="选择规则生效的池" />
          </Form.Item>
          <Form.Item label="规则名称" required>
            <Input v-model:value="assignModal.name" :maxlength="64" placeholder="如：华东区线索分配" />
          </Form.Item>
          <Form.Item label="优先级" extra="越小越先评估">
            <InputNumber v-model:value="assignModal.priority" :min="1" class="w-full" />
          </Form.Item>
          <Form.Item label="启用状态">
            <Switch v-model:checked="assignModal.enabled" :checked-value="1" :un-checked-value="0" checked-children="启用" un-checked-children="停用" />
          </Form.Item>
        </div>
        <Form.Item label="规则条目（JSON）" extra="高级模式：[{conditions:[{field:'source',op:'eq',value:6}],targetType:4,targetIds:[],mode:1,weight:{},sort:1}]；空条件=全命中；targetType 1=用户 2=职位 3=群组 4=池成员；mode 1=指定 2=轮询(按持有数均衡) 3=权重">
          <Textarea v-model:value="assignModal.entriesJson" :rows="8" />
        </Form.Item>
      </Form>
    </Modal>

    <!-- 审批备注弹窗 -->
    <Modal
      v-model:visible="auditModal.visible"
      :confirm-loading="auditModal.saving"
      :title="`${auditModal.pass ? '通过' : '拒绝'}${auditModal.type === 'apply' ? '申领' : '延期申请'}`"
      @ok="submitAudit"
    >
      <Form layout="vertical">
        <Form.Item label="审批备注">
          <Input
            v-model:value="auditModal.remark"
            :maxlength="200"
            :placeholder="auditModal.pass ? '备注（可选）' : '拒绝原因（建议填写）'"
            :rows="3"
          />
        </Form.Item>
      </Form>
    </Modal>
  </Page>
</template>

<style scoped>
.action-btns {
  display: inline-flex;
  gap: 12px;
  align-items: center;
  font-size: 13px;
}

.action-btn {
  color: #1677ff;
  text-decoration: none;
  cursor: pointer;
}

.action-btn:hover {
  color: #4096ff;
}

.action-btn.danger {
  color: #ff4d4f;
}

.config-group-title {
  padding: 8px 12px;
  margin: 12px 0;
  font-weight: 500;
  background: #fafafa;
  border-left: 3px solid #1677ff;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0 24px;
}
</style>
