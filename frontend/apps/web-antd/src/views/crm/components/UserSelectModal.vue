<script lang="ts" setup>
/**
 * 员工选择弹窗组件
 *
 * 用法：
 * <UserSelectModal v-model:visible="visible" @select="onSelect" />
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { LucideSearch } from '@vben/icons';

import {
  Button,
  Input,
  Modal,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getUserListApi } from '#/api/core/system/user';

const props = withDefaults(defineProps<{
  /** 弹窗是否可见 */
  visible: boolean;
  /** 额外的过滤条件 */
  extraParams?: Record<string, any>;
  /** 弹窗宽度 */
  width?: string | number;
  /** 需要排除（禁用）的用户ID列表 */
  excludeIds?: number[];
}>(), {
  width: '780px',
  excludeIds: () => [],
});

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'select', row: any): void;
}>();

const innerVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

const keywords = ref('');

const gridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  height: 420,
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const result = await getUserListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          userName: keywords.value || undefined,
          ...props.extraParams,
        });
        return result;
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    { title: '用户名', field: 'userName', width: 140, headerAlign: 'center', align: 'center' },
    {
      title: '姓名', field: 'nickName', minWidth: 120,
      formatter: ({ row }: any) => row.nickName || row.realName || row.name || row.userName || '-',
    },
    {
      title: '部门', field: 'deptName', width: 140,
      formatter: ({ cellValue, row }: any) => cellValue || row.departmentName || row.dept || '-',
    },
    {
      title: '手机号', field: 'mobile', width: 130,
      formatter: ({ cellValue, row }: any) => cellValue || row.phone || '-',
    },
    {
      title: '状态', field: 'status', width: 80,
      slots: { default: 'statusSlot' },
    },
    {
      title: '操作', field: 'action', fixed: 'right', slots: { default: 'action' }, width: 80,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  keywords.value = '';
  gridApi.query();
}

function isExcluded(row: any): boolean {
  return props.excludeIds.includes(Number(row.id));
}

function handleSelect(row: any) {
  if (isExcluded(row)) return;
  emit('select', row);
}

function handleRowDblClick({ row }: { row: any }) {
  handleSelect(row);
}

watch(() => props.visible, (val) => {
  if (val) {
    keywords.value = '';
    setTimeout(() => gridApi.query(), 100);
  }
});
</script>

<template>
  <Modal
    :open="innerVisible"
    title="选择员工"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    @cancel="innerVisible = false"
  >
    <div class="flex items-center gap-2 mb-3">
      <Input
        v-model:value="keywords"
        placeholder="输入姓名/用户名搜索"
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

    <Grid @row-dblclick="handleRowDblClick">
      <template #statusSlot="{ row }">
        <Tag :color="row.status === 1 ? 'green' : 'default'" size="small">
          {{ row.status === 1 ? '启用' : '禁用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button v-if="!isExcluded(row)" type="primary" size="small" @click="handleSelect(row)">选择</Button>
        <Button v-else size="small" disabled>已添加</Button>
      </template>
    </Grid>

    <div class="mt-2 text-xs text-gray-400 text-right">
      提示：双击行可快速选择
    </div>
  </Modal>
</template>

<style scoped>
:deep(.vxe-table--body-wrapper) {
  cursor: pointer;
}
:deep(.vxe-table--body-wrapper .vxe-body--row:hover td) {
  background-color: #e6f4ff !important;
}
</style>
