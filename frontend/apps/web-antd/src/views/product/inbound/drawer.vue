<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Alert,
  Button,
  Input,
  InputNumber,
  message,
  Modal,
  Table,
  Textarea,
  Tooltip,
} from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createInboundApi,
  getInboundInfoApi,
  updateInboundApi,
} from '#/api/core/product/inbound';
import { getWarehouseListApi } from '#/api/core/product/warehouse';
import { $t } from '#/locales';

import ProductSelectModal from '../../sale/components/ProductSelectModal.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

// ============ 修改原因弹窗（编辑已完成单据时使用） ============
const changeReasonVisible = ref(false);
const changeReason = ref('');
const pendingSaveData = ref<any>(null);

const drawerClass = computed(() => [
  'inbound-drawer',
  { 'inbound-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// 入库类型选项
const inboundTypeOptions = [
  { label: $t('page.product.inbound.type.purchase'), value: 'purchase' },
  { label: $t('page.product.inbound.type.return'), value: 'return' },
  { label: $t('page.product.inbound.type.surplus'), value: 'surplus' },
  { label: $t('page.product.inbound.type.initial'), value: 'initial' },
  { label: $t('page.product.inbound.type.other'), value: 'other' },
];

// 仓库选项（异步加载）
const warehouseOptions = ref<{ label: string; value: number }[]>([]);
// 当前选中仓库ID（传给 ProductSelectModal 用于查询库存）
const selectedWarehouseId = ref<number | undefined>();
// 仓库名称显示
const selectedWarehouseName = ref('');
// 仓库选择弹窗
const warehouseSelectVisible = ref(false);

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
    console.error('[入库] 加载仓库列表失败:', error);
  }
}

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  const id = Number(warehouse.id);
  const name = warehouse.warehouseName ?? warehouse.name ?? '';
  selectedWarehouseId.value = id;
  selectedWarehouseName.value = name;
  // 同步到表单
  mainFormApi.setFieldValue('warehouseId', id);
  mainFormApi.setFieldValue('warehouseName', name);
}

function clearWarehouse() {
  selectedWarehouseId.value = undefined;
  selectedWarehouseName.value = '';
  mainFormApi.setFieldValue('warehouseId', undefined);
  mainFormApi.setFieldValue('warehouseName', '');
}

// ============ 产品明细 ============
const tableItems = ref<any[]>([]);
const productSelectVisible = ref(false);

// 已添加产品的排除列表（computed 确保响应式）
const excludeProductIds = computed(() =>
  tableItems.value
    .filter((i: any) => !i.skuId || i.skuId === 0)
    .map((i: any) => Number(i.productId)),
);
const excludeSkuKeys = computed(() =>
  tableItems.value
    .filter((i: any) => i.skuId && i.skuId > 0)
    .map((i: any) => `${i.productId}-${i.skuId}`),
);
const excludeSkuCodes = computed(() =>
  tableItems.value
    .filter((i: any) => i.productSku && i.productSku !== '')
    .map((i: any) => i.productSku),
);

const itemColumns = computed(() => [
  {
    title: '产品编码',
    dataIndex: 'productCode',
    width: 110,
    ellipsis: true,
  },
  {
    title: '产品名称',
    dataIndex: 'productName',
    width: 140,
    ellipsis: true,
  },
  {
    title: '规格',
    dataIndex: 'spec',
    width: 120,
    ellipsis: true,
  },
  {
    title: '单位',
    dataIndex: 'unit',
    width: 60,
    align: 'center' as const,
  },
  {
    title: '数量',
    dataIndex: 'quantity',
    width: 110,
  },
  {
    title: '单价',
    dataIndex: 'unitPrice',
    width: 110,
  },
  {
    title: '金额',
    dataIndex: 'amount',
    width: 100,
    align: 'right' as const,
  },
  {
    title: '批次号',
    dataIndex: 'batchNo',
    width: 120,
  },
  {
    title: '备注',
    dataIndex: 'remark',
    width: 140,
  },
  {
    title: $t('ui.table.action'),
    dataIndex: 'action',
    width: 60,
    fixed: 'right' as const,
  },
]);

function openProductSelect() {
  if (!selectedWarehouseId.value) {
    message.warning('请先选择入库仓库');
    return;
  }
  productSelectVisible.value = true;
}

function onProductSelected(selectedItems: any[]) {
  for (const item of selectedItems) {
    tableItems.value.push({
      productId: item.productId,
      productName: item.productName,
      productCode: item.productCode,
      skuId: item.skuId || 0,
      productSku: item.skuCode || '',
      spec: item.spec || '',
      unit: item.unit || '',
      quantity: 1,
      unitPrice: item.unitPrice || 0,
      batchNo: '',
      remark: '',
    });
  }
  productSelectVisible.value = false;
}

