<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import type { VbenFormSchema } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { $t } from '#/locales';
import {
  getOutboundInfoApi,
  createOutboundApi,
  updateOutboundApi,
} from '#/api/core/product/outbound';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { getInventoryListApi } from '#/api/core/product/inventory';
import {
  Button,
  Input,
  InputNumber,
  Table,
  Tooltip,
  message,
} from 'ant-design-vue';
import ProductSelectModal from '../../sale/components/ProductSelectModal.vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'outbound-drawer',
  { 'outbound-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const outboundTypeOptions = [
  { label: $t('page.product.outbound.type.sale'), value: 'sale' },
  { label: $t('page.product.outbound.type.material'), value: 'material' },
  { label: $t('page.product.outbound.type.shortage'), value: 'shortage' },
  { label: $t('page.product.outbound.type.scrap'), value: 'scrap' },
  { label: $t('page.product.outbound.type.freeze'), value: 'freeze' },
  { label: $t('page.product.outbound.type.other'), value: 'other' },
];

const warehouseOptions = ref<{ label: string; value: number }[]>([]);

async function loadWarehouseOptions() {
  try {
    const resp = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const list = resp?.data ?? resp ?? [];
    warehouseOptions.value = (Array.isArray(list) ? list : []).map((w: any) => ({
      label: w.warehouseName ?? w.name ?? w.label,
      value: Number(w.id ?? w.value),
    }));
  } catch (e) {
    console.error('[出库] 加载仓库选项失败:', e);
  }
}

// ============ 产品明细 ============
interface OutboundItem {
  productId: number;
  productName: string;
  productCode?: string;
  productSku: string;
  spec?: string;
  unit?: string;
  /** 当前仓库可用库存 */
  stock: number;
  /** 出库数量 */
  quantity: number;
  batchNo: string;
  remark: string;
}

const tableItems = ref<OutboundItem[]>([]);
const productSelectVisible = ref(false);
const selectedWarehouseId = ref<number | undefined>();

// 已添加产品的排除列表（防止重复添加）
const excludeProductIds = computed(() =>
  tableItems.value
    .filter((i) => !i.productSku || i.productSku === '')
    .map((i) => Number(i.productId)),
);
const excludeSkuCodes = computed(() =>
  tableItems.value
    .filter((i) => i.productSku && i.productSku !== '')
    .map((i) => i.productSku),
);

// 拉取当前仓库库存，回填到表格行（用于库存校验）
async function fetchStock() {
  const wid = selectedWarehouseId.value;
  if (!wid) return;
  try {
    const resp: any = await getInventoryListApi({
      page: 1,
      pageSize: 999,
      warehouseId: wid,
    });
    const raw = resp?.data ?? resp;
    const list =
      raw?.items ?? raw?.list ?? (Array.isArray(raw) ? raw : []);
    const stockMap = new Map<number, number>();
    for (const it of list) {
      const pid = Number(it.productId ?? it.product_id);
      if (pid) stockMap.set(pid, Number(it.quantity ?? 0));
    }
    tableItems.value.forEach((item) => {
      const s = stockMap.get(item.productId);
      item.stock = s !== undefined ? s : 0;
    });
  } catch (e) {
    console.error('[出库] 获取库存失败:', e);
  }
}

const itemColumns = computed(() => [
  {
    title: $t('page.product.outbound.drawer.item.productName'),
    dataIndex: 'productName',
    width: 160,
    ellipsis: true,
  },
  {
    title: $t('page.product.outbound.drawer.item.spec'),
    dataIndex: 'spec',
    width: 120,
    ellipsis: true,
  },
  {
    title: $t('page.product.outbound.drawer.item.unit'),
    dataIndex: 'unit',
    width: 70,
    align: 'center' as const,
  },
  {
    title: $t('page.product.outbound.drawer.item.stock'),
    dataIndex: 'stock',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.product.outbound.drawer.item.quantity'),
    dataIndex: 'quantity',
    width: 140,
  },
  {
    title: $t('page.product.outbound.drawer.item.batchNo'),
    dataIndex: 'batchNo',
    width: 140,
  },
  {
    title: $t('page.product.outbound.drawer.item.remark'),
    dataIndex: 'remark',
    width: 150,
  },
  {
    title: $t('ui.table.action'),
    dataIndex: 'action',
    width: 70,
    fixed: 'right' as const,
  },
]);

function isShortage(record: any) {
  return Number(record.quantity) > Number(record.stock);
}

function rowClassName(record: any) {
  return isShortage(record) ? 'outbound-row-warn' : '';
}

async function openProductSelect() {
  const values = await mainFormApi.getValues();
  const wid = values?.warehouseId;
  if (!wid) {
    message.warning($t('page.product.outbound.drawer.selectWarehouseFirst'));
    return;
  }
  selectedWarehouseId.value = Number(wid);
  productSelectVisible.value = true;
}

