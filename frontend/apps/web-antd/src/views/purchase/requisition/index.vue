<script lang="ts" setup>
import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideCheck, LucideFilePenLine, LucideSend, LucideTrash2, LucideUndo2, LucideX } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Modal, Popconfirm, Select, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  convertToPoApi,
  deleteRequisitionApi,
  getRequisitionListApi,
  getSupplierListApi,
  rejectRequisitionApi,
  submitRequisitionApi,
  approveRequisitionApi,
  withdrawRequisitionApi,
} from '#/api';
import { $t } from '#/locales';

import RequisitionDrawer from './drawer.vue';

const accessStore = useAccessStore();

const typeOptions = [
  { label: '缺货补货', value: 'replenish' },
  { label: '辅材采购', value: 'consumable' },
  { label: '备货采购', value: 'stock' },
  { label: '其他', value: 'other' },
];

const statusOptions = [
  { label: '草稿', value: 0 },
  { label: '待审批', value: 1 },
  { label: '已通过', value: 2 },
  { label: '已驳回', value: 3 },
  { label: '已撤回', value: 4 },
  { label: '已转采购订单', value: 5 },
  { label: '已完成', value: 6 },
];

const statusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '待审批',
  2: '已通过',
  3: '已驳回',
  4: '已撤回',
  5: '已转采购订单',
  6: '已完成',
};

const statusColorMap: Record<number, string> = {
  0: 'default',
  1: 'orange',
  2: 'green',
  3: 'red',
  4: 'default',
  5: 'blue',
  6: 'green',
};

