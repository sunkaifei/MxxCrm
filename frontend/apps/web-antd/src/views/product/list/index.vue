<script lang="ts" setup>
import { computed, h, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideImageOff } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Popconfirm, Spin, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  deleteProductApi,
  getBrandListApi,
  getCategoryListApi,
  getProductListApi,
  getProductSpecsApi,
  getWarehouseListApi,
} from '#/api';
import { $t } from '#/locales';

import ProductDrawer from './drawer.vue';

const accessStore = useAccessStore();
const router = useRouter();

const warehouseOptions = ref<{ label: string; value: number }[]>([]);
const categoryOptions = ref<{ label: string; value: number }[]>([]);
const brandOptions = ref<{ label: string; value: number }[]>([]);

/** 当前选中的仓库，用于在表格标题中展示仓库名称 */
const selectedWarehouseId = ref<number | undefined>();

/** 多规格产品的 SKU 数据缓存（按产品 ID 索引） */
const skuMap = ref<Record<number, any[]>>({});

/** 解析 SKU 的 specs 字段为键值对 */
function parseSpecs(specs: any): Record<string, string> {
  if (!specs) return {};
  if (typeof specs === 'string') {
    try {
      return JSON.parse(specs);
    } catch {
      return {};
    }
  }
  return specs;
}

/** 加载仓库下拉选项 */
async function loadWarehouses() {
  try {
    const res: any = await getWarehouseListApi({ pageSize: 200 });
    const list = res?.list || res?.items || res || [];
    warehouseOptions.value = list.map((w: any) => ({
      label: w.name || w.warehouseName,
      value: Number(w.id),
    }));
  } catch {
    warehouseOptions.value = [];
  }
}

/** 加载分类下拉选项 */
async function loadCategories() {
  try {
    const res: any = await getCategoryListApi({ pageSize: 200 });
    const list = res?.list || res?.items || res || [];
    categoryOptions.value = list.map((c: any) => ({
      label: c.name || c.categoryName,
      value: Number(c.id),
    }));
  } catch {
    categoryOptions.value = [];
  }
}

/** 加载品牌下拉选项 */
async function loadBrands() {
  try {
    const res: any = await getBrandListApi({ pageSize: 200 });
    const list = res?.list || res?.items || res || [];
    brandOptions.value = list.map((b: any) => ({
      label: b.name || b.brandName,
      value: Number(b.id),
    }));
  } catch {
    brandOptions.value = [];
  }
}

/** 加载某个多规格产品的 SKU 列表（带缓存） */
async function loadSkuData(productId: number) {
  if (skuMap.value[productId]) return;
  try {
    const res: any = await getProductSpecsApi(productId);
    const data = res?.data ?? res;
    skuMap.value[productId] = data?.skus ?? data?.skuList ?? [];
  } catch {
    skuMap.value[productId] = [];
  }
}

/** 行展开/收起事件：展开多规格产品时按需加载 SKU 数据 */
function onToggleExpand({
  expanded,
  row,
}: {
  expanded: boolean;
  row: any;
}) {
  if (expanded && row?.specType === 'multiple') {
    loadSkuData(Number(row.id));
  }
}

/** 跳转到库存管理页 */
function goToInventory(row: any) {
  router.push(`/product/inventory?productId=${row.id}`);
}

