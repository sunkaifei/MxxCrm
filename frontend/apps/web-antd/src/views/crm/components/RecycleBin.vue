<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h } from 'vue';

import { LucideTrash2, LucideUndo2 } from '@vben/icons';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Modal, Popconfirm } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getRecycleListApi, purgeRecycleApi, restoreRecycleApi } from '#/api';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

// 内嵌回收站视图：由各模块列表页的"回收站"Tab 挂载，仅展示当前模块的回收数据
const props = defineProps<{ module: string }>();

const userStore = useUserStore();
const { isSuperAdmin } = useSuperAdminGuard();

async function handleRestore(row: any) {
  try {
    await restoreRecycleApi(props.module, Number(row.id));
    message.success('还原成功');
    gridApi.query();
  } catch {
    // 全局拦截器处理
  }
}

// 彻底删除：仅超管可见，二次确认（规划 G10）
function handlePurge(row: any) {
  Modal.confirm({
    title: '彻底删除',
    content: `彻底删除后数据无法恢复，确定删除「${row.title}」吗？`,
    okText: '确认删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await purgeRecycleApi(props.module, Number(row.id));
        message.success('已彻底删除');
        gridApi.query();
      } catch {
        // 全局拦截器处理
      }
    },
  });
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '输入标题关键词', allowClear: true },
    },
  ],
};

// 无数据/请求异常时固定 600px 空态高度（避免列表塌缩），有数据恢复内容自适应
function applyRecycleGridHeight(empty: boolean) {
  const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
  if (!gridEl) return;
  if (empty) {
    gridEl.style.setProperty('height', '600px', 'important');
  } else {
    gridEl.style.removeProperty('height');
  }
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        let result: any;
        try {
          result = await getRecycleListApi({
            pageNum: page.currentPage,
            pageSize: page.pageSize,
            module: props.module,
            ...formValues,
          });
        } catch (error) {
          // 请求异常同样按无数据处理，保持 600px 空态高度
          applyRecycleGridHeight(true);
          throw error;
        }
        // 无数据 600px（避免空态列表塌缩），有数据按内容自适应
        const items = (result as any)?.items ?? [];
        applyRecycleGridHeight(items.length === 0);
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: '标题',
      field: 'title',
      minWidth: 220,
      headerAlign: 'center',
      align: 'left',
    },
    {
      title: '删除人',
      field: 'deleteByName',
      width: 110,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
      slots: { default: 'createTimeSlot' },
    },
    {
      title: '删除时间',
      field: 'deleteTime',
      width: 160,
      slots: { default: 'deleteTimeSlot' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 180,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Grid :table-title="$t('page.crm.recycle.title')">
    <!-- 透传：调用方可把范围选项卡等渲染进筛选卡片（对齐 #form-header 模式） -->
    <template #form-header>
      <slot name="form-header"></slot>
    </template>
    <template #createTimeSlot="{ row }">
      {{ row.createTime ? formatDateTime(row.createTime) : '-' }}
    </template>

    <template #deleteTimeSlot="{ row }">
      {{ row.deleteTime ? formatDateTime(row.deleteTime) : '-' }}
    </template>

    <template #action="{ row }">
      <Popconfirm
        v-if="isSuperAdmin || row.deleteBy === userStore.userInfo?.userId"
        title="确定还原该数据？"
        ok-text="确认"
        cancel-text="取消"
        @confirm="handleRestore(row)"
      >
        <Button type="link" :icon="h(LucideUndo2)">
          {{ $t('page.crm.recycle.button.restore') }}
        </Button>
      </Popconfirm>
      <Button
        v-if="isSuperAdmin"
        type="link"
        danger
        :icon="h(LucideTrash2)"
        @click="handlePurge(row)"
      >
        {{ $t('page.crm.recycle.button.purge') }}
      </Button>
    </template>
  </Grid>
</template>
