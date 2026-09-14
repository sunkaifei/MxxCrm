<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';

import { LucideImageOff, LucidePlus, LucideTrash2 } from '@vben/icons';

import {
  Button,
  Drawer,
  Empty,
  Input,
  InputNumber,
  message,
  Modal,
  Select,
  Switch,
  Table,
  TabPane,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { articleApi } from '#/api/core/website/article';
import {
  categoryWebsiteProductApi,
  getProductLibraryListApi,
  getWebsiteProductCategoriesApi,
  getWebsiteProductListApi,
  getWebsiteProductSkusApi,
  quantityWebsiteProductApi,
  recommendWebsiteProductApi,
  saveWebsiteProductSkuPricesApi,
  saveWebsiteProductSkuQuantitiesApi,
  updateWebsiteProductRelatedApi,
  updateWebsiteProductSeoApi,
  shelfWebsiteProductApi,
  sortWebsiteProductApi,
  updateWebsiteProductSkusApi,
} from '#/api/core/website/product';

const props = defineProps<{
  open?: boolean;
  row?: any;
}>();

const emit = defineEmits<{
  (e: 'saved'): void;
  (e: 'update:open', open: boolean): void;
}>();

const form = reactive({
  categoryId: undefined as number | undefined,
  quantity: 0,
  sort: 0,
  status: 1,
  skuIds: [] as number[],
});

/** SKU 前台零售价（skuId → 价格；仅作用于前台在线销售，不写回产品库） */
const priceMap = reactive<Record<number, any>>({});
const origPriceMap = reactive<Record<number, number>>({});
/** SKU 前台销售库存（skuId → 数量；钳制为不超过该 SKU 仓储库存） */
const qtyMap = reactive<Record<number, number>>({});
/** SKU 已保存的销售库存（未配置过则为 undefined） */
const origQtyMap = reactive<Record<number, number | undefined>>({});

const catOptions = ref<{ id: number; name: string }[]>([]);
const skuList = ref<any[]>([]);
/** 是否存在 SKU（有 SKU 时库存按规格分别管理，不再有全局库存） */
const hasSkus = computed(() => skuList.value.length > 0);
/** 推荐/相关产品/相关文章/SEO */
const formExtra = reactive({
  isRecommend: 0,
  relatedProductIds: [] as number[],
  relatedArticleIds: [] as number[],
  seoTitle: '',
  seoKeywords: '',
  seoDescription: '',
});
const relatedOptions = ref<{ id: number; name: string; imageUrl?: string }[]>([]);
const articleOptions = ref<{ id: number; title: string }[]>([]);
const productId = ref<number>(0);
const totalStock = ref<number>(0);
const productName = ref('');

/** 相关产品选择弹窗状态 */
const relProductModal = reactive({
  open: false,
  keywords: '',
  rows: [] as any[],
  loading: false,
  total: 0,
  page: 1,
  pageSize: 10,
  selectedKeys: [] as number[],
});
/** 相关文章选择弹窗状态 */
const relArticleModal = reactive({
  open: false,
  keywords: '',
  rows: [] as any[],
  loading: false,
  total: 0,
  page: 1,
  pageSize: 10,
  selectedKeys: [] as number[],
});

const relProductCols = [
  { title: '图片', dataIndex: 'imageUrl', key: 'imageUrl', width: 64 },
  { title: '产品名称', dataIndex: 'name', key: 'name', minWidth: 200 },
  { title: '销售价', dataIndex: 'salePrice', key: 'salePrice', width: 90 },
  { title: '库存', dataIndex: 'totalStock', key: 'totalStock', width: 80 },
];

const relArticleCols = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 70 },
  { title: '文章标题', dataIndex: 'title', key: 'title', minWidth: 220 },
  { title: '发布时间', dataIndex: 'publishTime', key: 'publishTime', width: 160 },
];

