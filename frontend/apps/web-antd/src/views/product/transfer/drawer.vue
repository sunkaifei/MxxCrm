<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Button,
  Input,
  InputNumber,
  message,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createTransferApi,
  getTransferInfoApi,
} from '#/api/core/product/transfer';
import { $t } from '#/locales';

import ProductSelectModal from '../../sale/components/ProductSelectModal.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'transfer-drawer',
  { 'transfer-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// ============ 仓库弹窗选择（源仓库 / 目标仓库共用） ============
const warehouseSelectVisible = ref(false);
// 标记当前选择的是源仓库还是目标仓库
const warehouseSelectTarget = ref<'from' | 'to'>('from');

// 选择仓库时排除另一个已选仓库（源仓库和目标仓库不能相同）
const warehouseExcludeId = computed(() => {
  if (warehouseSelectTarget.value === 'from') {
    // 选源仓库时，排除已选的目标仓库
    return toWarehouseId.value;
  }
  // 选目标仓库时，排除已选的源仓库
  return fromWarehouseId.value;
});

function openWarehouseSelect(target: 'from' | 'to') {
  warehouseSelectTarget.value = target;
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  const id = String(warehouse.id);
  const name = warehouse.warehouseName ?? warehouse.name ?? '';

  if (warehouseSelectTarget.value === 'from') {
    mainFormApi.setFieldValue('fromWarehouseId', id);
    mainFormApi.setFieldValue('fromWarehouseDisplay', name);
    fromWarehouseId.value = Number(warehouse.id);
    // 源仓库切换时清空已选产品（库存数据失效）
    if (tableItems.value.length > 0) {
      tableItems.value = [];
    }
  } else {
    mainFormApi.setFieldValue('toWarehouseId', id);
    mainFormApi.setFieldValue('toWarehouseDisplay', name);
    toWarehouseId.value = Number(warehouse.id);
  }
}

// ============ 产品明细 ============
interface TransferItem {
  productId: number;
  productName: string;
  productCode?: string;
  productSku?: string;
  spec?: string;
  unit?: string;
  /** 源仓库库存（ProductSelectModal 按 warehouseId 返回） */
  stock?: null | number;
  /** 调拨数量 */
  quantity?: null | number;
  remark?: string;
}

const tableItems = ref<TransferItem[]>([]);
const productSelectVisible = ref(false);
// 源仓库ID（传给 ProductSelectModal 查询源仓库库存）
const fromWarehouseId = ref<number | undefined>();
// 目标仓库ID（用于排除重复选择）
const toWarehouseId = ref<number | undefined>();

// 已添加产品的排除列表（computed 确保响应式）
const excludeProductIds = computed(() =>
  tableItems.value.filter((i) => !i.productSku).map((i) => Number(i.productId)),
);
const excludeSkuCodes = computed(() =>
  tableItems.value
    .map((i) => i.productSku)
    .filter((sku): sku is string => !!sku && sku !== ''),
);

// 调拨数量超过源仓库库存校验
function isOverStock(record: any): boolean {
  if (record.quantity === null || record.quantity === undefined) return false;
  if (record.stock === null || record.stock === undefined) return false;
  return Number(record.quantity) > Number(record.stock);
}

// 是否存在库存不足的项
const hasStockError = computed(() =>
  tableItems.value.some((i) => isOverStock(i)),
);

function openProductSelect() {
  mainFormApi.getValues().then((values) => {
    if (!values?.fromWarehouseId) {
      message.warning('请先选择源仓库');
      return;
    }
    fromWarehouseId.value = Number(values.fromWarehouseId);
    productSelectVisible.value = true;
  });
}

function onProductSelected(items: any[]) {
  const existingKeys = new Set(
    tableItems.value.map((i) => `${i.productId}-${i.productSku || ''}`),
  );
  let added = 0;
  for (const item of items) {
    const key = `${item.productId}-${item.skuCode || ''}`;
    if (!existingKeys.has(key)) {
      tableItems.value.push({
        productId: item.productId,
        productName: item.productName,
        productCode: item.productCode,
        productSku: item.skuCode || '',
        spec: item.spec || '',
        unit: item.unit,
        stock: item.stock ?? null,
        quantity: undefined,
        remark: '',
      });
      added++;
    }
  }
  productSelectVisible.value = false;
  if (added > 0) {
    message.success(`已添加 ${added} 个产品`);
  }
}

function removeItem(index: number) {
  tableItems.value.splice(index, 1);
}

const itemColumns = computed(() => [
  { title: '产品名称', dataIndex: 'productName', width: 160, ellipsis: true },
  { title: '规格', dataIndex: 'spec', width: 130, ellipsis: true },
  { title: '单位', dataIndex: 'unit', width: 70, align: 'center' as const },
  {
    title: '源仓库库存',
    dataIndex: 'stock',
    width: 100,
    align: 'right' as const,
  },
  { title: '调拨数量', dataIndex: 'quantity', width: 140 },
  { title: '备注', dataIndex: 'remark', width: 160 },
  {
    title: $t('ui.table.action'),
    dataIndex: 'action',
    width: 70,
    fixed: 'right' as const,
  },
]);

const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({
      default: () => $t('page.product.inventory.transfer.drawer.info'),
    }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'fromWarehouseDisplay',
    label: $t('page.product.inventory.transfer.drawer.sourceWarehouse'),
    rules: 'required',
    componentProps: {
      placeholder: '点击选择源仓库',
      readOnly: true,
      style: { cursor: 'pointer' },
      onClick: () => openWarehouseSelect('from'),
    },
  },
  {
    component: 'Input',
    fieldName: 'fromWarehouseId',
    label: '',
    formItemClass: 'hidden',
  },
  {
    component: 'Input',
    fieldName: 'toWarehouseDisplay',
    label: $t('page.product.inventory.transfer.drawer.targetWarehouse'),
    rules: 'required',
    componentProps: {
      placeholder: '点击选择目标仓库',
      readOnly: true,
      style: { cursor: 'pointer' },
      onClick: () => openWarehouseSelect('to'),
    },
  },
  {
    component: 'Input',
    fieldName: 'toWarehouseId',
    label: '',
    formItemClass: 'hidden',
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: $t('page.product.inventory.transfer.drawer.remark'),
    componentProps: {
      placeholder: $t(
        'page.product.inventory.transfer.drawer.remarkPlaceholder',
      ),
      allowClear: true,
      rows: 2,
    },
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
        message.warning('请添加调拨明细');
        return;
      }

      // 数量与库存校验
      for (const item of tableItems.value) {
        if (
          item.quantity === null ||
          item.quantity === undefined ||
          Number(item.quantity) <= 0
        ) {
          message.warning(`请填写产品「${item.productName}」的调拨数量`);
          return;
        }
        if (isOverStock(item)) {
          message.warning(`产品「${item.productName}」调拨数量超过源仓库库存`);
          return;
        }
      }

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();

      const data = {
        fromWarehouseId: Number(values.fromWarehouseId),
        toWarehouseId: Number(values.toWarehouseId),
        remark: values.remark,
        items: tableItems.value.map((i) => ({
          productId: i.productId,
          productName: i.productName,
          productSku: i.productSku,
          quantity: Number(i.quantity),
          remark: i.remark,
        })),
      };

      if (drawerData.value.create) {
        await createTransferApi(data);
        message.success($t('ui.notification.create_success'));
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
      drawerData.value = drawerApi.getData<{
        create: boolean;
        row?: any;
      }>() || { create: true };
      mainFormApi.resetForm();
      confirmLoading.value = false;
      tableItems.value = [];
      fromWarehouseId.value = undefined;
      toWarehouseId.value = undefined;
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getTransferInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) =>
      v === null || v === undefined ? undefined : Number(v);
    const main = data.main ?? data;
    const items = data.items ?? [];

    const fromId = num(main.from_warehouse_id ?? main.fromWarehouseId);
    const toId = num(main.to_warehouse_id ?? main.toWarehouseId);

    mainFormApi.setValues({
      fromWarehouseId: fromId ? String(fromId) : undefined,
      fromWarehouseDisplay:
        main.from_warehouse_name ?? main.fromWarehouseName ?? '',
      toWarehouseId: toId ? String(toId) : undefined,
      toWarehouseDisplay: main.to_warehouse_name ?? main.toWarehouseName ?? '',
      remark: main.remark,
    });
    fromWarehouseId.value = fromId;
    toWarehouseId.value = toId;

    tableItems.value = items.map((item: any) => ({
      productId: Number(item.product_id ?? item.productId ?? 0),
      productName: item.product_name ?? item.productName ?? '',
      productSku: item.product_sku ?? item.productSku ?? '',
      spec: '',
      unit: '',
      stock: null,
      quantity:
        item.quantity !== null && item.quantity !== undefined
          ? Number(item.quantity)
          : undefined,
      remark: item.remark ?? '',
    }));
  } catch (error) {
    console.error('[调拨] 加载详情失败:', error);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="
      drawerData.create
        ? $t('page.product.inventory.transfer.drawer.title.create')
        : $t('page.product.inventory.transfer.drawer.title.edit')
    "
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip
        :title="
          isFullscreen
            ? $t('page.product.inventory.transfer.drawer.restore')
            : $t('page.product.inventory.transfer.drawer.fullscreen')
        "
      >
        <button
          type="button"
          class="transfer-drawer__fs-btn"
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

    <div class="transfer-drawer__body">
      <!-- 调拨信息表单 -->
      <MainForm />

      <!-- 调拨明细分隔线 + 统计 -->
      <div class="transfer-items-header">
        <span class="transfer-items-title">{{
          $t('page.product.inventory.transfer.drawer.detail')
        }}</span>
        <div v-if="tableItems.length > 0" class="transfer-items-stats">
          <Tag>共 {{ tableItems.length }}</Tag>
          <Tag v-if="hasStockError" color="error">库存不足</Tag>
        </div>
      </div>

      <!-- 工具栏 -->
      <div class="transfer-items-toolbar">
        <Button type="primary" size="small" @click="openProductSelect">
          添加产品
        </Button>
      </div>

      <!-- 调拨明细表格 -->
      <Table
        :columns="itemColumns"
        :data-source="tableItems"
        :row-key="(record) => `${record.productId}-${record.productSku || ''}`"
        :pagination="false"
        size="small"
        :scroll="{ x: 900, y: 360 }"
        class="transfer-items-table"
      >
        <template #bodyCell="{ column, record, index }">
          <!-- 源仓库库存（只读） -->
          <template v-if="column.dataIndex === 'stock'">
            <span
              v-if="record.stock === null || record.stock === undefined"
              class="text-gray-400"
              >-</span
            >
            <span v-else class="font-medium">{{ record.stock }}</span>
          </template>

          <!-- 调拨数量（可编辑，超库存标红） -->
          <template v-else-if="column.dataIndex === 'quantity'">
            <InputNumber
              :value="record.quantity"
              size="small"
              style="width: 100%"
              :min="0"
              :precision="2"
              :step="1"
              placeholder="输入"
              :status="isOverStock(record) ? 'error' : ''"
              @update:value="(val) => (record.quantity = val)"
            />
            <div v-if="isOverStock(record)" class="transfer-stock-warn">
              超出库存 {{ Number(record.quantity) - Number(record.stock) }}
            </div>
          </template>

          <!-- 备注（可编辑） -->
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
            <Button type="link" danger size="small" @click="removeItem(index)">
              {{ $t('ui.button.delete') }}
            </Button>
          </template>
        </template>

        <template #emptyText>
          <div class="transfer-items-empty">请添加调拨明细</div>
        </template>
      </Table>
    </div>

    <!-- 产品选择弹窗 -->
    <ProductSelectModal
      :visible="productSelectVisible"
      :exclude-ids="excludeProductIds"
      :exclude-sku-codes="excludeSkuCodes"
      :warehouse-id="fromWarehouseId"
      @update:visible="(val) => (productSelectVisible = val)"
      @select="onProductSelected"
    />

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      :exclude-id="warehouseExcludeId"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
    />
  </Drawer>
</template>

<style>
.transfer-drawer {
  width: 75vw !important;
}

.transfer-drawer--fullscreen {
  width: 100vw !important;
}

.transfer-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  color: rgb(0 0 0 / 45%);
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 4px;
  transition: all 0.2s;
}

.transfer-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}

.transfer-drawer__body {
  height: calc(100vh - 150px);
  padding: 0 8px;
  overflow-y: auto;
}

.transfer-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.transfer-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.transfer-items-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 12px;
  margin: 16px 0 8px;
  border-top: 1px solid #f0f0f0;
}

.transfer-items-title {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.transfer-items-stats {
  display: flex;
  gap: 4px;
}

.transfer-items-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.transfer-items-table {
  margin-bottom: 8px;
}

.transfer-items-empty {
  padding: 24px 0;
  font-size: 13px;
  color: #999;
  text-align: center;
}

.transfer-stock-warn {
  margin-top: 2px;
  font-size: 11px;
  line-height: 1.2;
  color: #ff4d4f;
}
</style>
