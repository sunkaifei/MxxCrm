<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { Button, Input, Modal, Table, Tag } from 'ant-design-vue';

import { getProductListApi, getProductSpecsApi } from '#/api';
import { $t } from '#/locales';

interface SelectedProduct {
  id: number;
  name: string;
  specType: string;
}

const props = withDefaults(
  defineProps<{
    /** 已存在规则的产品 ID（行内标注"已有规则"） */
    addedProductIds?: number[];
    /** 已存在规则的 SKU 标识（格式 `productId-skuId`，行内标注"已添加"） */
    addedSkuKeys?: string[];
    visible: boolean;
  }>(),
  {
    addedProductIds: () => [],
    addedSkuKeys: () => [],
  },
);

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', product: SelectedProduct): void;
}>();

const addedProductIdSet = computed(
  () => new Set((props.addedProductIds ?? []).map(Number)),
);
const addedSkuKeySet = computed(() => new Set(props.addedSkuKeys ?? []));

function isProductAdded(record: any): boolean {
  return addedProductIdSet.value.has(Number(record.id));
}

function isSkuAdded(productId: number, sku: any): boolean {
  return addedSkuKeySet.value.has(`${Number(productId)}-${Number(sku.id)}`);
}

function isMultiSpec(product: any) {
  return product.specType === 'multiple';
}

const innerVisible = ref(false);

watch(
  () => props.visible,
  (val) => {
    innerVisible.value = val;
    if (val) {
      keyword.value = '';
      pagination.value.current = 1;
      expandedRowKeys.value = [];
      skuMap.value = {};
      loadProducts();
    }
  },
);

function closeModal() {
  innerVisible.value = false;
  emit('update:visible', false);
}

const keyword = ref('');
const loading = ref(false);
const productList = ref<any[]>([]);
const pagination = ref({ current: 1, pageSize: 10, total: 0 });
const expandedRowKeys = ref<number[]>([]);
const skuMap = ref<Record<number, any[]>>({});
const skuLoadingMap = ref<Record<number, boolean>>({});

const productColumns = [
  {
    dataIndex: 'name',
    ellipsis: true,
    key: 'name',
    title: $t('page.product.inventory.alert.productSelect.colProduct'),
  },
  {
    ellipsis: true,
    key: 'code',
    title: $t('page.product.inventory.alert.productSelect.colCode'),
    width: 140,
  },
  {
    dataIndex: 'specType',
    key: 'specType',
    title: $t('page.product.inventory.alert.productSelect.colSpec'),
    width: 100,
  },
  {
    dataIndex: 'action',
    key: 'action',
    title: $t('page.product.inventory.alert.productSelect.colAction'),
    width: 80,
  },
];

const skuColumns = [
  {
    key: 'specText',
    title: $t('page.product.inventory.alert.productSelect.specCol'),
  },
  {
    key: 'ruleStatus',
    title: $t('page.product.inventory.alert.productSelect.ruleStatus'),
    width: 100,
  },
];

async function loadProducts() {
  loading.value = true;
  try {
    const res: any = await getProductListApi({
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: keyword.value,
    });
    const list = res?.list || res?.items || res || [];
    productList.value = list.map((p: any) => ({ ...p, key: p.id }));
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
        skuMap.value[record.id] = data?.skus || data?.skuList || data || [];
      } catch {
        skuMap.value[record.id] = [];
      } finally {
        skuLoadingMap.value[record.id] = false;
      }
    }
  } else {
    expandedRowKeys.value = expandedRowKeys.value.filter(
      (k) => k !== record.id,
    );
  }
}

function parseSpecsObj(specs: any): null | Record<string, any> {
  if (!specs) return null;
  if (typeof specs === 'string') {
    try {
      const parsed = JSON.parse(specs);
      return typeof parsed === 'object' && parsed !== null ? parsed : null;
    } catch {
      return null;
    }
  }
  return typeof specs === 'object' && specs !== null ? specs : null;
}