watch(
  () => props.open,
  async (isOpen) => {
    if (!isOpen || !props.row) return;
    const row = props.row;
    productId.value = Number(row.productId);
    productName.value = row.productName || '';
    totalStock.value = Number(row.totalStock) || 0;
    form.quantity = Number(row.quantity) || 0;
    form.sort = Number(row.sort) || 0;
    form.status = row.status ?? 1;
    form.categoryId = row.categoryId ?? undefined;
    form.skuIds = Array.isArray(row.skuIds) ? [...row.skuIds] : [];
    // 零售价初始化：清单行已设置的优先，否则用产品库 SKU 价格
    const savedPrices: Record<string, number> = row.skuPrices ?? {};
    // 销售库存初始化：已设置的值钳制为不超过该 SKU 仓储库存；未设置默认取仓储库存
    let savedQtys: Record<string, number> = {};
    if (row.skuQuantities && typeof row.skuQuantities === 'object') {
      savedQtys = row.skuQuantities;
    } else if (typeof row.skuQuantities === 'string' && row.skuQuantities) {
      try {
        savedQtys = JSON.parse(row.skuQuantities);
      } catch {
        savedQtys = {};
      }
    }
    if (catOptions.value.length === 0) {
      try {
        const res: any = await getWebsiteProductCategoriesApi();
        catOptions.value = Array.isArray(res) ? res : (res?.items || []);
      } catch {
        catOptions.value = [];
      }
    }
    try {
      const res: any = await getWebsiteProductSkusApi(productId.value);
      skuList.value = Array.isArray(res) ? res : (res?.items || []);
      for (const s of skuList.value) {
        const sid = Number(s.id);
        const saved = savedPrices[String(sid)];
        priceMap[sid] = saved ?? (s.price !== undefined && s.price !== null ? Number(s.price) : null);
        origPriceMap[sid] = priceMap[sid] ?? 0;
        const stock = Number(s.stock) || 0;
        const savedQty = savedQtys[String(sid)];
        const base = savedQty !== undefined && savedQty !== null ? Number(savedQty) : stock;
        qtyMap[sid] = Math.min(base, stock);
        if (savedQty !== undefined && savedQty !== null) {
          origQtyMap[sid] = Math.min(Number(savedQty), stock);
        }
      }
      // 未在 SKU 列表中的已存价格（SKU 被删）忽略
    } catch {
      skuList.value = [];
    }
    // 推荐/相关/SEO
    formExtra.isRecommend = row.isRecommend ?? 0;
    formExtra.relatedProductIds = Array.isArray(row.relatedProductIds)
      ? [...row.relatedProductIds]
      : [];
    formExtra.relatedArticleIds = Array.isArray(row.relatedArticleIds)
      ? [...row.relatedArticleIds]
      : [];
    formExtra.seoTitle = row.seoTitle || '';
    formExtra.seoKeywords = row.seoKeywords || '';
    formExtra.seoDescription = row.seoDescription || '';
    try {
      const res: any = await getWebsiteProductListApi({ pageNum: 1, pageSize: 100 });
      const rows: any[] = Array.isArray(res) ? res : (res?.items || res?.list || []);
      relatedOptions.value = rows
        .filter((r: any) => Number(r.productId) !== productId.value)
        .map((r: any) => ({
          id: Number(r.productId),
          name: r.productName || `产品 ${r.productId}`,
          imageUrl: r.productImage,
        }));
    } catch {
      relatedOptions.value = [];
    }
    try {
      const res: any = await (articleApi.list as any)({ page: 1, pageSize: 100 });
      const rows: any[] = Array.isArray(res) ? res : (res?.items || res?.list || []);
      articleOptions.value = rows.map((r: any) => ({ id: Number(r.id), title: r.title || `文章 ${r.id}` }));
    } catch {
      articleOptions.value = [];
    }
  },
);

/** 已选相关产品列表（富化名称/图片） */
const relatedSelectedList = computed(() =>
  formExtra.relatedProductIds.map((pid) => {
    const opt = relatedOptions.value.find((o) => o.id === pid);
    return { id: pid, name: opt?.name || `产品 ${pid}`, imageUrl: opt?.imageUrl };
  }),
);
/** 已选相关文章列表（富化标题） */
const articleSelectedList = computed(() =>
  formExtra.relatedArticleIds.map((id) => {
    const opt = articleOptions.value.find((o) => o.id === id);
    return { id, title: opt?.title || `文章 ${id}` };
  }),
);

