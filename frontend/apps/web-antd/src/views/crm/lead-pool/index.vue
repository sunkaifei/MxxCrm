<script lang="ts" setup>
/**
 * 公海工作台（v3.0 公海优化 §6.2）
 * 池 Tab（我可见的池）+ 线索列表（池过滤 + 脱敏渲染）+ 操作集：
 * - 普通成员：领取（模式1）/ 申领（模式2）/ 详情 / 归属历史
 * - 管理员：+ 指派 / 收回；批量操作
 */
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Alert,
  Button,
  Drawer,
  Dropdown,
  Menu,
  message,
  Modal,
  Popconfirm,
  Select,
  TabPane,
  Tabs,
  Tag,
  Textarea,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  applyPoolLeadsApi,
  assignPoolLeadsApi,
  claimPoolLeadsApi,
  deleteLeadPoolApi,
  getAssignHistoryApi,
  getLeadPoolListApi,
  recyclePoolLeadsApi,
} from '#/api/core/crm/lead-pool';
import {
  getMyPoolsApi,
  getPoolMemberCandidatesApi,
} from '#/api/core/crm/pool';
import type { MyPoolVO, PoolMemberSimpleVO } from '#/api/core/crm/pool';
import { $t } from '#/locales';

import LeadDetail from '../lead/detail.vue';
import AuditModal from './audit-modal.vue';

const accessStore = useAccessStore();

const sourceLabelMap: Record<string, string> = {
  website: '官网',
  exhibition: '展会',
  social: '社交媒体',
  referral: '客户转介',
  cold_call: '陌生拜访',
  customs: '海关数据',
  email: '邮件营销',
  alibaba: '阿里国际站',
  amazon: 'Amazon',
  tiktok: 'TikTok',
  wechat: '微信',
  other: '其他',
};

const industryLabelMap: Record<number, string> = {
  1: '零售',
  2: '批发',
  3: '制造',
  4: '贸易代理',
  5: '电商',
  6: '微商',
  7: '社交电商',
  8: '其他',
};

const statusLabelMap: Record<number, string> = {
  1: '新客',
  2: '跟进中',
  3: '已成交',
  4: '无效线索',
  5: '已回收',
  6: '未核查',
  7: '核查中',
  8: '线索池',
};
const statusColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'green',
  4: 'default',
  5: 'orange',
  6: 'default',
  7: 'processing',
  8: 'success',
};

// ==================== 池 Tab ====================
const myPools = ref<MyPoolVO[]>([]);
const activePoolId = ref<string>('all');

const currentPool = computed(() =>
  myPools.value.find((p) => String(p.id) === activePoolId.value),
);

// 当前池领取模式（"全部" Tab 下不启用领取类操作）
const currentClaimMode = computed(() =>
  activePoolId.value === 'all' ? undefined : currentPool.value?.claimMode,
);

// 池管理员（具体池 Tab 且 isManager）或有 assign 权限码
const isManagerView = computed(
  () => activePoolId.value !== 'all' && !!currentPool.value?.isManager,
);

const maskTip = computed(() => {
  const fields = currentPool.value?.maskFields || [];
  if (fields.length === 0 || activePoolId.value === 'all') return '';
  const labelMap: Record<string, string> = {
    mobile: '手机号',
    phone: '电话',
    email: '邮箱',
    wechat: '微信',
  };
  return `本池已开启字段脱敏（${fields.map((f) => labelMap[f] || f).join('、')}），普通成员查看时将打码展示`;
});

async function loadMyPools() {
  myPools.value = (await getMyPoolsApi()) || [];
}

loadMyPools();

watch(activePoolId, () => {
  gridApi.query();
});

// ==================== 审批中心（§5.8 审批入口） ====================
const auditVisible = ref(false);
const auditTab = ref<'apply' | 'retain'>('apply');

const canAuditApply = computed(() =>
  accessStore.hasAccessCode('crm:lead-pool:apply-audit'),
);
const canAuditRetain = computed(() =>
  accessStore.hasAccessCode('crm:lead-pool:retain-audit'),
);

function openAudit(tab: 'apply' | 'retain') {
  auditTab.value = tab;
  auditVisible.value = true;
}

/** 审批动作影响池线索数/归属，完成后联动刷新 */
function onAuditChanged() {
  gridApi.query();
  void loadMyPools();
}

