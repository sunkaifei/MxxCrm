<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, nextTick, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideMoreHorizontal, LucideChevronDown } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Alert, Button, Popconfirm, Drawer, Dropdown, Menu, Modal, Tabs, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { useVbenDrawer } from '#/adapter/drawer';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import {
  convertOpportunityToOrderApi,
  deleteOpportunityApi,
  getOpportunityListApi,
  getSalesFlowModeApi,
  type SalesFlowMode,
} from '#/api';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import OpportunityDetail from './detail.vue';
import QuotationDrawer from '../../sale/quotation/drawer.vue';
import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';
import SalesProcessGuide from '../../sale/components/SalesProcessGuide.vue';

// 商机管理使用说明步骤数（与 i18n 中 page.crm.opportunity.guide.steps 数组对齐）
const guideStepCount = 5;

const accessStore = useAccessStore();
const userStore = useUserStore();
const { isSuperAdmin, guardBusiness } = useSuperAdminGuard();

// 销售流程模式：A=仅标准(转报价单) B=仅简易(转订单) both=两种都允许
const flowMode = ref<SalesFlowMode>('both');
const loadFlowMode = async () => {
  try {
    flowMode.value = await getSalesFlowModeApi();
  } catch {
    flowMode.value = 'both';
  }
};
// 是否显示"转报价单"入口
const canConvertToQuotation = computed(
  () => flowMode.value === 'A' || flowMode.value === 'both',
);
// 是否显示"转订单"入口
const canConvertToOrder = computed(
  () => flowMode.value === 'B' || flowMode.value === 'both',
);
// 是否显示"更多"下拉（仅当至少有一个转换入口可用时）
const showMoreActions = computed(
  () => canConvertToQuotation.value || canConvertToOrder.value,
);

// 转报价单：打开报价单新建页，自动带入商机信息（商机、客户、联系人）
function handleConvertToQuotation(row: any) {
  quotationDrawerApi.setData({ create: true, fromOpportunity: row });
  quotationDrawerApi.open();
}

// 转订单（简易流程）
async function handleConvertToOrder(row: any) {
  const id = row.id ?? row.id_;
  if (!id) return;
  Modal.confirm({
    title: '转订单',
    content: `确定要将商机「${row.title || ''}」直接转为订单吗？转换后将创建订单草稿，可继续完善明细。`,
    okText: '确定',
    cancelText: '取消',
    onOk: async () => {
      try {
        await convertOpportunityToOrderApi(Number(id));
        message.success('已转为订单');
        gridApi.query();
      } catch {
        /* ignore */
      }
    },
  });
}

// 全部商机 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 1;
});

// 下属商机 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const activeTab = ref('my');
// 是否为下属视图（下属视图下只能查看，不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');
const allTabList = [
  { key: 'all', label: '全部商机' },
  { key: 'my', label: '我的商机' },
  { key: 'subordinate', label: '下属商机' },
];
const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter(t => keys.includes(t.key));
});
// 当Tab权限变化时，确保当前激活的Tab仍然可见
watch(tabList, (newTabs) => {
  const keys = newTabs.map(t => t.key);
  if (!keys.includes(activeTab.value) && keys.length > 0) {
    activeTab.value = keys[0]!;
  }
  // 根据当前tab控制"负责人"列显隐（"我的商机"下隐藏）
  nextTick(() => {
    if (activeTab.value === 'my') {
      gridApi.grid?.hideColumn('assigneeName');
    } else {
      gridApi.grid?.showColumn('assigneeName');
    }
  });
}, { immediate: true });

function handleTabChange(key: string | number) {
  activeTab.value = key as string;
  // "我的商机"tab下隐藏"负责人"列（只看自己的，无需显示）；其他tab显示
  if (key === 'my') {
    gridApi.grid?.hideColumn('assigneeName');
  } else {
    gridApi.grid?.showColumn('assigneeName');
  }
  gridApi.query();
}

// 来源映射 - 对齐后端 LeadSource 枚举（数字值）
const sourceLabelMap: Record<string, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
};

