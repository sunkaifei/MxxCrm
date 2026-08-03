<script lang="ts" setup>
/**
 * 联系人选择弹窗组件
 * 用于在订单/合同等场景中选择联系人
 *
 * 用法：
 * <ContactSelectModal v-model:visible="visible" :customer-id="customerId" @select="onSelect" />
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
import { getContactListApi } from '#/api/core/crm/contact';

const props = withDefaults(defineProps<{
  /** 弹窗是否可见 */
  visible: boolean;
  /** 按客户ID过滤联系人（可选） */
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

// 角色映射
const roleLabelMap: Record<number, string> = {
  1: '首要', 2: '普通', 3: '其他',
};
const roleColorMap: Record<number, string> = {
  1: 'success', 2: 'default', 3: 'default',
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  height: 420,
  cellConfig: { isHover: true } as any,
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
        };
        if (props.customerId) params.customerId = props.customerId;
        return await getContactListApi(params);
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    { title: '姓名', field: 'name', width: 120, align: 'left', slots: { default: 'nameSlot' } },
    { title: '当前公司', field: 'companyName', minWidth: 180, align: 'left' },
    { title: '职位', field: 'title', width: 120 },
    {
      title: '角色', field: 'roleType', width: 80, align: 'center', slots: { default: 'roleSlot' },
    },
    { title: '手机', field: 'mobile', width: 130 },
    { title: '邮箱', field: 'email', width: 180 },
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

/** 选择联系人 */
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
    :title="customerId ? '选择该客户的联系人' : '选择联系人'"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    @cancel="innerVisible = false"
  >
    <!-- 搜索栏 -->
    <div class="flex items-center gap-2 mb-3">
      <Input
        v-model:value="keywords"
        placeholder="输入联系人姓名/手机/邮箱搜索"
        allow-clear
        class="flex-1"
        @press-enter="handleSearch"
      >
        <template #prefix>
          <LucideSearch class="w-4 h-4" style="color: hsl(var(--muted-foreground))" />
        </template>
      </Input>
      <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">搜索</Button>
      <Button @click="handleReset">重置</Button>
    </div>

    <!-- 联系人列表表格 -->
    <Grid @row-dblclick="handleRowDblClick">
      <template #nameSlot="{ row }">
        <span style="color: hsl(var(--primary))" class="font-medium">{{ row.name || '-' }}</span>
      </template>

      <template #roleSlot="{ row }">
        <Tag :color="roleColorMap[row.roleType] || 'default'" size="small">
          {{ roleLabelMap[row.roleType] || row.roleType || '-' }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button type="primary" size="small" @click="handleSelect(row)">选择</Button>
      </template>
    </Grid>

    <div class="mt-2 text-xs text-right" style="color: hsl(var(--muted-foreground) / 0.6)">
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
  background-color: hsl(var(--primary) / 0.06) !important;
}
</style>
