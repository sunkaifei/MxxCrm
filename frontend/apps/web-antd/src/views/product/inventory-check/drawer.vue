<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Button,
  InputNumber,
  message,
  Modal,
  Select,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createCheckApi,
  getCheckInfoApi,
  toFrontendType,
  updateCheckApi,
} from '#/api/core/product/check';
import { getInventoryListApi } from '#/api/core/product/inventory';
import { getAdminOptionsApi } from '#/api/core/system/user';
import { $t } from '#/locales';

import ProductSelectModal from '../../sale/components/ProductSelectModal.vue';
import WarehouseSelectModal from './WarehouseSelectModal.vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

// 盘点人列表
const assigneeOptions = ref<{ label: string; value: number }[]>([]);
const selectedRowKeys = ref<number[]>([]);
const batchAssignVisible = ref(false);
const batchAssigneeIds = ref<number[]>([]);

async function loadAssignees() {
  try {
    const resp: any = await getAdminOptionsApi();
    const data = resp?.data ?? resp;
    const list = Array.isArray(data) ? data : (data?.list ?? []);
    assigneeOptions.value = list.map((u: any) => ({
      label: u.nickName ?? u.nickname ?? u.name ?? u.label,
      value: Number(u.id ?? u.value),
    }));
  } catch {
    // 忽略
  }
}

function openBatchAssign() {
  if (selectedRowKeys.value.length === 0) {
    message.warning('请先勾选要分配的产品');
    return;
  }
  batchAssigneeIds.value = [];
  batchAssignVisible.value = true;
}

function confirmBatchAssign() {
  for (const id of selectedRowKeys.value) {
    const item = tableItems.value.find((i) => i.id === id);
    if (item) {
      item.assigneeIds = [...batchAssigneeIds.value];
    }
  }
  batchAssignVisible.value = false;
  message.success(`已分配 ${selectedRowKeys.value.length} 个产品的盘点人`);
  selectedRowKeys.value = [];
}

