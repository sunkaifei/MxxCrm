<script lang="ts" setup>
// 审批工作台：待审批 / 已审批 双 Tab
// - 待审批：我待处理（当前节点候选审批人，状态 1/2）
// - 已审批：我处理过的全部实例（任意状态，可按状态筛选）
// - 操作列仅保留「详情」，所有处理操作（通过/驳回/退回/转办/委派/加签/抄送/取消）在详情抽屉内完成
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';
import { LucideEye } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getApprovalListApi } from '#/api';
import { $t } from '#/locales';

import ApprovalDetailDrawer from './approval-detail-drawer.vue';

const route = useRoute();
const router = useRouter();

const businessTypeMap: Record<string, { color: string; label: string }> = {
  contract: { label: '合同', color: 'geekblue' },
  expense: { label: '报销', color: 'volcano' },
  invoice: { label: '发票', color: 'purple' },
  leave: { label: '请假', color: 'orange' },
  order: { label: '订单', color: 'cyan' },
  payment: { label: '回款', color: 'gold' },
  purchase: { label: '采购', color: 'magenta' },
  quotation: { label: '报价单', color: 'blue' },
  refund: { label: '退款', color: 'red' },
  visit: { label: '外勤', color: 'lime' },
  hire: { label: '员工入职', color: 'geekblue' },
  user: { label: '员工入职', color: 'geekblue' },
  inbound: { label: '入库', color: 'lime' },
  outbound: { label: '出库', color: 'volcano' },
};

// 实例状态：1=待审批,2=审批中,3=已通过,4=已驳回,5=已撤回,6=待修改
const approvalStatusList: Record<number, { color: string; label: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

// 业务类型选项（筛选用）
const businessTypeOptions = Object.entries(businessTypeMap).map(
  ([value, { label }]) => ({ label, value }),
);

// 当前范围：todo=待审批，done=已审批
const activeTab = ref<'todo' | 'done'>('todo');

// 范围选项卡数据源（与客户列表「我的客户/全部客户」同款 Tabs 结构）
const scopeTabs = [
  { key: 'todo', label: '待审批' },
  { key: 'done', label: '已审批' },
];

// 已办视角：我在每张单上的最新处理动作文案（action 枚举与审批日志一致）
const myActionText: Record<number, string> = {
  0: '我已提交',
  1: '我已同意',
  2: '我已驳回',
  3: '我已转办',
  4: '我已委派',
  5: '我已加签',
  6: '我已退回',
  7: '我已撤回',
  8: '我已抄送',
};

// 仅已办 Tab 展示「我的动作」小徽标（待办视角 my_action 为空）
function showMyAction(row: any) {
  return (
    activeTab.value === 'done' &&
    row.myAction !== null &&
    row.myAction !== undefined
  );
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'businessTitle',
      label: '业务标题',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'businessType',
      label: '业务类型',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: businessTypeOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: Object.entries(approvalStatusList).map(([value, opt]) => ({
          label: opt.label,
          value: Number(value),
        })),
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  cellConfig: {},
  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      title: $t('ui.table.action'),
      width: 100,
    },
    {
      field: 'businessTitle',
      minWidth: 180,
      title: '业务标题',
    },
    {
      field: 'businessType',
      slots: { default: 'businessType' },
      title: '业务类型',
      width: 110,
    },
    {
      field: 'submitterName',
      title: '提交人',
      width: 120,
    },
    {
      field: 'submittedAt',
      slots: { default: 'submittedAt' },
      title: '提交时间',
      width: 170,
    },
    {
      field: 'status',
      slots: { default: 'status' },
      title: '状态',
      width: 110,
    },
  ],
  height: 'auto',
  pagerConfig: {},
  proxyConfig: {
    ajax: {
      query: async ({ page }, formValues) => {
        return await getApprovalListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          scope: activeTab.value,
          ...formValues,
        });
      },
    },
    autoLoad: true,
  },
  stripe: true,
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 选项卡切换（antd Tabs change 回调）：重新加载列表
function handleTabChange(key: number | string) {
  activeTab.value = key as 'todo' | 'done';
  gridApi.query();
}

// ============ 详情抽屉 ============
const detailVisible = ref(false);
const detailRow = ref<any>(null);

function openDetail(row: any) {
  detailRow.value = row;
  detailVisible.value = true;
}

function handleDetailSuccess() {
  gridApi.query();
}

// 从工作台待办跳转：通过 query 参数 instanceId 自动打开详情
onMounted(async () => {
  const instanceId = route.query.instanceId;
  if (instanceId) {
    // 构造最小行数据，详情抽屉按 id 加载完整数据
    openDetail({ id: Number(instanceId) });
    // 清除 query 参数，避免刷新后重复打开
    router.replace({ query: {} });
  }
});

// 已办 Tab 中提交人列加标记（当前实例已非待我处理）
const tabTitle = computed(() =>
  activeTab.value === 'todo'
    ? '待审批'
    : '已审批',
);
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="tabTitle">
      <!-- 范围选项卡注入 Grid 的 form-header 插槽，与筛选表单合并为同一张卡片 -->
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-2"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in scopeTabs"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>

      <template #businessType="{ row }">
        <Tag :color="businessTypeMap[row.businessType]?.color || 'default'">
          {{ businessTypeMap[row.businessType]?.label || row.businessType }}
        </Tag>
      </template>

      <template #submittedAt="{ row }">
        {{ formatDateTime(row.submittedAt || row.createdAt) }}
      </template>

      <template #status="{ row }">
        <div class="flex items-center gap-1">
          <Tag :color="approvalStatusList[row.status]?.color || 'default'">
            {{ approvalStatusList[row.status]?.label || '未知' }}
          </Tag>
          <!-- 已办视角：我在这张单上的最新动作，与实例终态并列展示 -->
          <Tag v-if="showMyAction(row)" color="default">
            {{ myActionText[row.myAction] || '已处理' }}
          </Tag>
        </div>
      </template>

      <template #action="{ row }">
        <Button
          :icon="h(LucideEye)"
          size="small"
          type="link"
          @click="openDetail(row)"
        >
          详情
        </Button>
      </template>
    </Grid>

    <!-- 审批详情抽屉（按业务类型分发详情 + 处理操作） -->
    <ApprovalDetailDrawer
      v-model:open="detailVisible"
      :row="detailRow"
      @success="handleDetailSuccess"
    />
  </Page>
</template>