function removeItem(index: number) {
  tableItems.value.splice(index, 1);
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({
      default: () => $t('page.product.inbound.drawer.basicInfo'),
    }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'inboundType',
    label: $t('page.product.inbound.drawer.inboundType'),
    rules: 'required',
    defaultValue: 'purchase',
    componentProps: {
      placeholder: $t('page.product.inbound.drawer.inboundTypePlaceholder'),
      options: inboundTypeOptions,
      allowClear: true,
    },
  },
  {
    component: 'Input',
    fieldName: 'warehouseName',
    label: $t('page.product.inbound.drawer.warehouse'),
    rules: 'required',
    componentProps: {
      placeholder: $t('page.product.inbound.drawer.warehousePlaceholder'),
      readonly: true,
      allowClear: true,
      style: { cursor: 'pointer' },
      onClick: () => openWarehouseSelect(),
      onChange: (e: any) => {
        if (!e?.target?.value) clearWarehouse();
      },
    },
  },
  {
    component: 'Input',
    fieldName: 'sourceOrderNo',
    label: $t('page.product.inbound.drawer.sourceOrderNo'),
    componentProps: {
      placeholder: $t('page.product.inbound.drawer.sourceOrderNoPlaceholder'),
      allowClear: true,
    },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: $t('page.product.inbound.drawer.remark'),
    componentProps: {
      placeholder: $t('page.product.inbound.drawer.remarkPlaceholder'),
      allowClear: true,
      rows: 2,
    },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Divider',
    fieldName: '_div2',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({
      default: () => $t('page.product.inbound.drawer.detail'),
    }),
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
        message.warning('请至少添加一个产品');
        return;
      }

      const hasInvalidQty = tableItems.value.some(
        (item) => !item.quantity || Number(item.quantity) <= 0,
      );
      if (hasInvalidQty) {
        message.warning('产品数量必须大于 0');
        return;
      }

      const values = await mainFormApi.getValues();

      const { _div1, _div2, _warehouseName, ...rest } = values as any;

      const items = tableItems.value
        .filter(
          (item) => item.productId !== null && item.productId !== undefined,
        )
        .map((item) => ({
          productId: Number(item.productId),
          productSku: item.productSku || undefined,
          quantity: Number(item.quantity),
          unitPrice:
            item.unitPrice !== undefined && item.unitPrice !== ''
              ? Number(item.unitPrice)
              : undefined,
          amount: Number(item.quantity) * Number(item.unitPrice || 0),
          batchNo: item.batchNo || undefined,
          remark: item.remark || undefined,
        }));

      const data = {
        ...rest,
        warehouseId: selectedWarehouseId.value,
        totalAmount: Number(
          items.reduce((sum, item) => sum + (item.amount || 0), 0).toFixed(2),
        ),
        items,
      };

      if (drawerData.value.create) {
        // 新建：直接保存
        confirmLoading.value = true;
        await createInboundApi(data);
        message.success($t('ui.notification.create_success'));
        drawerApi.setData({ needRefresh: true });
        drawerApi.close();
      } else {
        // 编辑：检查是否为已完成单据
        const rowStatus = Number(drawerData.value.row?.status ?? 0);
        if (rowStatus === 3) {
          // 已完成单据：弹出修改原因弹窗
          pendingSaveData.value = data;
          changeReason.value = '';
          changeReasonVisible.value = true;
        } else {
          // 草稿/待审核：直接保存
          confirmLoading.value = true;
          await updateInboundApi({ ...data, id: drawerData.value.row.id });
          message.success($t('ui.notification.update_success'));
          drawerApi.setData({ needRefresh: true });
          drawerApi.close();
        }
      }
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
    const resp = await getInboundInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) =>
      v === null || v === undefined ? undefined : Number(v);
    const main = data.main ?? data;
    const items = data.items ?? [];

    mainFormApi.setValues({
      inboundType: main.inboundType ?? 'purchase',
      warehouseId: main.warehouseId ? num(main.warehouseId) : undefined,
      warehouseName: main.warehouseName ?? '',
      sourceOrderNo: main.sourceOrderNo,
      remark: main.remark,
    });
    selectedWarehouseId.value =
      (main.warehouseId ?? main.warehouse_id)
        ? Number(main.warehouseId ?? main.warehouse_id)
        : undefined;
    selectedWarehouseName.value = main.warehouseName ?? '';

    // 回填产品明细
    tableItems.value = items.map((item: any) => ({
      productId: Number(item.productId ?? item.product_id),
      productName: item.productName ?? '',
      productCode: item.productCode ?? '',
      skuId: 0,
      productSku: item.productSku ?? item.product_sku ?? '',
      spec: item.spec ?? '',
      unit: item.unit ?? '',
      quantity: num(item.quantity) ?? 0,
      unitPrice: num(item.unitPrice ?? item.unit_price) ?? 0,
      batchNo: item.batchNo ?? item.batch_no ?? '',
      remark: item.remark ?? '',
    }));
  } catch (error) {
    console.error('[入库] 加载详情失败:', error);
  }
}

