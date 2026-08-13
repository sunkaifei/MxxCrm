<script lang="ts" setup>
/**
 * 库存流水抽屉
 *
 * 用法：
 * <StockLogDrawer v-model:visible="visible" :product-id="productId" :product-name="productName" />
 */
import { ref, watch } from 'vue';

import { Drawer, Tag, Empty } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getStockLogListApi } from '#/api/core/product/stock-log';
import { $t } from '#/locales';

const props = defineProps<{
  visible: boolean;
  productId?: number | null;
  productName?: string;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

// 变动类型标签映射
const changeTypeMap: Record<string, { label: string; color: string }> = {
  inbound: { label: $t('page.inventory.inbound.title'), color: 'green' },
  outbound: { label: $t('page.inventory.outbound.title'), color: 'red' },
  transfer_in: { label: $t('page.inventory.changeType.transferIn'), color: 'blue' },
  transfer_out: { label: $t('page.inventory.changeType.transferOut'), color: 'orange' },
  check: { label: $t('page.inventory.changeType.check'), color: 'purple' },
  freeze: { label: $t('page.inventory.changeType.freeze'), color: 'geekblue' },
  unfreeze: { label: $t('page.inventory.changeType.unfreeze'), color: 'cyan' },
  setup: { label: $t('page.inventory.changeType.setup'), color: 'default' },
  adjust: { label: $t('page.inventory.changeType.adjust'), color: 'warning' },
};

function getChangeTypeTag(type: string) {
  return changeTypeMap[type] || { label: type, color: 'default' };
}

const gridOptions: VxeGridProps = {
  height: 'auto',
  pagerConfig: { pageSize: 10 },
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        if (!props.productId) return { items: [], total: 0 };
        const res: any = await getStockLogListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productId: props.productId,
        });
        const items = res?.list ?? res?.items ?? [];
        return { items, total: res?.total ?? 0 };
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 50 },
    {
      title: '变动类型',
      field: 'changeType',
      width: 100,
      slots: { default: 'changeType' },
    },
    { title: '仓库', field: 'warehouseName', width: 100 },
    { title: '变动前', field: 'quantityBefore', width: 80, align: 'right' },
    { title: '变动量', field: 'changeQuantity', width: 80, align: 'right', slots: { default: 'changeQuantity' } },
    { title: '变动后', field: 'quantityAfter', width: 80, align: 'right' },
    { title: '业务单号', field: 'bizNo', minWidth: 140, ellipsis: true },
    { title: '时间', field: 'createTime', width: 150 },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

watch(
  () => props.visible,
  (val) => {
    if (val && props.productId) {
      gridApi.query();
    }
  },
);
</script>

<template>
  <Drawer
    :open="visible"
    :title="`库存流水 - ${productName || ''}`"
    :width="800"
    placement="right"
    @close="emit('update:visible', false)"
  >
    <Grid>
      <template #changeType="{ row }">
        <Tag :color="getChangeTypeTag(row.changeType).color" :bordered="false">
          {{ getChangeTypeTag(row.changeType).label }}
        </Tag>
      </template>

      <template #changeQuantity="{ row }">
        <span :class="Number(row.changeQuantity) > 0 ? 'text-green-600 font-medium' : 'text-red-500 font-medium'">
          {{ Number(row.changeQuantity) > 0 ? '+' : '' }}{{ row.changeQuantity }}
        </span>
      </template>
    </Grid>
  </Drawer>
</template>
