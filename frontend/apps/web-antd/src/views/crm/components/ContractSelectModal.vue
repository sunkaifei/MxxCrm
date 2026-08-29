<script lang="ts" setup>
/**
 * 合同选择弹窗组件
 * 用于发票等单据关联合同场景：展示当前用户自己签订的合同，并标注已开票金额
 *
 * 用法：
 * <ContractSelectModal v-model:visible="visible" @select="onSelect" />
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { LucideSearch } from '@vben/icons';

import { Button, Input, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getContractSelectListApi } from '#/api/core/crm/contract';

const props = withDefaults(
  defineProps<{
    visible: boolean;
    width?: number | string;
  }>(),
  {
    width: '980px',
  },
);

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'select', row: any): void;
}>();

const innerVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

const keywords = ref('');

const statusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '已签署',
  3: '执行中',
  4: '已完成',
  5: '已终止',
};
const statusColorMap: Record<number, string> = {
  1: 'default',
  2: 'blue',
  3: 'cyan',
  4: 'green',
  5: 'red',
};

function formatAmount(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  const num = Number(val);
  if (Number.isNaN(num)) return val;
  return num.toLocaleString('zh-CN', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  });
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  height: 440,
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const result = await getContractSelectListApi({
          keywords: keywords.value || undefined,
          page: page.currentPage,
          pageSize: page.pageSize,
        });
        return result;
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    {
      title: '合同编号',
      field: 'contractNo',
      width: 150,
      headerAlign: 'center',
      align: 'center',
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '合同标题',
      field: 'title',
      minWidth: 200,
      align: 'left',
      slots: { default: 'title' },
    },
    {
      title: '客户名称',
      field: 'customerName',
      minWidth: 160,
      align: 'left',
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '合同金额',
      field: 'totalAmount',
      width: 120,
      align: 'right',
      formatter: ({ row }: any) => formatAmount(row.totalAmount ?? row.amount),
    },
    {
      title: '已开票金额',
      field: 'invoicedAmount',
      width: 130,
      align: 'right',
      slots: { default: 'invoicedSlot' },
    },
    {
      title: '状态',
      field: 'status',
      width: 90,
      align: 'center',
      slots: { default: 'statusSlot' },
    },
    {
      title: '负责人',
      field: 'assignedToName',
      width: 90,
      formatter: ({ cellValue }: any) => cellValue || '-',
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

/** 选择合同 */
function handleSelect(row: any) {
  emit('select', row);
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
    title="选择合同"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    :z-index="2100"
    @cancel="innerVisible = false"
  >
    <!-- 搜索栏 -->
    <div class="mb-3 flex items-center gap-2">
      <Input
        v-model:value="keywords"
        placeholder="输入合同标题/编号搜索"
        allow-clear
        class="flex-1"
        @press-enter="handleSearch"
      >
        <template #prefix>
          <LucideSearch class="h-4 w-4 text-gray-400" />
        </template>
      </Input>
      <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">
        搜索
      </Button>
      <Button @click="handleReset">重置</Button>
    </div>

    <!-- 合同列表表格 -->
    <Grid @row-dblclick="handleRowDblClick">
      <template #title="{ row }">
        <div>
          <span class="font-medium text-blue-600">{{ row.title || '-' }}</span>
          <div v-if="row.orderNo" class="mt-0.5 text-xs text-gray-400">
            关联订单：{{ row.orderNo }}
          </div>
        </div>
      </template>

      <template #invoicedSlot="{ row }">
        <span
          :class="
            Number(row.invoicedAmount) > 0
              ? 'font-medium text-orange-600'
              : 'text-gray-400'
          "
        >
          {{ formatAmount(row.invoicedAmount) }}
        </span>
      </template>

      <template #statusSlot="{ row }">
        <Tag :color="statusColorMap[row.status] || 'default'" size="small">
          {{ statusLabelMap[row.status] || row.status || '-' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button type="primary" size="small" @click="handleSelect(row)">
          选择
        </Button>
      </template>
    </Grid>

    <div class="mt-2 text-right text-xs text-gray-400">
      提示：仅显示我签订的合同，双击行可快速选择
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