const drawerClass = computed(() => [
  'check-drawer',
  { 'check-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const checkTypeOptions = [
  { label: $t('page.product.inventory.check.type.1'), value: 1 },
  { label: $t('page.product.inventory.check.type.2'), value: 2 },
  { label: $t('page.product.inventory.check.type.3'), value: 3 },
];

// ============ 差异原因选项（主流ERP标准） ============
const diffReasonOptions = [
  { label: '正常损耗', value: 'normal_loss' },
  { label: '损坏/破损', value: 'damaged' },
  { label: '丢失', value: 'lost' },
  { label: '计数错误', value: 'count_error' },
  { label: '过期', value: 'expired' },
  { label: '被盗', value: 'theft' },
  { label: '调拨未完成', value: 'transfer_pending' },
  { label: '入库未登记', value: 'inbound_unrecorded' },
  { label: '其他', value: 'other' },
];

// ============ 处理方式选项 ============
const handlingOptions = [
  { label: '调整库存', value: 'adjust' },
  { label: '报废处理', value: 'scrap' },
  { label: '挂账待查', value: 'pending' },
  { label: '调查处理', value: 'investigate' },
  { label: '自行承担', value: 'absorb' },
];

// ============ 仓库弹窗选择 ============
const warehouseSelectVisible = ref(false);
const selectedWarehouseName = ref('');
const selectedWarehouseId = ref<number | undefined>();

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  mainFormApi.setFieldValue('warehouseId', String(warehouse.id));
  mainFormApi.setFieldValue('warehouseDisplay', warehouse.warehouseName);
  selectedWarehouseName.value = warehouse.warehouseName;
  selectedWarehouseId.value = Number(warehouse.id);
  // 仓库变化时刷新已有产品的系统库存
  if (tableItems.value.length > 0) {
    fetchSystemStock();
  }
}

// ============ 产品明细 ============
interface StocktakeItem {
  id?: number;
  productId: number;
  /** SKU ID（多规格产品按SKU盘点时使用，0表示无SKU） */
  skuId?: number;
  productName: string;
  productSku?: string;
  productCode?: string;
  systemQuantity?: null | number;
  actualQuantity?: null | number;
  difference?: null | number;
  diffReason?: string;
  handling?: string;
  remark?: string;
  assigneeIds?: number[];
}

const tableItems = ref<StocktakeItem[]>([]);
const productSelectVisible = ref(false);
const stockLoading = ref(false);

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

// 获取选中仓库的库存数据
async function fetchSystemStock() {
  const values = await mainFormApi.getValues();
  const warehouseId = values?.warehouseId;
  if (!warehouseId) return;

  stockLoading.value = true;
  try {
    const resp: any = await getInventoryListApi({
      page: 1,
      pageSize: 999,
      warehouseId: Number(warehouseId),
    });
    const raw = resp?.data ?? resp;
    const list = raw?.items ?? raw?.list ?? (Array.isArray(raw) ? raw : []);

    // 构建 productId → quantity 映射
    const stockMap = new Map<number, number>();
    for (const item of list) {
      const pid = Number(item.productId ?? item.product_id);
      if (pid) {
        stockMap.set(pid, Number(item.quantity ?? 0));
      }
    }

    // 回填系统库存到表格
    tableItems.value.forEach((item) => {
      const stock = stockMap.get(item.productId);
      item.systemQuantity = stock === undefined ? 0 : stock;
    });
  } catch (error) {
    console.error('[库存盘点] 获取库存失败:', error);
  } finally {
    stockLoading.value = false;
  }
}

// 计算差异
function computeDiff(item: Record<string, any> | StocktakeItem): null | number {
  if (item.actualQuantity === null || item.actualQuantity === undefined)
    return null;
  const system = Number(item.systemQuantity ?? 0);
  const actual = Number(item.actualQuantity);
  return actual - system;
}

// 差异类型标签
function getDiffTag(diff: null | number) {
  if (diff === null) return { label: '-', color: 'default' };
  if (diff > 0) return { label: '盘盈', color: 'success' };
  if (diff < 0) return { label: '盘亏', color: 'error' };
  return { label: '一致', color: 'default' };
}

// 实盘数量变化时自动计算差异
function onActualQuantityChange(
  item: Record<string, any> | StocktakeItem,
  val: null | number | string,
) {
  item.actualQuantity = val === null ? null : Number(val);
  item.difference = computeDiff(item);
}

const itemColumns = computed(() => [
  {
    title: $t('page.product.inventory.check.field.productCode'),
    dataIndex: 'productCode',
    width: 110,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.productName'),
    dataIndex: 'productName',
    width: 150,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.productSku'),
    dataIndex: 'productSku',
    width: 100,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.systemQuantity'),
    dataIndex: 'systemQuantity',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.product.inventory.check.field.actualQuantity'),
    dataIndex: 'actualQuantity',
    width: 110,
  },
  {
    title: $t('page.product.inventory.check.field.difference'),
    dataIndex: 'difference',
    width: 80,
    align: 'right' as const,
  },
  {
    title: $t('page.product.inventory.check.field.differenceType'),
    dataIndex: 'differenceType',
    width: 75,
    align: 'center' as const,
  },
  {
    title: '盘点人',
    dataIndex: 'assignees',
    width: 130,
  },
  {
    title: $t('page.product.inventory.check.field.diffReason'),
    dataIndex: 'diffReason',
    width: 120,
  },
  {
    title: $t('page.product.inventory.check.field.handling'),
    dataIndex: 'handling',
    width: 110,
  },
  {
    title: $t('ui.table.action'),
    dataIndex: 'action',
    width: 60,
    fixed: 'right' as const,
  },
]);

const rowSelection = computed(() => ({
  selectedRowKeys: selectedRowKeys.value,
  onChange: (keys: any[]) => {
    selectedRowKeys.value = keys;
  },
}));

function openProductSelect() {
  // 先校验仓库是否已选
  mainFormApi.getValues().then((values) => {
    if (!values?.warehouseId) {
      message.warning('请先选择盘点仓库');
      return;
    }
    productSelectVisible.value = true;
  });
}

function onProductSelected(items: any[]) {
  const existingKeys = new Set(
    tableItems.value.map((i) => `${i.productId}-${i.skuId || 0}`),
  );
  let added = 0;
  for (const item of items) {
    const key = `${item.productId}-${item.skuId || 0}`;
    if (!existingKeys.has(key)) {
      tableItems.value.push({
        productId: item.productId,
        skuId: item.skuId || 0,
        productName: item.productName,
        productCode: item.productCode,
        productSku: item.skuCode || '',
        systemQuantity: null,
        actualQuantity: null,
        difference: null,
        diffReason: '',
        handling: '',
        remark: '',
        assigneeIds: [],
      });
      added++;
    }
  }
  productSelectVisible.value = false;

  // 添加完后获取系统库存
  if (added > 0) {
    fetchSystemStock();
    message.success(`已添加 ${added} 个产品`);
  }
}

function removeItem(index: number) {
  tableItems.value.splice(index, 1);
}

function removeSelected() {
  if (selectedRowKeys.value.length === 0) {
    message.warning('请先选择要删除的产品');
    return;
  }
  tableItems.value = tableItems.value.filter(
    (item) => item.id === undefined || !selectedRowKeys.value.includes(item.id),
  );
  selectedRowKeys.value = [];
}

// 盘盈/盘亏/一致统计
const summary = computed(() => {
  let surplus = 0;
  let shortage = 0;
  let match = 0;
  let pending = 0;
  for (const item of tableItems.value) {
    const diff = computeDiff(item);
    if (diff === null) {
      pending++;
    } else if (diff > 0) {
      surplus++;
    } else if (diff < 0) {
      shortage++;
    } else {
      match++;
    }
  }
  return { surplus, shortage, match, pending, total: tableItems.value.length };
});

// ============ 表单 ============
const formSchema: VbenFormSchema[] = [
  {
    component: 'Divider',
    fieldName: '_div1',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({
      default: () => $t('page.product.inventory.check.drawer.basicInfo'),
    }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'checkType',
    label: $t('page.product.inventory.check.field.checkType'),
    defaultValue: 2,
    rules: 'required',
    componentProps: {
      placeholder: $t(
        'page.product.inventory.check.drawer.checkTypePlaceholder',
      ),
      options: checkTypeOptions,
      allowClear: true,
    },
  },
  {
    component: 'Input',
    fieldName: 'warehouseDisplay',
    label: $t('page.product.inventory.check.field.warehouse'),
    rules: 'required',
    componentProps: {
      placeholder: '点击选择仓库',
      readOnly: true,
      style: { cursor: 'pointer' },
      onClick: () => openWarehouseSelect(),
    },
  },
  {
    component: 'Input',
    fieldName: 'warehouseId',
    label: '',
    formItemClass: 'hidden',
    dependencies: { triggerFields: ['warehouseId'] },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: $t('page.product.inventory.check.field.remark'),
    componentProps: {
      placeholder: $t('page.product.inventory.check.drawer.remarkPlaceholder'),
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
        message.warning($t('page.product.inventory.check.action.noItems'));
        return;
      }

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();

      const { _div1, _div2, _warehouseDisplay, warehouseId, ...rest } =
        values as any;

      const data = {
        ...rest,
        checkType: Number(rest.checkType),
        warehouseId: warehouseId ? Number(warehouseId) : undefined,
        items: tableItems.value.map((item) => ({
          productId: item.productId,
          skuId: item.skuId && item.skuId > 0 ? item.skuId : undefined,
          productName: item.productName,
          productSku: item.productSku,
          systemQuantity: item.systemQuantity,
          assigneeIds: item.assigneeIds || [],
          diffReason: item.diffReason || '',
          handling: item.handling || '',
        })),
      };

      if (drawerData.value.create) {
        await createCheckApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateCheckApi({ ...data, id: drawerData.value.row.id });
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
  async onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{
        create: boolean;
        row?: any;
      }>() || {
        create: true,
      };
      await mainFormApi.resetForm();
      confirmLoading.value = false;
      tableItems.value = [];
      selectedRowKeys.value = [];
      selectedWarehouseName.value = '';
      selectedWarehouseId.value = undefined;
      loadAssignees();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      } else {
        mainFormApi.setFieldValue('checkType', 2);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getCheckInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const main = data.main ?? data;
    const items = data.items ?? [];

    mainFormApi.setValues({
      checkType: toFrontendType(main.stocktakeType),
      warehouseId: main.warehouseId ? String(main.warehouseId) : undefined,
      warehouseDisplay: main.warehouseName ?? '',
      remark: main.remark,
    });
    selectedWarehouseName.value = main.warehouseName ?? '';
    selectedWarehouseId.value = main.warehouseId
      ? Number(main.warehouseId)
      : undefined;

    tableItems.value = items.map((item: any) => ({
      id: item.id,
      productId: Number(item.productId),
      skuId: Number(item.skuId ?? item.sku_id ?? 0),
      productName: item.productName ?? '',
      productCode: item.productCode ?? '',
      productSku: item.productSku ?? '',
      systemQuantity:
        item.systemQuantity !== null && item.systemQuantity !== undefined
          ? Number(item.systemQuantity)
          : null,
      actualQuantity:
        item.actualQuantity !== null && item.actualQuantity !== undefined
          ? Number(item.actualQuantity)
          : null,
      difference:
        item.difference !== null && item.difference !== undefined
          ? Number(item.difference)
          : null,
      diffReason: '',
      handling: '',
      remark: item.remark ?? '',
    }));
  } catch (error) {
    console.error('[库存盘点] 加载详情失败:', error);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="
      drawerData.create
        ? $t('page.product.inventory.check.drawer.createTitle')
        : $t('page.product.inventory.check.drawer.editTitle')
    "
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip
        :title="
          isFullscreen
            ? $t('page.product.inventory.check.drawer.restore')
            : $t('page.product.inventory.check.drawer.fullscreen')
        "
      >
        <button
          type="button"
          class="check-drawer__fs-btn"
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

    <div class="check-drawer__body">
      <!-- 基本信息表单 -->
      <MainForm />

      <!-- 产品明细分隔线 + 统计 -->
      <div class="check-items-header">
        <span class="check-items-title">{{
          $t('page.product.inventory.check.drawer.detail')
        }}</span>
        <div v-if="tableItems.length > 0" class="check-items-stats">
          <Tag>共 {{ summary.total }}</Tag>
          <Tag v-if="summary.surplus > 0" color="success">
            盘盈 {{ summary.surplus }}
          </Tag>
          <Tag v-if="summary.shortage > 0" color="error">
            盘亏 {{ summary.shortage }}
          </Tag>
          <Tag v-if="summary.match > 0" color="default">
            一致 {{ summary.match }}
          </Tag>
          <Tag v-if="summary.pending > 0" color="warning">
            待盘 {{ summary.pending }}
          </Tag>
        </div>
      </div>

      <!-- 工具栏 -->
      <div class="check-items-toolbar">
        <Button type="primary" size="small" @click="openProductSelect">
          {{ $t('page.product.inventory.check.drawer.addItem') }}
        </Button>
        <Button
          size="small"
          :disabled="selectedRowKeys.length === 0"
          @click="openBatchAssign"
        >
          批量分配盘点人
          <span v-if="selectedRowKeys.length > 0"
            >({{ selectedRowKeys.length }})</span
          >
        </Button>
        <Button
          size="small"
          danger
          :disabled="selectedRowKeys.length === 0"
          @click="removeSelected"
        >
          {{ $t('page.product.inventory.check.drawer.removeSelected') }}
          <span v-if="selectedRowKeys.length > 0"
            >({{ selectedRowKeys.length }})</span
          >
        </Button>
      </div>

      <!-- 产品明细表格 -->
      <Table
        :columns="itemColumns"
        :data-source="tableItems"
        :row-selection="rowSelection"
        :row-key="(record) => record.id"
        :pagination="false"
        size="small"
        :scroll="{ x: 1200, y: 380 }"
        :loading="stockLoading"
        class="check-items-table"
      >
        <template #bodyCell="{ column, record, index }">
          <!-- 系统数量 -->
          <template v-if="column.dataIndex === 'systemQuantity'">
            <span v-if="record.systemQuantity === null" class="text-gray-400"
              >加载中</span
            >
            <span v-else class="font-medium">{{ record.systemQuantity }}</span>
          </template>

          <!-- 实盘数量：可编辑 -->
          <template v-else-if="column.dataIndex === 'actualQuantity'">
            <InputNumber
              :value="record.actualQuantity"
              size="small"
              style="width: 100%"
              :precision="0"
              :step="1"
              placeholder="输入"
              @update:value="(val) => onActualQuantityChange(record, val)"
            />
          </template>

          <!-- 差异：自动计算 -->
          <template v-else-if="column.dataIndex === 'difference'">
            <span v-if="computeDiff(record) === null" class="text-gray-400"
              >-</span
            >
            <span
              v-else
              :class="{
                'text-green-600 font-medium': computeDiff(record)! > 0,
                'text-red-600 font-medium': computeDiff(record)! < 0,
                'text-gray-500': computeDiff(record) === 0,
              }"
            >
              {{ computeDiff(record)! > 0 ? '+' : '' }}{{ computeDiff(record) }}
            </span>
          </template>

          <!-- 差异类型：自动标签 -->
          <template v-else-if="column.dataIndex === 'differenceType'">
            <Tag
              :color="getDiffTag(computeDiff(record)).color"
              :bordered="false"
              style="font-size: 11px"
            >
              {{ getDiffTag(computeDiff(record)).label }}
            </Tag>
          </template>

          <!-- 盘点人：多选下拉 -->
          <template v-else-if="column.dataIndex === 'assignees'">
            <Select
              v-model:value="record.assigneeIds"
              mode="multiple"
              size="small"
              style="width: 100%; min-width: 90px"
              :options="assigneeOptions"
              placeholder="分配"
              :max-tag-count="1"
              :max-tag-text-length="6"
              allow-clear
            />
          </template>

          <!-- 差异原因：下拉选择 -->
          <template v-else-if="column.dataIndex === 'diffReason'">
            <Select
              v-model:value="record.diffReason"
              size="small"
              style="width: 100%"
              placeholder="选择"
              :options="diffReasonOptions"
              allow-clear
              :disabled="
                computeDiff(record) === 0 || computeDiff(record) === null
              "
            />
          </template>

          <!-- 处理方式：下拉选择 -->
          <template v-else-if="column.dataIndex === 'handling'">
            <Select
              v-model:value="record.handling"
              size="small"
              style="width: 100%"
              placeholder="选择"
              :options="handlingOptions"
              allow-clear
              :disabled="
                computeDiff(record) === 0 || computeDiff(record) === null
              "
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
          <div class="check-items-empty">
            {{ $t('page.product.inventory.check.drawer.emptyHint') }}
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

    <!-- 批量分配盘点人弹窗 -->
    <Modal
      v-model:open="batchAssignVisible"
      title="批量分配盘点人"
      width="420px"
      @ok="confirmBatchAssign"
    >
      <p style="margin-bottom: 12px; font-size: 13px; color: #666">
        将为选中的
        <strong>{{ selectedRowKeys.length }}</strong> 个产品分配盘点人：
      </p>
      <Select
        v-model:value="batchAssigneeIds"
        mode="multiple"
        style="width: 100%"
        :options="assigneeOptions"
        placeholder="选择一个或多个盘点人"
      />
    </Modal>
  </Drawer>
</template>

<style>
.check-drawer {
  width: 80vw !important;
}

.check-drawer--fullscreen {
  width: 100vw !important;
}

.check-drawer__fs-btn {
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

.check-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}

.check-drawer__body {
  height: calc(100vh - 150px);
  padding: 0 8px;
  overflow-y: auto;
}

.check-items-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 12px;
  margin: 16px 0 8px;
  border-top: 1px solid #f0f0f0;
}

.check-items-title {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.check-items-stats {
  display: flex;
  gap: 4px;
}

.check-items-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.check-items-table {
  margin-bottom: 8px;
}

.check-items-empty {
  padding: 24px 0;
  font-size: 13px;
  color: #999;
  text-align: center;
}
</style>
