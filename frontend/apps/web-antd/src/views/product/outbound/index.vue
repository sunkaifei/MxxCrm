<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Button, Dropdown, Menu, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteOutboundApi,
  getOutboundListApi,
} from '#/api/core/product/outbound';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { getSettingConfigApi } from '#/api/core/system/setting';
import { requestClient } from '#/api/request';
import { $t } from '#/locales';

import InventoryProcessGuide from '../components/InventoryProcessGuide.vue';
import WarehouseDetailDrawer from '../components/WarehouseDetailDrawer.vue';
import OutboundDetailDrawer from './detail-drawer.vue';
import OutboundDrawer from './drawer.vue';

const accessStore = useAccessStore();

// 出库审核开关
const outboundAuditEnabled = ref(true);

const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map(
      (w: any) => ({
        label: w.warehouseName ?? w.name ?? w.label,
        value: Number(w.id ?? w.value),
      }),
    );
  } catch (error) {
    console.error('[出库] 加载仓库选项失败:', error);
  }
}

onMounted(() => {
  loadWarehouseOptions();
  loadAuditEnabled();
});

async function loadAuditEnabled() {
  try {
    const data = await getSettingConfigApi();
    outboundAuditEnabled.value = data?.outboundAuditEnabled ?? true;
  } catch {
    // 忽略加载错误，默认开启审核
  }
}

const outboundTypeOptions = [
  { label: $t('page.product.outbound.type.sale'), value: 'sale' },
  { label: $t('page.product.outbound.type.material'), value: 'material' },
  { label: $t('page.product.outbound.type.shortage'), value: 'shortage' },
  { label: $t('page.product.outbound.type.scrap'), value: 'scrap' },
  { label: $t('page.product.outbound.type.freeze'), value: 'freeze' },
  { label: $t('page.product.outbound.type.other'), value: 'other' },
];

