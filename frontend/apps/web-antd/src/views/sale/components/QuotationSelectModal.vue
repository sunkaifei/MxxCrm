<script lang="ts" setup>
/**
 * 报价单选择弹窗组件
 * 用于在订单等场景中选择关联报价单
 *
 * 用法：
 * <QuotationSelectModal v-model:visible="visible" @select="onSelect" />
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { LucideSearch } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import { Button, Input, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getQuotationListApi } from '#/api';

const props = withDefaults(
  defineProps<{
    /** 弹窗是否可见 */
    visible: boolean;
    /** 弹窗宽度 */
    width?: number | string;
  }>(),
  {
    width: '860px',
  },
);

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

// 币种符号映射
const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
  7: 'A$',
};

// 审批状态映射
const approvalStatusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '草稿',
  2: '审批中',
  3: '已通过',
  4: '已驳回',
};
const approvalStatusColorMap: Record<number, string> = {
  0: 'default',
  1: 'default',
  2: 'processing',
  3: 'success',
  4: 'error',
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  height: 420,
  cellConfig: { isHover: true } as any,
  rowConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: keywords.value || undefined,
          // 只展示当前用户负责的报价单
          listType: 'my',
        };
        return await getQuotationListApi(params);
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    {
      title: '报价编号',
      field: 'quotationNo',
      width: 150,
      align: 'left',
      slots: { default: 'quotationNoSlot' },
    },
    { title: '标题', field: 'title', minWidth: 200, align: 'left' },
    { title: '客户名称', field: 'customerName', width: 150 },
    {
      title: '报价金额',
      field: 'grandTotal',
      width: 130,
      align: 'right',
      slots: { default: 'amountSlot' },
    },
    {
      title: '审批状态',
      field: 'approvalStatus',
      width: 100,
      align: 'center',
      slots: { default: 'statusSlot' },
    },
    { title: '报价日期', field: 'quotationDate', width: 120 },
    {
      title: '创建时间',
      field: 'createTime',
      width: 150,
      slots: { default: 'createdAt' },
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 80,
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

/** 选择报价单 */
function handleSelect(row: any) {
  emit('select', row);
  innerVisible.value = false;
}

/** 双击行也触发选择 */
function handleRowDblClick({ row }: { row: any }) {
  handleSelect(row);
}

// 弹窗打开时自动加载数据
watch(
  () => props.visible,
  (val) => {
    if (val) {
      keywords.value = '';
      setTimeout(() => gridApi.query(), 100);
    }
  },
);
</script>

<template>
  <Modal
    :open="innerVisible"
    title="选择关联报价单"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    @cancel="innerVisible = false"
  >
    <!-- 搜索栏 -->
    <div class="flex items-center gap-2 mb-3">
      <Input
        v-model:value="keywords"
        placeholder="输入报价单号/标题搜索"
        allow-clear
        class="flex-1"
        @press-enter="handleSearch"
      >
        <template #prefix>
          <LucideSearch
            class="w-4 h-4"
            style="color: hsl(var(--muted-foreground))"
          />
        </template>
      </Input>
      <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">
        搜索
      </Button>
      <Button @click="handleReset">重置</Button>
    </div>

    <!-- 报价单列表表格 -->
    <Grid @row-dblclick="handleRowDblClick">
      <template #quotationNoSlot="{ row }">
        <span style="color: hsl(var(--primary))" class="font-medium">{{
          row.quotationNo || '-'
        }}</span>
      </template>

      <template #amountSlot="{ row }">
        <span v-if="row.grandTotal != null" class="font-medium">
          {{ currencySymbolMap[row.currency] || '¥' }}
          {{
            Number(row.grandTotal).toLocaleString('zh-CN', {
              minimumFractionDigits: 2,
            })
          }}
        </span>
        <span v-else style="color: hsl(var(--muted-foreground) / 50%)">-</span>
      </template>

      <template #statusSlot="{ row }">
        <Tag
          :color="approvalStatusColorMap[row.approvalStatus] || 'default'"
          size="small"
        >
          {{ approvalStatusLabelMap[row.approvalStatus] || '草稿' }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button type="primary" size="small" @click="handleSelect(row)">
          选择
        </Button>
      </template>
    </Grid>

    <div
      class="mt-2 text-xs text-right"
      style="color: hsl(var(--muted-foreground) / 60%)"
    >
      提示：双击行可快速选择
    </div>
  </Modal>
</template>

<style scoped>
/* 行悬停高亮 */
:deep(.vxe-table--body-wrapper) {
  cursor: pointer;
}

:deep(.vxe-table--body-wrapper .vxe-body--row:hover td) {
  background-color: hsl(var(--primary) / 6%) !important;
}
</style>