function toggleSku(skuId: number) {
  const idx = form.skuIds.indexOf(skuId);
  if (idx >= 0) {
    form.skuIds.splice(idx, 1);
  } else {
    form.skuIds.push(skuId);
  }
}

function selectAllSkus() {
  form.skuIds = skuList.value.map((s) => Number(s.id));
}

function clearSkus() {
  form.skuIds = [];
}

/** 展示数量变更：超仓储库存自动钳制并提示 */
function onQuantityChange(value: any) {
  const stock = totalStock.value;
  let v = Math.max(0, Math.floor(Number(value) || 0));
  if (v > stock) {
    v = stock;
    message.warning(`展示数量不能超过仓储库存 ${stock}，已自动调整为 ${stock}`);
  }
  form.quantity = v;
}

/** 销售库存变更：超该 SKU 仓储库存自动钳制并提示（防超卖） */
function onQtyChange(sku: any, value: any) {
  const sid = Number(sku.id);
  const stock = Number(sku.stock) || 0;
  let v = Math.max(0, Math.floor(Number(value) || 0));
  if (v > stock) {
    v = stock;
    message.warning(`销售库存不能超过仓储库存 ${stock}，已自动调整为 ${stock}`);
  }
  qtyMap[sid] = v;
}

function removeRelatedProduct(id: number) {
  formExtra.relatedProductIds = formExtra.relatedProductIds.filter((pid) => pid !== id);
}

function removeRelatedArticle(id: number) {
  formExtra.relatedArticleIds = formExtra.relatedArticleIds.filter((a) => a !== id);
}

function openRelProductModal() {
  relProductModal.keywords = '';
  relProductModal.selectedKeys = [];
  relProductModal.open = true;
  loadRelProducts(1);
}

function openRelArticleModal() {
  relArticleModal.keywords = '';
  relArticleModal.selectedKeys = [];
  relArticleModal.open = true;
  loadRelArticles(1);
}

async function loadRelProducts(page: number) {
  relProductModal.loading = true;
  try {
    const res: any = await getProductLibraryListApi({
      page,
      pageSize: relProductModal.pageSize,
      keywords: relProductModal.keywords || undefined,
    });
    const rows: any[] = Array.isArray(res) ? res : (res?.items || res?.list || []);
    relProductModal.rows = rows;
    relProductModal.total = Array.isArray(res) ? rows.length : (res?.total ?? rows.length);
    relProductModal.page = page;
    const ids = new Set(rows.map((r) => Number(r.id)));
    relProductModal.selectedKeys = formExtra.relatedProductIds.filter((pid) => ids.has(pid));
  } catch {
    relProductModal.rows = [];
    relProductModal.total = 0;
  } finally {
    relProductModal.loading = false;
  }
}

async function loadRelArticles(page: number) {
  relArticleModal.loading = true;
  try {
    const res: any = await (articleApi.list as any)({
      page,
      pageSize: relArticleModal.pageSize,
      keyword: relArticleModal.keywords || undefined,
    });
    const rows: any[] = Array.isArray(res) ? res : (res?.items || res?.list || []);
    relArticleModal.rows = rows;
    relArticleModal.total = Array.isArray(res) ? rows.length : (res?.total ?? rows.length);
    relArticleModal.page = page;
    const ids = new Set(rows.map((r) => Number(r.id)));
    relArticleModal.selectedKeys = formExtra.relatedArticleIds.filter((aid) => ids.has(aid));
  } catch {
    relArticleModal.rows = [];
    relArticleModal.total = 0;
  } finally {
    relArticleModal.loading = false;
  }
}

function onRelProductSelect(keys: any[]) {
  relProductModal.selectedKeys = (keys || []).map(Number);
}

function onRelArticleSelect(keys: any[]) {
  relArticleModal.selectedKeys = (keys || []).map(Number);
}