// 商品类型映射
const productTypeNames: Record<number, string> = {
  1: '实物', 2: '虚拟', 3: '服务', 4: '订阅',
};
const productTypeColors: Record<number, string> = {
  1: 'blue', 2: 'purple', 3: 'orange', 4: 'green',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '产品名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'sku',
      label: 'SKU',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'warehouseId',
      label: '仓库',
      componentProps: {
        placeholder: '全部仓库',
        allowClear: true,
        showSearch: true,
        options: [],
        onChange: (val: any) => {
          selectedWarehouseId.value = val;
        },
      },
    },
    {
      component: 'Select',
      fieldName: 'categoryId',
      label: '分类',
      componentProps: {
        placeholder: '全部分类',
        allowClear: true,
        showSearch: true,
        options: [],
      },
    },
    {
      component: 'Select',
      fieldName: 'brandId',
      label: '品牌',
      componentProps: {
        placeholder: '全部品牌',
        allowClear: true,
        showSearch: true,
        options: [],
      },
    },
    {
      component: 'Select',
      fieldName: 'isActive',
      label: '状态',
      componentProps: {
        placeholder: '全部状态',
        allowClear: true,
        options: [
          { label: '启用', value: true },
          { label: '停用', value: false },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'productType',
      label: '商品类型',
      componentProps: {
        placeholder: '全部类型',
        allowClear: true,
        options: [
          { label: '实物商品', value: 1 },
          { label: '虚拟商品', value: 2 },
          { label: '服务商品', value: 3 },
          { label: '订阅商品', value: 4 },
        ],
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
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  expandConfig: {
    visibleMethod: ({ row }: any) => row?.specType === 'multiple',
  } as any,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getProductListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.name,
          sku: formValues.sku,
          warehouseId: formValues.warehouseId || undefined,
          categoryId: formValues.categoryId || undefined,
          brandId: formValues.brandId || undefined,
          isActive: formValues.isActive,
          productType: formValues.productType || undefined,
        });
      },
    },
  },

  columns: [
    {
      title: '',
      type: 'expand',
      width: 50,
      slots: { content: 'expandContent' },
    },
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '商品图',
      field: 'imageUrl',
      width: 70,
      slots: { default: 'productImage' },
    },
    {
      title: '产品名称',
      field: 'name',
      minWidth: 160,
      align: 'left',
    },
    {
      title: '产品编号',
      field: 'productNo',
      width: 140,
    },
    {
      title: '品牌',
      field: 'brandName',
      width: 120,
    },
    {
      title: '规格',
      field: 'specType',
      width: 90,
      slots: { default: 'specType' },
    },
    {
      title: '单位',
      field: 'unit',
      width: 80,
    },
    {
      title: '销售价',
      field: 'salePrice',
      width: 100,
    },
    {
      title: '库存',
      field: 'totalStock',
      width: 120,
      slots: { default: 'totalStock' },
    },
    {
      title: '类型',
      field: 'productType',
      slots: { default: 'productType' },
      width: 80,
    },
    {
      title: $t('ui.table.status'),
      field: 'isActive',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 140,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({
  gridOptions,
  formOptions,
  gridEvents: {
    toggleRowExpand: onToggleExpand,
  },
});

const tableTitle = computed(() => {
  const base = $t('page.product.list.title');
  if (!selectedWarehouseId.value) return base;
  const name = warehouseOptions.value.find(
    (w) => w.value === selectedWarehouseId.value,
  )?.label;
  return name ? `${base} — ${name}` : base;
});

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ProductDrawer,
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

function handleSkuManage(row: any) {
  router.push(`/product/sku?productId=${row.id}`);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteProductApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleCreate() {
  openDrawer(true);
}

onMounted(async () => {
  await Promise.all([loadWarehouses(), loadCategories(), loadBrands()]);
  gridApi.formApi?.updateSchema?.([
    {
      fieldName: 'warehouseId',
      componentProps: { options: warehouseOptions.value },
    },
    {
      fieldName: 'categoryId',
      componentProps: { options: categoryOptions.value },
    },
    {
      fieldName: 'brandId',
      componentProps: { options: brandOptions.value },
    },
  ]);
});
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="tableTitle">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:product:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.list.button.create') }}
        </Button>
      </template>

      <template #productImage="{ row }">
        <div v-if="row.imageUrl || row.coverImage" class="w-10 h-10 rounded-lg border border-gray-200 overflow-hidden flex-shrink-0">
          <img :src="row.imageUrl || row.coverImage" alt="产品主图" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-10 h-10 rounded-lg border border-gray-200 flex-shrink-0 flex items-center justify-center bg-gray-50">
          <LucideImageOff class="w-5 h-5 text-gray-400" />
        </div>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #status="{ row }">
        <Tag :color="row.isActive ? 'green' : 'red'">{{ row.isActive ? '启用' : '停用' }}</Tag>
      </template>

      <template #productType="{ row }">
        <Tag :color="productTypeColors[row.productType] || 'default'">
          {{ productTypeNames[row.productType] || '实物' }}
        </Tag>
      </template>

      <template #specType="{ row }">
        <Tag v-if="row.specType === 'multiple'" color="purple">多规格</Tag>
        <Tag v-else color="default">单规格</Tag>
      </template>

      <template #totalStock="{ row }">
        <div class="flex items-center gap-1">
          <span
            class="cursor-pointer hover:text-blue-500"
            :class="{
              'text-red-500 font-medium': row.totalStock === 0 || row.totalStock == null,
              'text-orange-500': row.totalStock > 0 && row.safetyStock && row.totalStock <= row.safetyStock,
              'text-green-600': row.totalStock > 0 && (!row.safetyStock || row.totalStock > row.safetyStock),
            }"
            @click="goToInventory(row)"
          >
            {{ row.totalStock ?? 0 }}
          </span>
          <Tag v-if="row.totalStock === 0 || row.totalStock == null" color="red" :bordered="false" style="font-size: 10px">缺货</Tag>
          <Tag v-else-if="row.safetyStock && row.totalStock <= row.safetyStock" color="orange" :bordered="false" style="font-size: 10px">不足</Tag>
          <Tag v-else color="green" :bordered="false" style="font-size: 10px">正常</Tag>
        </div>
      </template>

      <template #expandContent="{ row }">
        <div v-if="row.specType === 'multiple'" class="bg-gray-50 p-3">
          <table v-if="skuMap[row.id] && skuMap[row.id].length > 0" class="w-full text-sm">
            <thead>
              <tr class="text-gray-500 text-xs border-b">
                <th class="text-left py-2 px-3">SKU编码</th>
                <th class="text-left py-2 px-3">规格</th>
                <th class="text-right py-2 px-3">价格</th>
                <th class="text-right py-2 px-3">库存</th>
                <th class="text-center py-2 px-3">状态</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="sku in skuMap[row.id]"
                :key="sku.id"
                class="border-b border-gray-100"
                :class="{ 'opacity-50': !sku.isActive }"
              >
                <td class="py-2 px-3 text-gray-600">{{ sku.skuCode || '-' }}</td>
                <td class="py-2 px-3">
                  <span
                    v-for="(val, key) in parseSpecs(sku.specs)"
                    :key="key"
                    class="inline-block bg-gray-200 rounded px-1.5 py-0.5 text-xs mr-1"
                  >
                    {{ key }}: {{ val }}
                  </span>
                </td>
                <td class="py-2 px-3 text-right">¥{{ sku.price ?? 0 }}</td>
                <td class="py-2 px-3 text-right" :class="{ 'text-red-500': !sku.stock || sku.stock === 0 }">{{ sku.stock ?? 0 }}</td>
                <td class="py-2 px-3 text-center">
                  <Tag :color="sku.isActive ? 'green' : 'red'" :bordered="false" style="font-size: 11px">
                    {{ sku.isActive ? '启用' : '停用' }}
                  </Tag>
                </td>
              </tr>
            </tbody>
          </table>
          <div v-else-if="skuMap[row.id] && skuMap[row.id].length === 0" class="text-center text-gray-400 py-4 text-sm">
            暂无规格数据
          </div>
          <div v-else class="flex justify-center py-4">
            <Spin size="small" />
          </div>
        </div>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:product:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.list.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:product:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
