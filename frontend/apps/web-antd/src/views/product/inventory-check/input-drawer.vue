<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import {
  Button,
  InputNumber,
  Select,
  Tag,
  Progress,
  Table,
  message,
  Modal,
  Tooltip,
} from 'ant-design-vue';
import { getCheckItemsApi, inputCheckApi } from '#/api/core/product/check';
import { getProductSpecsApi } from '#/api/core/product/spec';
import { getAdminOptionsApi } from '#/api/core/system/user';
import { useUserStore } from '@vben/stores';
import { $t } from '#/locales';
import ProductSelectModal from '../../sale/components/ProductSelectModal.vue';

const userStore = useUserStore();
const confirmLoading = ref(false);
const isFullscreen = ref(false);
const drawerData = ref<{ row?: any }>({});
const items = ref<any[]>([]);
const productSelectVisible = ref(false);
const selectedRowKeys = ref<number[]>([]);
const activeTab = ref<'all' | 'mine' | 'uninputted' | 'inputted'>('all');

// 已添加产品的排除列表（computed 确保响应式）
// 单规格产品（无 productCode）：整体排除 productId
const excludeProductIds = computed(() =>
  items.value
    .filter((i: any) => !i.skuId || i.skuId === 0)
    .map((i: any) => Number(i.productId)),
);
// 有 skuId 的新数据：按 productId-skuId 排除
const excludeSkuKeys = computed(() =>
  items.value
    .filter((i: any) => i.skuId && i.skuId > 0)
    .map((i: any) => `${i.productId}-${i.skuId}`),
);
// 有 productCode（SKU编码）的项：按编码排除（兼容历史数据无 sku_id）
const excludeSkuCodes = computed(() =>
  items.value
    .filter((i: any) => i.productCode && i.productCode !== '')
    .map((i: any) => i.productCode),
);

// 盘点人列表
const assigneeOptions = ref<{ label: string; value: number }[]>([]);
const currentUserId = Number(userStore.userInfo?.userId ?? 0);

// 批量分配弹窗
const batchAssignVisible = ref(false);
const batchAssigneeIds = ref<number[]>([]);

// 差异原因选项
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

const handlingOptions = [
  { label: '调整库存', value: 'adjust' },
  { label: '报废处理', value: 'scrap' },
  { label: '挂账待查', value: 'pending' },
  { label: '调查处理', value: 'investigate' },
  { label: '自行承担', value: 'absorb' },
];

// 加载盘点人列表
async function loadAssignees() {
  try {
    const resp: any = await getAdminOptionsApi();
    const data = resp?.data ?? resp;
    const list = Array.isArray(data) ? data : data?.list ?? [];
    assigneeOptions.value = list.map((u: any) => ({
      label: u.nickName ?? u.nickname ?? u.name ?? u.label,
      value: Number(u.id ?? u.value),
    }));
  } catch (e) {
    console.error('[盘点] 加载盘点人失败:', e);
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    const uninputted = items.value.filter(
      (item) =>
        item.actualQuantity === null ||
        item.actualQuantity === undefined ||
        item.actualQuantity === '',
    );
    if (uninputted.length > 0) {
      message.warning(`还有 ${uninputted.length} 个产品未录入实盘数量`);
      return;
    }

    // 检查有差异但未复盘的产品（建议但不强制）
    const diffUnrechecked = items.value.filter((item) => {
      const diff = computeDiff(item);
      return diff !== null && diff !== 0 && (item.recheckQuantity === null || item.recheckQuantity === undefined);
    });
    if (diffUnrechecked.length > 0) {
      // 弹窗确认是否跳过复盘
      const confirmed = await new Promise<boolean>((resolve) => {
        Modal.confirm({
          title: '复盘提示',
          content: `有 ${diffUnrechecked.length} 个存在差异的产品尚未复盘，建议先完成复盘再提交。是否继续提交？`,
          okText: '继续提交',
          cancelText: '返回复盘',
          onOk: () => resolve(true),
          onCancel: () => resolve(false),
        });
      });
      if (!confirmed) return;
    }

    confirmLoading.value = true;
    try {
      const payload = items.value.map((item) => ({
        id: item.id,
        actualQuantity: Number(item.actualQuantity),
        remark: item.remark ?? '',
        assigneeIds: item.assigneeIds ?? [],
        recheckQuantity: item.recheckQuantity ?? null,
        recheckAssigneeIds: item.recheckAssigneeIds ?? [],
        diffReason: item.diffReason ?? '',
        handling: item.handling ?? '',
      }));
      await inputCheckApi(drawerData.value.row.id, payload);
      message.success('录入成功');
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
      drawerData.value = drawerApi.getData<{ row?: any }>() || {};
      items.value = [];
      selectedRowKeys.value = [];
      activeTab.value = 'all';
      loadAssignees();
      if (drawerData.value.row?.id) {
        loadItems(drawerData.value.row.id);
      }
    }
  },
});


