<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { Modal, Input, Table, Button, Tag, Empty } from 'ant-design-vue';

import { getProductListApi } from '#/api';
import { getProductSpecsApi } from '#/api';
import { getInventoryListApi } from '#/api/core/product/inventory';

interface SelectedSku {
  productId: number;
  productName: string;
  productCode: string;
  unit: string;
  weight: number;
  skuId?: number;
  skuCode?: string;
  spec?: string;
  unitPrice: number;
  stock?: number;
  imageUrl?: string;
}

const props = defineProps<{
  visible: boolean;
  excludeIds?: number[];
  /** 已添加的 SKU 标识列表（格式 `productId-skuId`，新数据） */
  excludeSkuKeys?: string[];
  /** 已添加的 SKU 编码列表（如 SKU-20-Black-Silicone，兼容历史数据） */
  excludeSkuCodes?: string[];
  /** 仓库ID（用于查询真实库存） */
  warehouseId?: number;
}>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', items: SelectedSku[]): void;
}>();

// computed: 完全响应式，props 变化时自动更新
const excludeIdSet = computed(
  () => new Set((props.excludeIds ?? []).map((id) => Number(id))),
);
const excludeSkuKeySet = computed(
  () => new Set(props.excludeSkuKeys ?? []),
);
const excludeSkuCodeSet = computed(
  () => new Set(props.excludeSkuCodes ?? []),
);

// 判断单规格产品是否已添加（多规格产品不整体排除）
function isProductAdded(record: any): boolean {
  if (isMultiSpec(record)) return false;
  return excludeIdSet.value.has(Number(record.id));
}

// 判断某个 SKU 是否已被添加过（支持 skuId 和 skuCode 两种匹配方式）
function isSkuAdded(productId: number, sku: any): boolean {
  // 1. 新数据：按 productId-skuId 匹配
  const key = `${Number(productId)}-${Number(sku.id)}`;
  if (excludeSkuKeySet.value.has(key)) return true;
  // 2. 历史数据：按 skuCode 匹配
  const code = sku.skuCode || sku.sku_code;
  if (code && excludeSkuCodeSet.value.has(code)) return true;
  return false;
}

const keyword = ref('');
const loading = ref(false);
const productList = ref<any[]>([]);
const pagination = ref({ current: 1, pageSize: 10, total: 0 });
const expandedRowKeys = ref<number[]>([]);
const skuMap = ref<Record<number, any[]>>({});
const skuLoadingMap = ref<Record<number, boolean>>({});
const selectedSkus = ref<SelectedSku[]>([]);

watch(
  () => props.visible,
  (val) => {
    if (val) {
      selectedSkus.value = [];
      keyword.value = '';
      pagination.value.current = 1;
      loadProducts();
    }
  },
);

async function loadProducts() {
  loading.value = true;
  try {
    const res = await getProductListApi({
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: keyword.value,
    });
    const list = res?.list || res?.items || res || [];
    productList.value = list.map((p: any) => ({
      ...p,
      key: p.id,
      specInfo: '', // 初始为空，异步加载后更新
    }));
    pagination.value.total = res?.total || list.length;

    // 异步加载规格信息，直接更新 productList 触发响应式
    loadProductSpecs();
    // 异步加载仓库真实库存
    loadWarehouseStock();
  } finally {
    loading.value = false;
  }
}

// 批量加载仓库真实库存，更新 productList 中的 stock 字段
async function loadWarehouseStock() {
  if (!props.warehouseId) return;
  const productIds = productList.value.map((p) => p.id);
  if (productIds.length === 0) return;
  try {
    const res = await getInventoryListApi({
      warehouseId: props.warehouseId,
      pageSize: 200,
    });
    const list = res?.list || res?.items || res || [];
    // 建立 productId → quantity 映射
    const stockMap = new Map<number, number>();
    for (const item of list) {
      const pid = Number(item.productId ?? item.product_id ?? 0);
      const qty = Number(item.quantity ?? 0);
      if (pid > 0) stockMap.set(pid, qty);
    }
    // 更新产品列表中的 stock
    productList.value = productList.value.map((p) => ({
      ...p,
      stock: stockMap.get(Number(p.id)) ?? p.stock ?? 0,
    }));
  } catch {
    // 库存查询失败，保持原有 stock 值
  }
}

