<script lang="ts" setup>
/**
 * 仓库详情抽屉（只读）
 *
 * 用法：
 * <WarehouseDetailDrawer v-model:visible="visible" :warehouse-id="warehouseId" />
 */
import { computed, ref, watch } from 'vue';

import {
  Button,
  Descriptions,
  DescriptionsItem,
  Divider,
  Drawer,
  Empty,
  Spin,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { getWarehouseInfoApi } from '#/api/core/product/warehouse';

const props = defineProps<{
  visible: boolean;
  warehouseId?: null | number;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const loading = ref(false);
const detail = ref<any>(null);

const isFullscreen = ref(false);
const drawerWidth = computed(() => (isFullscreen.value ? '100%' : '75%'));

const warehouseTypeMap: Record<number, { color: string; text: string }> = {
  1: { text: '原材料仓', color: 'orange' },
  2: { text: '成品仓', color: 'green' },
  3: { text: '半成品仓', color: 'blue' },
  4: { text: '退货仓', color: 'red' },
  5: { text: '中转仓', color: 'cyan' },
};

const pickingStrategyMap: Record<string, string> = {
  fifo: '先进先出 (FIFO)',
  fefo: '先到期先出 (FEFO)',
  lifo: '后进先出 (LIFO)',
  assigned: '指定库位',
};

const logisticsMap: Record<string, string> = {
  sf: '顺丰速运',
  jd: '京东物流',
  zto: '中通快递',
  yto: '圆通速递',
  yd: '韵达快递',
  sto: '申通快递',
  htky: '百世快递',
  db: '德邦物流',
  ems: 'EMS',
  jt: '极兔速递',
  pickup: '自提',
};

function formatLogistics(val: any): string {
  if (!val) return '-';
  return String(val)
    .split(',')
    .filter(Boolean)
    .map((v) => logisticsMap[v.trim()] || v.trim())
    .join('、');
}

async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getWarehouseInfoApi(id);
    detail.value = res?.data ?? res;
  } catch {
    detail.value = null;
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.visible, props.warehouseId] as const,
  ([v, id]) => {
    if (v && id) {
      loadDetail(id);
    }
    if (!v) {
      detail.value = null;
    }
  },
);
</script>

<template>
  <Drawer
    :open="visible"
    title="仓库详情"
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
        <!-- 仓库头部 -->
        <div class="flex items-center gap-3 mb-2">
          <div
            class="flex-shrink-0 w-[48px] h-[48px] rounded-lg bg-primary/10 flex items-center justify-center"
          >
            <svg
              viewBox="0 0 24 24"
              width="28"
              height="28"
              fill="none"
              stroke="hsl(var(--primary))"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M3 21V8l9-5 9 5v13" />
              <path d="M3 21h18" />
              <path d="M9 21v-7h6v7" />
              <path d="M7 11h.01M11 11h.01M15 11h.01" />
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <h2 class="text-lg font-semibold text-foreground mb-1 truncate">
              {{ detail.warehouseName || '-' }}
            </h2>
            <div class="flex flex-wrap items-center gap-1.5">
              <Tag
                v-if="detail.warehouseType"
                :color="
                  warehouseTypeMap[detail.warehouseType]?.color || 'default'
                "
              >
                {{
                  warehouseTypeMap[detail.warehouseType]?.text ||
                  `类型${detail.warehouseType}`
                }}
              </Tag>
              <Tag :color="detail.isActive ? 'green' : 'red'">
                {{ detail.isActive ? '启用' : '停用' }}
              </Tag>
              <span class="text-xs text-muted-foreground"
                >编码：{{ detail.code || '-' }}</span
              >
            </div>
          </div>
        </div>

        <Divider style="margin: 12px 0" />

        <!-- 统计卡片 -->
        <div class="grid grid-cols-3 gap-2 mb-3">
          <div
            class="rounded-lg border border-solid border-border p-3 text-center bg-muted/20"
          >
            <div class="text-xs text-muted-foreground mb-1">面积(㎡)</div>
            <div class="text-base font-semibold text-foreground">
              {{ detail.areaSqm ?? '-' }}
            </div>
          </div>
          <div
            class="rounded-lg border border-solid border-border p-3 text-center bg-muted/20"
          >
            <div class="text-xs text-muted-foreground mb-1">拣货策略</div>
            <div class="text-sm font-medium text-foreground">
              {{
                pickingStrategyMap[detail.pickingStrategy] ||
                detail.pickingStrategy ||
                '-'
              }}
            </div>
          </div>
          <div
            class="rounded-lg border border-solid border-border p-3 text-center bg-muted/20"
          >
            <div class="text-xs text-muted-foreground mb-1">排序</div>
            <div class="text-base font-semibold text-foreground">
              {{ detail.sortOrder ?? '-' }}
            </div>
          </div>
        </div>

        <!-- 基本信息 -->
        <Descriptions
          title="基本信息"
          :column="2"
          size="small"
          bordered
          :label-style="{ width: '90px' }"
        >
          <DescriptionsItem label="仓库名称">
            {{ detail.warehouseName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="仓库编码">
            {{ detail.code || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="仓库类型">
            <Tag
              v-if="detail.warehouseType"
              :color="warehouseTypeMap[detail.warehouseType]?.color"
            >
              {{ warehouseTypeMap[detail.warehouseType]?.text || '-' }}
            </Tag>
            <span v-else>-</span>
          </DescriptionsItem>
          <DescriptionsItem label="所属区域">
            {{ detail.region || '-' }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 联系信息 -->
        <Descriptions
          title="联系信息"
          :column="1"
          size="small"
          bordered
          class="mt-4"
          :label-style="{ width: '90px' }"
        >
          <DescriptionsItem label="仓库地址">
            {{ detail.address || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="联系人">
            {{ detail.contactPerson || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="联系电话">
            {{ detail.contactPhone || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="备用电话">
            {{ detail.backupPhone || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="物流类型">
            {{ formatLogistics(detail.logisticsTypes) }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 其他信息 -->
        <Descriptions
          v-if="detail.remark"
          title="备注"
          :column="1"
          size="small"
          bordered
          class="mt-4"
        >
          <DescriptionsItem label="备注内容">
            {{ detail.remark }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 时间信息 -->
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