async function loadItems(stocktakeId: number) {
  try {
    const resp = await getCheckItemsApi(stocktakeId);
    const data = resp?.data ?? resp;
    const list = Array.isArray(data) ? data : data?.items ?? data?.list ?? [];

    items.value = list.map((item: any) => {
      const skuVal = item.product_sku ?? item.productSku ?? '';
      const actual =
        item.actual_quantity !== null && item.actual_quantity !== undefined
          ? Number(item.actual_quantity)
          : item.actualQuantity !== null && item.actualQuantity !== undefined
            ? Number(item.actualQuantity)
            : null;

      return {
        id: item.id,
        productId: Number(item.product_id ?? item.productId ?? 0),
        skuId: Number(item.sku_id ?? item.skuId ?? 0),
        productName: item.product_name ?? item.productName ?? '',
        productCode: item.product_sku ?? item.productSku ?? item.product_code ?? item.productCode ?? '',
        productSku: '',
        specInfo: '',
        specInfoList: [] as { label: string; value: string }[],
        systemQuantity: Number(
          item.system_quantity ?? item.systemQuantity ?? 0,
        ),
        actualQuantity: actual,
        difference: actual !== null ? actual - Number(item.system_quantity ?? 0) : null,
        diffReason: item.diff_reason ?? item.diffReason ?? '',
        handling: item.handling ?? '',
        remark: item.remark ?? '',
        assigneeIds: item.assignee_ids
          ? (typeof item.assignee_ids === 'string'
              ? JSON.parse(item.assignee_ids)
              : item.assignee_ids)
          : item.assigneeIds ?? [],
        recheckQuantity: item.recheck_quantity
          ? Number(item.recheck_quantity)
          : null,
        recheckAssigneeIds: item.recheck_assignee_ids
          ? (typeof item.recheck_assignee_ids === 'string'
              ? JSON.parse(item.recheck_assignee_ids)
              : item.recheck_assignee_ids)
          : [],
        _isNew: false,
        _checked: actual !== null,
      };
    });

    // 为多规格产品加载规格信息
    await loadSpecInfoForItems();
  } catch (e) {
    console.error('[盘点录入] 加载明细失败:', e);
  }
}

// 为表格中的产品加载规格信息（显示具体规格值如：颜色:红色/尺码:XXL）
async function loadSpecInfoForItems() {
  const uniqueProductIds = [...new Set(items.value.map((i) => i.productId))];
  for (const pid of uniqueProductIds) {
    try {
      const resp: any = await getProductSpecsApi(pid);
      const data = resp?.data ?? resp;
      const skus = data?.skus ?? data?.skuList ?? [];

      if (skus.length === 0) continue;

      let specText = '';
      let specList: { label: string; value: string }[] = [];

      if (skus.length === 1) {
        // 单规格：直接显示该 SKU 的规格值
        const skuSpecs = skus[0]?.specs;
        if (skuSpecs && typeof skuSpecs === 'object' && Object.keys(skuSpecs).length > 0) {
          specList = Object.entries(skuSpecs).map(([k, v]) => ({
            label: k,
            value: String(v),
          }));
          specText = specList.map((s) => `${s.label}:${s.value}`).join(' / ');
        }
      } else {
        // 多规格：显示各维度的可选值汇总
        // 收集所有规格维度
        const specMap = new Map<string, Set<string>>();
        for (const sku of skus) {
          const skuSpecs = sku?.specs;
          if (skuSpecs && typeof skuSpecs === 'object') {
            for (const [key, val] of Object.entries(skuSpecs)) {
              if (!specMap.has(key)) specMap.set(key, new Set());
              specMap.get(key)!.add(String(val));
            }
          }
        }
        if (specMap.size > 0) {
          specList = [...specMap.entries()].map(([key, vals]) => ({
            label: key,
            value: [...vals].join(','),
          }));
          specText = specList.map((s) => `${s.label}:${s.value}`).join(' / ');
        }
      }

      if (specText) {
        items.value.forEach((item) => {
          if (item.productId === pid) {
            item.specInfo = specText;
            item.specInfoList = specList;
          }
        });
      }
    } catch {
      // 忽略单个产品规格加载失败
    }
  }
}