// ==================== 统一详情抽屉 ====================
const detailVisible = ref(false);
const detailId = ref<null | number>(null);
const detailCreate = ref(false);
const detailKey = ref(0);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) {
    message.error('线索ID不存在');
    return;
  }
  detailCreate.value = false;
  detailId.value = Number(id);
  detailKey.value++;
  detailVisible.value = true;
}

function openCreate() {
  detailCreate.value = true;
  detailId.value = null;
  detailKey.value++;
  detailVisible.value = true;
}

function closeDetail() {
  detailVisible.value = false;
  detailId.value = null;
  detailCreate.value = false;
}

function handleDetailSaved() {
  gridApi.query();
  loadMyPools();
}

// ==================== 批量操作结果提示 ====================
function reportOpResults(results: any) {
  if (!Array.isArray(results)) return;
  const ok = results.filter((r) => r.success).length;
  const fail = results.filter((r) => !r.success) as {
    message?: null | string;
  }[];
  if (fail.length === 0) {
    message.success(`操作成功（${ok} 条）`);
  } else {
    const firstMsg = fail[0]?.message || '未知原因';
    message.warning(
      `成功 ${ok} 条，失败 ${fail.length} 条：${firstMsg}${fail.length > 1 ? ' 等' : ''}`,
    );
  }
}

// ==================== 工具栏批量操作入口 ====================
function getCheckedRows(): any[] {
  const rows = getSelectedRows();
  if (rows.length === 0) {
    message.warning('请先勾选线索');
  }
  return rows;
}

function batchClaim() {
  const rows = getCheckedRows();
  if (rows.length > 0) handleClaimRows(rows.map((r) => r.id));
}

function batchApply() {
  const rows = getCheckedRows();
  if (rows.length > 0) openApply(rows);
}

function batchAssign() {
  const rows = getCheckedRows();
  if (rows.length > 0) openAssign(rows);
}

function batchRecycle() {
  const rows = getCheckedRows();
  if (rows.length > 0) openRecycle(rows);
}

// ==================== 领取 / 申领 ====================
async function handleClaimRows(ids: (number | string)[]) {
  await claimPoolLeadsApi(ids);
  message.success('领取成功，线索已转私海');
  gridApi.query();
  loadMyPools();
}

function confirmClaim(row: any) {
  Modal.confirm({
    title: '领取线索',
    content: `确定领取线索「${row.companyName}」吗？领取后将转入您的私海。`,
    onOk: () => handleClaimRows([row.id]),
  });
}

// 申领理由弹窗
const applyVisible = ref(false);
const applyIds = ref<(number | string)[]>([]);
const applyReason = ref('');
const applySaving = ref(false);

function openApply(rows: any[]) {
  applyIds.value = rows.map((r) => r.id);
  applyReason.value = '';
  applyVisible.value = true;
}

async function submitApply() {
  if (applyIds.value.length === 0) return;
  applySaving.value = true;
  try {
    await applyPoolLeadsApi(applyIds.value, applyReason.value.trim());
    message.success('申领已提交，等待审批');
    applyVisible.value = false;
    gridApi.query();
  } finally {
    applySaving.value = false;
  }
}

// ==================== 指派（管理员） ====================
const assignVisible = ref(false);
const assignIds = ref<(number | string)[]>([]);
const assignCandidates = ref<PoolMemberSimpleVO[]>([]);
const assignUserIds = ref<string[]>([]);
const assignSaving = ref(false);

async function openAssign(rows: any[]) {
  if (activePoolId.value === 'all') {
    message.warning('请先选择具体公海池');
    return;
  }
  assignIds.value = rows.map((r) => r.id);
  assignUserIds.value = [];
  assignCandidates.value =
    (await getPoolMemberCandidatesApi(activePoolId.value)) || [];
  assignVisible.value = true;
}

async function submitAssign() {
  if (assignUserIds.value.length === 0) {
    message.warning('请选择目标成员');
    return;
  }
  assignSaving.value = true;
  try {
    const results = await assignPoolLeadsApi(
      assignIds.value,
      assignUserIds.value,
    );
    reportOpResults(results);
    assignVisible.value = false;
    gridApi.query();
  } finally {
    assignSaving.value = false;
  }
}

// ==================== 收回（管理员） ====================
const recycleVisible = ref(false);
const recycleIds = ref<(number | string)[]>([]);
const recycleRemark = ref('');
const recycleSaving = ref(false);

