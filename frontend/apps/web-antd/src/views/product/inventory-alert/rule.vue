<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { useRoute } from 'vue-router';

import {
  Button,
  Checkbox,
  Drawer,
  Form,
  Input,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Select,
  Switch,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getProductSpecsApi } from '#/api';
import {
  batchSaveAlertRuleApi,
  createAlertRuleApi,
  deleteAlertRuleApi,
  getAlertRuleInfoApi,
  getAlertRuleListApi,
  updateAlertRuleApi,
} from '#/api/core/product/alert';
import { UserPickerModal } from '#/components/UserPickerModal';
import { $t } from '#/locales';

import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

import ProductSelectModal from './ProductSelectModal.vue';

const accessStore = useAccessStore();
const route = useRoute();

// ============ 已有规则索引（防重复 + 覆盖更新回显） ============
const addedProductIds = ref<number[]>([]);
const addedSkuKeys = ref<string[]>([]);
// key: `productId-warehouseId-skuId`（0 表示"全部"维度）→ 现有规则阈值
const existingRuleMap = ref<Map<string, ExistingRule>>(new Map());

interface ExistingRule {
  id: number;
  maxQuantity?: number;
  minQuantity?: number;
}

function currentRuleKey(skuId?: number): string {
  return `${Number(editForm.value.productId ?? 0)}-${Number(
    editForm.value.warehouseId ?? 0,
  )}-${Number(skuId ?? 0)}`;
}

async function loadExistingRules() {
  try {
    const res: any = await getAlertRuleListApi({ page: 1, pageSize: 1000 });
    const items = res?.items ?? res?.list ?? [];
    const pids: number[] = [];
    const skeys: string[] = [];
    const map = new Map<string, ExistingRule>();
    for (const r of items) {
      const pid = Number(r.productId ?? 0);
      if (pid > 0) pids.push(pid);
      const sid = Number(r.skuId ?? 0);
      if (pid > 0 && sid > 0) skeys.push(`${pid}-${sid}`);
      // 列表按更新时间倒序，同维度多条历史脏数据取最新一条
      const key = `${pid}-${Number(r.warehouseId ?? 0)}-${sid}`;
      if (!map.has(key)) {
        map.set(key, {
          id: Number(r.id),
          minQuantity:
            r.minQuantity == null ? undefined : Number(r.minQuantity),
          maxQuantity:
            r.maxQuantity == null ? undefined : Number(r.maxQuantity),
        });
      }
    }
    addedProductIds.value = pids;
    addedSkuKeys.value = skeys;
    existingRuleMap.value = map;
  } catch {
    addedProductIds.value = [];
    addedSkuKeys.value = [];
    existingRuleMap.value = new Map();
  }
}

function isRowAdded(row: { skuId: number }): boolean {
  return existingRuleMap.value.has(currentRuleKey(row.skuId));
}

// 勾选已有规则的规格时回显现有阈值（不覆盖已手输的值），语义 = 在现值基础上调整
function onRowCheckChange(row: {
  checked: boolean;
  maxQuantity: undefined | number;
  minQuantity: undefined | number;
  skuId: number;
}) {
  if (!row.checked) return;
  const existing = existingRuleMap.value.get(currentRuleKey(row.skuId));
  if (!existing) return;
  if (
    row.minQuantity == null &&
    existing.minQuantity != null &&
    existing.minQuantity > 0
  ) {
    row.minQuantity = existing.minQuantity;
  }
  if (
    row.maxQuantity == null &&
    existing.maxQuantity != null &&
    existing.maxQuantity > 0
  ) {
    row.maxQuantity = existing.maxQuantity;
  }
}

// ============ 产品弹窗选择 ============
const productSelectVisible = ref(false);

async function openProductSelect() {
  await loadExistingRules();
  productSelectVisible.value = true;
}

function onProductSelected(product: {
  id: number;
  name: string;
  specType: string;
}) {
  if (editForm.value.productId === product.id) return;
  editForm.value.productId = product.id;
  editForm.value.productName = product.name;
  onProductChange(product.id);
}