function onSearch() {
  pagination.value.current = 1;
  loadProducts();
}

// 异步加载产品列表中每个产品的规格摘要，直接写入 productList 触发更新
async function loadProductSpecs() {
  const products = productList.value;
  // 并行请求所有产品的规格
  await Promise.allSettled(
    products.map(async (product: any, i: number) => {
      try {
        const res: any = await getProductSpecsApi(product.id);
        const data = res?.data ?? res;
        const skus = data?.skus ?? data?.skuList ?? [];
        let specText = '';

        if (skus.length === 1) {
          // 单规格：优先从 specs 对象提取 key:value
          const specs = parseSpecsObj(skus[0]?.specs);
          if (specs) {
            specText = Object.entries(specs)
              .map(([k, v]) => `${k}:${v}`)
              .join(' / ');
          } else {
            specText = '';
          }
        } else if (skus.length > 1) {
          // 多规格：汇总各维度可选值
          const specMap = new Map<string, Set<string>>();
          for (const sku of skus) {
            const specs = parseSpecsObj(sku?.specs);
            if (specs) {
              for (const [key, val] of Object.entries(specs)) {
                if (!specMap.has(key)) specMap.set(key, new Set());
                specMap.get(key)!.add(String(val));
              }
            }
          }
          specText = [...specMap.entries()]
            .map(([key, vals]) => `${key}:${[...vals].join(',')}`)
            .join(' / ');
        }

        product.specInfo = specText;

        // 多规格产品：库存显示所有 SKU 库存累加值
        if (skus.length > 0) {
          const totalStock = skus.reduce(
            (sum: number, s: any) => sum + Number(s.stock ?? 0),
            0,
          );
          product.stock = totalStock;
        }

        // 强制替换数组引用，确保 Table 组件重新渲染
        productList.value = [...productList.value];
      } catch (e) {
        console.error('[ProductSelectModal] 加载规格失败:', product.id, e);
      }
    }),
  );
}

function onPageChange(page: number, pageSize: number) {
  pagination.value.current = page;
  pagination.value.pageSize = pageSize;
  loadProducts();
}

async function onExpand(expanded: boolean, record: any) {
  if (expanded) {
    expandedRowKeys.value = [...expandedRowKeys.value, record.id];
    if (!skuMap.value[record.id]) {
      skuLoadingMap.value[record.id] = true;
      try {
        const res = await getProductSpecsApi(record.id);
        const data = res?.data || res;
        skuMap.value[record.id] = data?.skus || data || [];
      } catch {
        skuMap.value[record.id] = [];
      } finally {
        skuLoadingMap.value[record.id] = false;
      }
    }
  } else {
    expandedRowKeys.value = expandedRowKeys.value.filter((k) => k !== record.id);
  }
}

function isMultiSpec(product: any) {
  return product.specType === 'multiple';
}

function selectSingleProduct(product: any) {
  const item: SelectedSku = {
    productId: Number(product.id),
    productName: product.name || product.productName,
    productCode: product.productNo || product.productCode || product.sku || '',
    unit: product.unit || '',
    weight: Number(product.weight || 0),
    unitPrice: Number(product.salePrice || product.marketPrice || 0),
    stock: product.stock,
    imageUrl: product.imageUrl || product.coverImage,
  };

  const existing = selectedSkus.value.findIndex(
    (s) => s.productId === item.productId && !s.skuId,
  );
  if (existing >= 0) {
    selectedSkus.value.splice(existing, 1);
  } else {
    selectedSkus.value.push(item);
  }
}

