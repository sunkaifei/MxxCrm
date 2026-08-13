<script lang="ts" setup>
import { onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Dropdown, Menu, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { requestClient } from '#/api/request';
import { deleteInboundApi, getInboundListApi } from '#/api/core/product/inbound';
import { getSettingConfigApi } from '#/api/core/system/setting';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';
import InboundDetailDrawer from './detail-drawer.vue';
import InboundDrawer from './drawer.vue';

const accessStore = useAccessStore();

// 入库审核开关
const inboundAuditEnabled = ref(true);

onMounted(async () => {
  try {
    const data = await getSettingConfigApi();
    inboundAuditEnabled.value = data?.inboundAuditEnabled ?? true;
  } catch {
    // 忽略加载错误，默认开启审核
  }
});

// 入库类型选项
const inboundTypeOptions = [
  { label: $t('page.product.inbound.type.purchase'), value: 'purchase' },
  { label: $t('page.product.inbound.type.return'), value: 'return' },
  { label: $t('page.product.inbound.type.surplus'), value: 'surplus' },
  { label: $t('page.product.inbound.type.initial'), value: 'initial' },
  { label: $t('page.product.inbound.type.other'), value: 'other' },
];

// 入库状态选项
const statusOptions = [
  { label: $t('page.product.inbound.status.0'), value: 0 },
  { label: $t('page.product.inbound.status.1'), value: 1 },
  { label: $t('page.product.inbound.status.2'), value: 2 },
  { label: $t('page.product.inbound.status.3'), value: 3 },
  { label: $t('page.product.inbound.status.4'), value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'inboundNo',
      label: $t('page.product.inbound.field.inboundNo'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.inboundNo'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'inboundType',
      label: $t('page.product.inbound.field.inboundType'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.inboundType'),
        options: inboundTypeOptions,
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'warehouseId',
      label: $t('page.product.inbound.field.warehouse'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.warehouse'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.inbound.field.status'),
      componentProps: {
        placeholder: $t('page.product.inbound.placeholder.status'),
        options: statusOptions,
        allowClear: true,
      },
    },
  ],
};

// 入库类型标签映射
function getInboundTypeTag(type: string) {
  const map: Record<string, { label: string; color: string }> = {
    purchase: { label: $t('page.product.inbound.type.purchase'), color: 'blue' },
    return: { label: $t('page.product.inbound.type.return'), color: 'orange' },
    surplus: { label: $t('page.product.inbound.type.surplus'), color: 'green' },
    initial: { label: $t('page.product.inbound.type.initial'), color: 'cyan' },
    other: { label: $t('page.product.inbound.type.other'), color: 'default' },
  };
  return map[type] || { label: $t('page.product.inbound.type.unknown'), color: 'default' };
}

// 入库状态标签映射
function getInboundStatusTag(status: number) {
  const map: Record<number, { label: string; color: string }> = {
    0: { label: $t('page.product.inbound.status.0'), color: 'default' },
    1: { label: $t('page.product.inbound.status.1'), color: 'processing' },
    2: { label: $t('page.product.inbound.status.2'), color: 'warning' },
    3: { label: $t('page.product.inbound.status.3'), color: 'success' },
    4: { label: $t('page.product.inbound.status.4'), color: 'error' },
  };
  return map[status] || { label: $t('page.product.inbound.status.unknown'), color: 'default' };
}

// 审核通过时不可编辑/删除
function isLockedByApproval(row: any): boolean {
  return inboundAuditEnabled.value && row.status === 3;
}

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

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const res: any = await getInboundListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          inboundNo: formValues.inboundNo,
          inboundType: formValues.inboundType,
          warehouseId: formValues.warehouseId,
          status: formValues.status,
        });
        return { items: res?.list ?? res?.items ?? [], total: res?.total ?? 0 };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.product.inbound.field.inboundNo'), field: 'inboundNo', width: 160, slots: { default: 'inboundNo' } },
    { title: $t('page.product.inbound.field.inboundType'), field: 'inboundType', width: 110, slots: { default: 'inboundType' } },
    { title: $t('page.product.inbound.field.warehouse'), field: 'warehouseName', minWidth: 120, slots: { default: 'warehouseName' } },
    { title: $t('page.product.inbound.field.status'), field: 'status', width: 100, slots: { default: 'status' } },
    { title: $t('page.product.inbound.field.totalQuantity'), field: 'totalQuantity', width: 100 },
    { title: $t('page.product.inbound.field.totalAmount'), field: 'totalAmount', width: 110 },
    { title: $t('page.product.inbound.field.createdBy'), field: 'createdByName', width: 100 },
    { title: '提交人', field: 'submittedByName', width: 100 },
    { title: $t('page.product.inbound.field.createTime'), field: 'createTime', width: 160 },
    { title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 220 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ===== 编辑/新建抽屉 =====
const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: InboundDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteInboundApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleCreate() {
  openDrawer(true);
}

// ===== 仓库详情抽屉 =====
const warehouseDetailVisible = ref(false);
const warehouseDetailId = ref<number | null>(null);

function openWarehouseDetail(row: any) {
  if (!row.warehouseId) return;
  warehouseDetailId.value = row.warehouseId;
  warehouseDetailVisible.value = true;
}

// ===== 入库单详情抽屉 =====
const inboundDetailVisible = ref(false);
const inboundDetailId = ref<number | null>(null);

function openInboundDetail(row: any) {
  inboundDetailId.value = row.id;
  inboundDetailVisible.value = true;
}

// ===== PDF 打印 =====
async function handlePrintPdf(row: any) {
  try {
    const resp: any = await requestClient.get(`/api/system/inbound/print/${row.id}`);
    const data = resp?.data ?? resp;
    const main = data?.main ?? {};
    const items = data?.items ?? [];
    const warehouse = data?.warehouse;
    const creator = data?.creator;
    const submitter = data?.submitter;
    const auditor = data?.auditor;

    const typeText = getInboundTypeTag(main.inboundType || '').label;
    const statusText = getInboundStatusTag(main.status ?? 0).label;
    const wName = warehouse?.name || main.warehouseName || '-';
    const cName = creator?.nick_name || creator?.user_name || '-';
    const sName = submitter?.nick_name || submitter?.user_name || '-';
    const aName = auditor?.nick_name || auditor?.user_name || '-';

    const itemsHtml = items.map((it: any, i: number) => `
      <tr>
        <td>${i + 1}</td>
        <td>${it.productCode || '-'}</td>
        <td>${it.productName || '-'}</td>
        <td>${it.spec || '-'}</td>
        <td>${it.unit || '-'}</td>
        <td style="text-align:right">${it.quantity ?? '-'}</td>
        <td style="text-align:right">${it.unitPrice ?? '-'}</td>
        <td style="text-align:right">${it.totalPrice ?? '-'}</td>
        <td>${it.remark || '-'}</td>
      </tr>`).join('');

    const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>入库单 - ${main.inboundNo || ''}</title>
      <style>
        body { font-family: 'Microsoft YaHei', sans-serif; padding: 30px; color: #333; }
        h1 { text-align: center; margin-bottom: 20px; }
        .info-table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
        .info-table td { padding: 6px 10px; border: 1px solid #ddd; }
        .info-label { background: #f5f5f5; font-weight: bold; width: 100px; }
        table.items { width: 100%; border-collapse: collapse; margin-top: 10px; }
        table.items th, table.items td { border: 1px solid #ddd; padding: 8px 10px; font-size: 13px; }
        table.items th { background: #f0f5ff; }
        .sign { margin-top: 40px; display: flex; justify-content: space-between; }
        @media print { .no-print { display: none; } }
      </style></head><body>
      <h1>入库单</h1>
      <table class="info-table">
        <tr><td class="info-label">入库单号</td><td>${main.inboundNo || '-'}</td>
            <td class="info-label">入库类型</td><td>${typeText}</td></tr>
        <tr><td class="info-label">仓库</td><td>${wName}</td>
            <td class="info-label">状态</td><td>${statusText}</td></tr>
        <tr><td class="info-label">总数量</td><td>${main.totalQuantity ?? '-'}</td>
            <td class="info-label">总金额</td><td>${main.totalAmount ?? '-'}</td></tr>
        <tr><td class="info-label">制单人</td><td>${cName}</td>
            <td class="info-label">创建时间</td><td>${main.createTime || '-'}</td></tr>
        <tr><td class="info-label">提交人</td><td>${sName}</td>
            <td class="info-label">审核人</td><td>${aName}</td></tr>
        <tr><td class="info-label">审核时间</td><td>${main.auditTime || '-'}</td>
            <td class="info-label">备注</td><td>${main.remark || '-'}</td></tr>
      </table>
      <table class="items">
        <thead><tr>
          <th>序号</th><th>产品编码</th><th>产品名称</th><th>规格</th><th>单位</th>
          <th>数量</th><th>单价</th><th>金额</th><th>备注</th>
        </tr></thead>
        <tbody>${itemsHtml || '<tr><td colspan="9" style="text-align:center">无明细</td></tr>'}</tbody>
      </table>
      <div class="sign">
        <div>制单人：${cName}</div>
        <div>提交人：${sName}</div>
        <div>审核人：${aName}</div>
        <div>打印时间：${new Date().toLocaleString()}</div>
      </div>
      <div class="no-print" style="text-align:center;margin-top:20px">
        <button onclick="window.print()" style="padding:8px 30px;font-size:14px;cursor:pointer">打印 / 保存为PDF</button>
      </div>
      <script>window.onload = function() { setTimeout(function() { window.print(); }, 300); }<\/script>
    </body></html>`;

    const win = window.open('', '_blank');
    if (!win) {
      window.$message.error($t('page.product.inbound.message.popupBlocked'));
      return;
    }
    win.document.write(html);
    win.document.close();
  } catch {
    window.$message.error($t('page.product.inbound.message.printFailed'));
  }
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="inbound" />
    <Grid :table-title="$t('page.product.inbound.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:inbound:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.inbound.button.create') }}
        </Button>
      </template>

      <template #inboundNo="{ row }">
        <Button type="link" class="!px-0" @click="openInboundDetail(row)">
          {{ row.inboundNo }}
        </Button>
      </template>

      <template #warehouseName="{ row }">
        <Button type="link" class="!px-0" @click="openWarehouseDetail(row)">
          {{ row.warehouseName }}
        </Button>
      </template>

      <template #inboundType="{ row }">
        <Tag :color="getInboundTypeTag(row.inboundType).color">
          {{ getInboundTypeTag(row.inboundType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="getInboundStatusTag(row.status).color">
          {{ getInboundStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <!-- 草稿单：打开详情页，查看内容确认后再提交审批（不在列表直接提交） -->
        <Button
          v-if="inboundAuditEnabled && row.status === 0 && accessStore.hasAccessCode('product:inbound:update')"
          type="link"
          @click="() => openInboundDetail(row)"
        >
          {{ $t('page.product.inbound.action.submitAudit') }}
        </Button>
        <!-- 审核中：打开详情处理（提交人可撤回/抄送，审批人可审核/加签/转办/委派/退回） -->
        <Button
          v-if="
            inboundAuditEnabled &&
            row.status === 1 &&
            (accessStore.hasAccessCode('product:inbound:update') ||
              accessStore.hasAccessCode('product:inbound:audit'))
          "
          type="link"
          @click="() => openInboundDetail(row)"
        >
          {{ $t('page.product.inbound.action.auditing') }}
        </Button>

        <!-- 更多操作下拉菜单 -->
        <Dropdown placement="bottomRight">
          <Button type="link" @click.prevent>
            {{ $t('page.product.inbound.action.more') }}
          </Button>
          <template #overlay>
            <Menu>
              <Menu.Item
                v-if="accessStore.hasAccessCode('product:inbound:update') && !isLockedByApproval(row)"
                key="edit"
                @click="() => handleEdit(row)"
              >
                <template #icon><LucideFilePenLine class="inline" :size="14" /></template>
                {{ $t('page.product.inbound.action.edit') }}
              </Menu.Item>
              <Menu.Item key="print" @click="() => handlePrintPdf(row)">
                {{ $t('page.product.inbound.action.downloadPdf') }}
              </Menu.Item>
              <Menu.Divider
                v-if="accessStore.hasAccessCode('product:inbound:delete') && !isLockedByApproval(row)"
              />
              <Popconfirm
                v-if="accessStore.hasAccessCode('product:inbound:delete') && !isLockedByApproval(row)"
                :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.product.inbound.title') })"
                :ok-text="$t('ui.button.ok')"
                :cancel-text="$t('ui.button.cancel')"
                @confirm="() => handleDelete(row)"
              >
                <Menu.Item key="delete" danger>
                  <template #icon><LucideTrash2 class="inline" :size="14" /></template>
                  {{ $t('page.product.inbound.action.delete') }}
                </Menu.Item>
              </Popconfirm>
            </Menu>
          </template>
        </Dropdown>
      </template>
    </Grid>

    <!-- 编辑/新建抽屉 -->
    <Drawer />

    <!-- 仓库详情抽屉 -->
    <WarehouseDetailDrawer v-model:visible="warehouseDetailVisible" :warehouse-id="warehouseDetailId" />

    <!-- 入库单详情抽屉 -->
    <InboundDetailDrawer v-model:visible="inboundDetailVisible" :inbound-id="inboundDetailId" @refresh="() => gridApi.query()" />
  </Page>
</template>
