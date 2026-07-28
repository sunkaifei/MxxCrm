<script lang="ts" setup>
/**
 * 订单选择弹窗组件
 * 用于在合同签署等场景中选择关联订单（只显示当前用户负责的订单）
 *
 * 用法：
 * <OrderSelectModal v-model:visible="visible" @select="onSelect" />
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { LucideSearch } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Input,
  Modal,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getOrderListApi } from '#/api/core/sale/order';

const props = withDefaults(defineProps<{
  /** 弹窗是否可见 */
  visible: boolean;
  /** 弹窗宽度 */
  width?: string | number;
}>(), {
  width: '900px',
});

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'select', row: any): void;
}>();

// 内部可见状态
const innerVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

// 搜索表单
const keywords = ref('');

// 订单状态映射
const orderStatusLabelMap: Record<number, string> = {
  1: '草稿', 2: '待确认', 3: '已确认', 4: '备货中',
  5: '部分发货', 6: '已发货', 7: '已取消', 8: '已交付',
  9: '已签收', 10: '已完成', 11: '已作废',
};
const orderStatusColorMap: Record<number, string> = {
  1: 'default', 2: 'blue', 3: 'blue', 4: 'orange',
  5: 'cyan', 6: 'purple', 7: 'red', 8: 'cyan',
  9: 'green', 10: 'blue', 11: 'red',
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  height: 420,
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: keywords.value || undefined,
          // 只展示当前用户负责的订单
          listType: 'my',
        };
        return await getOrderListApi(params);
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    { title: '订单编号', field: 'orderNo', width: 160 },
    { title: '订单标题', field: 'title', minWidth: 180, align: 'left', slots: { default: 'titleSlot' } },
    { title: '客户', field: 'customerName', width: 150 },
    { title: '联系人', field: 'contactName', width: 100 },
    {
      title: '订单状态', field: 'orderStatus', width: 100, slots: { default: 'statusSlot' },
    },
    {
      title: '订单金额', field: 'totalAmount', width: 130, align: 'right', slots: { default: 'amountSlot' },
    },
    { title: '下单日期', field: 'orderDate', width: 120 },
    { title: '创建时间', field: 'createTime', width: 150, slots: { default: 'createdAt' } },
    {
      title: '操作', field: 'action', fixed: 'right', slots: { default: 'action' }, width: 80,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

/** 搜索 */
function handleSearch() {
  gridApi.query();
}

/** 重置搜索 */
function handleReset() {
  keywords.value = '';
  gridApi.query();
}

/** 选择订单 */
function handleSelect(row: any) {
  emit('select', row);
}

/** 双击行也触发选择 */
function handleRowDblClick({ row }: { row: any }) {
  handleSelect(row);
}

// 弹窗打开时自动加载数据
watch(() => props.visible, (visible) => {
  if (visible) {
    keywords.value = '';
    setTimeout(() => gridApi.query(), 100);
  }
});
</script>

<template>
  <Modal
    :open="innerVisible"
    title="选择关联订单"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    @cancel="innerVisible = false"
  >
    <!-- 搜索栏 -->
    <div class="flex items-center gap-2 mb-3">
      <Input
        v-model:value="keywords"
        placeholder="输入订单名称搜索"
        allow-clear
        class="flex-1"
        @press-enter="handleSearch"
      >
        <template #prefix>
          <LucideSearch class="w-4 h-4 text-gray-400" />
        </template>
      </Input>
      <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">搜索</Button>
      <Button @click="handleReset">重置</Button>
    </div>

    <!-- 订单列表表格 -->
    <Grid @row-dblclick="handleRowDblClick">
      <template #titleSlot="{ row }">
        <span class="text-blue-600 font-medium">{{ row.title || '-' }}</span>
      </template>

      <template #statusSlot="{ row }">
        <Tag :color="orderStatusColorMap[row.orderStatus] || 'default'" size="small">
          {{ orderStatusLabelMap[row.orderStatus] || row.orderStatus || '-' }}
        </Tag>
      </template>

      <template #amountSlot="{ row }">
        <span v-if="row.totalAmount != null" class="font-medium">
          ¥{{ Number(row.totalAmount).toLocaleString('zh-CN', { minimumFractionDigits: 2 }) }}
        </span>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button type="primary" size="small" @click="handleSelect(row)">选择</Button>
      </template>
    </Grid>

    <div class="mt-2 text-xs text-gray-400 text-right">
      提示：双击行可快速选择
    </div>
  </Modal>
</template>

<style scoped>
/* 行悬停高亮 - 可选择状态 */
:deep(.vxe-table--body-wrapper) {
  cursor: pointer;
}
:deep(.vxe-table--body-wrapper .vxe-body--row:hover td) {
  background-color: #e6f4ff !important;
}
</style>