function selectSku(product: any, sku: any) {
  // 优先用 specs 生成带规格名的文本（如 颜色:红色），label 作为兜底
  const specsObj = parseSpecsObj(sku.specs);
  let specText = '';
  if (specsObj) {
    specText = Object.entries(specsObj)
      .map(([k, v]) => `${k}:${v}`)
      .join(' / ');
  } else {
    specText = sku.label || '';
  }

  const item: SelectedSku = {
    productId: Number(product.id),
    productName: product.name || product.productName,
    productCode: product.productNo || product.productCode || '',
    unit: product.unit || '',
    weight: Number(sku.weight || product.weight || 0),
    skuId: Number(sku.id),
    skuCode: sku.skuCode || '',
    spec: specText,
    unitPrice: Number(sku.price || product.salePrice || 0),
    stock: sku.stock,
    imageUrl: sku.imageUrl || product.imageUrl || product.coverImage,
  };

  const existing = selectedSkus.value.findIndex((s) => s.skuId === item.skuId);
  if (existing >= 0) {
    selectedSkus.value.splice(existing, 1);
  } else {
    selectedSkus.value.push(item);
  }
}

function isSkuSelected(productId: number, skuId?: number) {
  const pid = Number(productId);
  if (skuId) {
    const sid = Number(skuId);
    return selectedSkus.value.some((s) => s.skuId === sid);
  }
  return selectedSkus.value.some((s) => s.productId === pid && !s.skuId);
}

function removeSelected(index: number) {
  selectedSkus.value.splice(index, 1);
}

function handleConfirm() {
  if (selectedSkus.value.length === 0) return;
  emit('select', [...selectedSkus.value]);
  emit('update:visible', false);
}

function handleCancel() {
  emit('update:visible', false);
}

const productColumns = [
  { title: '产品名称', dataIndex: 'name', key: 'name', width: 180 },
  { title: '编码', dataIndex: 'productNo', key: 'productNo', width: 110 },
  { title: '规格信息', key: 'specInfo', width: 200 },
  { title: '单价', key: 'price', width: 90 },
  { title: '库存', dataIndex: 'stock', key: 'stock', width: 70 },
  { title: '单位', dataIndex: 'unit', key: 'unit', width: 55 },
  { title: '操作', key: 'action', width: 80 },
];

const skuColumns = [
  { title: '规格', key: 'specText', width: 280 },
  { title: '单价', dataIndex: 'price', key: 'price', width: 100 },
  { title: '库存', dataIndex: 'stock', key: 'stock', width: 80 },
  { title: '操作', key: 'action', width: 80 },
];

// 把 specs（可能是 JSON 字符串或对象）统一解析为对象
function parseSpecsObj(specs: any): Record<string, any> | null {
  if (!specs) return null;
  if (typeof specs === 'string') {
    try {
      const parsed = JSON.parse(specs);
      if (typeof parsed === 'object' && parsed !== null) return parsed;
    } catch {
      return null;
    }
    return null;
  }
  if (typeof specs === 'object' && specs !== null) return specs;
  return null;
}

function formatSpecs(specs: any): string {
  if (!specs) return '-';
  let obj: Record<string, any>;
  if (typeof specs === 'string') {
    try {
      obj = JSON.parse(specs);
    } catch {
      return specs;
    }
  } else {
    obj = specs;
  }
  if (typeof obj !== 'object' || obj === null) return String(obj);
  const entries = Object.entries(obj);
  if (entries.length === 0) return '-';
  return entries.map(([k, v]) => `${k}: ${v}`).join('  /  ');
}
</script>

