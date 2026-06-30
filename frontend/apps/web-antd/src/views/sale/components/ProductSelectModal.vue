<script lang="ts" setup>
import { ref, watch } from 'vue';

import { Modal, Input, Table, Button, Tag, Empty } from 'ant-design-vue';

import { getProductListApi } from '#/api';
import { getProductSpecsApi } from '#/api';

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

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', items: SelectedSku[]): void;
}>();

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
    }));
    pagination.value.total = res?.total || list.length;
  } finally {
    loading.value = false;
  }
}

function onSearch() {
  pagination.value.current = 1;
  loadProducts();
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
    imageUrl: product.coverImage,
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
  const specText = sku.label || formatSpecs(sku.specs) || '';

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
    imageUrl: sku.imageUrl || product.coverImage,
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
  { title: '产品名称', dataIndex: 'name', key: 'name', width: 200 },
  { title: '编码', dataIndex: 'productNo', key: 'productNo', width: 120 },
  { title: '单价', key: 'price', width: 100 },
  { title: '库存', dataIndex: 'stock', key: 'stock', width: 80 },
  { title: '单位', dataIndex: 'unit', key: 'unit', width: 60 },
  { title: '操作', key: 'action', width: 80 },
];

const skuColumns = [
  { title: '规格', key: 'specText', width: 280 },
  { title: '单价', dataIndex: 'price', key: 'price', width: 100 },
  { title: '库存', dataIndex: 'stock', key: 'stock', width: 80 },
  { title: '操作', key: 'action', width: 80 },
];

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
      size="small"
      row-key="id"
      :scroll="{ y: 400 }"
      @expand="onExpand"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'price'">
          ¥{{ Number(record.salePrice || record.marketPrice || 0).toFixed(2) }}
        </template>
        <template v-else-if="column.key === 'action'">
          <Button
            v-if="!isMultiSpec(record)"
            type="primary"
            size="small"
            :ghost="isSkuSelected(record.id)"
            @click="selectSingleProduct(record)"
          >
            {{ isSkuSelected(record.id) ? '已选' : '选择' }}
          </Button>
          <Button
            v-else
            type="link"
            size="small"
            @click="onExpand(!expandedRowKeys.includes(record.id), record)"
          >
            {{ expandedRowKeys.includes(record.id) ? '收起' : '选规格' }}
          </Button>
        </template>
      </template>

      <!-- 展开行：SKU列表 -->
      <template #expandedRowRender="{ record }">
        <div v-if="skuLoadingMap[record.id]" class="py-4 text-center text-gray-400">
          加载中...
        </div>
        <div v-else-if="!skuMap[record.id] || skuMap[record.id].length === 0">
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
              <span v-if="sku.label" class="font-medium">{{ sku.label }}</span>
              <span v-else-if="sku.specs" class="font-medium">{{ formatSpecs(sku.specs) }}</span>
              <span v-else class="text-gray-400">无规格</span>
            </template>
            <template v-else-if="column.key === 'price'">
              ¥{{ Number(sku.price || 0).toFixed(2) }}
            </template>
            <template v-else-if="column.key === 'action'">
              <Button
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
          </div>
          <Button type="link" danger size="small" @click="removeSelected(idx)">移除</Button>
        </div>
      </div>
    </div>
  </Modal>
</template>