// ============ 计算函数 ============
function computeDiff(item: any): number | null {
  if (
    item.actualQuantity === null ||
    item.actualQuantity === undefined ||
    item.actualQuantity === ''
  ) {
    return null;
  }
  return Number(item.actualQuantity) - Number(item.systemQuantity ?? 0);
}

function getDiffTag(diff: number | null) {
  if (diff === null) return { label: '-', color: 'default' };
  if (diff > 0) return { label: '盘盈', color: 'success' };
  if (diff < 0) return { label: '盘亏', color: 'error' };
  return { label: '一致', color: 'default' };
}

// ============ 添加产品 ============
function openProductSelect() {
  productSelectVisible.value = true;
}

function onProductSelected(selectedItems: any[]) {
  const existingKeys = new Set(items.value.map((i) => `${i.productId}-${i.skuId || 0}`));
  let added = 0;
  for (const item of selectedItems) {
    const key = `${item.productId}-${item.skuId || 0}`;
    if (!existingKeys.has(key)) {
      items.value.push({
        id: Date.now() + added,
        productId: item.productId,
        skuId: item.skuId || 0,
        productName: item.productName,
        productCode: item.productCode ?? '',
        productSku: '',
        systemQuantity: 0,
        actualQuantity: null,
        difference: null,
        diffReason: '',
        handling: '',
        remark: '',
        assigneeIds: [],
        recheckQuantity: null,
        recheckAssigneeIds: [],
        _isNew: true,
        _checked: false,
      });
      added++;
    }
  }
  productSelectVisible.value = false;
  if (added > 0) message.success(`已添加 ${added} 个产品`);
}

// ============ 标记已盘（自动保存到后端） ============
const savingIds = ref<Set<number>>(new Set());

async function markAsChecked(record: any) {
  if (
    record.actualQuantity === null ||
    record.actualQuantity === undefined ||
    record.actualQuantity === ''
  ) {
    message.warning('请先填写实盘数量');
    return;
  }

  // 新增产品尚未入库，无法单独保存
  if (record._isNew) {
    record._checked = true;
    message.info('新增产品请点底部「确定」保存');
    return;
  }

  // 已有 DB 记录，调用后端保存
  savingIds.value.add(record.id);
  try {
    const payload = [
      {
        id: record.id,
        actualQuantity: Number(record.actualQuantity),
        remark: record.remark ?? '',
        assigneeIds: record.assigneeIds ?? [],
        recheckQuantity: record.recheckQuantity ?? null,
        recheckAssigneeIds: record.recheckAssigneeIds ?? [],
        diffReason: record.diffReason ?? '',
        handling: record.handling ?? '',
      },
    ];
    await inputCheckApi(drawerData.value.row.id, payload);
    record._checked = true;
    message.success('已保存并标记为已盘');
  } catch (e) {
    console.error('[盘点录入] 保存失败:', e);
    message.error('保存失败，请重试');
  } finally {
    savingIds.value.delete(record.id);
  }
}

function uncheck(record: any) {
  record._checked = false;
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}

function removeSelected() {
  if (selectedRowKeys.value.length === 0) {
    message.warning('请先选择要删除的产品');
    return;
  }
  items.value = items.value.filter(
    (_, idx) => !selectedRowKeys.value.includes(idx),
  );
  selectedRowKeys.value = [];
}

// ============ 批量分配盘点人 ============
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
    const item = items.value.find((i) => i.id === id);
    if (item) {
      item.assigneeIds = [...batchAssigneeIds.value];
    }
  }
  batchAssignVisible.value = false;
  message.success(`已分配 ${selectedRowKeys.value.length} 个产品的盘点人`);
  selectedRowKeys.value = [];
}

// ============ 批量填0 ============
function batchFillZero() {
  let count = 0;
  items.value.forEach((item) => {
    if (
      item.actualQuantity === null ||
      item.actualQuantity === undefined ||
      item.actualQuantity === ''
    ) {
      item.actualQuantity = 0;
      count++;
    }
  });
  if (count > 0) message.success(`已填充 ${count} 个空白项为 0`);
}