function onRelProductPageChange(pagination: any) {
  loadRelProducts(pagination.current);
}

function onRelArticlePageChange(pagination: any) {
  loadRelArticles(pagination.current);
}

function confirmRelProducts() {
  const pageIds = new Set(relProductModal.rows.map((r) => Number(r.id)));
  const selected = new Set(relProductModal.selectedKeys.map(Number));
  const pid = productId.value;
  // 当前页按勾选状态增删；其他页保持不动；始终排除自身
  const merged = formExtra.relatedProductIds.filter((id) => {
    if (id === pid) return false;
    if (pageIds.has(id)) return selected.has(id);
    return true;
  });
  for (const id of selected) {
    if (id !== pid && !merged.includes(id)) merged.push(id);
  }
  formExtra.relatedProductIds = merged;
  relProductModal.open = false;
}

function confirmRelArticles() {
  const pageIds = new Set(relArticleModal.rows.map((r) => Number(r.id)));
  const selected = new Set(relArticleModal.selectedKeys.map(Number));
  const merged = formExtra.relatedArticleIds.filter((id) => {
    if (pageIds.has(id)) return selected.has(id);
    return true;
  });
  for (const id of selected) {
    if (!merged.includes(id)) merged.push(id);
  }
  formExtra.relatedArticleIds = merged;
  relArticleModal.open = false;
}

function close() {
  emit('update:open', false);
}

async function onSave() {
  const rowId = Number(props.row?.id);
  // 无 SKU（单规格）才保存全局展示数量；有 SKU 时库存按规格管理，不写全局数量
  if (!hasSkus.value) {
    if (form.quantity < 0) {
      message.warning('展示数量不能为负数');
      return;
    }
    await quantityWebsiteProductApi(rowId, form.quantity);
  }
  await categoryWebsiteProductApi(rowId, Number(form.categoryId) || 0);
  await updateWebsiteProductSkusApi(rowId, form.skuIds);
  // 零售价：仅提交有修改的 SKU
  const changed: Record<string, number> = {};
  for (const [sid, price] of Object.entries(priceMap)) {
    if (price !== null && price !== undefined && Number(price) !== origPriceMap[Number(sid)]) {
      changed[sid] = Number(price);
    }
  }
  if (Object.keys(changed).length > 0) {
    await saveWebsiteProductSkuPricesApi(rowId, changed);
  }
  // 销售库存：提交已配置过或本次修改过的 SKU；未配置的 SKU 以前台仓储库存为上限（自动跟随库存变化）
  if (hasSkus.value) {
    const qtyObj: Record<string, number> = {};
    for (const s of skuList.value) {
      const sid = Number(s.id);
      const stock = Number(s.stock) || 0;
      const v = Number(qtyMap[sid] ?? stock);
      const orig = origQtyMap[sid] === undefined ? stock : Number(origQtyMap[sid]);
      if (origQtyMap[sid] !== undefined || v !== orig) {
        qtyObj[String(sid)] = v;
      }
    }
    if (Object.keys(qtyObj).length > 0) {
      await saveWebsiteProductSkuQuantitiesApi(rowId, qtyObj);
    }
  }
  await sortWebsiteProductApi(rowId, form.sort);
  await recommendWebsiteProductApi([rowId], formExtra.isRecommend);
  await updateWebsiteProductRelatedApi(rowId, formExtra.relatedProductIds, formExtra.relatedArticleIds);
  await updateWebsiteProductSeoApi(rowId, formExtra.seoTitle || undefined, formExtra.seoKeywords || undefined, formExtra.seoDescription || undefined);
  if ((props.row?.status ?? 1) !== form.status) {
    await shelfWebsiteProductApi([rowId], form.status);
  }
  message.success('保存成功');
  emit('saved');
  close();
}
</script>