const statusOptions = [
  { label: $t('page.product.outbound.status.0'), value: 0 },
  { label: $t('page.product.outbound.status.1'), value: 1 },
  { label: $t('page.product.outbound.status.2'), value: 2 },
  { label: $t('page.product.outbound.status.3'), value: 3 },
  { label: $t('page.product.outbound.status.4'), value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'outboundNo',
      label: $t('page.product.outbound.field.outboundNo'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.outboundNo'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'outboundType',
      label: $t('page.product.outbound.field.outboundType'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.outboundType'),
        options: outboundTypeOptions,
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'warehouseId',
      label: $t('page.product.outbound.field.warehouse'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.warehouse'),
        options: warehouseOptions,
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.product.outbound.field.status'),
      componentProps: {
        placeholder: $t('page.product.outbound.placeholder.status'),
        options: statusOptions,
        allowClear: true,
      },
    },
  ],
};

function getOutboundTypeTag(type: string) {
  const map: Record<string, { color: string; label: string }> = {
    sale: { label: $t('page.product.outbound.type.sale'), color: 'blue' },
    material: {
      label: $t('page.product.outbound.type.material'),
      color: 'cyan',
    },
    shortage: {
      label: $t('page.product.outbound.type.shortage'),
      color: 'orange',
    },
    scrap: { label: $t('page.product.outbound.type.scrap'), color: 'red' },
    freeze: { label: $t('page.product.outbound.type.freeze'), color: 'purple' },
    other: { label: $t('page.product.outbound.type.other'), color: 'default' },
  };
  return (
    map[type] || {
      label: $t('page.product.outbound.type.unknown'),
      color: 'default',
    }
  );
}

function getStatusTag(status: number) {
  const map: Record<number, { color: string; label: string }> = {
    0: { label: $t('page.product.outbound.status.0'), color: 'default' },
    1: { label: $t('page.product.outbound.status.1'), color: 'processing' },
    2: { label: $t('page.product.outbound.status.2'), color: 'warning' },
    3: { label: $t('page.product.outbound.status.3'), color: 'success' },
    4: { label: $t('page.product.outbound.status.4'), color: 'error' },
  };
  return (
    map[status] || {
      label: $t('page.product.outbound.status.unknown'),
      color: 'default',
    }
  );
}

// 审核通过时不可编辑/删除
function isLockedByApproval(row: any): boolean {
  return outboundAuditEnabled.value && row.status === 3;
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
        const res: any = await getOutboundListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          outboundNo: formValues.outboundNo,
          outboundType: formValues.outboundType,
          warehouseId: formValues.warehouseId,
          status: formValues.status,
        });
        return { items: res?.list ?? res?.items ?? [], total: res?.total ?? 0 };
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 60,
    },
    {
      title: $t('page.product.outbound.field.outboundNo'),
      field: 'outboundNo',
      minWidth: 140,
      slots: { default: 'outboundNo' },
    },
    {
      title: $t('page.product.outbound.field.outboundType'),
      field: 'outboundType',
      width: 110,
      slots: { default: 'outboundType' },
    },
    {
      title: $t('page.product.outbound.field.warehouse'),
      field: 'warehouseName',
      minWidth: 120,
      slots: { default: 'warehouseName' },
    },
    {
      title: $t('page.product.outbound.field.status'),
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    {
      title: $t('page.product.outbound.field.totalQuantity'),
      field: 'totalQuantity',
      width: 100,
    },
    {
      title: $t('page.product.outbound.field.totalAmount'),
      field: 'totalAmount',
      width: 110,
    },
    {
      title: $t('page.product.outbound.field.createdBy'),
      field: 'createdByName',
      width: 100,
    },
    {
      title: '提交人',
      field: 'submittedByName',
      width: 100,
    },
    {
      title: $t('page.product.outbound.field.createTime'),
      field: 'createTime',
      width: 160,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: OutboundDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteOutboundApi([row.id]);
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
const warehouseDetailId = ref<null | number>(null);

function openWarehouseDetail(row: any) {
  if (!row.warehouseId) return;
  warehouseDetailId.value = row.warehouseId;
  warehouseDetailVisible.value = true;
}

// ===== 出库单详情抽屉 =====
const outboundDetailVisible = ref(false);
const outboundDetailId = ref<null | number>(null);

function openOutboundDetail(row: any) {
  outboundDetailId.value = row.id;
  outboundDetailVisible.value = true;
}

// ===== PDF 打印 =====
async function handlePrintPdf(row: any) {
  try {
    const resp: any = await requestClient.get(
      `/api/system/outbound/print/${row.id}`,
    );
    const data = resp?.data ?? resp;
    const main = data?.main ?? {};
    const items = data?.items ?? [];
    const warehouse = data?.warehouse;
    const creator = data?.creator;
    const submitter = data?.submitter;
    const auditor = data?.auditor;

    const typeText = getOutboundTypeTag(main.outboundType || '').label;
    const statusText = getStatusTag(main.status ?? 0).label;
    const wName = warehouse?.name || main.warehouseName || '-';
    const cName = creator?.nick_name || creator?.user_name || '-';
    const sName = submitter?.nick_name || submitter?.user_name || '-';
    const aName = auditor?.nick_name || auditor?.user_name || '-';

    const itemsHtml = items
      .map(
        (it: any, i: number) => `
      <tr>
        <td>${i + 1}</td>
        <td>${it.productCode || '-'}</td>
        <td>${it.productName || '-'}</td>
        <td>${it.spec || '-'}</td>
        <td>${it.unit || '-'}</td>
        <td style="text-align:right">${it.quantity ?? '-'}</td>
        <td>${it.batchNo || '-'}</td>
        <td>${it.remark || '-'}</td>
      </tr>`,
      )
      .join('');

    // 结束 script 标签的斜杠用 \u002f 转义，避免字面闭合标签截断 .vue 的 SFC script 块
    const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>出库单 - ${main.outboundNo || ''}</title>
      <style>
        body { font-family: 'Microsoft YaHei', sans-serif; padding: 30px; color: #333; }
        h1 { text-align: center; margin-bottom: 20px; }
        .info-table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
        .info-table td { padding: 6px 10px; border: 1px solid #ddd; }
        .info-label { background: #f5f5f5; font-weight: bold; width: 100px; }
        table.items { width: 100%; border-collapse: collapse; margin-top: 10px; }
        table.items th, table.items td { border: 1px solid #ddd; padding: 8px 10px; font-size: 13px; }
        table.items th { background: #fff5f0; }
        .sign { margin-top: 40px; display: flex; justify-content: space-between; }
        @media print { .no-print { display: none; } }
      </style></head><body>
      <h1>出库单</h1>
      <table class="info-table">
        <tr><td class="info-label">出库单号</td><td>${main.outboundNo || '-'}</td>
            <td class="info-label">出库类型</td><td>${typeText}</td></tr>
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
          <th>数量</th><th>批次号</th><th>备注</th>
        </tr></thead>
        <tbody>${itemsHtml || '<tr><td colspan="8" style="text-align:center">无明细</td></tr>'}</tbody>
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
      <script>window.onload = function() { setTimeout(function() { window.print(); }, 300); }<\u002Fscript>
    </body></html>`;

    const win = window.open('', '_blank');
    if (!win) {
      window.$message.error($t('page.product.outbound.message.popupBlocked'));
      return;
    }
    win.document.write(html);
    win.document.close();
  } catch {
    window.$message.error($t('page.product.outbound.message.printFailed'));
  }
}
</script>

<template>
  <Page auto-content-height>
    <InventoryProcessGuide current-step="outbound" />
    <Grid :table-title="$t('page.product.outbound.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:outbound:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.outbound.button.create') }}
        </Button>
      </template>

      <template #outboundNo="{ row }">
        <Button type="link" class="!px-0" @click="openOutboundDetail(row)">
          {{ row.outboundNo }}
        </Button>
      </template>

      <template #warehouseName="{ row }">
        <Button type="link" class="!px-0" @click="openWarehouseDetail(row)">
          {{ row.warehouseName }}
        </Button>
      </template>

      <template #outboundType="{ row }">
        <Tag :color="getOutboundTypeTag(row.outboundType).color">
          {{ getOutboundTypeTag(row.outboundType).label }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="getStatusTag(row.status).color">
          {{ getStatusTag(row.status).label }}
        </Tag>
      </template>

      <template #action="{ row }">
        <!-- 草稿单：打开详情页，查看内容确认后再提交审批（不在列表直接提交） -->
        <Button
          v-if="
            outboundAuditEnabled &&
            row.status === 0 &&
            accessStore.hasAccessCode('product:outbound:update')
          "
          type="link"
          @click="() => openOutboundDetail(row)"
        >
          {{ $t('page.product.outbound.action.submitAudit') }}
        </Button>
        <!-- 审核中：打开详情处理（提交人可撤回/抄送，审批人可审核/加签/转办/委派/退回） -->
        <Button
          v-if="outboundAuditEnabled && row.status === 1"
          type="link"
          @click="() => openOutboundDetail(row)"
        >
          {{ $t('page.product.outbound.action.audit') }}
        </Button>

        <!-- 更多操作下拉菜单 -->
        <Dropdown placement="bottomRight">
          <Button type="link" @click.prevent>
            {{ $t('page.product.outbound.action.more') }}
          </Button>
          <template #overlay>
            <Menu>
              <Menu.Item
                v-if="
                  accessStore.hasAccessCode('product:outbound:update') &&
                  !isLockedByApproval(row)
                "
                key="edit"
                @click="() => handleEdit(row)"
              >
                <template #icon>
                  <LucideFilePenLine class="inline" :size="14" />
                </template>
                {{ $t('page.product.outbound.action.edit') }}
              </Menu.Item>
              <Menu.Item key="print" @click="() => handlePrintPdf(row)">
                {{ $t('page.product.outbound.action.downloadPdf') }}
              </Menu.Item>
              <Menu.Divider
                v-if="
                  accessStore.hasAccessCode('product:outbound:delete') &&
                  !isLockedByApproval(row)
                "
              />
              <Popconfirm
                v-if="
                  accessStore.hasAccessCode('product:outbound:delete') &&
                  !isLockedByApproval(row)
                "
                :title="
                  $t('ui.text.do_you_want_delete', {
                    moduleName: $t('page.product.outbound.title'),
                  })
                "
                :ok-text="$t('ui.button.ok')"
                :cancel-text="$t('ui.button.cancel')"
                @confirm="() => handleDelete(row)"
              >
                <Menu.Item key="delete" danger>
                  <template #icon>
                    <LucideTrash2 class="inline" :size="14" />
                  </template>
                  {{ $t('page.product.outbound.action.delete') }}
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
    <WarehouseDetailDrawer
      v-model:visible="warehouseDetailVisible"
      :warehouse-id="warehouseDetailId"
    />

    <!-- 出库单详情抽屉 -->
    <OutboundDetailDrawer
      v-model:visible="outboundDetailVisible"
      :outbound-id="outboundDetailId"
      @refresh="() => gridApi.query()"
    />
  </Page>
</template>
