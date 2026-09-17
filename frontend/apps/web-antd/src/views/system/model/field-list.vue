<script lang="ts" setup>
/**
 * 系统模型 · 字段设置面板（抽屉内嵌）
 * 固定展示某个模块的字段清单（系统字段+自定义字段）：
 * 编辑（抽屉）/ 字段权限（抽屉）/ 删除 / 停用。系统字段仅可改显示名。
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import {
  LucideFilePenLine,
  LucideLock,
  LucideTrash2,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';
import { IconifyIcon } from '@vben/icons';

import { Button, Input, message, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteFieldApi,
  FIELD_TYPE_LABELS,
  getFieldListApi,
  setFieldStatusApi,
} from '#/api';
import { $t } from '#/locales';

import FieldDrawer from '../field/drawer.vue';
import PermDrawer from '../field/perm-drawer.vue';

const props = defineProps<{ module: string }>();

const keyword = ref('');

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    refresh: true,
    zoom: true,
  },
  height: 480,
  pagerConfig: {},
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        return await getFieldListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          module: props.module,
          keyword: keyword.value || undefined,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '显示名', field: 'fieldLabel', minWidth: 120 },
    { title: '字段键', field: 'fieldKey', minWidth: 140 },
    {
      title: '类型',
      field: 'fieldType',
      width: 90,
      formatter: ({ cellValue }: any) => FIELD_TYPE_LABELS[cellValue] ?? cellValue,
    },
    {
      title: '必填',
      field: 'required',
      width: 64,
      align: 'center',
      formatter: ({ cellValue }: any) => (Number(cellValue) === 1 ? '是' : '否'),
    },
    {
      title: '状态',
      field: 'status',
      width: 86,
      align: 'center',
      slots: { default: 'status' },
    },
    {
      title: '更新时间',
      field: 'updateTime',
      width: 150,
      formatter: ({ cellValue }: any) => cellValue || '',
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 110,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

// 新增字段：按当前模型预填所属模块
function openCreate() {
  drawerApi.setData({ create: true, presetModule: props.module });
  drawerApi.open();
}

async function handleStatusChanged(row: any, checked: boolean | number) {
  row.pending = true;
  const target = checked ? 1 : 0;
  try {
    await setFieldStatusApi({ id: row.id, status: target });
    row.status = target;
    message.success('状态已更新');
  } catch {
    /* 错误由全局拦截器处理 */
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: FieldDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

const [PermDrawerHost, permDrawerApi] = useVbenDrawer({
  connectedComponent: PermDrawer,
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function openPerm(row: any) {
  permDrawerApi.setData({ row });
  permDrawerApi.open();
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteFieldApi(row.id);
    message.success('删除成功');
  } catch {
    /* 错误由全局拦截器处理 */
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <div>
    <div class="mb-2 flex items-center gap-2">
      <Input
        v-model:value="keyword"
        allow-clear
        class="w-56"
        placeholder="字段键 / 显示名"
        @press-enter="gridApi.query()"
      />
      <Button @click="gridApi.query()">查询</Button>
      <Button class="ml-auto" type="primary" ghost @click="openCreate">
        <template #icon><IconifyIcon icon="lucide:plus" /></template>
        新增字段
      </Button>
    </div>
    <Grid>
      <template #status="{ row }">
        <Switch
          :checked="Number(row.status)"
          :checked-value="1"
          :disabled="Number(row.isSystem) === 1"
          :loading="row.pending"
          :un-checked-value="0"
          checked-children="启用"
          un-checked-children="停用"
          @change="(checked: any) => handleStatusChanged(row, checked)"
        />
      </template>
      <template #action="{ row }">
        <Button
          type="link"
          size="small"
          :icon="h(LucideFilePenLine)"
          @click="openDrawer(false, row)"
        />
        <Popconfirm
          v-if="Number(row.isSystem) !== 1"
          title="删除后同名同类型可重建，历史数据保留在库中，确认删除？"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button type="link" size="small" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
        <Button
          type="link"
          size="small"
          :icon="h(LucideLock)"
          title="字段权限"
          @click="openPerm(row)"
        />
      </template>
    </Grid>
    <Drawer />
    <PermDrawerHost />
  </div>
</template>