<template>
  <Modal
    :open="props.visible"
    title="选择产品"
    :width="900"
    :destroy-on-close="true"
    @ok="handleConfirm"
    @cancel="handleCancel"
  >
    <!-- 搜索栏 -->
    <div class="mb-4 flex gap-2">
      <Input
        v-model:value="keyword"
        placeholder="搜索产品名称/编码"
        allow-clear
        style="width: 300px"
        @press-enter="onSearch"
      />
      <Button type="primary" @click="onSearch">搜索</Button>
    </div>

    <!-- 产品列表 -->
    <Table
      :columns="productColumns"
      :data-source="productList"
      :loading="loading"
      :pagination="{
        current: pagination.current,
        pageSize: pagination.pageSize,
        total: pagination.total,
        showSizeChanger: true,
        showTotal: (t: number) => `共 ${t} 条`,
        onChange: onPageChange,
      }"
      :expanded-row-keys="expandedRowKeys"
      :row-expandable="(record: any) => isMultiSpec(record)"
      size="small"
      row-key="id"
      :scroll="{ y: 400 }"
      @expand="onExpand"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'specInfo'">
          <span v-if="record.specInfo" class="text-xs">{{ record.specInfo }}</span>
          <span v-else-if="isMultiSpec(record)">
            <Tag color="blue" style="font-size: 11px">多规格</Tag>
          </span>
          <span v-else class="text-gray-400">-</span>
        </template>
        <template v-else-if="column.key === 'price'">
          ¥{{ Number(record.salePrice || record.marketPrice || 0).toFixed(2) }}
        </template>
        <template v-else-if="column.key === 'action'">
          <!-- 单规格产品已添加：禁用 -->
          <Button
            v-if="isProductAdded(record)"
            type="primary"
            size="small"
            ghost
            disabled
          >
            已添加
          </Button>
          <!-- 单规格产品 -->
          <Button
            v-else-if="!isMultiSpec(record)"
            type="primary"
            size="small"
            :ghost="isSkuSelected(record.id)"
            @click="selectSingleProduct(record)"
          >
            {{ isSkuSelected(record.id) ? '已选' : '选择' }}
          </Button>
          <!-- 多规格产品：展开/收起 SKU 列表 -->
          <Button
            v-else
            type="link"
            size="small"
            @click="onExpand(!expandedRowKeys.includes(record.id), record)"
          >
            {{ expandedRowKeys.includes(record.id) ? '收起' : '展开' }}
          </Button>
        </template>
      </template>

      <!-- 展开行：SKU列表 -->
      <template #expandedRowRender="{ record }">
        <div v-if="skuLoadingMap[record.id]" class="py-4 text-center text-gray-400">
          加载中...
        </div>
        <div v-else-if="!skuMap[record.id] || skuMap[record.id]?.length === 0">
          <Empty description="暂无SKU数据" :image="Empty.PRESENTED_IMAGE_SIMPLE" />
        </div>
        <Table
          v-else
          :columns="skuColumns"
          :data-source="skuMap[record.id]"
          :pagination="false"
          size="small"
          row-key="id"
        >
          <template #bodyCell="{ column, record: sku }">
            <template v-if="column.key === 'specText'">
              <span v-if="parseSpecsObj(sku.specs)" class="font-medium">{{ formatSpecs(sku.specs) }}</span>
              <span v-else-if="sku.label" class="font-medium">{{ sku.label }}</span>
              <span v-else class="text-gray-400">无规格</span>
            </template>
            <template v-else-if="column.key === 'price'">
              ¥{{ Number(sku.price || 0).toFixed(2) }}
            </template>
            <template v-else-if="column.key === 'action'">
              <Button
                v-if="isSkuAdded(record.id, sku)"
                type="primary"
                size="small"
                ghost
                disabled
              >
                已添加
              </Button>
              <Button
                v-else
                type="primary"
                size="small"
                :ghost="isSkuSelected(record.id, sku.id)"
                @click="selectSku(record, sku)"
              >
                {{ isSkuSelected(record.id, sku.id) ? '已选' : '选择' }}
              </Button>
            </template>
          </template>
        </Table>
      </template>
    </Table>

    <!-- 已选列表 -->
    <div v-if="selectedSkus.length > 0" class="mt-4 border-t pt-3">
      <div class="mb-2 text-sm font-medium">已选产品（{{ selectedSkus.length }}）</div>
      <div class="space-y-2">
        <div
          v-for="(item, idx) in selectedSkus"
          :key="idx"
          class="flex items-center justify-between rounded bg-blue-50 px-3 py-2 text-sm"
        >
          <div class="flex items-center gap-2">
            <span class="font-medium">{{ item.productName }}</span>
            <Tag v-if="item.spec" color="blue">{{ item.spec }}</Tag>
            <span class="text-gray-400">¥{{ item.unitPrice.toFixed(2) }}/{{ item.unit || '个' }}</span>
            <Tag v-if="item.stock !== undefined && item.stock !== null" color="orange">库存: {{ item.stock }}</Tag>
          </div>
          <Button type="link" danger size="small" @click="removeSelected(idx)">移除</Button>
        </div>
      </div>
    </div>
  </Modal>
</template>