// ============ 筛选 ============
const filteredItems = computed(() => {
  if (activeTab.value === 'mine') {
    return items.value.filter((i) =>
      (i.assigneeIds ?? []).includes(currentUserId),
    );
  }
  if (activeTab.value === 'uninputted') {
    return items.value.filter((i) => !i._checked);
  }
  if (activeTab.value === 'inputted') {
    return items.value.filter((i) => i._checked);
  }
  return items.value;
});

// ============ 统计 ============
const summary = computed(() => {
  let surplus = 0;
  let shortage = 0;
  let match = 0;
  let pending = 0;
  for (const item of items.value) {
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
  // 有差异但未复盘的数量
  const diffUnrechecked = items.value.filter((item) => {
    const diff = computeDiff(item);
    return diff !== null && diff !== 0 && (item.recheckQuantity === null || item.recheckQuantity === undefined);
  }).length;
  return { surplus, shortage, match, pending, diffUnrechecked, total: items.value.length };
});

const inputtedCount = computed(
  () => items.value.filter((i) => i._checked).length,
);

const progressPercent = computed(() => {
  if (items.value.length === 0) return 0;
  return Math.round((inputtedCount.value / items.value.length) * 100);
});

// ============ 表格列 ============
const columns = computed(() => [
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
    title: '规格信息',
    dataIndex: 'specInfo',
    width: 200,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.systemQuantity'),
    dataIndex: 'systemQuantity',
    width: 80,
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
    width: 70,
    align: 'right' as const,
  },
  {
    title: $t('page.product.inventory.check.field.differenceType'),
    dataIndex: 'differenceType',
    width: 80,
    align: 'center' as const,
  },
  {
    title: '盘点人',
    dataIndex: 'assignees',
    width: 130,
  },
  {
    title: '复盘数量',
    dataIndex: 'recheckQuantity',
    width: 100,
  },
  {
    title: '复盘人',
    dataIndex: 'recheckAssignees',
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
    title: '操作',
    dataIndex: 'action',
    width: 120,
    fixed: 'right' as const,
  },
]);