function openRecycle(rows: any[]) {
  recycleIds.value = rows.map((r) => r.id);
  recycleRemark.value = '';
  recycleVisible.value = true;
}

async function submitRecycle() {
  recycleSaving.value = true;
  try {
    const results = await recyclePoolLeadsApi(
      recycleIds.value,
      recycleRemark.value.trim() || undefined,
    );
    reportOpResults(results);
    recycleVisible.value = false;
    gridApi.query();
    loadMyPools();
  } finally {
    recycleSaving.value = false;
  }
}

// ==================== 归属历史 ====================
const historyVisible = ref(false);
const historyLoading = ref(false);
const historyList = ref<any[]>([]);
const historyLeadName = ref('');

async function openHistory(row: any) {
  historyLeadName.value = row.companyName || '';
  historyVisible.value = true;
  historyLoading.value = true;
  try {
    historyList.value = (await getAssignHistoryApi(row.id)) || [];
  } finally {
    historyLoading.value = false;
  }
}

// ==================== 删除 ====================
async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteLeadPoolApi([row.id]);
    message.success($t('ui.notification.delete_success'));
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

// ==================== 表格 ====================
function getSelectedRows(): any[] {
  return gridApi.grid?.getCheckboxRecords() || [];
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '公司名称',
      componentProps: { placeholder: '输入公司名称搜索', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'contactName',
      label: '联系人',
      componentProps: { placeholder: '输入联系人', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: '手机号',
      componentProps: { placeholder: '输入手机号', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '来源',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: Object.entries(sourceLabelMap).map(([value, label]) => ({
          label,
          value,
        })),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: Object.entries(statusLabelMap).map(([value, label]) => ({
          label,
          value: Number(value),
        })),
      },
    },
    {
      component: 'Select',
      fieldName: 'industry',
      label: '行业',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: Object.entries(industryLabelMap).map(([value, label]) => ({
          label,
          value: Number(value),
        })),
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' as any },
  stripe: true,
  checkboxConfig: { checkField: 'checked' },
  height: 'auto',

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getLeadPoolListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          poolId: activePoolId.value === 'all' ? undefined : activePoolId.value,
          ...formValues,
        });
        return result;
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
      headerAlign: 'center',
    },
    {
      title: '公司名称',
      field: 'companyName',
      minWidth: 180,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'companyName' },
    },
    {
      title: '联系人',
      field: 'contactName',
      width: 100,
      headerAlign: 'center',
    },
    {
      title: '手机号',
      field: 'mobile',
      width: 130,
      headerAlign: 'center',
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '来源',
      field: 'source',
      width: 100,
      headerAlign: 'center',
      formatter: ({ cellValue }: any) =>
        sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '状态',
      field: 'status',
      width: 90,
      headerAlign: 'center',
      slots: { default: 'status' },
    },
    {
      title: '行业',
      field: 'industry',
      width: 90,
      headerAlign: 'center',
      formatter: ({ cellValue }: any) =>
        industryLabelMap[cellValue] || cellValue || '-',
    },
    { title: '国家', field: 'country', width: 80, headerAlign: 'center' },
    {
      title: '创建人',
      field: 'createdByName',
      width: 90,
      headerAlign: 'center',
    },
    {
      title: '最近跟进',
      field: 'lastFollowUpAt',
      width: 150,
      headerAlign: 'center',
      formatter: ({ cellValue }: any) =>
        cellValue ? formatDateTime(cellValue) : '未跟进',
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
      width: 160,
      headerAlign: 'center',
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 220,
      headerAlign: 'center',
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #toolbar-tools>
        <!-- 池 Tab（表格工具栏左侧内嵌，跟随切换查询） -->
        <Tabs
          :active-key="activePoolId"
          size="small"
          class="pool-tabs mr-2"
          @update:active-key="(k: any) => (activePoolId = String(k))"
        >
          <TabPane key="all">
            <template #tab>全部</template>
          </TabPane>
          <TabPane v-for="p in myPools" :key="String(p.id)">
            <template #tab>
              {{ p.name }}
              <Tag class="pool-count-tag ml-1">{{ p.leadCount ?? 0 }}</Tag>
            </template>
          </TabPane>
        </Tabs>

        <Button
          v-if="accessStore.hasAccessCode('crm:lead:save')"
          type="primary"
          class="mr-2"
          @click="openCreate"
        >
          {{ $t('page.crm.leadPool.button.create') }}
        </Button>

        <!-- 领取（模式1） -->
        <Button
          v-if="currentClaimMode === 1"
          v-access:code="['crm:lead-pool:claim']"
          class="mr-2"
          @click="batchClaim"
        >
          批量领取
        </Button>

        <!-- 申领（模式2） -->
        <Button
          v-if="currentClaimMode === 2"
          v-access:code="['crm:lead-pool:apply']"
          class="mr-2"
          @click="batchApply"
        >
          批量申领
        </Button>

        <!-- 指派 / 收回（管理员，具体池 Tab） -->
        <Button
          v-if="isManagerView || accessStore.hasAccessCode('crm:lead-pool:assign')"
          v-access:code="['crm:lead-pool:assign']"
          class="mr-2"
          @click="batchAssign"
        >
          批量指派
        </Button>
        <Button
          v-if="isManagerView || accessStore.hasAccessCode('crm:lead-pool:recycle')"
          v-access:code="['crm:lead-pool:recycle']"
          class="mr-2"
          @click="batchRecycle"
        >
          批量收回
        </Button>

        <!-- 审批中心（主管/总监：申领审批 / 延期审批） -->
        <Dropdown v-if="canAuditApply || canAuditRetain">
          <template #overlay>
            <Menu>
              <Menu.Item
                v-if="canAuditApply"
                key="apply"
                @click="openAudit('apply')"
              >
                申领审批
              </Menu.Item>
              <Menu.Item
                v-if="canAuditRetain"
                key="retain"
                @click="openAudit('retain')"
              >
                延期审批
              </Menu.Item>
            </Menu>
          </template>
          <Button class="mr-2">审批</Button>
        </Dropdown>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #companyName="{ row }">
        <div>
          <a
            class="cursor-pointer text-blue-600 hover:text-blue-800"
            @click="() => openDetail(row)"
            >{{ row.companyName }}</a
          >
          <div
            v-if="row.tags && row.tags.length > 0"
            class="mt-1 flex flex-wrap gap-1"
          >
            <Tag
              v-for="tag in row.tags"
              :key="tag.id"
              :color="tag.tagColor || 'blue'"
              class="!mr-0 !mb-1"
              style="font-size: 12px; line-height: 18px"
            >
              {{ tag.tagName }}
            </Tag>
          </div>
        </div>
      </template>

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status] || 'default'">
          {{ statusLabelMap[row.status] || row.status || '-' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <span class="action-btns">
          <a
            v-if="currentClaimMode === 1"
            v-access:code="['crm:lead-pool:claim']"
            class="action-btn"
            @click="() => confirmClaim(row)"
            >领取</a
          >
          <a
            v-else-if="currentClaimMode === 2"
            v-access:code="['crm:lead-pool:apply']"
            class="action-btn"
            @click="() => openApply([row])"
            >申领</a
          >
          <a class="action-btn" @click="() => openDetail(row)">详情</a>
          <a class="action-btn" @click="() => openHistory(row)">历史</a>
          <a
            v-if="currentClaimMode === 3 && (isManagerView || accessStore.hasAccessCode('crm:lead-pool:assign'))"
            v-access:code="['crm:lead-pool:assign']"
            class="action-btn"
            @click="() => openAssign([row])"
            >指派</a
          >
          <Popconfirm
            :title="
              $t('ui.text.do_you_want_delete', {
                moduleName: $t('page.crm.leadPool.title'),
              })
            "
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="handleDelete(row)"
          >
            <a
              v-if="accessStore.hasAccessCode('crm:lead-pool:delete')"
              class="action-btn danger"
              >删除</a
            >
          </Popconfirm>
        </span>
      </template>
    </Grid>

    <!-- 脱敏提示 -->
    <Alert
      v-if="maskTip"
      :message="maskTip"
      type="info"
      show-icon
      class="mask-alert"
      banner
    />

    <!-- 统一详情/新建 抽屉 -->
    <Drawer
      v-model:open="detailVisible"
      :width="1100"
      placement="right"
      :destroy-on-close="false"
      :mask-closable="false"
      :closable="true"
      :title="detailCreate ? '新建线索' : '线索详情'"
      :body-style="{ padding: 0, overflow: 'auto', height: '100%' }"
      @close="closeDetail"
    >
      <LeadDetail
        v-if="detailVisible"
        :key="detailKey"
        :id="detailId"
        :create="detailCreate"
        @saved="handleDetailSaved"
      />
    </Drawer>

    <!-- 申领理由 -->
    <Modal
      v-model:open="applyVisible"
      title="提交申领"
      :confirm-loading="applySaving"
      ok-text="提交"
      @ok="submitApply"
    >
      <p class="mb-2 text-gray-500">将申领 {{ applyIds.length }} 条线索，请填写申领理由：</p>
      <Textarea
        v-model:value="applyReason"
        :rows="3"
        :maxlength="200"
        placeholder="请说明申领理由（如：该客户与我负责区域匹配）"
      />
    </Modal>

    <!-- 指派 -->
    <Modal
      v-model:open="assignVisible"
      title="指派线索"
      :confirm-loading="assignSaving"
      ok-text="确认指派"
      @ok="submitAssign"
    >
      <p class="mb-2 text-gray-500">
        将指派 {{ assignIds.length }} 条线索，选择目标成员（多人按顺序均摊）：
      </p>
      <Select
        v-model:value="assignUserIds"
        mode="multiple"
        style="width: 100%"
        placeholder="请选择池成员"
        :options="
          assignCandidates.map((c) => ({
            label: `${c.userName || c.userId}（私海 ${c.holdingCount ?? 0} 条）`,
            value: String(c.userId),
          }))
        "
      />
    </Modal>

    <!-- 收回 -->
    <Modal
      v-model:open="recycleVisible"
      title="收回线索"
      :confirm-loading="recycleSaving"
      ok-text="确认收回"
      @ok="submitRecycle"
    >
      <p class="mb-2 text-gray-500">
        将把 {{ recycleIds.length }} 条私海线索收回至当前公海池，可填写收回备注：
      </p>
      <Textarea
        v-model:value="recycleRemark"
        :rows="3"
        :maxlength="200"
        placeholder="收回备注（可选）"
      />
    </Modal>

    <!-- 归属历史 -->
    <Drawer
      v-model:open="historyVisible"
      :title="`归属历史 - ${historyLeadName}`"
      :width="480"
    >
      <div v-if="historyLoading" class="text-center py-8 text-gray-400">
        加载中...
      </div>
      <Timeline v-else-if="historyList.length > 0" class="mt-4">
        <TimelineItem v-for="h in historyList" :key="h.id" :color="h.actionType === 2 || h.actionType === 5 || h.actionType === 6 ? 'red' : 'blue'">
          <div class="font-medium">
            {{ h.actionLabel || '变更' }}
            <span v-if="h.poolName" class="text-gray-500">（{{ h.poolName }}）</span>
          </div>
          <div class="text-xs text-gray-500 mt-1">
            <div v-if="h.adminName">负责人：{{ h.adminName }}</div>
            <div>
              {{ h.startTime ? formatDateTime(h.startTime) : '-' }} ～
              {{ h.endTime ? formatDateTime(h.endTime) : '至今' }}
            </div>
            <div v-if="h.operatedByName">操作人：{{ h.operatedByName }}</div>
            <div v-if="h.reason || h.remark">
              {{ h.remark || h.reason }}
            </div>
          </div>
        </TimelineItem>
      </Timeline>
      <div v-else class="text-center py-8 text-gray-400">暂无归属记录</div>
    </Drawer>

    <!-- 公海审批中心（申领审批 / 延期审批） -->
    <AuditModal
      v-model:visible="auditVisible"
      :default-tab="auditTab"
      @changed="onAuditChanged"
    />
  </Page>
</template>

<style scoped>
.action-btns {
  display: inline-flex;
  gap: 15px;
  align-items: center;
  font-size: 13px;
}

.action-btn {
  line-height: 1;
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

.action-btn.danger:hover {
  color: #ff7875;
}

.pool-tabs :deep(.ant-tabs-nav) {
  margin: 0;
}

.pool-count-tag {
  padding: 0 5px;
  font-size: 12px;
  line-height: 16px;
}

.mask-alert {
  margin-top: 8px;
}

:deep(.vxe-table--fixed-right-wrapper .vxe-body--column .vxe-cell) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  height: 100% !important;
}
</style>
