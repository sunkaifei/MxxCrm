<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import { Alert, Button, message, Popconfirm } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteCategoryApi, getCategoryListApi } from '#/api';
import { $t } from '#/locales';

import CategoryDrawer from './drawer.vue';

const accessStore = useAccessStore();

// 分类数量过多时显示提示
const showCountAlert = ref(false);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '分类名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {
    enabled: false,
  },
  treeConfig: {
    parentField: 'parentId',
    rowField: 'id',
    transform: true,
  },
  cellConfig: { isHover: true } as any,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async (_, formValues) => {
        const res: any = await getCategoryListApi({
          page: 1,
          pageSize: 99_999,
          keywords: formValues.keywords,
        });
        const list = res?.list || res?.items || res || [];
        // 超过1000条时显示提示
        showCountAlert.value = list.length > 1000;
        return list;
      },
    },
  },

  columns: [
    {
      title: '分类图片',
      field: 'image',
      width: 90,
      slots: { default: 'image' },
    },
    {
      title: '分类名称',
      field: 'name',
      minWidth: 200,
      treeNode: true,
    },
    {
      title: '排序号',
      field: 'sortOrder',
      width: 100,
    },
    {
      title: '描述',
      field: 'description',
      minWidth: 200,
      showOverflow: true,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 展开/折叠全部树节点
function handleExpandAll() {
  gridApi.grid?.setAllTreeExpand?.(true);
}

function handleCollapseAll() {
  gridApi.grid?.clearTreeExpand?.();
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: CategoryDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteCategoryApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function handleCreate() {
  openDrawer(true);
}
</script>

<template>
  <Page auto-content-height>
    <Alert
      v-if="showCountAlert"
      type="warning"
      show-icon
      message="分类数量较多，加载数据可能较慢，如有性能问题请联系官方获取最佳解决方案。"
      style="margin-bottom: 8px"
      closable
    />
    <Grid :table-title="$t('page.product.category.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:category:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.product.category.button.create') }}
        </Button>
        <Button
          type="default"
          size="small"
          class="mr-2"
          @click="handleExpandAll"
        >
          展开全部
        </Button>
        <Button
          type="default"
          size="small"
          class="mr-2"
          @click="handleCollapseAll"
        >
          全部收缩
        </Button>
      </template>

      <template #image="{ row }">
        <img
          v-if="row.image"
          :src="row.image"
          alt=""
          class="w-10 h-10 rounded object-cover border border-gray-100"
        />
        <span v-else class="text-xs text-gray-300">-</span>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:category:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.product.category.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('product:category:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