function onProductSelected(items: any[]) {
  const existingKeys = new Set(
    tableItems.value.map((i) => `${i.productId}-${i.productSku || ''}`),
  );
  let added = 0;
  for (const item of items) {
    const key = `${item.productId}-${item.skuCode || ''}`;
    if (existingKeys.has(key)) continue;
    tableItems.value.push({
      productId: item.productId,
      productName: item.productName,
      productCode: item.productCode,
      productSku: item.skuCode || '',
      spec: item.spec || '',
      unit: item.unit || '',
      stock: item.stock ?? 0,
      quantity: 1,
      batchNo: '',
      remark: '',
    });
    existingKeys.add(key);
    added++;
  }
  productSelectVisible.value = false;
  if (added > 0) {
    message.success(`已添加 ${added} 个产品`);
  }
}

function removeItem(index: number) {
  tableItems.value.splice(index, 1);
}

// 仓库变更时同步仓库ID并刷新已有明细的库存
function onWarehouseChange(val: any) {
  selectedWarehouseId.value = val ? Number(val) : undefined;
  if (tableItems.value.length > 0) {
    fetchStock();
  }
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => $t('page.product.outbound.drawer.basicInfo') }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'outboundType',
    label: $t('page.product.outbound.drawer.outboundType'),
    rules: 'required',
    defaultValue: 'sale',
    componentProps: {
      placeholder: $t('page.product.outbound.drawer.outboundTypePlaceholder'),
      options: outboundTypeOptions,
      allowClear: true,
    },
  },
  {
    component: 'Select',
    fieldName: 'warehouseId',
    label: $t('page.product.outbound.drawer.warehouse'),
    rules: 'required',
    componentProps: {
      placeholder: $t('page.product.outbound.drawer.warehousePlaceholder'),
      options: warehouseOptions,
      allowClear: true,
      showSearch: true,
      filterOption: (input: string, option: any) =>
        (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      onChange: (val: any) => onWarehouseChange(val),
    },
  },
  {
    component: 'Input',
    fieldName: 'sourceOrderNo',
    label: $t('page.product.outbound.drawer.sourceOrderNo'),
    componentProps: { placeholder: $t('page.product.outbound.drawer.sourceOrderNoPlaceholder'), allowClear: true },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: $t('page.product.outbound.drawer.remark'),
    componentProps: { placeholder: $t('page.product.outbound.drawer.remarkPlaceholder'), allowClear: true, rows: 2 },
    formItemClass: 'col-span-2',
  },
];

