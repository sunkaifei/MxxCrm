<script setup lang="ts">
import { reactive, ref, watch } from 'vue';

import { Button, Drawer, Input, InputNumber, message, Select, Switch, Tag } from 'ant-design-vue';

import { articleApi } from '#/api/core/website/article';
import {
  categoryWebsiteProductApi,
  getWebsiteProductCategoriesApi,
  getWebsiteProductListApi,
  getWebsiteProductSkusApi,
  quantityWebsiteProductApi,
  recommendWebsiteProductApi,
  saveWebsiteProductSkuPricesApi,
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

const catOptions = ref<{ id: number; name: string }[]>([]);
const skuList = ref<any[]>([]);
/** 推荐/相关产品/相关文章/SEO */
const formExtra = reactive({
  isRecommend: 0,
  relatedProductIds: [] as number[],
  relatedArticleIds: [] as number[],
  seoTitle: '',
  seoKeywords: '',
  seoDescription: '',
});
const relatedOptions = ref<{ id: number; name: string }[]>([]);
const articleOptions = ref<{ id: number; title: string }[]>([]);
const productId = ref<number>(0);
const totalStock = ref<number>(0);
const productName = ref('');

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
        .map((r: any) => ({ id: Number(r.productId), name: r.productName || `产品 ${r.productId}` }));
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

function close() {
  emit('update:open', false);
}

async function onSave() {
  if (form.quantity < 0) {
    message.warning('展示数量不能为负数');
    return;
  }
  const rowId = Number(props.row?.id);
  // 依次保存：数量（后端钳制不超库存）/ 栏目分类 / SKU 展示 / 零售价 / 排序 / 状态
  await quantityWebsiteProductApi(rowId, form.quantity);
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
          v-model:value="form.quantity"
          class="w-40"
        />
        <span class="ml-2 text-xs text-[hsl(var(--muted-foreground))]">
          保存时自动钳制为不超过库存
        </span>
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
        <div class="mb-1 flex items-center gap-2">
          <span class="text-sm font-medium">展示 SKU</span>
          <Tag v-if="skuList.length === 0" color="default">单品（未配置 SKU）</Tag>
        </div>
        <template v-if="skuList.length > 0">
          <div class="mb-2 flex items-center gap-2">
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
              蓝色 = 前台展示；零售价仅作用于前台在线销售，不影响仓储价格
            </span>
          </div>
          <div class="flex flex-col gap-2 rounded-md border border-[hsl(var(--border))] p-3">
            <div
              v-for="s in skuList"
              :key="s.id"
              class="flex flex-wrap items-center gap-2"
            >
              <Tag
                class="cursor-pointer select-none"
                :color="form.skuIds.length === 0 || form.skuIds.includes(Number(s.id)) ? 'blue' : 'default'"
                @click="toggleSku(Number(s.id))"
              >
                {{ s.label || s.skuCode || `SKU ${s.id}` }}
              </Tag>
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
        </template>
      </div>

      <div>
        <div class="mb-1 text-sm font-medium">相关产品</div>
        <Select
          v-model:value="formExtra.relatedProductIds"
          mode="multiple"
          :options="relatedOptions"
          :field-names="{ label: 'name', value: 'id' }"
          placeholder="选择关联上架的其他产品"
          allow-clear
          class="w-full"
        />
      </div>

      <div>
        <div class="mb-1 text-sm font-medium">相关文章</div>
        <Select
          v-model:value="formExtra.relatedArticleIds"
          mode="multiple"
          :options="articleOptions"
          :field-names="{ label: 'title', value: 'id' }"
          placeholder="选择关联的文章"
          allow-clear
          class="w-full"
        />
      </div>

      <div>
        <div class="mb-1 text-sm font-medium">SEO 设置（仅作用于前台产品详情页）</div>
        <div class="flex flex-col gap-2">
          <Input v-model:value="formExtra.seoTitle" placeholder="SEO 标题（留空使用产品名称）" allow-clear />
          <Input v-model:value="formExtra.seoKeywords" placeholder="SEO 关键词（留空使用产品默认）" allow-clear />
          <Input v-model:value="formExtra.seoDescription" placeholder="SEO 描述（留空使用产品默认）" allow-clear />
        </div>
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

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="close">取消</Button>
        <Button type="primary" @click="onSave">保存</Button>
      </div>
    </template>
  </Drawer>
</template>
