<script lang="ts" setup>
/**
 * 产品详情抽屉（只读）
 *
 * 用法：
 * <ProductDetailDrawer v-model:visible="visible" :product-id="productId" />
 *
 * 或通过 productName/warehouseName 列表点击触发：
 * <span class="detail-link" @click="openProductDetail(row.productId)">{{ row.productName }}</span>
 */
import { computed, ref, watch } from 'vue';

import {
  Button,
  Descriptions,
  DescriptionsItem,
  Divider,
  Drawer,
  Empty,
  Image,
  Spin,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { getProductInfoApi } from '#/api/core/product/product';

const props = defineProps<{
  productId?: null | number;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const isFullscreen = ref(false);
const drawerWidth = computed(() => (isFullscreen.value ? '100%' : '75%'));

const loading = ref(false);
const detail = ref<any>(null);
const specs = ref<any[]>([]);

const productTypeMap: Record<number, { color: string; text: string }> = {
  1: { text: '实物商品', color: 'blue' },
  2: { text: '虚拟商品', color: 'cyan' },
  3: { text: '服务', color: 'purple' },
  4: { text: '订阅', color: 'magenta' },
};

const specTypeMap: Record<string, string> = {
  single: '单规格',
  multiple: '多规格',
};

function formatPrice(val: any): string {
  if (val === null || val === undefined || val === '') return '-';
  return `¥ ${Number(val).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function formatDimensions(val: any): string {
  if (!val) return '-';
  return val;
}

async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getProductInfoApi(id);
    detail.value = res?.product ?? res?.data?.product ?? res?.data ?? res;
    specs.value = res?.specs ?? res?.data?.specs ?? [];
  } catch {
    detail.value = null;
    specs.value = [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.visible, props.productId] as const,
  ([v, id]) => {
    if (v && id) {
      loadDetail(id);
    }
    if (!v) {
      detail.value = null;
      specs.value = [];
    }
  },
);
</script>

<template>
  <Drawer
    :open="visible"
    title="产品详情"
    :width="drawerWidth"
    placement="right"
    @close="emit('update:visible', false)"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <Button type="text" size="small" @click="isFullscreen = !isFullscreen">
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </Button>
      </Tooltip>
    </template>
    <Spin :spinning="loading">
      <div v-if="detail" class="space-y-1">
        <!-- 产品头部：图片 + 名称 -->
        <div class="flex items-start gap-4 mb-2">
          <div
            class="flex-shrink-0 w-[88px] h-[88px] rounded-lg overflow-hidden border border-solid border-border bg-muted/40 flex items-center justify-center"
          >
            <Image
              v-if="detail.imageUrl"
              :src="detail.imageUrl"
              :width="88"
              :height="88"
              style="object-fit: cover"
            />
            <div
              v-else
              class="w-full h-full flex items-center justify-center text-3xl text-muted-foreground/40"
            >
              <svg
                viewBox="0 0 24 24"
                width="36"
                height="36"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
              >
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <circle cx="8.5" cy="8.5" r="1.5" />
                <path d="M21 15l-5-5L5 21" />
              </svg>
            </div>
          </div>
          <div class="flex-1 min-w-0">
            <h2 class="text-lg font-semibold text-foreground mb-1 truncate">
              {{ detail.name || '-' }}
            </h2>
            <div class="flex flex-wrap items-center gap-1.5">
              <Tag
                v-if="detail.productType"
                :color="productTypeMap[detail.productType]?.color || 'default'"
              >
                {{
                  productTypeMap[detail.productType]?.text ||
                  `类型${detail.productType}`
                }}
              </Tag>
              <Tag :color="detail.isActive ? 'green' : 'red'">
                {{ detail.isActive ? '在售' : '停售' }}
              </Tag>
              <Tag v-if="detail.abcClass" color="gold">
                ABC {{ detail.abcClass }}
              </Tag>
            </div>
            <div class="text-xs text-muted-foreground mt-1">
              编号：{{ detail.productNo || '-' }}
              <span v-if="detail.sku" class="ml-3">SKU：{{ detail.sku }}</span>
            </div>
          </div>
        </div>

        <Divider style="margin: 12px 0" />

        <!-- 价格信息 -->
        <div class="grid grid-cols-3 gap-2 mb-3">
          <div
            class="rounded-lg border border-solid border-border p-3 text-center bg-muted/20"
          >
            <div class="text-xs text-muted-foreground mb-1">成本价</div>
            <div class="text-base font-semibold text-foreground">
              {{ formatPrice(detail.costPrice) }}
            </div>
          </div>
          <div
            class="rounded-lg border border-solid border-border p-3 text-center bg-muted/20"
          >
            <div class="text-xs text-muted-foreground mb-1">销售价</div>
            <div class="text-base font-semibold text-primary">
              {{ formatPrice(detail.salePrice) }}
            </div>
          </div>
          <div
            class="rounded-lg border border-solid border-border p-3 text-center bg-muted/20"
          >
            <div class="text-xs text-muted-foreground mb-1">市场价</div>
            <div class="text-base font-semibold text-muted-foreground">
              {{ formatPrice(detail.marketPrice) }}
            </div>
          </div>
        </div>

        <!-- 基本信息 -->
        <Descriptions
          title="基本信息"
          :column="2"
          size="small"
          bordered
          :label-style="{ width: '100px' }"
        >
          <DescriptionsItem label="产品编号">
            {{ detail.productNo || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="SKU编码">
            {{ detail.sku || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="条码">
            {{ detail.barcode || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="单位">
            {{ detail.unit || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="规格类型">
            {{ specTypeMap[detail.specType] || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="币种">
            {{ detail.currency || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="重量(kg)">
            {{ detail.weight ?? '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="尺寸">
            {{ formatDimensions(detail.dimensions) }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 库存预警参数 -->
        <Descriptions
          title="库存参数"
          :column="2"
          size="small"
          bordered
          class="mt-4"
          :label-style="{ width: '100px' }"
        >
          <DescriptionsItem label="安全库存">
            {{ detail.safetyStock ?? '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="最大库存">
            {{ detail.maxStock ?? '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="预警天数">
            {{ detail.warningDays ?? '-' }}天
          </DescriptionsItem>
          <DescriptionsItem label="是否自产">
            {{ detail.isSelfProduced === 1 ? '是' : '否' }}
          </DescriptionsItem>
          <DescriptionsItem label="生产提前期">
            {{
              detail.productionLeadTime ? `${detail.productionLeadTime}天` : '-'
            }}
          </DescriptionsItem>
          <DescriptionsItem label="虚拟库存">
            {{ detail.isVirtualStock === 1 ? '是' : '否' }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 多规格信息 -->
        <Descriptions
          v-if="specs && specs.length > 0"
          title="规格信息"
          :column="1"
          size="small"
          bordered
          class="mt-4"
        >
          <DescriptionsItem
            v-for="(spec, idx) in specs"
            :key="idx"
            :label="spec.specName || spec.name || `规格${idx + 1}`"
          >
            <span class="font-mono text-sm">{{
              spec.skuCode || spec.sku || '-'
            }}</span>
            <span class="ml-3 text-muted-foreground"
              >¥{{ spec.salePrice ?? spec.price ?? '-' }}</span
            >
            <span class="ml-3 text-muted-foreground"
              >库存: {{ spec.stock ?? '-' }}</span
            >
          </DescriptionsItem>
        </Descriptions>

        <!-- 产品描述 -->
        <div v-if="detail.description" class="mt-4">
          <div class="text-sm font-medium text-foreground mb-2">产品描述</div>
          <div
            class="text-sm text-muted-foreground leading-relaxed rounded-lg bg-muted/20 p-3"
          >
            {{ detail.description }}
          </div>
        </div>

        <!-- 创建/更新时间 -->
        <div class="mt-4 text-xs text-muted-foreground flex justify-between">
          <span>创建时间：{{ detail.createTime || '-' }}</span>
          <span>更新时间：{{ detail.updateTime || '-' }}</span>
        </div>
      </div>

      <Empty v-else-if="!loading" description="暂无数据" />
    </Spin>
  </Drawer>
</template>

<style scoped>
:deep(.ant-descriptions-header) {
  margin-bottom: 8px;
}
</style>
