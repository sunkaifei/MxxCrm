<script setup lang="ts">
import type { VbenFormProps } from '#/adapter/form';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideImageOff, LucideTrash2 } from '@vben/icons';

import { Button, InputNumber, message, Popconfirm, Select, Switch, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  addWebsiteProductApi,
  categoryWebsiteProductApi,
  deleteWebsiteProductApi,
  getWebsiteProductCategoriesApi,
  getWebsiteProductListApi,
  quantityWebsiteProductApi,
  recommendWebsiteProductApi,
  shelfWebsiteProductApi,
  sortWebsiteProductApi,
} from '#/api/core/website/product';
import { $t } from '#/locales';
import { useAccessStore } from '@vben/stores';

import ProductDetailDrawer from '#/views/product/components/ProductDetailDrawer.vue';
import EditDrawer from './edit-drawer.vue';
import PickerModal from './picker.vue';

const accessStore = useAccessStore();
const canAdd = accessStore.hasAccessCode('website:product:add');
const canUpdate = accessStore.hasAccessCode('website:product:update');
const canDelete = accessStore.hasAccessCode('website:product:delete');

const addedProductIds = ref<number[]>([]);
const pickerOpen = ref(false);
/** 栏目分类选项（内容类型=产品的栏目） */
const catOptions = ref<{ id: number; name: string }[]>([]);

// ============ 产品详情抽屉（点击产品名称打开） ============
const detailVisible = ref(false);
const detailProductId = ref<null | number>(null);
function openProductDetail(row: any) {
  detailProductId.value = Number(row.productId);
  detailVisible.value = true;
}

// ============ 编辑抽屉 ============
const editOpen = ref(false);
const editingRow = ref<any>(null);
function openEdit(row: any) {
  editingRow.value = row;
  editOpen.value = true;
}

function openPicker() {
  pickerOpen.value = true;
}