// 币种标签映射 - 对齐后端 CurrencyCode 枚举（数字值）
const currencyLabelMap: Record<number, string> = {
  1: 'CNY', 2: 'USD', 3: 'EUR', 4: 'GBP', 5: 'JPY', 6: 'HKD', 7: 'AUD',
};

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<number | null>(null);
const detailTitle = computed(() => detailId.value ? '商机详情' : '新建商机');

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('商机ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
// 详情页内已支持内联编辑，edit 事件仅刷新列表
function handleDetailEdit() { gridApi.query(); }

// 客户详情抽屉
const customerDetailVisible = ref(false);
const customerDetailId = ref<number | string | undefined>(undefined);

function openCustomerDetail(row: any) {
  const id = row.customerId ?? row.customer_id;
  if (!id) { message.error('客户ID不存在'); return; }
  customerDetailId.value = Number(id);
  customerDetailVisible.value = true;
}

function handleConverted(_quotationId: number | string) {
  gridApi.query();
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '商机名称',
      componentProps: { placeholder: '输入商机名称/编号搜索', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'stage',
      label: '销售阶段',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '初步沟通', value: 1 },
          { label: '需求确认', value: 2 },
          { label: '方案沟通', value: 3 },
          { label: '已报价', value: 4 },
          { label: '成交/丢单', value: 5 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '商机来源',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '官网', value: 1 },
          { label: '展会', value: 2 },
          { label: '社交媒体', value: 3 },
          { label: '客户转介', value: 4 },
          { label: '陌生拜访', value: 5 },
          { label: '海关数据', value: 6 },
          { label: '邮件营销', value: 7 },
          { label: '阿里国际站', value: 8 },
          { label: 'Amazon', value: 9 },
          { label: 'TikTok', value: 10 },
          { label: '微信', value: 11 },
          { label: '其他', value: 12 },
        ],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: { placeholder: '客户ID', allowClear: true, min: 0 },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' as any },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getOpportunityListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          ...formValues,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    
    { title: '商机名称', field: 'title', minWidth: 200, align: 'left', headerAlign: 'center', slots: { default: 'title' } },
    { title: '客户', field: 'customerName', width: 150, align: 'left', headerAlign: 'center', slots: { default: 'customerName' } },
    {
      title: '销售阶段', field: 'stage', width: 110,
      formatter: ({ cellValue }: any) => {
        const stageMap: Record<number, string> = { 1: '初步沟通', 2: '需求确认', 3: '方案沟通', 4: '已报价', 5: '成交/丢单' };
        return stageMap[cellValue] ?? '-';
      },
    },
    {
      title: '预算金额', field: 'amount', width: 140,
      formatter: ({ cellValue, row }: any) => {
        if (cellValue == null) return '-';
        const currencyLabel = currencyLabelMap[row.currency] || '';
        return `${currencyLabel} ${Number(cellValue).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
      },
    },
    { title: '报价次数', field: 'quoteCount', width: 90, align: 'center', formatter: ({ cellValue }: any) => cellValue ?? 0 },
    {
      title: '概率', field: 'probability', width: 80, align: 'center',
      formatter: ({ cellValue }: any) => (cellValue == null ? '-' : `${cellValue}%`),
    },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '预计成交日', field: 'expectedCloseDate', width: 120 },
    { title: '负责人', field: 'assigneeName', width: 90, visible: true },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 150,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 报价单抽屉（从商机转入）
const [QuotationFormDrawer, quotationDrawerApi] = useVbenDrawer({
  connectedComponent: QuotationDrawer,
  onClosed() {
    const data = quotationDrawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function handleCreate() {
  if (guardBusiness('商机')) return;
  detailId.value = null;
  detailVisible.value = true;
}

function handleCreated(id: number | string) {
  detailId.value = Number(id);
  gridApi.query();
}

// 编辑改为打开详情页（详情页内已有内联编辑表单）
function handleEdit(row: any) { openDetail(row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteOpportunityApi([row.id]); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的商机'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个商机？`,
    onOk: async () => {
      try {
        const ids = records.map((r: any) => r.id);
        await deleteOpportunityApi(ids);
        message.success(`已删除 ${records.length} 个商机`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}

// 页面初始化时加载销售流程模式
loadFlowMode();
</script>

<template>
  <Page>
    <PageUsageGuide
      :title="$t('page.crm.opportunity.guide.title')"
      :brief="$t('page.crm.opportunity.guide.brief')"
      :expand-text="$t('page.crm.opportunity.guide.expand')"
      :collapse-text="$t('page.crm.opportunity.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.crm.opportunity.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.crm.opportunity.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <SalesProcessGuide current-step="opportunity" />
    <a-alert
      v-if="isSuperAdmin"
      type="info"
      show-icon
      message="您当前是超级管理员，仅可查看数据。创建商机等业务操作请使用业务账号登录。"
      style="margin-bottom: 12px;"
    />
    <Grid :table-title="$t('page.crm.opportunity.title')">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button v-if="!isSubordinateView && !isSuperAdmin && accessStore.hasAccessCode('crm:opportunity:save')" type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.crm.opportunity.button.create') }}
        </Button>
        <Button v-if="!isSubordinateView" @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #title="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.title }}</a>
      </template>

      <template #customerName="{ row }">
        <a v-if="row.customerId" class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openCustomerDetail(row)">{{ row.customerName || '-' }}</a>
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <template #action="{ row }">
        <template v-if="isSubordinateView">
          <!-- 下属视图不展示操作按钮，用占位符保持行高与其他列一致 -->
          <span class="text-gray-400">-</span>
        </template>
        <template v-else>
          <a
            v-if="accessStore.hasAccessCode('crm:opportunity:update')"
            class="text-blue-600 cursor-pointer mr-3"
            @click="() => handleEdit(row)"
          >
            修改
          </a>
          <Popconfirm
            :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.opportunity.title') })"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="handleDelete(row)"
          >
            <a
              v-if="accessStore.hasAccessCode('crm:opportunity:delete')"
              class="text-red-500 cursor-pointer mr-3"
            >
              删除
            </a>
          </Popconfirm>
          <Dropdown
            v-if="showMoreActions && accessStore.hasAccessCode('crm:opportunity:update')"
            :trigger="['click']"
          >
            <a class="text-blue-600 cursor-pointer" @click.prevent>
              更多<LucideChevronDown class="inline-block ml-0.5" :size="12" />
            </a>
            <template #overlay>
              <Menu>
                <Menu.Item v-if="canConvertToQuotation" key="toQuotation" @click="handleConvertToQuotation(row)">
                  一键转报价单
                </Menu.Item>
                <Menu.Item v-if="canConvertToOrder" key="toOrder" @click="handleConvertToOrder(row)">
                  一键转订单
                </Menu.Item>
              </Menu>
            </template>
          </Dropdown>
        </template>
      </template>
    </Grid>

    <Drawer v-model:open="detailVisible" :width="1200" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" :title="detailTitle" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <OpportunityDetail :id="detailId ?? undefined" @edit="handleDetailEdit" @converted="handleConverted" @created="handleCreated" />
    </Drawer>

    <CustomerDetailDrawer v-model:visible="customerDetailVisible" :id="customerDetailId" />
    <QuotationFormDrawer />
  </Page>
</template>