function clearProduct() {
  editForm.value.productId = undefined;
  editForm.value.productName = '';
  editForm.value.skuId = undefined;
  skuOptions.value = [];
  specRows.value = [];
}

// ============ SKU 选项（选择产品后加载，多规格产品可对单一规格设阈值） ============
const skuOptions = ref<{ label: string; value: number }[]>([]);
const skuLoading = ref(false);

// 多规格批量添加行（勾选 + 每行独立阈值）
const specRows = ref<{
  checked: boolean;
  label: string;
  maxQuantity: undefined | number;
  minQuantity: undefined | number;
  skuId: number;
}[]>([]);

function parseSpecsObj(v: any): Record<string, string> | null {
  if (!v) return null;
  try {
    const obj = typeof v === 'string' ? JSON.parse(v) : v;
    return obj && typeof obj === 'object' && !Array.isArray(obj)
      ? (obj as Record<string, string>)
      : null;
  } catch {
    return null;
  }
}

async function onProductChange(pid?: number, keepSku = false) {
  if (!keepSku) {
    editForm.value.skuId = undefined;
  }
  skuOptions.value = [];
  if (!pid) return;
  skuLoading.value = true;
  try {
    const res: any = await getProductSpecsApi(Number(pid));
    const data = res?.data ?? res;
    const skus = data?.skus ?? data?.skuList ?? [];
    if (skus.length > 1) {
      skuOptions.value = skus
        .filter((s: any) => Number(s?.id) > 0)
        .map((s: any) => {
          const specs = parseSpecsObj(s?.specs);
          const text = specs
            ? Object.entries(specs)
                .map(([k, v]) => `${k}:${v}`)
                .join(' / ')
            : s?.skuCode || '';
          return { label: text || `SKU#${s.id}`, value: Number(s.id) };
        });
      if (!keepSku) {
        specRows.value = skuOptions.value.map((o) => ({
          skuId: o.value,
          label: o.label,
          checked: false,
          minQuantity: undefined,
          maxQuantity: undefined,
        }));
      }
    }
  } catch {
    skuOptions.value = [];
  } finally {
    skuLoading.value = false;
  }
}

// ============ 仓库弹窗选择 ============
const warehouseSelectVisible = ref(false);

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  editForm.value.warehouseId = Number(warehouse.id);
  editForm.value.warehouseName =
    warehouse.warehouseName ?? warehouse.name ?? '';
}

function clearWarehouse() {
  editForm.value.warehouseId = undefined;
  editForm.value.warehouseName = '';
}