// ============ 盘点人显示 ============
function getAssigneeNames(ids: number[]): string {
  if (!ids || ids.length === 0) return '';
  return ids
    .map((id) => assigneeOptions.value.find((a) => a.value === id)?.label)
    .filter(Boolean)
    .join(', ');
}
const drawerClass = computed(() => [
  'check-input-drawer',
  { 'check-input-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    title="录入实盘数量"
    :confirm-loading="confirmLoading"
    width="85%"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button
          type="button"
          class="input-drawer__fs-btn"
          @click="toggleFullscreen"
        >
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24" width="16" height="16"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round"
          >
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24" width="16" height="16"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round"
          >
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>
    <!-- 顶部信息 -->
    <div class="drawer-topbar">
      <div class="topbar-info">
        <span class="topbar-no">{{ drawerData.row?.stocktakeNo ?? drawerData.row?.checkNo ?? '' }}</span>
        <Tag color="blue">{{ drawerData.row?.warehouseName ?? '' }}</Tag>
        <Tag color="orange">盘点中</Tag>
      </div>
      <div class="topbar-progress">
        <span class="progress-text">{{ inputtedCount }} / {{ items.length }} 项</span>
        <Progress :percent="progressPercent" size="small" style="width: 160px" />
      </div>
    </div>

    <!-- 工具栏 -->
    <div class="drawer-toolbar">
      <Button type="primary" size="small" @click="openProductSelect">
        {{ $t('page.product.inventory.check.drawer.addItem') }}
      </Button>
      <Button size="small" @click="openBatchAssign" :disabled="selectedRowKeys.length === 0">
        批量分配盘点人
        <span v-if="selectedRowKeys.length > 0">({{ selectedRowKeys.length }})</span>
      </Button>
      <Button size="small" @click="batchFillZero">{{ $t('page.product.inventory.check.drawer.batchFillZero') }}</Button>
      <Button size="small" danger :disabled="selectedRowKeys.length === 0" @click="removeSelected">
        {{ $t('page.product.inventory.check.drawer.removeSelected') }}
      </Button>

      <!-- Tab 筛选 -->
      <div class="toolbar-tabs">
        <button
          :class="['tab-btn', { active: activeTab === 'all' }]"
          @click="activeTab = 'all'"
        >全部</button>
        <button
          :class="['tab-btn', { active: activeTab === 'mine' }]"
          @click="activeTab = 'mine'"
        >我的</button>
        <button
          :class="['tab-btn', { active: activeTab === 'uninputted' }]"
          @click="activeTab = 'uninputted'"
        >未盘点</button>
        <button
          :class="['tab-btn', { active: activeTab === 'inputted' }]"
          @click="activeTab = 'inputted'"
        >已盘点</button>
      </div>
    </div>

    <!-- 统计标签 -->
    <div class="drawer-stats">
      <Tag>共 {{ summary.total }}</Tag>
      <Tag v-if="summary.surplus > 0" color="success">盘盈 {{ summary.surplus }}</Tag>
      <Tag v-if="summary.shortage > 0" color="error">盘亏 {{ summary.shortage }}</Tag>
      <Tag v-if="summary.match > 0">一致 {{ summary.match }}</Tag>
      <Tag v-if="summary.pending > 0" color="warning">待盘 {{ summary.pending }}</Tag>
      <Tag v-if="summary.diffUnrechecked > 0" color="orange">待复盘 {{ summary.diffUnrechecked }}</Tag>
    </div>

    <!-- 表格 -->
    <Table
      :columns="columns"
      :data-source="filteredItems"
      :row-key="(record) => record.id"
      :row-class-name="(record) => (record._checked ? 'row-checked' : '')"
      :row-selection="{
        selectedRowKeys,
        onChange: (keys: any) => (selectedRowKeys = keys),
      }"
      :pagination="false"
      size="small"
      :scroll="{ x: 1200, y: 'calc(100vh - 320px)' }"
      bordered
    >
      <template #bodyCell="{ column, record, index }">
        <!-- 产品编码 -->
        <template v-if="column.dataIndex === 'productCode'">
          {{ record.productCode || '-' }}
        </template>

        <!-- 产品名称 -->
        <template v-else-if="column.dataIndex === 'productName'">
          {{ record.productName || '-' }}
        </template>

        <!-- 规格信息 -->
        <template v-else-if="column.dataIndex === 'specInfo'">
          <div v-if="record.specInfo" class="spec-info-cell">
            <span v-for="(spec, idx) in record.specInfoList" :key="idx" class="spec-item">
              <span class="spec-label">{{ spec.label }}:</span>
              <span class="spec-value">{{ spec.value }}</span>
            </span>
          </div>
          <span v-else class="text-gray-400">-</span>
        </template>

        <!-- 系统数量 -->
        <template v-else-if="column.dataIndex === 'systemQuantity'">
          <span class="font-medium">{{ record.systemQuantity ?? 0 }}</span>
        </template>

        <!-- 实盘数量 -->
        <template v-else-if="column.dataIndex === 'actualQuantity'">
          <InputNumber
            v-model:value="record.actualQuantity"
            size="small"
            style="width: 100%"
            :precision="0"
            :step="1"
            placeholder="输入"
          />
        </template>

        <!-- 差异 -->
        <template v-else-if="column.dataIndex === 'difference'">
          <span v-if="computeDiff(record) === null" class="text-gray-400">-</span>
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

        <!-- 差异类型 -->
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
            :placeholder="getAssigneeNames(record.assigneeIds) || '分配'"
            :max-tag-count="1"
            :max-tag-text-length="6"
            allow-clear
          />
        </template>

        <!-- 复盘数量：有差异时可填 -->
        <template v-else-if="column.dataIndex === 'recheckQuantity'">
          <InputNumber
            v-if="computeDiff(record) !== null && computeDiff(record) !== 0"
            v-model:value="record.recheckQuantity"
            size="small"
            style="width: 100%"
            :precision="0"
            :step="1"
            placeholder="复盘"
          />
          <span v-else class="text-gray-400">-</span>
        </template>

        <!-- 复盘人：有差异时可选 -->
        <template v-else-if="column.dataIndex === 'recheckAssignees'">
          <Select
            v-if="computeDiff(record) !== null && computeDiff(record) !== 0"
            v-model:value="record.recheckAssigneeIds"
            mode="multiple"
            size="small"
            style="width: 100%; min-width: 90px"
            :options="assigneeOptions"
            :placeholder="getAssigneeNames(record.recheckAssigneeIds) || '分配'"
            :max-tag-count="1"
            :max-tag-text-length="6"
            allow-clear
          />
          <span v-else class="text-gray-400">-</span>
        </template>

        <!-- 差异原因 -->
        <template v-else-if="column.dataIndex === 'diffReason'">
          <Select
            v-model:value="record.diffReason"
            size="small"
            style="width: 100%"
            placeholder="选择"
            :options="diffReasonOptions"
            allow-clear
            :disabled="computeDiff(record) === 0 || computeDiff(record) === null"
          />
        </template>

        <!-- 处理方式 -->
        <template v-else-if="column.dataIndex === 'handling'">
          <Select
            v-model:value="record.handling"
            size="small"
            style="width: 100%"
            placeholder="选择"
            :options="handlingOptions"
            allow-clear
            :disabled="computeDiff(record) === 0 || computeDiff(record) === null"
          />
        </template>

        <!-- 操作 -->
        <template v-else-if="column.dataIndex === 'action'">
          <div class="action-cell">
            <Button
              v-if="!record._checked"
              type="link"
              size="small"
              style="padding: 0 4px"
              :loading="savingIds.has(record.id)"
              @click="markAsChecked(record)"
            >
              已盘
            </Button>
            <Button
              v-else
              type="link"
              size="small"
              style="padding: 0 4px; color: #999"
              @click="uncheck(record)"
            >
              撤销
            </Button>
            <Button type="link" danger size="small" style="padding: 0 4px" @click="removeItem(index)">
              删除
            </Button>
          </div>
        </template>
      </template>
    </Table>

    <!-- 产品选择弹窗 -->
    <ProductSelectModal
      :visible="productSelectVisible"
      :exclude-ids="excludeProductIds"
      :exclude-sku-keys="excludeSkuKeys"
      :exclude-sku-codes="excludeSkuCodes"
      :warehouse-id="drawerData.row?.warehouseId"
      @update:visible="(val) => (productSelectVisible = val)"
      @select="onProductSelected"
    />

    <!-- 批量分配盘点人弹窗 -->
    <Modal
      v-model:open="batchAssignVisible"
      title="批量分配盘点人"
      width="420px"
      @ok="confirmBatchAssign"
    >
      <p style="margin-bottom: 12px; color: #666; font-size: 13px">
        将为选中的 <strong>{{ selectedRowKeys.length }}</strong> 个产品分配盘点人：
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
.check-input-drawer {
  width: 85vw !important;
}

.check-input-drawer--fullscreen {
  width: 100vw !important;
}

.input-drawer__fs-btn {
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

.input-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.product-info-cell {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.product-info-cell .product-code {
  font-size: 11px;
  color: #86909c;
  line-height: 1.3;
}

.product-info-cell .product-name {
  font-size: 13px;
  color: #1f2329;
  font-weight: 500;
  line-height: 1.3;
}

.spec-info-cell {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.spec-info-cell .spec-item {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 1px 6px;
  background: #f2f3f5;
  border-radius: 4px;
  font-size: 11px;
}

.spec-info-cell .spec-label {
  color: #86909c;
}

.spec-info-cell .spec-value {
  color: #1f2329;
  font-weight: 500;
}

.action-cell {
  display: flex;
  align-items: center;
  gap: 0;
  white-space: nowrap;
}

:deep(.row-checked td) {
  background-color: #f6ffed !important;
}

.drawer-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(135deg, #f0f5ff 0%, #e6f7ff 100%);
  border-radius: 8px;
  margin-bottom: 12px;
}

.topbar-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.topbar-no {
  font-size: 15px;
  font-weight: 600;
  color: #1f2329;
}

.topbar-progress {
  display: flex;
  align-items: center;
  gap: 8px;
}

.topbar-progress .progress-text {
  font-size: 12px;
  white-space: nowrap;
  color: #666;
}

.drawer-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.toolbar-tabs {
  display: flex;
  gap: 4px;
  margin-left: auto;
}

.tab-btn {
  padding: 2px 12px;
  border: 1px solid #e5e6eb;
  border-radius: 6px;
  background: #fff;
  color: #86909c;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.tab-btn:hover {
  border-color: #3370ff;
  color: #3370ff;
}

.tab-btn.active {
  background: #3370ff;
  border-color: #3370ff;
  color: white;
}

.drawer-stats {
  display: flex;
  gap: 4px;
  margin-bottom: 12px;
}
</style>
