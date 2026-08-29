<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, reactive, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideEye, LucidePencil, LucidePlus } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  DatePicker,
  Drawer,
  Form,
  FormItem,
  Input,
  message,
  Modal,
  Popconfirm,
  Radio,
  Select,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createFollowupApi,
  deleteFollowupApi,
  getFollowupListApi,
  getLeadListApi,
  updateFollowupApi,
} from '#/api';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

import CustomerSelectModal from '../components/CustomerSelectModal.vue';
import OpportunitySelectModal from '../components/OpportunitySelectModal.vue';
import CustomerDetail from '../customer/detail.vue';
import LeadDetail from '../lead/detail.vue';
import OpportunityDetail from '../opportunity/detail.vue';
import FollowupDetail from './detail.vue';
import RecycleBin from '../components/RecycleBin.vue';

// 跟进方式映射
const activityLabelMap: Record<number, string> = {
  1: '电话',
  2: '拜访',
  3: '邮件',
  4: '会议',
  5: 'WhatsApp',
  6: '微信',
  7: '其他',
};

// 跟进来源类型映射：1=线索跟进, 2=客户跟进, 3=商机跟进
const sourceTypeLabelMap: Record<number, string> = {
  1: '线索跟进',
  2: '客户跟进',
  3: '商机跟进',
};
const sourceTypeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'green',
  3: 'orange',
};

const accessStore = useAccessStore();
const userStore = useUserStore();

// 删除显隐：本人创建 + 24h 内（后端删除守卫为准，前端仅预判）
const CRM_DELETE_WINDOW_MS = 24 * 60 * 60 * 1000;
function canDeleteFollowup(row: any): boolean {
  if (row.createdBy !== userStore.userInfo?.userId) return false;
  if (!row.createTime) return false;
  return (
    Date.now() - new Date(row.createTime).getTime() <= CRM_DELETE_WINDOW_MS
  );
}

// data_scope 决定可见的 Tab
// 1=全部数据 → 全部Tab  2=自定义 → my+subordinate+todayFollow
// 3=本部门 → my+todayFollow  4=本部门及以下 → all+my+subordinate+todayFollow
// 5=仅本人 → my+todayFollow
// 超管（user_type=1）/系统管理员（data_scope=1）统一按 dataScope=1 处理，与后端一致
const { dataScope } = useDataScopeTabs();

const activeTab = ref('my');

// 是否为下属视图（下属视图下只能查看，不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');

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
    const firstKey = keys[0];
    if (!keys.includes(activeTab.value) && firstKey !== undefined) {
      activeTab.value = firstKey;
    }
  },
  { immediate: true },
);

const { isSuperAdmin } = useSuperAdminGuard();

function handleTabChange(key: number | string) {
  activeTab.value = String(key);
  if (key === 'recycle') return;
  gridApi.query();
}

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<null | number>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) {
    message.error('跟进记录ID不存在');
    return;
  }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() {
  detailVisible.value = false;
  detailId.value = null;
}

// 线索详情抽屉
const leadDetailVisible = ref(false);
const leadDetailId = ref<null | number>(null);
const leadDetailKey = ref(0);

// 客户详情抽屉
const customerDetailVisible = ref(false);
const customerDetailId = ref<null | number>(null);
const customerDetailKey = ref(0);

// 商机详情抽屉
const opportunityDetailVisible = ref(false);
const opportunityDetailId = ref<null | number>(null);
const opportunityDetailKey = ref(0);