const [MainForm, mainFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: formSchema,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    try {
      const valid = await mainFormApi.validate();
      if (!valid.valid) return;

      if (tableItems.value.length === 0) {
        message.warning($t('page.product.outbound.drawer.noItems'));
        return;
      }

      // 库存校验：出库数量不能超过可用库存
      for (const item of tableItems.value) {
        if (Number(item.quantity) > Number(item.stock)) {
          message.error(
            $t('page.product.outbound.drawer.stockShortage', {
              name: item.productName || item.productCode,
              stock: item.stock,
              quantity: item.quantity,
            }),
          );
          return;
        }
      }

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();
      const { _div1, ...rest } = values as any;

      const data = {
        ...rest,
        warehouseId: rest.warehouseId ? Number(rest.warehouseId) : undefined,
        items: tableItems.value.map((item) => ({
          productId: item.productId,
          productSku: item.productSku || undefined,
          quantity: Number(item.quantity),
          batchNo: item.batchNo || undefined,
          remark: item.remark || undefined,
        })),
      };

      if (drawerData.value.create) {
        await createOutboundApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateOutboundApi({ ...data, id: drawerData.value.row.id });
        message.success($t('ui.notification.update_success'));
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      confirmLoading.value = false;
    }
  },
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value =
        drawerApi.getData<{ create: boolean; row?: any }>() || {
          create: true,
        };
      mainFormApi.resetForm();
      confirmLoading.value = false;
      tableItems.value = [];
      selectedWarehouseId.value = undefined;
      loadWarehouseOptions();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getOutboundInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const main = data.main ?? data;
    const items = data.items ?? [];
    const num = (v: any) =>
      v === null || v === undefined ? undefined : Number(v);

    const wid = main.warehouseId ? num(main.warehouseId) : undefined;
    mainFormApi.setValues({
      outboundType: main.outboundType ?? 'sale',
      warehouseId: wid,
      sourceOrderNo: main.sourceOrderNo,
      remark: main.remark,
    });
    selectedWarehouseId.value = wid;

    tableItems.value = items.map((item: any) => ({
      productId: Number(item.productId ?? 0),
      productName: item.productName ?? '',
      productCode: item.productCode ?? '',
      productSku: item.productSku ?? '',
      spec: item.spec ?? '',
      unit: item.unit ?? '',
      stock: 0,
      quantity:
        item.quantity !== null && item.quantity !== undefined
          ? Number(item.quantity)
          : 0,
      batchNo: item.batchNo ?? '',
      remark: item.remark ?? '',
    }));

    // 回显后拉取当前仓库库存以支持库存校验
    if (wid) {
      fetchStock();
    }
  } catch (e) {
    console.error('[出库] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? $t('page.product.outbound.drawer.title.create') : $t('page.product.outbound.drawer.title.edit')"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? $t('page.product.outbound.drawer.restore') : $t('page.product.outbound.drawer.fullscreen')">
        <button
          type="button"
          class="outbound-drawer__fs-btn"
          @click="toggleFullscreen"
        >
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="outbound-drawer__body">
      <MainForm />

      <!-- 产品明细标题 + 工具栏 -->
      <div class="outbound-items-header">
        <span class="outbound-items-title">{{ $t('page.product.outbound.drawer.detail') }}</span>
        <Button type="primary" size="small" @click="openProductSelect">
          {{ $t('page.product.outbound.drawer.addItem') }}
        </Button>
      </div>

      <!-- 产品明细表格 -->
      <Table
        :columns="itemColumns"
        :data-source="tableItems"
        :row-key="(record: any) => `${record.productId}-${record.productSku || ''}`"
        :row-class-name="rowClassName"
        :pagination="false"
        size="small"
        :scroll="{ x: 1000, y: 380 }"
        class="outbound-items-table"
      >
        <template #bodyCell="{ column, record, index }">
          <!-- 可用库存：只读，库存不足时标红 -->
          <template v-if="column.dataIndex === 'stock'">
            <span :class="{ 'outbound-stock-warn': isShortage(record) }">
              {{ record.stock }}
            </span>
          </template>

          <!-- 出库数量：可编辑，超出库存标红告警 -->
          <template v-else-if="column.dataIndex === 'quantity'">
            <div class="outbound-qty-cell">
              <InputNumber
                :value="record.quantity"
                size="small"
                style="width: 100%"
                :min="0"
                :precision="2"
                :step="1"
                :status="isShortage(record) ? 'error' : ''"
                @update:value="(val) => (record.quantity = val ?? 0)"
              />
              <div v-if="isShortage(record)" class="outbound-qty-warn">
                库存不足
              </div>
            </div>
          </template>

          <!-- 批次号 -->
          <template v-else-if="column.dataIndex === 'batchNo'">
            <Input
              v-model:value="record.batchNo"
              size="small"
              placeholder="批次号"
              allow-clear
            />
          </template>

          <!-- 备注 -->
          <template v-else-if="column.dataIndex === 'remark'">
            <Input
              v-model:value="record.remark"
              size="small"
              placeholder="备注"
              allow-clear
            />
          </template>

          <!-- 操作：删除 -->
          <template v-else-if="column.dataIndex === 'action'">
            <Button
              type="link"
              danger
              size="small"
              @click="removeItem(index)"
            >
              {{ $t('ui.button.delete') }}
            </Button>
          </template>
        </template>

        <template #emptyText>
          <div class="outbound-items-empty">
            {{ $t('page.product.outbound.drawer.emptyHint') }}
          </div>
        </template>
      </Table>
    </div>

    <!-- 产品选择弹窗 -->
    <ProductSelectModal
      :visible="productSelectVisible"
      :exclude-ids="excludeProductIds"
      :exclude-sku-codes="excludeSkuCodes"
      :warehouse-id="selectedWarehouseId"
      @update:visible="(val) => (productSelectVisible = val)"
      @select="onProductSelected"
    />
  </Drawer>
</template>

<style>
.outbound-drawer {
  width: 75vw !important;
}

.outbound-drawer--fullscreen {
  width: 100vw !important;
}

.outbound-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}

.outbound-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.outbound-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.outbound-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.outbound-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.outbound-items-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 16px 0 8px;
  padding-top: 12px;
  border-top: 1px solid #f0f0f0;
}

.outbound-items-title {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.outbound-items-table {
  margin-bottom: 8px;
}

.outbound-items-empty {
  padding: 24px 0;
  color: #999;
  font-size: 13px;
  text-align: center;
}

/* 库存不足的行整体高亮 */
.outbound-items-table .outbound-row-warn > td {
  background-color: #fff1f0 !important;
}

/* 可用库存数值标红 */
.outbound-stock-warn {
  color: #ff4d4f;
  font-weight: 600;
}

.outbound-qty-cell {
  width: 100%;
}

.outbound-qty-warn {
  margin-top: 2px;
  font-size: 11px;
  color: #ff4d4f;
  line-height: 1.2;
}
</style>