<template>
  <Drawer
    :open="open"
    :title="`编辑上架产品 - ${productName}`"
    placement="right"
    width="75%"
    @close="close"
  >
    <Tabs>
      <!-- 基础设置 -->
      <TabPane key="base" tab="基础设置">
        <div class="flex flex-col gap-4">
          <div class="flex items-center gap-3">
            <span class="text-sm font-medium">推荐设置</span>
            <Switch
              :checked="formExtra.isRecommend === 1"
              checked-children="推荐"
              un-checked-children="否"
              @change="(checked: any) => (formExtra.isRecommend = checked ? 1 : 0)"
            />
            <span class="text-xs text-[hsl(var(--muted-foreground))]">
              推荐产品可通过前台模板标签 get_recommend_products 在推荐位展示
            </span>
          </div>

          <!-- 无 SKU（单规格）：使用全局展示数量 -->
          <template v-if="!hasSkus">
            <div>
              <div class="mb-1 text-sm font-medium">当前仓储库存</div>
              <div class="text-lg" :class="totalStock > 0 ? '' : 'text-red-500'">
                {{ totalStock }}
              </div>
              <div class="text-xs text-[hsl(var(--muted-foreground))]">
                展示数量不能超过仓储库存，库存减少时前台展示数量自动跟随下降
              </div>
            </div>

            <div>
              <div class="mb-1 text-sm font-medium">展示数量</div>
              <InputNumber
                :min="0"
                :max="Math.max(totalStock, form.quantity)"
                :value="form.quantity"
                class="w-40"
                @change="onQuantityChange"
              />
              <span class="ml-2 text-xs text-[hsl(var(--muted-foreground))]">
                保存时自动钳制为不超过库存
              </span>
            </div>
          </template>
          <!-- 有 SKU：库存与销售数量按各规格分别管理，不再有全局库存 -->
          <div
            v-else
            class="rounded-md bg-[hsl(var(--muted))] px-3 py-2 text-xs text-[hsl(var(--muted-foreground))]"
          >
            该产品按规格分别管理库存与销售数量，请在「多规格 SKU」选项卡中逐个规格设置（无全局库存）。
          </div>

          <div>
            <div class="mb-1 text-sm font-medium">栏目分类</div>
            <Select
              v-model:value="form.categoryId"
              :options="catOptions"
              :field-names="{ label: 'name', value: 'id' }"
              placeholder="未归类（可选栏目管理中内容类型为产品的栏目）"
              allow-clear
              class="w-full"
            />
          </div>

          <div>
            <div class="mb-1 text-sm font-medium">排序（越小越靠前）</div>
            <InputNumber :min="0" v-model:value="form.sort" class="w-40" />
          </div>

          <div>
            <div class="mb-1 text-sm font-medium">上架状态</div>
            <Switch
              :checked="form.status === 1"
              checked-children="上架"
              un-checked-children="下架"
              @change="(checked: any) => (form.status = checked ? 1 : 0)"
            />
          </div>
        </div>
      </TabPane>

      <!-- 规格库存（有 SKU 时显示）：库存与销售数量按规格分别管理 -->
      <TabPane
        v-if="hasSkus"
        key="sku"
        :tab="skuList.length > 1 ? '多规格 SKU' : '规格库存'"
      >
        <div class="mb-3 flex items-center gap-2">
          <Tag
            class="cursor-pointer select-none"
            :color="form.skuIds.length === 0 ? 'blue' : 'default'"
            @click="clearSkus()"
          >
            全部 SKU
          </Tag>
          <Tag color="default" class="cursor-pointer select-none" @click="selectAllSkus()">
            全选
          </Tag>
          <span class="text-xs text-[hsl(var(--muted-foreground))]">
            蓝色 = 前台展示该 SKU；销售库存不能超过该 SKU 仓储库存，超出自动钳制；库存为 0 的规格前台不可购买
          </span>
        </div>
        <div class="flex flex-col gap-2">
          <div
            v-for="s in skuList"
            :key="s.id"
            class="flex flex-wrap items-center gap-3 rounded-md border px-3 py-2"
            :class="
              (Number(s.stock) || 0) <= 0
                ? 'border-dashed border-[hsl(var(--border))] bg-[hsl(var(--muted))] opacity-60'
                : 'border-[hsl(var(--border))]'
            "
          >
            <Tag
              class="cursor-pointer select-none"
              :color="form.skuIds.length === 0 || form.skuIds.includes(Number(s.id)) ? 'blue' : 'default'"
              @click="toggleSku(Number(s.id))"
            >
              {{ s.label || s.skuCode || `SKU ${s.id}` }}
            </Tag>
            <Tag v-if="(Number(s.stock) || 0) <= 0" color="red">缺货</Tag>
            <span class="text-xs text-[hsl(var(--muted-foreground))]">仓储库存</span>
            <span
              class="text-sm font-medium"
              :class="(Number(s.stock) || 0) > 0 ? '' : 'text-red-500'"
            >
              {{ s.stock ?? 0 }}
            </span>
            <span class="text-xs text-[hsl(var(--muted-foreground))]">销售库存</span>
            <InputNumber
              size="small"
              :min="0"
              :max="Number(s.stock) || 0"
              :value="qtyMap[Number(s.id)]"
              :disabled="(Number(s.stock) || 0) <= 0"
              placeholder="销售库存"
              class="w-24"
              @change="(v: any) => onQtyChange(s, v)"
            />
            <span class="text-xs text-[hsl(var(--muted-foreground))]">零售价</span>
            <InputNumber
              size="small"
              :min="0"
              :precision="2"
              v-model:value="priceMap[Number(s.id)]"
              placeholder="前台零售价"
              class="w-32"
            />
            <span class="text-xs text-[hsl(var(--muted-foreground))]">
              (仓储价 ¥{{ s.price ?? '-' }})
            </span>
          </div>
        </div>
      </TabPane>

      <!-- 相关产品 -->
      <TabPane key="related" tab="相关产品">
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <span class="text-sm text-[hsl(var(--muted-foreground))]">
              已选 {{ relatedSelectedList.length }} 个相关产品
            </span>
            <Button type="primary" size="small" @click="openRelProductModal">
              <LucidePlus class="mr-1 size-4" />
              添加相关产品
            </Button>
          </div>
          <div v-if="relatedSelectedList.length === 0" class="py-6">
            <Empty :image="Empty.PRESENTED_IMAGE_SIMPLE" description="暂无相关产品" />
          </div>
          <div v-else class="flex flex-col gap-2">
            <div
              v-for="item in relatedSelectedList"
              :key="item.id"
              class="flex items-center gap-3 rounded-md border border-[hsl(var(--border))] px-3 py-2"
            >
              <div
                v-if="item.imageUrl"
                class="flex h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
              >
                <img :src="item.imageUrl" alt="" class="h-full w-full object-cover" />
              </div>
              <div
                v-else
                class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
              >
                <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
              </div>
              <span class="flex-1 text-sm">{{ item.name }}</span>
              <Button type="link" danger size="small" @click="removeRelatedProduct(item.id)">
                <LucideTrash2 class="size-4" />
                移除
              </Button>
            </div>
          </div>
        </div>
      </TabPane>

      <!-- 相关文章 -->
      <TabPane key="articles" tab="相关文章">
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <span class="text-sm text-[hsl(var(--muted-foreground))]">
              已选 {{ articleSelectedList.length }} 篇相关文章
            </span>
            <Button type="primary" size="small" @click="openRelArticleModal">
              <LucidePlus class="mr-1 size-4" />
              添加相关文章
            </Button>
          </div>
          <div v-if="articleSelectedList.length === 0" class="py-6">
            <Empty :image="Empty.PRESENTED_IMAGE_SIMPLE" description="暂无相关文章" />
          </div>
          <div v-else class="flex flex-col gap-2">
            <div
              v-for="item in articleSelectedList"
              :key="item.id"
              class="flex items-center gap-3 rounded-md border border-[hsl(var(--border))] px-3 py-2"
            >
              <span class="flex-1 text-sm">{{ item.title }}</span>
              <Button type="link" danger size="small" @click="removeRelatedArticle(item.id)">
                <LucideTrash2 class="size-4" />
                移除
              </Button>
            </div>
          </div>
        </div>
      </TabPane>

      <!-- SEO -->
      <TabPane key="seo" tab="SEO 设置">
        <div class="mb-1 text-sm font-medium">仅作用于前台产品详情页</div>
        <div class="flex flex-col gap-2">
          <Input v-model:value="formExtra.seoTitle" placeholder="SEO 标题（留空使用产品名称）" allow-clear />
          <Input v-model:value="formExtra.seoKeywords" placeholder="SEO 关键词（留空使用产品默认）" allow-clear />
          <Input v-model:value="formExtra.seoDescription" placeholder="SEO 描述（留空使用产品默认）" allow-clear />
        </div>
      </TabPane>
    </Tabs>

    <!-- 相关产品选择弹窗 -->
    <Modal
      :open="relProductModal.open"
      title="选择相关产品"
      width="760px"
      ok-text="确认添加"
      cancel-text="取消"
      @cancel="relProductModal.open = false"
      @ok="confirmRelProducts"
    >
      <div class="mb-3 flex items-center gap-2">
        <Input
          v-model:value="relProductModal.keywords"
          placeholder="搜索产品名称"
          allow-clear
          class="w-64"
          @press-enter="loadRelProducts(1)"
        />
        <Button type="primary" @click="loadRelProducts(1)">搜索</Button>
        <span class="ml-auto text-xs text-[hsl(var(--muted-foreground))]">
          勾选添加，取消勾选将移除（仅对当前页生效）
        </span>
      </div>
      <Table
        :columns="relProductCols"
        :data-source="relProductModal.rows"
        :loading="relProductModal.loading"
        :pagination="{
          current: relProductModal.page,
          pageSize: relProductModal.pageSize,
          total: relProductModal.total,
          showSizeChanger: false,
        }"
        :row-selection="{ selectedRowKeys: relProductModal.selectedKeys, onChange: onRelProductSelect }"
        row-key="id"
        size="small"
        :scroll="{ y: 380 }"
        @change="onRelProductPageChange"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'imageUrl'">
            <div
              v-if="record.imageUrl"
              class="flex h-10 w-10 overflow-hidden rounded-lg border border-[hsl(var(--border))]"
            >
              <img :src="record.imageUrl" alt="" class="h-full w-full object-cover" />
            </div>
            <div
              v-else
              class="flex h-10 w-10 items-center justify-center rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
            >
              <LucideImageOff class="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
            </div>
          </template>
        </template>
      </Table>
    </Modal>

    <!-- 相关文章选择弹窗 -->
    <Modal
      :open="relArticleModal.open"
      title="选择相关文章"
      width="720px"
      ok-text="确认添加"
      cancel-text="取消"
      @cancel="relArticleModal.open = false"
      @ok="confirmRelArticles"
    >
      <div class="mb-3 flex items-center gap-2">
        <Input
          v-model:value="relArticleModal.keywords"
          placeholder="搜索文章标题"
          allow-clear
          class="w-64"
          @press-enter="loadRelArticles(1)"
        />
        <Button type="primary" @click="loadRelArticles(1)">搜索</Button>
        <span class="ml-auto text-xs text-[hsl(var(--muted-foreground))]">
          勾选添加，取消勾选将移除（仅对当前页生效）
        </span>
      </div>
      <Table
        :columns="relArticleCols"
        :data-source="relArticleModal.rows"
        :loading="relArticleModal.loading"
        :pagination="{
          current: relArticleModal.page,
          pageSize: relArticleModal.pageSize,
          total: relArticleModal.total,
          showSizeChanger: false,
        }"
        :row-selection="{ selectedRowKeys: relArticleModal.selectedKeys, onChange: onRelArticleSelect }"
        row-key="id"
        size="small"
        :scroll="{ y: 380 }"
        @change="onRelArticlePageChange"
      />
    </Modal>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="close">取消</Button>
        <Button type="primary" @click="onSave">保存</Button>
      </div>
    </template>
  </Drawer>
</template>