// ============ 修改原因弹窗：提交 ============
async function submitChangeReason() {
  if (!changeReason.value.trim()) {
    message.warning('请填写修改原因');
    return;
  }

  try {
    confirmLoading.value = true;
    changeReasonVisible.value = false;

    const data = pendingSaveData.value;
    await updateInboundApi({
      ...data,
      id: drawerData.value.row.id,
      changeReason: changeReason.value.trim(),
    });
    message.success($t('ui.notification.update_success'));
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch (error) {
    console.error('[入库] 修改已完成单据失败:', error);
  } finally {
    confirmLoading.value = false;
    pendingSaveData.value = null;
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="
      drawerData.create
        ? $t('page.product.inbound.drawer.title.create')
        : $t('page.product.inbound.drawer.title.edit')
    "
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip
        :title="
          isFullscreen
            ? $t('page.product.inbound.drawer.restore')
            : $t('page.product.inbound.drawer.fullscreen')
        "
      >
        <button
          type="button"
          class="inbound-drawer__fs-btn"
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

    <div class="inbound-drawer__body">
      <MainForm />

      <!-- 产品明细工具栏 -->
      <div class="inbound-items-toolbar">
        <Button type="primary" size="small" @click="openProductSelect">
          添加产品
        </Button>
        <span v-if="tableItems.length > 0" class="inbound-items-count"
          >共 {{ tableItems.length }} 项</span
        >
      </div>

      <!-- 产品明细表格 -->
      <Table
        :columns="itemColumns"
        :data-source="tableItems"
        :row-key="(record: any) => `${record.productId}-${record.skuId || 0}`"
        :pagination="false"
        size="small"
        :scroll="{ x: 1100, y: 360 }"
        class="inbound-items-table"
      >
        <template #bodyCell="{ column, record, index }">
          <!-- 数量：可编辑 -->
          <template v-if="column.dataIndex === 'quantity'">
            <InputNumber
              v-model:value="record.quantity"
              size="small"
              style="width: 100%"
              :precision="0"
              :min="0"
              :step="1"
              placeholder="数量"
            />
          </template>

          <!-- 单价：可编辑 -->
          <template v-else-if="column.dataIndex === 'unitPrice'">
            <InputNumber
              v-model:value="record.unitPrice"
              size="small"
              style="width: 100%"
              :precision="2"
              :min="0"
              :step="0.01"
              placeholder="单价"
            />
          </template>

          <!-- 金额：自动计算 -->
          <template v-else-if="column.dataIndex === 'amount'">
            <span class="font-medium">{{
              (
                Number(record.quantity || 0) * Number(record.unitPrice || 0)
              ).toFixed(2)
            }}</span>
          </template>

          <!-- 批次号：可编辑 -->
          <template v-else-if="column.dataIndex === 'batchNo'">
            <Input
              v-model:value="record.batchNo"
              size="small"
              placeholder="批次号"
            />
          </template>

          <!-- 备注：可编辑 -->
          <template v-else-if="column.dataIndex === 'remark'">
            <Input
              v-model:value="record.remark"
              size="small"
              placeholder="备注"
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
          <div class="inbound-items-empty">
            请点击「添加产品」选择需要入库的产品
          </div>
        </template>
      </Table>
    </div>

    <!-- 产品选择弹窗 -->
    <ProductSelectModal
      :visible="productSelectVisible"
      :exclude-ids="excludeProductIds"
      :exclude-sku-keys="excludeSkuKeys"
      :exclude-sku-codes="excludeSkuCodes"
      :warehouse-id="selectedWarehouseId"
      @update:visible="(val) => (productSelectVisible = val)"
      @select="onProductSelected"
    />

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
    />

    <!-- 修改原因弹窗（编辑已完成单据时显示） -->
    <Modal
      v-model:open="changeReasonVisible"
      title="修改已完成单据"
      :confirm-loading="confirmLoading"
      :mask-closable="false"
      :width="480"
      ok-text="确认修改"
      cancel-text="取消"
      @ok="submitChangeReason"
    >
      <Alert
        type="warning"
        message="该单据已完成，修改后将自动调整库存。请填写修改原因，此操作将记录到修改日志。"
        show-icon
        class="mb-4"
      />
      <div class="mb-2 font-medium">
        修改原因 <span class="text-red-500">*</span>
      </div>
      <Textarea
        v-model:value="changeReason"
        placeholder="请填写修改原因，例如：录入数量错误，实际入库50件"
        :rows="4"
        :maxlength="500"
        show-count
      />
    </Modal>
  </Drawer>
</template>

<style>
.inbound-drawer {
  width: 75vw !important;
}

.inbound-drawer--fullscreen {
  width: 100vw !important;
}

.inbound-drawer__fs-btn {
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

.inbound-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}

.inbound-drawer__body {
  height: calc(100vh - 150px);
  padding: 0 8px;
  overflow-y: auto;
}

.inbound-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.inbound-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.inbound-items-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.inbound-items-count {
  font-size: 13px;
  color: #666;
}

.inbound-items-table {
  margin-bottom: 8px;
}

.inbound-items-empty {
  padding: 24px 0;
  font-size: 13px;
  color: #999;
  text-align: center;
}
</style>
