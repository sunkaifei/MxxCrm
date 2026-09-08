<script setup lang="ts">
import type { VxeGridProps } from '#/adapter/vxe-table';

import { nextTick, reactive, ref, watch } from 'vue';

import { LucideImageOff } from '@vben/icons';

import {
  Button,
  Drawer,
  Input,
  InputNumber,
  message,
  TabPane,
  Tabs,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  getWebsiteProductSkusApi,
  getProductLibraryListApi,
} from '#/api/core/website/product';
import { articleApi } from '#/api/core/website/article';
import { $t } from '#/locales';

const props = defineProps<{
  addedIds?: number[];
  open?: boolean;
}>();

const emit = defineEmits<{
  (
    e: 'confirm',
    items: { productId: number; quantity: number; skuIds: number[] }[],
    relatedProductIds: number[],
    relatedArticleIds: number[],
  ): void;
  (e: 'update:open', open: boolean): void;
}>();

const addedIdSet = ref<Set<number>>(new Set());
const state = reactive({ keywords: '' });

/** 每个产品的上架数量（productId → 数量） */
const quantityMap = reactive<Record<number, number>>({});
/** 每个产品勾选的 SKU（productId → Set；未展开/空 = 全部 SKU） */
const skuSelMap = reactive<Record<number, Set<number>>>({});
/** SKU 列表缓存（productId → SKU 数组） */
const skuListMap = reactive<Record<number, any[]>>({});
/** 相关产品勾选（productId Set）与网格 */
const relatedSel = ref<Set<number>>(new Set());
const relatedGridState = reactive({ keywords: '' });
/** 相关文章勾选与网格 */
const articleSel = ref<Set<number>>(new Set());
const articleGridState = reactive({ keywords: '' });

void relatedGridState;
void articleGridState;

const relatedGridOptions: VxeGridProps = {
  columns: [
    { type: 'checkbox', width: 50 },
    { title: 'ID', field: 'id', width: 70 },
    {
      title: '图片',
      field: 'imageUrl',
      width: 76,
      slots: { default: 'relatedImage' },
    },
    { title: '产品名称', field: 'name', minWidth: 170 },
    { title: '销售价', field: 'salePrice', width: 90 },
  ],
  checkboxConfig: {},
  pagerConfig: {},
  height: 420,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const res: any = await getProductLibraryListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: relatedGridState.keywords || undefined,
        });
        const rows: any[] = Array.isArray(res) ? res : (res?.items || res?.list || []);
        return {
          items: rows,
          total: Array.isArray(res) ? rows.length : (res?.total ?? rows.length),
        };
      },
    },
  },
};

const [RelatedGrid, relatedGridApi] = useVbenVxeGrid({ gridOptions: relatedGridOptions });

const articleGridOptions: VxeGridProps = {
  columns: [
    { type: 'checkbox', width: 50 },
    { title: 'ID', field: 'id', width: 70 },
    { title: '文章标题', field: 'title', minWidth: 220 },
    { title: '发布时间', field: 'createTime', width: 160 },
  ],
  checkboxConfig: {},
  pagerConfig: {},
  height: 420,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const res: any = await articleApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          keyword: articleGridState.keywords || undefined,
        });
        const rows: any[] = Array.isArray(res) ? res : (res?.items || res?.list || []);
        return {
          items: rows,
          total: Array.isArray(res) ? rows.length : (res?.total ?? rows.length),
        };
      },
    },
  },
};

const [ArticleGrid, articleGridApi] = useVbenVxeGrid({ gridOptions: articleGridOptions });

const gridOptions: VxeGridProps = {
  columns: [
    { type: 'checkbox', width: 50 },
    {
      title: '图片',
      field: 'imageUrl',
      width: 76,
      slots: { default: 'image' },
    },
    { title: '产品名称', field: 'name', minWidth: 170 },
    { title: '销售价', field: 'salePrice', width: 90 },
    {
      title: '当前库存',
      field: 'totalStock',
      width: 90,
      slots: { default: 'stock' },
    },
    {
      title: '上架数量',
      field: 'quantity',
      width: 130,
      slots: { default: 'quantity' },
    },
    {
      title: '展示SKU',
      field: 'skus',
      width: 100,
      slots: { default: 'skus' },
    },
  ],
  checkboxConfig: {
    checkMethod: ({ row }) => !addedIdSet.value.has(Number(row.id)),
  },
  pagerConfig: {},
  height: 560,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const res: any = await getProductLibraryListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: state.keywords || undefined,
        });
        const rows: any[] = Array.isArray(res)
          ? res
          : (res?.items || res?.list || []);
        // 初始化数量默认值 = 当前库存
        for (const r of rows) {
          const pid = Number(r.id);
          if (quantityMap[pid] === undefined) {
            quantityMap[pid] = Number(r.totalStock) || 0;
          }
        }
        return {
          items: rows,
          total: Array.isArray(res) ? rows.length : (res?.total ?? rows.length),
        };
      },
    },
  },
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