function formatSpecs(specs: any): string {
  const obj = parseSpecsObj(specs);
  if (!obj) return '';
  const entries = Object.entries(obj);
  return entries.length === 0
    ? ''
    : entries.map(([k, v]) => `${k}:${v}`).join(' / ');
}

// 点击"选择"直接选中产品并关闭（规格在抽屉内继续配置）
function onSelectClick(record: any) {
  closeModal();
  emit('select', {
    id: Number(record.id),
    name: String(record.name ?? record.productName ?? ''),
    specType: String(record.specType ?? 'single'),
  });
}
</script>

<template>
  <Modal
    v-model:open="innerVisible"
    :title="$t('page.product.inventory.alert.productSelect.title')"
    width="760px"
    :footer="null"
    @cancel="closeModal"
  >
    <div class="mb-3 flex gap-2">
      <Input
        v-model:value="keyword"
        :placeholder="
          $t('page.product.inventory.alert.productSelect.searchPlaceholder')
        "
        allow-clear
        style="width: 280px"
        @press-enter="onSearch"
      />
      <Button type="primary" @click="onSearch">
        {{ $t('page.product.inventory.alert.productSelect.searchButton') }}
      </Button>
    </div>

    <Table
      :columns="productColumns"
      :data-source="productList"
      :loading="loading"
      :row-key="(record) => record.id"
      :pagination="{
        current: pagination.current,
        pageSize: pagination.pageSize,
        total: pagination.total,
        showSizeChanger: true,
        showTotal: (t: number) =>
          $t('page.product.inventory.alert.productSelect.total', { total: t }),
        onChange: onPageChange,
      }"
      :expanded-row-keys="expandedRowKeys"
      :row-expandable="(record: any) => isMultiSpec(record)"
      size="small"
      :scroll="{ y: 400 }"
      @expand="onExpand"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'code'">
          {{ record.productNo || record.productCode || record.code || '-' }}
        </template>
        <template v-else-if="column.key === 'name'">
          <span>{{ record.name || record.productName }}</span>
          <Tag v-if="isProductAdded(record)" color="warning" class="ml-1">
            {{ $t('page.product.inventory.alert.productSelect.hasRule') }}
          </Tag>
        </template>
        <template v-else-if="column.key === 'specType'">
          <Tag v-if="isMultiSpec(record)" color="blue">
            {{ $t('page.product.inventory.alert.productSelect.multiSpec') }}
          </Tag>
          <span v-else class="text-neutral-400">-</span>
        </template>
        <template v-else-if="column.key === 'action'">
          <Button type="link" size="small" @click="onSelectClick(record)">
            {{ $t('page.product.inventory.alert.productSelect.select') }}
          </Button>
        </template>
      </template>

      <template #expandedRowRender="{ record }">
        <div
          v-if="skuLoadingMap[record.id]"
          class="py-3 text-center text-neutral-400"
        >
          {{ $t('page.product.inventory.alert.productSelect.loading') }}
        </div>
        <div
          v-else-if="!skuMap[record.id] || skuMap[record.id]?.length === 0"
          class="py-3 text-center text-neutral-400"
        >
          {{ $t('page.product.inventory.alert.productSelect.noSku') }}
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
              <span v-if="formatSpecs(sku.specs)">
                {{ formatSpecs(sku.specs) }}
              </span>
              <span v-else-if="sku.label">{{ sku.label }}</span>
              <span v-else class="text-neutral-400">
                {{ sku.skuCode || sku.sku_code || `SKU#${sku.id}` }}
              </span>
            </template>
            <template v-else-if="column.key === 'ruleStatus'">
              <Tag v-if="isSkuAdded(record.id, sku)" color="warning">
                {{ $t('page.product.inventory.alert.productSelect.skuAdded') }}
              </Tag>
              <span v-else class="text-neutral-400">-</span>
            </template>
          </template>
        </Table>
        <Empty v-if="false" />
      </template>
    </Table>
  </Modal>
</template>
