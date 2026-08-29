<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideFilePenLine,
  LucideLayers,
  LucideLock,
  LucideTrash2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Modal, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  accelerateFieldApi,
  deleteFieldApi,
  FIELD_TYPE_LABELS,
  getFieldListApi,
  getModuleListApi,
  setFieldStatusApi,
} from '#/api';
import { $t } from '#/locales';

import FieldDrawer from './drawer.vue';
import PermDrawer from './perm-drawer.vue';

const accessStore = useAccessStore();

// 最近一次筛选值缓存（工具栏「一键加速」取当前选中模块）
let lastFormValues: Record<string, any> = {};

const moduleOptions = ref<Array<{ label: string; value: string }>>([]);

async function loadModuleOptions() {
  try {
    const res: any = await getModuleListApi();
    const list = Array.isArray(res) ? res : (res?.list ?? []);
    moduleOptions.value = list.map((m: any) => ({
      label: m.label ?? m.module,
      value: m.module,
    }));
  } catch {
    /* ignore */
  }
}
loadModuleOptions();

const statusOptions = [
  { label: '启用', value: 1 },
  { label: '停用', value: 0 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'module',
      label: '所属模块',
      componentProps: {
        options: moduleOptions,
        placeholder: '请选择模块',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'keyword',
      label: '关键字',
      componentProps: {
        placeholder: '字段键 / 显示名',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: statusOptions,
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        lastFormValues = formValues || {};
        return await getFieldListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          module: formValues.module,
          keyword: formValues.keyword,
          status: formValues.status,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    { title: '显示名', field: 'fieldLabel', minWidth: 120 },
    { title: '字段键', field: 'fieldKey', minWidth: 140 },
    { title: '模块', field: 'module', width: 110 },
    {
      title: '类型',
      field: 'fieldType',
      width: 90,
      formatter: ({ cellValue }: any) => FIELD_TYPE_LABELS[cellValue] ?? cellValue,
    },
    {
      title: '必填',
      field: 'required',
      width: 70,
      align: 'center',
      formatter: ({ cellValue }: any) => (Number(cellValue) === 1 ? '是' : '否'),
    },
    {
      title: '列表显示',
      field: 'listVisible',
      width: 90,
      align: 'center',
      formatter: ({ cellValue }: any) => (Number(cellValue) === 1 ? '显示' : '隐藏'),
    },
    {
      title: '筛选',
      field: 'filterable',
      width: 70,
      align: 'center',
      formatter: ({ cellValue }: any) => (Number(cellValue) === 1 ? '是' : '否'),
    },
    {
      title: '索引',
      field: 'indexed',
      width: 80,
      align: 'center',
      formatter: ({ cellValue }: any) => (Number(cellValue) === 1 ? '已建' : '未建'),
    },
    {
      title: '状态',
      field: 'status',
      width: 90,
      align: 'center',
      slots: { default: 'status' },
    },
    {
      title: '更新时间',
      field: 'updateTime',
      width: 160,
      slots: { default: 'updatedAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 140,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

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

/** 一键加速：为模块内 filterable 字段批量创建表达式索引（CONCURRENTLY 无长阻塞） */
async function handleAccelerate() {
  const module = lastFormValues?.module;
  if (!module) {
    message.warning('请先在筛选栏选择要加速的模块');
    return;
  }
  try {
    const report: any = await accelerateFieldApi(module);
    const created: string[] = report?.created ?? [];
    const skipped: string[] = report?.skipped ?? [];
    const failed: Array<{ fieldKey: string; reason: string }> = report?.failed ?? [];
    if (failed.length > 0) {
      Modal.warning({
        title: '加速完成（部分失败）',
        content: `新建 ${created.length} 个索引，跳过 ${skipped.length} 个，失败 ${failed.length} 个：${failed
          .slice(0, 3)
          .map((f) => `${f.fieldKey}（${f.reason}）`)
          .join('；')}`,
      });
    } else {
      message.success(`加速完成：新建 ${created.length} 个索引，跳过 ${skipped.length} 个`);
    }
    gridApi.query();
  } catch {
    /* 错误由全局拦截器处理 */
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

// 标准字段权限配置抽屉（P2-1）：无需刷新字段清单，仅打开即加载
const [PermDrawerHost, permDrawerApi] = useVbenDrawer({
  connectedComponent: PermDrawer,
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
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
  <Page auto-content-height>
    <Grid>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('system:field_perm:save')"
          class="mr-2"
          :icon="h(LucideLock)"
          @click="permDrawerApi.open()"
        >
          标准字段权限
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('system:field:update')"
          class="mr-2"
          :icon="h(LucideLayers)"
          @click="handleAccelerate"
        >
          一键加速
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('system:field:save')"
          type="primary"
          class="mr-2"
          @click="openDrawer(true)"
        >
          新增字段
        </Button>
      </template>

      <template #updatedAt="{ row }">
        {{ formatDateTime(row.updateTime) }}
      </template>

      <template #status="{ row }">
        <Switch
          :checked="Number(row.status)"
          :checked-value="1"
          :disabled="!accessStore.hasAccessCode('system:field:status')"
          :loading="row.pending"
          :un-checked-value="0"
          checked-children="启用"
          un-checked-children="停用"
          @change="(checked: any) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('system:field:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="openDrawer(false, row)"
        />
        <Popconfirm
          title="删除后同名同类型可重建，历史数据保留在库中，确认删除？"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('system:field:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
    <PermDrawerHost />
  </Page>
</template>