/** SKU 配置小窗状态 */
const skuPanel = reactive({
  open: false,
  productId: 0,
  productName: '',
});

function openSkuPanel(row: any) {
  const pid = Number(row.id);
  skuPanel.productId = pid;
  skuPanel.productName = row.name || '';
  loadSkus(pid);
  skuPanel.open = true;
}

watch(
  () => props.open,
  async (isOpen) => {
    if (!isOpen) return;
    addedIdSet.value = new Set((props.addedIds || []).map(Number));
    state.keywords = '';
    // 抽屉动画期间 grid 可能未就绪，双保险延迟触发查询
    await nextTick();
    setTimeout(() => gridApi.query(), 400);
    setTimeout(() => gridApi.query(), 1500);
    setTimeout(() => relatedGridApi.query(), 600);
    setTimeout(() => articleGridApi.query(), 800);
  },
);

function close() {
  emit('update:open', false);
}

async function onSearch() {
  await gridApi.query();
}

/** 拉取该产品的 SKU 列表 */
async function loadSkus(productId: number) {
  if (skuListMap[productId]) return;
  const res: any = await getWebsiteProductSkusApi(productId);
  const list: any[] = Array.isArray(res) ? res : (res?.items || []);
  skuListMap[productId] = list;
  // 默认勾选全部 SKU（未主动选择过时）
  if (!skuSelMap[productId]) {
    skuSelMap[productId] = new Set(list.map((s) => Number(s.id)));
  }
}

function toggleSku(productId: number, skuId: number) {
  const set = skuSelMap[productId] || new Set<number>();
  if (set.has(skuId)) {
    set.delete(skuId);
  } else {
    set.add(skuId);
  }
  skuSelMap[productId] = new Set(set);
}

function selectAllSkus(productId: number, skus: any[]) {
  skuSelMap[productId] = new Set(skus.map((s) => Number(s.id)));
}

function clearSkus(productId: number) {
  skuSelMap[productId] = new Set();
}

function onRelatedCheck() {
  const records: any[] = relatedGridApi.grid.getCheckboxRecords();
  relatedSel.value = new Set(records.map((r) => Number(r.id)));
}

function onArticleCheck() {
  const records: any[] = articleGridApi.grid.getCheckboxRecords();
  articleSel.value = new Set(records.map((r) => Number(r.id)));
}

function onQuantityChange(row: any, value: any) {
  const pid = Number(row.id);
  const stock = Number(row.totalStock) || 0;
  let v = Number(value) || 0;
  if (v > stock) {
    v = stock;
    message.warning(`上架数量不能超过当前库存 ${stock}`);
  }
  quantityMap[pid] = Math.max(0, v);
}

