<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideEye } from '@vben/icons';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, message, Popconfirm, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteFollowupApi, getFollowupListApi } from '#/api';
import { $t } from '#/locales';
import LeadDetail from '../lead/detail.vue';
import CustomerDetail from '../customer/detail.vue';
import OpportunityDetail from '../opportunity/detail.vue';
import FollowupDetail from './detail.vue';

// 跟进方式映射
const activityLabelMap: Record<number, string> = {
  1: '电话', 2: '拜访', 3: '邮件', 4: '会议',
  5: 'WhatsApp', 6: '微信', 7: '其他',
};

// 跟进来源类型映射：1=线索跟进, 2=客户跟进, 3=商机跟进
const sourceTypeLabelMap: Record<number, string> = {
  1: '线索跟进', 2: '客户跟进', 3: '商机跟进',
};
const sourceTypeColorMap: Record<number, string> = {
  1: 'blue', 2: 'green', 3: 'orange',
};

const userStore = useUserStore();

// data_scope 决定可见的 Tab
// 1=全部数据 → 全部Tab  2=自定义 → my+subordinate+todayFollow
// 3=本部门 → my+todayFollow  4=本部门及以下 → all+my+subordinate+todayFollow
// 5=仅本人 → my+todayFollow
const dataScope = computed(() => {
  const scope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  const roles = userStore.userInfo?.roles ?? [];
  if (roles.includes('super_admin') || roles.includes('system_admin')) return 1;
  return typeof scope === 'number' ? scope : 5;
});

const activeTab = ref('my');
const allTabList = [
  { key: 'all', label: '全部跟进' },
  { key: 'my', label: '我的跟进' },
  { key: 'subordinate', label: '下属跟进' },
  { key: 'todayFollow', label: '今日跟进' },
];
// 根据 data_scope 过滤可见的Tab
const tabList = computed(() => {
  const scope = dataScope.value;
  let allowedKeys: string[];
  switch (scope) {
    case 1: {
      allowedKeys = ['all', 'my', 'subordinate', 'todayFollow'];
      break;
    }
    case 2:
    case 4: {
      allowedKeys = ['my', 'subordinate', 'todayFollow'];
      break;
    }
    case 3:
    case 5:
    default: {
      allowedKeys = ['my', 'todayFollow'];
      break;
    }
  }
  return allTabList.filter((t) => allowedKeys.includes(t.key));
});
// 当Tab权限变化时，确保当前激活的Tab仍然可见
watch(
  tabList,
  (newTabs) => {
    const keys = newTabs.map((t) => t.key);
    if (!keys.includes(activeTab.value) && keys.length > 0) {
      activeTab.value = keys[0]!;
    }
  },
  { immediate: true },
);