// ============ 列表筛选 ============
// 从预警列表行级「查看规则」跳转过来时：表单预填名称，首次查询精确按该产品/仓库过滤
const rowFilter = {
  productId: route.query.productId ? Number(route.query.productId) : undefined,
  warehouseId: route.query.warehouseId
    ? Number(route.query.warehouseId)
    : undefined,
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'productName',
      label: $t('page.product.inventory.alert.field.productName'),
      defaultValue: (route.query.productName as string) || undefined,
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'warehouseName',
      label: $t('page.product.inventory.alert.field.warehouseName'),
      defaultValue: (route.query.warehouseName as string) || undefined,
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
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
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getAlertRuleListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          warehouseName: formValues.warehouseName,
          productId: rowFilter.productId,
          warehouseId: rowFilter.warehouseId,
        });
        // 行级过滤参数仅首次查询生效，后续筛选由表单驱动
        rowFilter.productId = undefined;
        rowFilter.warehouseId = undefined;
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          if (items.length === 0) {
            gridEl.style.setProperty('height', '200px', 'important');
          } else {
            gridEl.style.removeProperty('height');
          }
        }
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { type: 'checkbox', width: 50 },
    {
      title: $t('page.product.inventory.alert.field.productName'),
      field: 'productName',
      minWidth: 140,
    },
    {
      title: $t('page.product.inventory.alert.field.specText'),
      field: 'specText',
      minWidth: 120,
      formatter: ({ cellValue }: any) => cellValue || $t('ui.all'),
    },
    {
      title: $t('page.product.inventory.alert.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.minQuantity'),
      field: 'minQuantity',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.maxQuantity'),
      field: 'maxQuantity',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.staleDays'),
      field: 'staleDays',
      width: 100,
    },
    {
      title: $t('page.product.inventory.alert.field.enableLowAlert'),
      field: 'enableLowAlert',
      width: 100,
      slots: { default: 'enableLowAlert' },
    },
    {
      title: $t('page.product.inventory.alert.field.enableHighAlert'),
      field: 'enableHighAlert',
      width: 100,
      slots: { default: 'enableHighAlert' },
    },
    {
      title: $t('page.product.inventory.alert.field.enableStaleAlert'),
      field: 'enableStaleAlert',
      width: 100,
      slots: { default: 'enableStaleAlert' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      width: 180,
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ============ 编辑/新增抽屉 ============
const drawerVisible = ref(false);
const drawerTitle = ref('');
const isEdit = ref(false);
const submitLoading = ref(false);

// 注意：fieldName 使用后端 camelCase 字段名
const editForm = ref({
  id: undefined as number | undefined,
  productId: undefined as number | undefined,
  productName: '',
  skuId: undefined as number | undefined,
  warehouseId: undefined as number | undefined,
  warehouseName: '',
  minQuantity: 0,
  maxQuantity: 0,
  staleDays: 90,
  enableLowAlert: true,
  enableHighAlert: false,
  enableStaleAlert: false,
  notifyUserId: undefined as number | undefined,
});

// 历史数据兼容：notifyUsers 存逗号分隔 ID，选择器为单选，回显取第一个
function parseFirstNotifyUser(raw?: null | string): number | undefined {
  if (!raw) return undefined;
  const first = raw.split(/[,，]/).find((p) => p.trim() !== '');
  const id = Number(first);
  return Number.isFinite(id) && id > 0 ? id : undefined;
}

function handleCreate() {
  isEdit.value = false;
  drawerTitle.value = $t('page.product.inventory.alert.action.create');
  editForm.value = {
    id: undefined,
    productId: undefined,
    productName: '',
    skuId: undefined,
    warehouseId: undefined,
    warehouseName: '',
    minQuantity: 0,
    maxQuantity: 0,
    staleDays: 90,
    enableLowAlert: true,
    enableHighAlert: false,
    enableStaleAlert: false,
    notifyUserId: undefined,
  };
  skuOptions.value = [];
  specRows.value = [];
  drawerVisible.value = true;
}

async function handleEdit(row: any) {
  isEdit.value = true;
  drawerTitle.value = $t('page.product.inventory.alert.action.edit');
  let data: any = row;
  try {
    const info = await getAlertRuleInfoApi(row.id);
    data = (info as any)?.data ?? row;
    editForm.value = {
      id: data.id,
      productId: data.productId ? Number(data.productId) : undefined,
      productName: data.productName || '',
      skuId: data.skuId ? Number(data.skuId) : undefined,
      warehouseId: data.warehouseId ? Number(data.warehouseId) : undefined,
      warehouseName: data.warehouseName || '',
      minQuantity: data.minQuantity ?? 0,
      maxQuantity: data.maxQuantity ?? 0,
      staleDays: data.staleDays ?? 90,
      enableLowAlert: data.enableLowAlert ?? true,
      enableHighAlert: data.enableHighAlert ?? false,
      enableStaleAlert: data.enableStaleAlert ?? false,
      notifyUserId: parseFirstNotifyUser(data.notifyUsers),
    };
  } catch {
    editForm.value = { ...row, productName: row.productName || '' };
    editForm.value.notifyUserId = parseFirstNotifyUser(row?.notifyUsers);
  }
  // 加载该产品的 SKU 选项（用于回显已选规格）
  const pid = editForm.value.productId;
  if (pid) {
    await onProductChange(pid, true);
  }
  // 已选 SKU 不在选项中（如 specs 为空）时，用列表返回的规格文本兜底显示
  const sid = editForm.value.skuId;
  if (sid && !skuOptions.value.some((o) => o.value === sid)) {
    skuOptions.value = [
      ...skuOptions.value,
      {
        label: data.specText || data.skuCode || `SKU#${sid}`,
        value: sid,
      },
    ];
  }
  drawerVisible.value = true;
}

function fillCheckedRows() {
  for (const row of specRows.value) {
    if (!row.checked) continue;
    row.minQuantity =
      editForm.value.minQuantity && editForm.value.minQuantity > 0
        ? editForm.value.minQuantity
        : undefined;
    row.maxQuantity =
      editForm.value.maxQuantity && editForm.value.maxQuantity > 0
        ? editForm.value.maxQuantity
        : undefined;
  }
}

async function handleSubmit() {
  const rf = 'page.product.inventory.alert.ruleForm';
  // 至少启用一种预警
  if (
    !editForm.value.enableLowAlert &&
    !editForm.value.enableHighAlert &&
    !editForm.value.enableStaleAlert
  ) {
    message.warning($t(`${rf}.needAlertType`));
    return;
  }
  // 低库存预警需要设置最低阈值
  if (
    editForm.value.enableLowAlert &&
    (!editForm.value.minQuantity || editForm.value.minQuantity <= 0)
  ) {
    message.warning($t(`${rf}.needMinQuantity`));
    return;
  }
  // 高库存预警需要设置最高阈值
  if (
    editForm.value.enableHighAlert &&
    (!editForm.value.maxQuantity || editForm.value.maxQuantity <= 0)
  ) {
    message.warning($t(`${rf}.needMaxQuantity`));
    return;
  }

  submitLoading.value = true;
  try {
    // 公共字段（每条规则共享）
    const basePayload = {
      productId: editForm.value.productId || undefined,
      warehouseId: editForm.value.warehouseId || undefined,
      staleDays: editForm.value.staleDays ?? 90,
      enableLowAlert: editForm.value.enableLowAlert,
      enableHighAlert: editForm.value.enableHighAlert,
      enableStaleAlert: editForm.value.enableStaleAlert,
      notifyUsers: editForm.value.notifyUserId
        ? String(editForm.value.notifyUserId)
        : undefined,
    };

    // 新增模式下勾选了多个规格 → 批量创建（一次提交多条规则）
    const checkedRows = isEdit.value
      ? []
      : specRows.value.filter((r) => r.checked);
    if (checkedRows.length > 0) {
      const invalid = checkedRows.some(
        (r) =>
          (editForm.value.enableLowAlert &&
            (!r.minQuantity || r.minQuantity <= 0)) ||
          (editForm.value.enableHighAlert &&
            (!r.maxQuantity || r.maxQuantity <= 0)),
      );
      if (invalid) {
        message.warning($t(`${rf}.needSpecThreshold`));
        return;
      }
      const doBatchSave = async () => {
        submitLoading.value = true;
        try {
          const rules = checkedRows.map((r) => ({
            ...basePayload,
            skuId: r.skuId,
            minQuantity: r.minQuantity || undefined,
            maxQuantity: r.maxQuantity || undefined,
          }));
          // 后端 upsert：已存在同维度规则走覆盖更新，返回新增/更新计数
          const res: any = await batchSaveAlertRuleApi(rules);
          const data = res?.data ?? res ?? {};
          const inserted = Number(data.inserted ?? 0);
          const updated = Number(data.updated ?? 0);
          message.success(
            updated > 0
              ? $t(`${rf}.batchUpsertSuccess`, { inserted, updated })
              : $t(`${rf}.batchCreateSuccess`, { count: rules.length }),
          );
          drawerVisible.value = false;
          gridApi.query();
        } finally {
          submitLoading.value = false;
        }
      };
      // 勾选行中已有规则的条数 → 覆盖更新前二次确认
      const updateCount = checkedRows.filter((r) => isRowAdded(r)).length;
      if (updateCount > 0) {
        submitLoading.value = false;
        Modal.confirm({
          title: $t(`${rf}.overwriteTitle`),
          content: $t(`${rf}.overwriteContent`, {
            count: updateCount,
            total: checkedRows.length,
          }),
          onOk: doBatchSave,
        });
        return;
      }
      await doBatchSave();
      return;
    }

    // 单条创建/编辑（不勾选规格 = 对所有规格生效）
    const payload = {
      ...basePayload,
      skuId: editForm.value.skuId || undefined,
      minQuantity: editForm.value.minQuantity || undefined,
      maxQuantity: editForm.value.maxQuantity || undefined,
    };

    if (isEdit.value) {
      await updateAlertRuleApi({ ...payload, id: editForm.value.id });
      message.success($t('ui.notification.update_success'));
    } else {
      // 防重复：同产品+仓库+规格维度已有未删除规则时阻止提交，引导编辑现有规则
      const checkKey = `${Number(editForm.value.productId ?? 0)}-${Number(
        editForm.value.warehouseId ?? 0,
      )}-${Number(editForm.value.skuId ?? 0)}`;
      const existRes: any = await getAlertRuleListApi({
        page: 1,
        pageSize: 1000,
        productId: editForm.value.productId,
      });
      const existItems = existRes?.items ?? existRes?.list ?? [];
      const duplicated = existItems.some(
        (it: any) =>
          `${Number(it.productId ?? 0)}-${Number(it.warehouseId ?? 0)}-${Number(
            it.skuId ?? 0,
          )}` === checkKey,
      );
      if (duplicated) {
        message.warning($t(`${rf}.duplicateRule`));
        return;
      }
      await createAlertRuleApi(payload);
      message.success($t('ui.notification.create_success'));
    }
    drawerVisible.value = false;
    gridApi.query();
  } finally {
    submitLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteAlertRuleApi([row.id]);
    message.success($t('ui.notification.delete_success'));
    gridApi.query();
  } catch {
    // 错误提示由拦截器处理
  }
}

async function handleBatchDelete() {
  const checked = gridApi.grid?.getCheckboxRecords() ?? [];
  const reserved = gridApi.grid?.getCheckboxReserveRecords() ?? [];
  const seen = new Set<number>();
  const records = [...checked, ...reserved].filter((r: any) => {
    if (seen.has(r.id)) {
      return false;
    }
    seen.add(r.id);
    return true;
  });
  if (!records.length) {
    message.warning(
      $t('page.product.inventory.alert.ruleForm.needSelectRecord'),
    );
    return;
  }
  Modal.confirm({
    title: $t('ui.text.batch_delete_title'),
    content: $t('ui.text.confirm_batch_delete', { count: records.length }),
    onOk: async () => {
      try {
        await deleteAlertRuleApi(records.map((r: any) => r.id));
        message.success($t('ui.notification.delete_success'));
        gridApi.query();
      } catch {
        // ignore
      }
    },
  });
}

onMounted(() => {
  // 从预警列表点「新增」跳转过来时自动打开新增抽屉
  if (route.query.create === '1') {
    handleCreate();
  }
});
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.inventory.alert.rule.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:update')"
          type="primary"
          class="mr-2"
          :icon="h(LucidePlus)"
          @click="handleCreate"
        >
          {{ $t('page.product.inventory.alert.action.create') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('product:alert:update')"
          class="mr-2"
          danger
          ghost
          :icon="h(LucideTrash2)"
          @click="handleBatchDelete"
        >
          {{ $t('ui.button.batch_delete') }}
        </Button>
      </template>

      <template #enableLowAlert="{ row }">
        <Tag :color="row.enableLowAlert ? 'green' : 'default'">
          {{ row.enableLowAlert ? $t('ui.enabled') : $t('ui.disabled') }}
        </Tag>
      </template>
      <template #enableHighAlert="{ row }">
        <Tag :color="row.enableHighAlert ? 'green' : 'default'">
          {{ row.enableHighAlert ? $t('ui.enabled') : $t('ui.disabled') }}
        </Tag>
      </template>
      <template #enableStaleAlert="{ row }">
        <Tag :color="row.enableStaleAlert ? 'green' : 'default'">
          {{ row.enableStaleAlert ? $t('ui.enabled') : $t('ui.disabled') }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:alert:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          {{ $t('page.product.inventory.alert.action.edit') }}
        </Button>
        <Popconfirm
          v-if="accessStore.hasAccessCode('product:alert:update')"
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.inventory.alert.module'),
            })
          "
          @confirm="() => handleDelete(row)"
        >
          <Button type="link" danger :icon="h(LucideTrash2)">
            {{ $t('page.product.inventory.alert.action.delete') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <!-- 新增/编辑抽屉 -->
    <Drawer
      v-model:open="drawerVisible"
      :title="drawerTitle"
      width="75%"
      placement="right"
      :mask-closable="true"
      :closable="true"
    >
      <Form layout="vertical">
        <!-- 产品选择：不选=全部产品 -->
        <Form.Item
          :label="$t('page.product.inventory.alert.ruleForm.productAll')"
        >
          <Input
            :value="editForm.productName || ''"
            :placeholder="
              $t('page.product.inventory.alert.ruleForm.productAllPlaceholder')
            "
            readonly
            allow-clear
            style="cursor: pointer"
            @click="openProductSelect"
            @change="
              (e: any) => {
                if (!e?.target?.value) clearProduct();
              }
            "
          />
        </Form.Item>
        <!-- 新增模式 + 多规格：批量勾选规格表格（多选 + 统一填充 + 逐行微调） -->
        <Form.Item
          v-if="!isEdit && skuOptions.length > 0"
          :label="$t('page.product.inventory.alert.field.sku')"
        >
          <div class="spec-hint">
            {{ $t('page.product.inventory.alert.ruleForm.skuHint') }}
          </div>
          <div class="spec-toolbar">
            <Button size="small" @click="fillCheckedRows">
              {{ $t('page.product.inventory.alert.ruleForm.fillChecked') }}
            </Button>
          </div>
          <div class="spec-table-wrap">
            <table class="spec-table">
              <thead>
                <tr>
                  <th class="spec-col-check"></th>
                  <th>
                    {{ $t('page.product.inventory.alert.ruleForm.specCol') }}
                  </th>
                  <th class="spec-col-num">
                    {{ $t('page.product.inventory.alert.ruleForm.minCol') }}
                  </th>
                  <th class="spec-col-num">
                    {{ $t('page.product.inventory.alert.ruleForm.maxCol') }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="row in specRows"
                  :key="row.skuId"
                  :class="{ 'spec-row-checked': row.checked }"
                >
                  <td class="spec-col-check">
                    <Checkbox
                      v-model:checked="row.checked"
                      @change="onRowCheckChange(row)"
                    />
                  </td>
                  <td class="spec-name" :title="row.label">
                    {{ row.label }}
                    <Tag
                      v-if="isRowAdded(row)"
                      :color="row.checked ? 'processing' : 'warning'"
                    >
                      {{
                        row.checked
                          ? $t(
                              'page.product.inventory.alert.ruleForm.skuWillUpdate',
                            )
                          : $t(
                              'page.product.inventory.alert.ruleForm.skuAdded',
                            )
                      }}
                    </Tag>
                  </td>
                  <td class="spec-col-num">
                    <InputNumber
                      v-model:value="row.minQuantity"
                      :min="0"
                      size="small"
                      style="width: 100%"
                    />
                  </td>
                  <td class="spec-col-num">
                    <InputNumber
                      v-model:value="row.maxQuantity"
                      :min="0"
                      size="small"
                      style="width: 100%"
                    />
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </Form.Item>
        <!-- 编辑模式：单条规则回显，规格下拉选择 -->
        <Form.Item
          v-else-if="skuOptions.length > 0"
          :label="$t('page.product.inventory.alert.field.sku')"
        >
          <Select
            v-model:value="editForm.skuId"
            :placeholder="
              $t('page.product.inventory.alert.ruleForm.skuAllPlaceholder')
            "
            allow-clear
            show-search
            :loading="skuLoading"
            :options="skuOptions"
            :filter-option="
              (input: string, option: any) =>
                (option?.label ?? '')
                  .toLowerCase()
                  .includes(input.toLowerCase())
            "
            style="width: 100%"
          />
        </Form.Item>

        <!-- 仓库选择：不选=全部仓库 -->
        <Form.Item
          :label="$t('page.product.inventory.alert.ruleForm.warehouseAll')"
        >
          <Input
            :value="editForm.warehouseName || ''"
            :placeholder="
              $t(
                'page.product.inventory.alert.ruleForm.warehouseAllPlaceholder',
              )
            "
            readonly
            allow-clear
            style="cursor: pointer"
            @click="openWarehouseSelect"
            @change="
              (e: any) => {
                if (!e?.target?.value) clearWarehouse();
              }
            "
          />
        </Form.Item>

        <Form.Item
          :label="$t('page.product.inventory.alert.field.minQuantity')"
        >
          <InputNumber
            v-model:value="editForm.minQuantity"
            style="width: 100%"
            :min="0"
            :placeholder="
              $t('page.product.inventory.alert.ruleForm.minPlaceholder')
            "
          />
        </Form.Item>
        <Form.Item
          :label="$t('page.product.inventory.alert.field.maxQuantity')"
        >
          <InputNumber
            v-model:value="editForm.maxQuantity"
            style="width: 100%"
            :min="0"
            :placeholder="
              $t('page.product.inventory.alert.ruleForm.maxPlaceholder')
            "
          />
        </Form.Item>
        <Form.Item :label="$t('page.product.inventory.alert.field.staleDays')">
          <InputNumber
            v-model:value="editForm.staleDays"
            style="width: 100%"
            :min="0"
            :placeholder="
              $t('page.product.inventory.alert.ruleForm.staleDaysPlaceholder')
            "
          />
        </Form.Item>
        <Form.Item
          :label="$t('page.product.inventory.alert.field.enableLowAlert')"
        >
          <Switch v-model:checked="editForm.enableLowAlert" />
        </Form.Item>
        <Form.Item
          :label="$t('page.product.inventory.alert.field.enableHighAlert')"
        >
          <Switch v-model:checked="editForm.enableHighAlert" />
        </Form.Item>
        <Form.Item
          :label="$t('page.product.inventory.alert.field.enableStaleAlert')"
        >
          <Switch v-model:checked="editForm.enableStaleAlert" />
        </Form.Item>
        <Form.Item
          :label="$t('page.product.inventory.alert.ruleForm.notifyUsers')"
        >
          <UserPickerModal
            v-model:value="editForm.notifyUserId"
            :placeholder="
              $t('page.product.inventory.alert.ruleForm.notifyUsersPlaceholder')
            "
          />
          <div class="spec-hint">
            {{ $t('page.product.inventory.alert.ruleForm.notifyUsersHint') }}
          </div>
        </Form.Item>
      </Form>
      <template #footer>
        <div style="text-align: right">
          <Button class="mr-2" @click="drawerVisible = false">
            {{ $t('ui.button.cancel') }}
          </Button>
          <Button type="primary" :loading="submitLoading" @click="handleSubmit">
            {{ $t('ui.button.ok') }}
          </Button>
        </div>
      </template>
    </Drawer>

    <!-- 仓库选择弹窗 -->
    <WarehouseSelectModal
      :visible="warehouseSelectVisible"
      @update:visible="(val) => (warehouseSelectVisible = val)"
      @select="onWarehouseSelected"
    />

    <!-- 产品选择弹窗 -->
    <ProductSelectModal
      :visible="productSelectVisible"
      :added-product-ids="addedProductIds"
      :added-sku-keys="addedSkuKeys"
      @update:visible="(val) => (productSelectVisible = val)"
      @select="onProductSelected"
    />
  </Page>
</template>

<style scoped>
.spec-hint {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 6px;
  line-height: 1.5;
}

.spec-toolbar {
  margin-bottom: 6px;
}

.spec-table-wrap {
  max-height: 260px;
  overflow-y: auto;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.spec-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.spec-table th,
.spec-table td {
  padding: 4px 8px;
  border-bottom: 1px solid hsl(var(--border));
  text-align: left;
}

.spec-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: hsl(var(--muted));
  color: hsl(var(--foreground));
  font-weight: 500;
}

.spec-col-check {
  width: 40px;
  text-align: center !important;
}

.spec-col-num {
  width: 110px;
}

.spec-name {
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.spec-row-checked {
  background: hsl(var(--primary) / 0.06);
}

.spec-table tbody tr:last-child td {
  border-bottom: none;
}
</style>