// 根据 sourceType 打开对应来源详情页
// 1=线索跟进 → 线索详情  2=客户跟进 → 客户详情  3=商机跟进 → 商机详情
function openSourceDetail(row: any) {
  const sourceType = row.sourceType;
  switch (sourceType) {
    case 1: {
      const id = row.leadId ?? row.lead_id;
      if (!id) {
        message.error('线索ID不存在');
        return;
      }
      leadDetailId.value = Number(id);
      leadDetailKey.value++;
      leadDetailVisible.value = true;

      break;
    }
    case 2: {
      const id = row.customerId ?? row.customer_id;
      if (!id) {
        message.error('客户ID不存在');
        return;
      }
      customerDetailId.value = Number(id);
      customerDetailKey.value++;
      customerDetailVisible.value = true;

      break;
    }
    case 3: {
      const id = row.opportunityId ?? row.opportunity_id;
      if (!id) {
        message.error('商机ID不存在');
        return;
      }
      opportunityDetailId.value = Number(id);
      opportunityDetailKey.value++;
      opportunityDetailVisible.value = true;

      break;
    }
    default: {
      message.warning('未知来源类型');
    }
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

// ========== 新增/编辑跟进 ==========
const formDrawerVisible = ref(false);
const formMode = ref<'create' | 'edit'>('create');
const editingId = ref<null | number>(null);
const formSubmitting = ref(false);

const formData = reactive({
  sourceType: 2,
  leadId: undefined as undefined | number,
  customerId: undefined as undefined | number,
  opportunityId: undefined as undefined | number,
  sourceName: '',
  activityType: 1,
  content: '',
  nextFollowDate: undefined as any,
});

const customerPickerVisible = ref(false);
const opportunityPickerVisible = ref(false);

// 线索选择（内嵌弹窗）
const leadPickerVisible = ref(false);
const leadKeyword = ref('');
const leadOptions = ref<any[]>([]);
const leadSearching = ref(false);

// 来源类型标签
const sourceLabel = computed(() => {
  const map: Record<number, string> = {
    1: '线索',
    2: '客户',
    3: '商机',
  };
  return map[formData.sourceType] || '来源';
});

const activityOptions = [1, 2, 3, 4, 5, 6, 7].map((v) => ({
  value: v,
  label: activityLabelMap[v],
}));

function resetForm() {
  formData.sourceType = 2;
  formData.leadId = undefined;
  formData.customerId = undefined;
  formData.opportunityId = undefined;
  formData.sourceName = '';
  formData.activityType = 1;
  formData.content = '';
  formData.nextFollowDate = undefined;
}

// 来源类型切换时清空已选来源
watch(
  () => formData.sourceType,
  () => {
    formData.leadId = undefined;
    formData.customerId = undefined;
    formData.opportunityId = undefined;
    formData.sourceName = '';
  },
);

function openCreate() {
  formMode.value = 'create';
  editingId.value = null;
  resetForm();
  formDrawerVisible.value = true;
}

function openEdit(row: any) {
  const id = row.id ?? row.id_;
  if (!id) return;
  formMode.value = 'edit';
  editingId.value = Number(id);
  resetForm();
  formData.sourceType = row.sourceType ?? 2;
  formData.leadId = row.leadId ?? row.lead_id;
  formData.customerId = row.customerId ?? row.customer_id;
  formData.opportunityId = row.opportunityId ?? row.opportunity_id;
  formData.activityType = row.activityType ?? row.activity_type ?? 1;
  formData.content = row.content ?? '';
  formData.nextFollowDate = row.nextFollowDate ?? row.next_follow_date;
  formData.sourceName =
    formData.sourceType === 1
      ? row.leadName || '-'
      : row.customerName || '-';
  formDrawerVisible.value = true;
}

// 打开来源选择器：1=线索内嵌弹窗，2=客户弹窗，3=商机弹窗
function openSourcePicker() {
  if (formData.sourceType === 1) {
    leadKeyword.value = '';
    handleLeadSearch('');
    leadPickerVisible.value = true;
  } else if (formData.sourceType === 2) {
    customerPickerVisible.value = true;
  } else if (formData.sourceType === 3) {
    opportunityPickerVisible.value = true;
  }
}

async function handleLeadSearch(keyword?: string) {
  leadSearching.value = true;
  try {
    const res: any = await getLeadListApi({
      page: 1,
      pageSize: 20,
      keywords: keyword || undefined,
    });
    leadOptions.value = (res as any)?.items ?? [];
  } catch {
    leadOptions.value = [];
  } finally {
    leadSearching.value = false;
  }
}

function onSelectCustomer(row: any) {
  formData.customerId = Number(row.id ?? row.id_);
  formData.sourceName = row.companyName || '-';
  customerPickerVisible.value = false;
}

function onSelectOpportunity(row: any) {
  formData.opportunityId = Number(row.id ?? row.id_);
  formData.sourceName =
    row.title || row.opportunityName || row.name || `#${row.id}`;
  opportunityPickerVisible.value = false;
}

function onSelectLead(row: any) {
  formData.leadId = Number(row.id ?? row.id_);
  formData.sourceName = row.companyName || '-';
  leadPickerVisible.value = false;
}

async function handleSubmit() {
  if (!formData.sourceName) {
    message.warning('请选择跟进来源');
    return;
  }
  if (!formData.content?.trim()) {
    message.warning('请填写跟进内容');
    return;
  }
  formSubmitting.value = true;
  try {
    const payload: any = {
      sourceType: formData.sourceType,
      leadId: formData.leadId,
      customerId: formData.customerId,
      opportunityId: formData.opportunityId,
      activityType: formData.activityType,
      content: formData.content,
      nextFollowDate: formData.nextFollowDate || undefined,
    };
    if (formMode.value === 'edit' && editingId.value !== null) {
      payload.id = String(editingId.value);
      await updateFollowupApi(payload);
      message.success('跟进记录已更新');
    } else {
      await createFollowupApi(payload);
      message.success('跟进记录已创建');
    }
    formDrawerVisible.value = false;
    gridApi.query();
  } catch {
    // 全局拦截器处理
  } finally {
    formSubmitting.value = false;
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
  cellConfig: { isHover: true } as any,
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

        // 无数据 600px，有数据最小 600px（数据超过 600px 自适应撑高）
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          if (items.length === 0) {
            gridEl.style.setProperty('height', '600px', 'important');
            gridEl.style.removeProperty('min-height');
          } else {
            gridEl.style.removeProperty('height');
            gridEl.style.setProperty('min-height', '600px', 'important');
          }
        }

        return { ...result, items };
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: '来源',
      field: 'sourceType',
      width: 100,
      align: 'center',
      slots: { default: 'sourceType' },
    },
    {
      title: '跟进内容',
      field: 'content',
      minWidth: 240,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'content' },
    },
    {
      title: '客户/线索',
      field: 'customerName',
      width: 150,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'customerName' },
    },
    {
      title: '跟进方式',
      field: 'activityType',
      width: 90,
      formatter: ({ cellValue }: any) =>
        cellValue === null || cellValue === undefined
          ? '-'
          : activityLabelMap[cellValue] || cellValue,
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
      title: '跟进时间',
      field: 'followTime',
      slots: { default: 'followTimeSlot' },
      width: 160,
    },
    {
      title: '下次跟进',
      field: 'nextFollowDate',
      width: 120,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '跟进人',
      field: 'createdByName',
      width: 100,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page>
    <Tabs
      v-model:active-key="activeTab"
      @change="handleTabChange"
      class="mb-4"
    >
      <Tabs.TabPane
        v-for="tab in tabList"
        :key="tab.key"
        :tab="tab.label"
      />
      <Tabs.TabPane v-if="isSuperAdmin" key="recycle" tab="回收站" />
    </Tabs>

    <Grid
      v-show="activeTab !== 'recycle'"
      :table-title="$t('page.crm.followup.title')"
    >

      <template #toolbar-tools>
        <Button
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('crm:followup:save')
          "
          type="primary"
          :icon="h(LucidePlus)"
          @click="openCreate"
        >
          新增跟进
        </Button>
      </template>

      <template #followTimeSlot="{ row }">
        {{ formatDateTime(row.followTime) }}
      </template>

      <template #sourceType="{ row }">
        <Tag
          v-if="row.sourceType != null"
          :color="sourceTypeColorMap[row.sourceType] || 'default'"
        >
          {{ sourceTypeLabelMap[row.sourceType] || row.sourceType }}
        </Tag>
        <span v-else>-</span>
      </template>

      <template #content="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openDetail(row)"
          >{{
            row.content?.length > 60
              ? `${row.content.slice(0, 60)}...`
              : row.content || '-'
          }}</a
        >
      </template>

      <template #customerName="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openSourceDetail(row)"
        >
          {{
            row.sourceType === 1 ? row.leadName || '-' : row.customerName || '-'
          }}
        </a>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('crm:followup:view')"
          type="link"
          :icon="h(LucideEye)"
          @click="() => openDetail(row)"
        />
        <Button
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('crm:followup:update')
          "
          type="link"
          :icon="h(LucidePencil)"
          @click="() => openEdit(row)"
        />
        <Popconfirm
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('crm:followup:delete') &&
            canDeleteFollowup(row)
          "
          title="确定删除该跟进记录？"
          ok-text="确认"
          cancel-text="取消"
          @confirm="handleDelete(row)"
        >
          <Button type="link" danger>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M3 6h18" />
              <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
              <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
            </svg>
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <RecycleBin v-show="activeTab === 'recycle'" :module="'followup'" />

    <Drawer
      v-model:open="detailVisible"
      width="min(860px, 92vw)"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      title="跟进记录详情"
      :body-style="{
        padding: 0,
        maxHeight: 'calc(100vh - 110px)',
        overflow: 'auto',
      }"
      @close="closeDetail"
    >
      <FollowupDetail v-if="detailId" :id="detailId" />
    </Drawer>

    <Drawer
      v-model:open="leadDetailVisible"
      width="min(1100px, 95vw)"
      placement="right"
      :destroy-on-close="false"
      :mask-closable="false"
      :closable="true"
      title="线索详情"
      :body-style="{ padding: 0, overflow: 'auto', height: '100%' }"
      @close="closeLeadDetail"
    >
      <LeadDetail
        v-if="leadDetailVisible"
        :key="leadDetailKey"
        :id="leadDetailId"
      />
    </Drawer>

    <Drawer
      v-model:open="customerDetailVisible"
      width="min(1000px, 95vw)"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      title="客户详情"
      :body-style="{
        padding: 0,
        maxHeight: 'calc(100vh - 110px)',
        overflow: 'auto',
      }"
      @close="closeCustomerDetail"
    >
      <CustomerDetail
        v-if="customerDetailId"
        :key="customerDetailKey"
        :id="customerDetailId"
      />
    </Drawer>

    <Drawer
      v-model:open="opportunityDetailVisible"
      width="min(1100px, 95vw)"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      title="商机详情"
      :body-style="{ padding: 0, overflow: 'auto', height: '100%' }"
      @close="closeOpportunityDetail"
    >
      <OpportunityDetail
        v-if="opportunityDetailId"
        :key="opportunityDetailKey"
        :id="opportunityDetailId"
      />
    </Drawer>

    <!-- 新增/编辑跟进 -->
    <Drawer
      v-model:open="formDrawerVisible"
      width="min(560px, 92vw)"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      :title="formMode === 'edit' ? '编辑跟进记录' : '新增跟进记录'"
    >
      <Form layout="vertical">
        <FormItem label="来源类型" required>
          <Radio.Group v-model:value="formData.sourceType">
            <Radio :value="1">线索跟进</Radio>
            <Radio :value="2">客户跟进</Radio>
            <Radio :value="3">商机跟进</Radio>
          </Radio.Group>
        </FormItem>
        <FormItem :label="`${sourceLabel}来源`" required>
          <Input
            v-model:value="formData.sourceName"
            readonly
            :placeholder="`点击右侧按钮选择${sourceLabel}`"
          />
          <Button class="mt-2" type="primary" ghost @click="openSourcePicker">
            选择{{ sourceLabel }}
          </Button>
        </FormItem>
        <FormItem label="跟进方式">
          <Select
            v-model:value="formData.activityType"
            :options="activityOptions"
            style="width: 100%"
          />
        </FormItem>
        <FormItem label="跟进内容" required>
          <Input.TextArea
            v-model:value="formData.content"
            :rows="4"
            placeholder="请输入跟进内容"
            :maxlength="1000"
            show-count
          />
        </FormItem>
        <FormItem label="下次跟进时间">
          <DatePicker
            v-model:value="formData.nextFollowDate"
            value-format="YYYY-MM-DD"
            style="width: 100%"
            placeholder="选择下次跟进时间"
          />
        </FormItem>
      </Form>
      <div class="flex justify-end gap-2 mt-4">
        <Button @click="formDrawerVisible = false">取消</Button>
        <Button type="primary" :loading="formSubmitting" @click="handleSubmit">
          保存
        </Button>
      </div>
    </Drawer>

    <!-- 线索选择弹窗 -->
    <Modal
      v-model:open="leadPickerVisible"
      title="选择线索"
      :footer="null"
      :width="640"
    >
      <div class="flex items-center gap-2 mb-3">
        <Input
          v-model:value="leadKeyword"
          placeholder="输入公司名称/联系人搜索"
          allow-clear
          @press-enter="handleLeadSearch(leadKeyword)"
        />
        <Button type="primary" @click="handleLeadSearch(leadKeyword)">
          搜索
        </Button>
      </div>
      <div v-if="leadSearching" class="py-8 text-center text-gray-400">
        加载中...
      </div>
      <div
        v-else-if="leadOptions.length === 0"
        class="py-8 text-center text-gray-400"
      >
        暂无数据
      </div>
      <div v-else class="max-h-80 overflow-auto border rounded">
        <div
          v-for="item in leadOptions"
          :key="item.id"
          class="flex items-center justify-between px-3 py-2 border-b last:border-b-0 cursor-pointer hover:bg-blue-50"
          @click="onSelectLead(item)"
        >
          <div>
            <div class="font-medium">{{ item.companyName || '-' }}</div>
            <div class="text-xs text-gray-500">
              {{ item.contactName ? `联系人：${item.contactName}` : '' }}
            </div>
          </div>
          <Button size="small" type="primary" @click.stop="onSelectLead(item)">
            选择
          </Button>
        </div>
      </div>
    </Modal>

    <!-- 客户选择弹窗 -->
    <CustomerSelectModal
      v-model:visible="customerPickerVisible"
      @select="onSelectCustomer"
    />
    <!-- 商机选择弹窗 -->
    <OpportunitySelectModal
      v-model:visible="opportunityPickerVisible"
      @select="onSelectOpportunity"
    />
  </Page>
</template>