async function onConfirm() {
  const records: any[] = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    message.warning($t('page.website.product.picker.confirm'));
    return;
  }
  const items = records.map((r) => {
    const pid = Number(r.id);
    return {
      productId: pid,
      quantity: quantityMap[pid] ?? Number(r.totalStock) ?? 0,
      skuIds: skuSelMap[pid] ? [...skuSelMap[pid]] : [],
    };
  });
  // 相关产品（排除主产品自身，避免自关联）
  const related: number[] = [...relatedSel.value].filter((pid) => !items.some((it) => it.productId === pid));
  emit('confirm', items, related, [...articleSel.value]);
  close();
}
</script>
<template>
  <Drawer
    :open="open"
    :title="$t('page.website.product.picker.title')"
    placement="right"
    width="75%"
    @close="close"
  >
    <Tabs>
      <TabPane key="products" tab="选择产品">
        <div class="mb-3 flex items-center gap-2">
          <Input
            v-model:value="state.keywords"
            :placeholder="$t('page.website.product.form.keywords')"
            allow-clear
            class="w-64"
            @press-enter="onSearch"
          />
          <Button type="primary" @click="onSearch">
            {{ $t('page.website.product.form.keywords') }}
          </Button>
          <span class="ml-auto text-sm text-[hsl(var(--muted-foreground))]">
            {{ $t('page.website.product.picker.added') }}：{{ addedIdSet.size }}
          </span>
        </div>
        <Grid>
          <template #image="{ row }">
            <div
              v-if="row.imageUrl"
              class="flex h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
            >
              <img :src="row.imageUrl" alt="" class="h-full w-full object-cover" />
            </div>
            <div
              v-else
              class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
            >
              <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
            </div>
          </template>
          <template #stock="{ row }">
            <span :class="(row.totalStock || 0) > 0 ? '' : 'text-red-500'">
              {{ row.totalStock ?? 0 }}
            </span>
          </template>
          <template #quantity="{ row }">
            <InputNumber
              size="small"
              :min="0"
              :max="Number(row.totalStock) || 0"
              :value="quantityMap[Number(row.id)] ?? Number(row.totalStock) ?? 0"
              :disabled="addedIdSet.has(Number(row.id))"
              class="w-24"
              @change="(v: any) => onQuantityChange(row, v)"
            />
            <div
              v-if="addedIdSet.has(Number(row.id))"
              class="text-xs text-[hsl(var(--muted-foreground))]"
            >
              {{ $t('page.website.product.picker.added') }}
            </div>
          </template>
          <template #skus="{ row }">
            <Button
              size="small"
              :disabled="addedIdSet.has(Number(row.id))"
              @click="openSkuPanel(row)"
            >
              {{
                (skuSelMap[Number(row.id)]?.size || 0) > 0
                  ? `SKU ${skuSelMap[Number(row.id)]?.size ?? 0}`
                  : '全部'
              }}
            </Button>
          </template>
        </Grid>
      </TabPane>
      <TabPane key="related" tab="相关产品">
        <div class="mb-3 flex items-center gap-2">
          <Input
            v-model:value="relatedGridState.keywords"
            placeholder="产品名称"
            allow-clear
            class="w-64"
            @press-enter="relatedGridApi.query()"
          />
          <Button type="primary" @click="relatedGridApi.query()">搜索</Button>
          <span class="text-sm text-[hsl(var(--muted-foreground))]">
            已勾选：{{ relatedSel.size }} 个（相关产品会一并加入上架清单）
          </span>
        </div>
        <RelatedGrid @checkbox-change="onRelatedCheck">
          <template #relatedImage="{ row }">
            <div
              v-if="row.imageUrl"
              class="flex h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
            >
              <img :src="row.imageUrl" alt="" class="h-full w-full object-cover" />
            </div>
            <div
              v-else
              class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
            >
              <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
            </div>
          </template>
        </RelatedGrid>
      </TabPane>
      <TabPane key="articles" tab="相关文章">
        <div class="mb-3 flex items-center gap-2">
          <Input
            v-model:value="articleGridState.keywords"
            placeholder="文章标题"
            allow-clear
            class="w-64"
            @press-enter="articleGridApi.query()"
          />
          <Button type="primary" @click="articleGridApi.query()">搜索</Button>
          <span class="text-sm text-[hsl(var(--muted-foreground))]">
            已勾选：{{ articleSel.size }} 篇相关文章
          </span>
        </div>
        <ArticleGrid @checkbox-change="onArticleCheck" />
      </TabPane>
    </Tabs>

    <!-- SKU 勾选小窗 -->
    <Modal
      :open="skuPanel.open"
      :title="`配置展示SKU - ${skuPanel.productName}`"
      width="min(640px, 92vw)"
      @cancel="skuPanel.open = false"
      @ok="skuPanel.open = false"
    >
      <div class="mb-3 flex items-center gap-2">
        <Button size="small" @click="selectAllSkus(skuPanel.productId, skuListMap[skuPanel.productId] || [])">
          全选
        </Button>
        <Button size="small" @click="clearSkus(skuPanel.productId)">清空</Button>
        <span class="text-xs text-[hsl(var(--muted-foreground))]">
          不勾选任何 SKU 时前台默认展示全部
        </span>
      </div>
      <div v-if="!skuListMap[skuPanel.productId]" class="py-4 text-center text-[hsl(var(--muted-foreground))]">
        加载中…
      </div>
      <div
        v-else-if="(skuListMap[skuPanel.productId] || []).length === 0"
        class="py-4 text-center text-[hsl(var(--muted-foreground))]"
      >
        该产品未配置 SKU，将按单品展示
      </div>
      <div v-else class="flex flex-wrap gap-2 py-2">
        <Tag
          v-for="s in skuListMap[skuPanel.productId]"
          :key="s.id"
          class="cursor-pointer select-none"
          :color="skuSelMap[skuPanel.productId]?.has(Number(s.id)) ? 'blue' : 'default'"
          @click="toggleSku(skuPanel.productId, Number(s.id))"
        >
          {{ s.label || s.skuCode || `SKU ${s.id}` }}
          <span class="ml-1 opacity-70">¥{{ s.price }}</span>
        </Tag>
      </div>
    </Modal>

    <template #footer>
      <div class="flex items-center justify-between">
        <span class="text-sm text-[hsl(var(--muted-foreground))]">
          数量默认取当前库存，上限不超过仓储库存，库存变化时展示数量自动跟随
        </span>
        <div class="flex gap-2">
          <Button @click="close">取消</Button>
          <Button type="primary" @click="onConfirm">
            {{ $t('page.website.product.picker.confirm') }}
          </Button>
        </div>
      </div>
    </template>
  </Drawer>
</template>
