<script lang="ts" setup>
/**
 * 商机选择弹窗组件
 * 用于在合同/订单等场景中选择关联商机
 *
 * 用法：
 * <OpportunitySelectModal v-model:visible="visible" :customer-id="customerId" @select="onSelect" />
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
import { getOpportunityListApi } from '#/api/core/crm/opportunity';

const props = withDefaults(defineProps<{
  /** 弹窗是否可见 */
  visible: boolean;
  /** 按客户ID过滤商机（可选） */
  customerId?: number | undefined;
  /** 弹窗宽度 */
  width?: string | number;
}>(), {
  width: '860px',
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

// 币种标签映射
const currencyLabelMap: Record<number, string> = {
  1: 'CNY', 2: 'USD', 3: 'EUR', 4: 'GBP', 5: 'JPY', 6: 'HKD', 7: 'AUD',
};

// 阶段映射
const stageLabelMap: Record<number, string> = {
  0: '资格审查', 1: '需求分析', 2: '方案报价', 3: '商务谈判', 4: '已成交', 5: '已输单',
};
const stageColorMap: Record<number, string> = {
  0: 'default', 1: 'processing', 2: 'warning', 3: 'processing', 4: 'success', 5: 'error',
};
// 来源映射
const sourceLabelMap: Record<number, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
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
          // 只展示当前用户负责的商机，不可选择他人商机
          listType: 'my',
        };
        if (props.customerId) params.customerId = props.customerId;
        return await getOpportunityListApi(params);
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    { title: '商机名称', field: 'title', minWidth: 200, align: 'left', slots: { default: 'titleSlot' } },
    { title: '客户', field: 'customerName', width: 150 },
    {
      title: '销售阶段', field: 'stage', width: 110, slots: { default: 'stageSlot' },
    },
    {
      title: '预算金额', field: 'amount', width: 130, align: 'right', slots: { default: 'amountSlot' },
    },
    {
      title: '概率', field: 'probability', width: 70, align: 'center',
      formatter: ({ cellValue }: any) => (cellValue == null ? '-' : `${cellValue}%`),
    },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '预计成交日', field: 'expectedCloseDate', width: 120 },
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

/** 选择商机 */
function handleSelect(row: any) {
  emit('select', row);
  innerVisible.value = false;
}

/** 双击行也触发选择 */
function handleRowDblClick({ row }: { row: any }) {
  handleSelect(row);
}

// 弹窗打开时自动加载数据；customerId 变化时也刷新
watch([() => props.visible, () => props.customerId], ([visible]) => {
  if (visible) {
    keywords.value = '';
    setTimeout(() => gridApi.query(), 100);
  }
});
</script>

<template>
  <Modal
    :open="innerVisible"
    :title="customerId ? `选择「${customerId}」客户的商机` : '选择关联商机'"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    @cancel="innerVisible = false"
  >
    <!-- 搜索栏 -->
    <div class="flex items-center gap-2 mb-3">
      <Input
        v-model:value="keywords"
        placeholder="输入商机名称搜索"
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

    <!-- 商机列表表格 -->
    <Grid @row-dblclick="handleRowDblClick">
      <template #titleSlot="{ row }">
        <span class="text-blue-600 font-medium">{{ row.title || row.name || '-' }}</span>
      </template>

      <template #stageSlot="{ row }">
        <Tag :color="stageColorMap[row.stage] || 'default'" size="small">
          {{ stageLabelMap[row.stage] || row.stage || '-' }}
        </Tag>
      </template>

      <template #amountSlot="{ row }">
        <span v-if="row.amount != null" class="font-medium">
          {{ currencyLabelMap[row.currency] || '' }} {{ Number(row.amount).toLocaleString('zh-CN', { minimumFractionDigits: 2 }) }}
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