const typeLabelMap: Record<string, string> = {
  replenish: '缺货补货',
  consumable: '辅材采购',
  stock: '备货采购',
  other: '其他',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '申请单号',
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
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: statusOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'type',
      label: '申请类型',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: typeOptions,
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
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getRequisitionListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          status: formValues.status,
          type: formValues.type,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { type: 'seq', title: $t('ui.table.seq'), width: 60 },
    { title: '申请单号', field: 'requisitionNo', width: 140 },
    { title: '标题', field: 'title', width: 180 },
    { title: '申请类型', field: 'type', width: 100, slots: { default: 'type' } },
    { title: '申请人', field: 'applicantName', width: 100 },
    { title: '部门', field: 'department', width: 100 },
    { title: '期望到货日', field: 'expectedDate', width: 110 },
    { title: '紧急程度', field: 'urgency', width: 90, slots: { default: 'urgency' } },
    { title: '预估总金额', field: 'estimatedAmount', width: 110 },
    { title: $t('ui.table.status'), field: 'status', width: 110, slots: { default: 'status' } },
    { title: $t('ui.table.createTime'), field: 'createTime', width: 160, slots: { default: 'createTime' } },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 280 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: RequisitionDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteRequisitionApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleSubmit(row: any) {
  row.pending = true;
  try {
    await submitRequisitionApi(row.id);
    window.$message.success('已提交审批');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleApprove(row: any) {
  row.pending = true;
  try {
    await approveRequisitionApi(row.id);
    window.$message.success('已审批通过');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleReject(row: any) {
  row.pending = true;
  try {
    await rejectRequisitionApi(row.id);
    window.$message.success('已驳回');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleWithdraw(row: any) {
  row.pending = true;
  try {
    await withdrawRequisitionApi(row.id);
    window.$message.success('已撤回');
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

// ========== 转采购订单 ==========
const convertVisible = ref(false);
const convertLoading = ref(false);
const convertPrIds = ref<number[]>([]);
const convertSupplierId = ref<number | undefined>(undefined);
const supplierOptions = ref<Array<{ value: number; label: string }>>([]);

async function loadSupplierOptions() {
  try {
    const res = await getSupplierListApi({ page: 1, pageSize: 999 });
    const list = (res as any)?.items || (res as any)?.rows || [];
    supplierOptions.value = list.map((s: any) => ({
      value: Number(s.id),
      label: s.companyName || s.shortName || `供应商#${s.id}`,
    }));
  } catch {
    // ignore
  }
}

function openConvertModal(row?: any) {
  if (row) {
    convertPrIds.value = [Number(row.id)];
  } else {
    const records = gridApi.grid?.getCheckboxRecords() ?? [];
    if (!records.length) {
      window.$message.warning('请先选择要转换的采购申请');
      return;
    }
    const invalid = records.filter((r: any) => r.status !== 2);
    if (invalid.length > 0) {
      window.$message.warning('只能转换已通过的采购申请');
      return;
    }
    convertPrIds.value = records.map((r: any) => Number(r.id));
  }
  convertSupplierId.value = undefined;
  convertVisible.value = true;
  loadSupplierOptions();
}

async function handleConvertToPo() {
  if (!convertSupplierId.value) {
    window.$message.warning('请选择供应商');
    return;
  }
  convertLoading.value = true;
  try {
    await convertToPoApi({
      prIds: convertPrIds.value,
      supplierId: convertSupplierId.value,
    });
    window.$message.success(`已转换 ${convertPrIds.value.length} 条采购申请为采购订单`);
    convertVisible.value = false;
    gridApi.query();
  } finally {
    convertLoading.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.purchase.requisition.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('purchase:requisition:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.purchase.requisition.button.create') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('purchase:requisition:convert')"
          class="mr-2"
          @click="openConvertModal()"
        >
          批量转采购订单
        </Button>
      </template>

      <template #type="{ row }">
        {{ typeLabelMap[row.type] || row.type }}
      </template>

      <template #urgency="{ row }">
        <Tag v-if="row.urgency === 0" color="default">普通</Tag>
        <Tag v-else-if="row.urgency === 1" color="orange">紧急</Tag>
        <Tag v-else-if="row.urgency === 2" color="red">非常紧急</Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status]">
          {{ statusLabelMap[row.status] || row.status }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('purchase:requisition:edit')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <template v-if="row.status === 0">
          <Popconfirm
            title="确认提交审批？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleSubmit(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('purchase:requisition:submit')"
              type="link"
              :icon="h(LucideSend)"
            />
          </Popconfirm>
        </template>
        <template v-if="row.status === 1">
          <Popconfirm
            title="确认审批通过？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleApprove(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('purchase:requisition:approve')"
              type="link"
              :icon="h(LucideCheck)"
            />
          </Popconfirm>
          <Popconfirm
            title="确认驳回？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleReject(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('purchase:requisition:reject')"
              type="link"
              :icon="h(LucideX)"
            />
          </Popconfirm>
          <Popconfirm
            title="确认撤回该申请？"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleWithdraw(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('purchase:requisition:withdraw')"
              type="link"
              :icon="h(LucideUndo2)"
            />
          </Popconfirm>
        </template>
        <template v-if="row.status === 2">
          <Button
            v-if="accessStore.hasAccessCode('purchase:requisition:convert')"
            type="link"
            @click="() => openConvertModal(row)"
          >
            转采购订单
          </Button>
        </template>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '采购申请' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('purchase:requisition:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />
    <Modal
      v-model:open="convertVisible"
      title="转采购订单"
      :confirm-loading="convertLoading"
      :ok-text="$t('ui.button.ok')"
      :cancel-text="$t('ui.button.cancel')"
      @ok="handleConvertToPo"
    >
      <div class="py-2">
        <p class="mb-3 text-gray-600">
          将 {{ convertPrIds.length }} 条采购申请合并转换为采购订单，请选择供应商：
        </p>
        <Select
          v-model:value="convertSupplierId"
          placeholder="请选择供应商"
          :options="supplierOptions"
          show-search
          :filter-option="(input: string, option: any) => option.label?.toLowerCase().includes(input.toLowerCase())"
          style="width: 100%"
        />
      </div>
    </Modal>
  </Page>
</template>