function handleTabChange(key: string | number) {
  activeTab.value = String(key);
  gridApi.query();
}

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('跟进记录ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }

// 线索详情抽屉
const leadDetailVisible = ref(false);
const leadDetailId = ref<number | null>(null);
const leadDetailKey = ref(0);

// 客户详情抽屉
const customerDetailVisible = ref(false);
const customerDetailId = ref<number | null>(null);
const customerDetailKey = ref(0);

// 商机详情抽屉
const opportunityDetailVisible = ref(false);
const opportunityDetailId = ref<number | null>(null);
const opportunityDetailKey = ref(0);

// 根据 sourceType 打开对应来源详情页
// 1=线索跟进 → 线索详情  2=客户跟进 → 客户详情  3=商机跟进 → 商机详情
function openSourceDetail(row: any) {
  const sourceType = row.sourceType;
  if (sourceType === 1) {
    const id = row.leadId ?? row.lead_id;
    if (!id) {
      message.error('线索ID不存在');
      return;
    }
    leadDetailId.value = Number(id);
    leadDetailKey.value++;
    leadDetailVisible.value = true;
  } else if (sourceType === 2) {
    const id = row.customerId ?? row.customer_id;
    if (!id) {
      message.error('客户ID不存在');
      return;
    }
    customerDetailId.value = Number(id);
    customerDetailKey.value++;
    customerDetailVisible.value = true;
  } else if (sourceType === 3) {
    const id = row.opportunityId ?? row.opportunity_id;
    if (!id) {
      message.error('商机ID不存在');
      return;
    }
    opportunityDetailId.value = Number(id);
    opportunityDetailKey.value++;
    opportunityDetailVisible.value = true;
  } else {
    message.warning('未知来源类型');
  }
}

function closeLeadDetail() {
  leadDetailVisible.value = false;
  leadDetailId.value = null;
}

function closeCustomerDetail() {
  customerDetailVisible.value = false;
  customerDetailId.value = null;
}

function closeOpportunityDetail() {
  opportunityDetailVisible.value = false;
  opportunityDetailId.value = null;
}

async function handleDelete(row: any) {
  const id = row.id ?? row.id_;
  if (!id) return;
  try {
    await deleteFollowupApi([Number(id)]);
    message.success('删除成功');
    gridApi.query();
  } catch {
    // 全局拦截器处理
  }
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'customerName',
      label: '客户',
      componentProps: { placeholder: '输入客户名称', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'activityType',
      label: '跟进方式',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '电话', value: 1 },
          { label: '拜访', value: 2 },
          { label: '邮件', value: 3 },
          { label: '会议', value: 4 },
          { label: 'WhatsApp', value: 5 },
          { label: '微信', value: 6 },
          { label: '其他', value: 7 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'sourceType',
      label: '跟进来源',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '线索跟进', value: 1 },
          { label: '客户跟进', value: 2 },
          { label: '商机跟进', value: 3 },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getFollowupListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
          listType: activeTab.value,
        });

        const items = (result as any)?.items ?? [];

        // 无数据 150px，有数据按内容自适应（避免内部滚动条）
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '150px' : '';
        }

        return { ...result, items };
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: '来源', field: 'sourceType', width: 100, align: 'center',
      slots: { default: 'sourceType' },
    },
    { title: '跟进内容', field: 'content', minWidth: 240, headerAlign: 'center', align: 'left', slots: { default: 'content' } },
    {
      title: '客户/线索', field: 'customerName', width: 150, headerAlign: 'center', align: 'left', slots: { default: 'customerName' },
    },
    {
      title: '跟进方式', field: 'activityType', width: 90,
      formatter: ({ cellValue }: any) => cellValue != null ? (activityLabelMap[cellValue] || cellValue) : '-',
      cellRender: {
        name: 'Tag',
        options: [
          { value: 1, label: '电话', color: 'blue' },
          { value: 2, label: '拜访', color: 'cyan' },
          { value: 3, label: '邮件', color: 'purple' },
          { value: 4, label: '会议', color: 'orange' },
          { value: 5, label: 'WhatsApp', color: 'lime' },
          { value: 6, label: '微信', color: 'lime' },
          { value: 7, label: '其他', color: 'default' },
        ],
      },
    },
    {
      title: '跟进时间', field: 'followTime', slots: { default: 'followTimeSlot' }, width: 160,
    },
    {
      title: '下次跟进', field: 'nextFollowDate', width: 120,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '跟进人', field: 'createdByName', width: 100,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.crm.followup.title')">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" @change="handleTabChange" class="mb-4">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>

      <template #followTimeSlot="{ row }">
        {{ formatDateTime(row.followTime) }}
      </template>

      <template #sourceType="{ row }">
        <Tag v-if="row.sourceType != null" :color="sourceTypeColorMap[row.sourceType] || 'default'">
          {{ sourceTypeLabelMap[row.sourceType] || row.sourceType }}
        </Tag>
        <span v-else>-</span>
      </template>

      <template #content="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.content?.length > 60 ? row.content.slice(0, 60) + '...' : row.content || '-' }}</a>
      </template>

      <template #customerName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openSourceDetail(row)">
          {{ row.sourceType === 1 ? (row.leadName || '-') : (row.customerName || '-') }}
        </a>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Popconfirm title="确定删除该跟进记录？" ok-text="确认" cancel-text="取消" @confirm="handleDelete(row)">
          <Button type="link" danger>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <Drawer v-model:open="detailVisible" :width="'min(860px, 92vw)'" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="跟进记录详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <FollowupDetail v-if="detailId" :id="detailId" />
    </Drawer>

    <Drawer v-model:open="leadDetailVisible" :width="'min(1100px, 95vw)'" placement="right" :destroy-on-close="false" :mask-closable="false" :closable="true" title="线索详情" :body-style="{ padding: 0, overflow: 'auto', height: '100%' }" @close="closeLeadDetail">
      <LeadDetail v-if="leadDetailVisible" :key="leadDetailKey" :id="leadDetailId" />
    </Drawer>

    <Drawer v-model:open="customerDetailVisible" :width="'min(1000px, 95vw)'" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="客户详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeCustomerDetail">
      <CustomerDetail v-if="customerDetailId" :key="customerDetailKey" :id="customerDetailId" />
    </Drawer>

    <Drawer v-model:open="opportunityDetailVisible" :width="'min(1100px, 95vw)'" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="商机详情" :body-style="{ padding: 0, overflow: 'auto', height: '100%' }" @close="closeOpportunityDetail">
      <OpportunityDetail v-if="opportunityDetailId" :key="opportunityDetailKey" :id="opportunityDetailId" />
    </Drawer>
  </Page>
</template>