/** 双击编辑状态：key = `${field}:${id}` */
const editingCell = ref('');

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: $t('page.website.product.form.keywords'),
      componentProps: { placeholder: $t('page.website.product.form.keywords'), allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.website.product.form.status'),
      componentProps: {
        placeholder: $t('page.website.product.form.all'),
        allowClear: true,
        options: [
          { label: $t('page.website.product.status.listed'), value: 1 },
          { label: $t('page.website.product.status.unlisted'), value: 0 },
        ] as any,
      },
    },
    {
      component: 'Select',
      fieldName: 'categoryId',
      label: '产品分类',
      componentProps: {
        placeholder: $t('page.website.product.form.all'),
        allowClear: true,
        options: catOptions,
        fieldNames: { label: 'name', value: 'id' },
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  stripe: true,
  checkboxConfig: {},
  columns: [
    { type: 'checkbox', width: 50 },
    { title: 'ID', field: 'id', width: 70 },
    {
      title: '图片',
      field: 'productImage',
      width: 76,
      slots: { default: 'productImage' },
    },
    {
      title: '产品名称',
      field: 'productName',
      minWidth: 180,
      slots: { default: 'productName' },
    },
    { title: '销售价', field: 'salePrice', width: 90 },
    {
      title: '产品分类',
      field: 'categoryId',
      minWidth: 130,
      slots: { default: 'category' },
    },
    {
      title: '推荐',
      field: 'isRecommend',
      width: 80,
      slots: { default: 'recommend' },
    },
    {
      title: '上架库存',
      field: 'effectiveQuantity',
      width: 110,
      slots: { default: 'shelfStock' },
    },
    {
      title: '排序',
      field: 'sort',
      width: 90,
      slots: { default: 'sort' },
    },
    {
      title: $t('page.website.product.form.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    { title: '添加时间', field: 'createTime', width: 160 },
    {
      field: 'actions',
      title: '操作',
      width: 150,
      fixed: 'right',
      slots: { default: 'actions' },
    },
  ],
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const res: any = await getWebsiteProductListApi({
          pageNum: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords || undefined,
          status: formValues.status ?? undefined,
          categoryId: formValues.categoryId || undefined,
        });
        // 后端 ResultPage: {items, total, currentPage, pageSize}
        const rows: any[] = Array.isArray(res)
          ? res
          : (res?.items || res?.list || []);
        addedProductIds.value = rows.map((r) => Number(r.productId));
        const total = Array.isArray(res)
          ? rows.length
          : (res?.total ?? rows.length);
        return { items: rows, total };
      },
    },
  },
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function getSelectedIds(): number[] {
  const records: any[] = gridApi.grid.getCheckboxRecords();
  return records.map((r) => Number(r.id));
}

function refresh() {
  gridApi.query();
}

async function loadCatOptions() {
  try {
    const res: any = await getWebsiteProductCategoriesApi();
    catOptions.value = Array.isArray(res) ? res : (res?.items || []);
  } catch {
    catOptions.value = [];
  }
}
onMounted(loadCatOptions);

async function onPickerConfirm(
  items: { productId: number; quantity: number; skuIds: number[] }[],
  relatedProductIds: number[],
  relatedArticleIds: number[],
) {
  const res: any = await addWebsiteProductApi(items, relatedProductIds, relatedArticleIds);
  const added = Number(res) || items.length;
  message.success(`${added} 个产品已上架`);
  refresh();
}

async function onStatusChange(row: any, checked: boolean) {
  await shelfWebsiteProductApi([Number(row.id)], checked ? 1 : 0);
  message.success(checked ? $t('page.website.product.button.shelf') : $t('page.website.product.button.offShelf'));
  refresh();
}

async function onCategoryChange(row: any, value: any) {
  await categoryWebsiteProductApi(Number(row.id), Number(value) || 0);
  message.success('分类已更新');
  refresh();
}

async function saveQuantity(row: any, value: number) {
  const stock = Number(row.totalStock) || 0;
  if (value > stock) {
    message.warning(`上架库存不能超过当前库存 ${stock}，已自动调整为 ${stock}`);
    return;
  }
  const res: any = await quantityWebsiteProductApi(Number(row.id), value);
  message.success(`展示数量已保存（生效 ${Number(res) || value}）`);
  refresh();
}

async function saveSort(row: any, value: number) {
  await sortWebsiteProductApi(Number(row.id), value);
  message.success('排序已保存');
  refresh();
}

async function onRecommendChange(row: any, checked: boolean) {
  await recommendWebsiteProductApi([Number(row.id)], checked ? 1 : 0);
  message.success(checked ? '已设为推荐' : '已取消推荐');
  refresh();
}

async function onBatchRecommend(recommend: number) {
  const ids = getSelectedIds();
  if (ids.length === 0) {
    message.warning('请先勾选要操作的清单行');
    return;
  }
  await recommendWebsiteProductApi(ids, recommend);
  message.success(recommend === 1 ? '已设为推荐' : '已取消推荐');
  refresh();
}

async function onBatchShelf(status: number) {
  const ids = getSelectedIds();
  if (ids.length === 0) {
    message.warning('请先勾选要操作的清单行');
    return;
  }
  await shelfWebsiteProductApi(ids, status);
  message.success(status === 1 ? $t('page.website.product.button.shelf') : $t('page.website.product.button.offShelf'));
  refresh();
}

async function onBatchDelete() {
  const ids = getSelectedIds();
  if (ids.length === 0) {
    message.warning('请先勾选要操作的清单行');
    return;
  }
  await deleteWebsiteProductApi(ids);
  message.success($t('page.website.product.button.delete'));
  refresh();
}
</script>

<template>
  <Page>
    <Grid>
      <template #toolbar-tools>
        <Button
          v-if="canAdd"
          type="primary"
          class="mr-2"
          @click="openPicker"
        >
          {{ $t('page.website.product.button.add') }}
        </Button>
        <Button
          v-if="canUpdate"
          class="mr-2"
          @click="onBatchShelf(1)"
        >
          {{ $t('page.website.product.button.shelf') }}
        </Button>
        <Button
          v-if="canUpdate"
          class="mr-2"
          @click="onBatchShelf(0)"
        >
          {{ $t('page.website.product.button.offShelf') }}
        </Button>
        <Button
          v-if="canUpdate"
          class="mr-2"
          @click="onBatchRecommend(1)"
        >
          设为推荐
        </Button>
        <Button
          v-if="canUpdate"
          class="mr-2"
          @click="onBatchRecommend(0)"
        >
          取消推荐
        </Button>
        <Button
          v-if="canDelete"
          danger
          class="mr-2"
          @click="onBatchDelete"
        >
          <LucideTrash2 class="mr-1 size-4" />
          {{ $t('page.website.product.button.delete') }}
        </Button>
      </template>
      <template #productName="{ row }">
        <span
          class="cursor-pointer font-medium hover:text-[hsl(var(--primary))]"
          title="点击查看产品详情"
          @click="openProductDetail(row)"
        >
          {{ row.productName }}
        </span>
      </template>
      <template #productImage="{ row }">
        <div
          v-if="row.productImage"
          class="flex h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
        >
          <img :src="row.productImage" alt="" class="h-full w-full object-cover" />
        </div>
        <div
          v-else
          class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
        >
          <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
        </div>
      </template>
      <template #category="{ row }">
        <Select
          v-if="canUpdate"
          size="small"
          :value="row.categoryId ?? undefined"
          :options="catOptions"
          :field-names="{ label: 'name', value: 'id' }"
          placeholder="未归类"
          allow-clear
          class="w-28 text-left"
          @change="(v: any) => onCategoryChange(row, v)"
        />
        <Tag v-else-if="row.categoryName" color="blue">{{ row.categoryName }}</Tag>
        <span v-else class="text-[hsl(var(--muted-foreground))]">未归类</span>
      </template>
      <template #recommend="{ row }">
        <Switch
          v-if="canUpdate"
          size="small"
          :checked="row.isRecommend === 1"
          checked-children="推荐"
          @change="(checked: any) => onRecommendChange(row, Boolean(checked))"
        />
        <span v-else>{{ row.isRecommend === 1 ? '推荐' : '-' }}</span>
      </template>
      <template #shelfStock="{ row }">
        <InputNumber
          v-if="canUpdate && editingCell === `qty:${row.id}`"
          size="small"
          :min="0"
          :max="row.totalStock ?? 0"
          :value="row.effectiveQuantity ?? row.quantity ?? 0"
          class="w-24"
          @change="(v: any) => saveQuantity(row, Number(v) || 0)"
          @blur="editingCell = ''"
          @press-enter="editingCell = ''"
        />
        <span
          v-else
          class="cursor-text"
          :title="`双击修改（当前库存 ${row.totalStock ?? 0}）`"
          @dblclick="canUpdate && (editingCell = `qty:${row.id}`)"
        >
          {{ row.effectiveQuantity ?? row.quantity ?? 0 }}
          <span
            v-if="(row.totalStock ?? 0) < (row.quantity ?? 0)"
            class="ml-1 text-xs text-orange-500"
          >
            (库存不足)
          </span>
        </span>
      </template>
      <template #sort="{ row }">
        <InputNumber
          v-if="canUpdate && editingCell === `sort:${row.id}`"
          size="small"
          :min="0"
          :value="row.sort"
          class="w-20"
          @change="(v: any) => saveSort(row, Number(v) || 0)"
          @blur="editingCell = ''"
          @press-enter="editingCell = ''"
        />
        <span
          v-else
          class="cursor-text"
          title="双击修改"
          @dblclick="canUpdate && (editingCell = `sort:${row.id}`)"
        >
          {{ row.sort }}
        </span>
      </template>
      <template #status="{ row }">
        <Switch
          v-if="canUpdate"
          :checked="row.status === 1"
          :checked-children="$t('page.website.product.status.listed')"
          :un-checked-children="$t('page.website.product.status.unlisted')"
          @change="(checked: any) => onStatusChange(row, Boolean(checked))"
        />
        <span v-else>{{ row.status === 1 ? $t('page.website.product.status.listed') : $t('page.website.product.status.unlisted') }}</span>
      </template>
      <template #actions="{ row }">
        <Button
          v-if="canUpdate"
          type="link"
          size="small"
          class="px-1"
          @click="openEdit(row)"
        >
          编辑
        </Button>
        <Popconfirm
          v-if="canDelete"
          title="确认移除该产品？移除后前台将不再展示"
          ok-text="确认移除"
          cancel-text="取消"
          @confirm="deleteWebsiteProductApi([Number(row.id)]).then(() => { message.success($t('page.website.product.button.delete')); refresh(); })"
        >
          <Button type="link" danger size="small">移除</Button>
        </Popconfirm>
      </template>
    </Grid>
    <PickerModal
      v-model:open="pickerOpen"
      :added-ids="addedProductIds"
      @confirm="onPickerConfirm"
    />
    <!-- 产品详情抽屉（点击产品名称打开） -->
    <ProductDetailDrawer
      v-model:visible="detailVisible"
      :product-id="detailProductId"
    />
    <!-- 编辑上架产品抽屉 -->
    <EditDrawer
      v-model:open="editOpen"
      :row="editingRow"
      @saved="refresh"
    />
  </Page>
</template>
