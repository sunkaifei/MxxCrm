<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine, LucidePlus } from '@vben/icons';
import { Button, Tag, Image, Modal, message } from 'ant-design-vue';
import SiteDrawer from './drawer.vue';
import { siteApi } from '#/api';

const detailModalVisible = ref(false);
const detailData = ref<any>(null);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '站点状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '正常', value: 1 },
          { label: '冻结', value: 2 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'keyword',
      label: '关键词',
      componentProps: {
        placeholder: '请输入网站名称',
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
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await siteApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status || undefined,
          keyword: formValues.keyword,
        });
      },
      delete: async ({ body }) => {
        await siteApi.delete(body);
      },
    },
  },

  columns: [
    {
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: '网站名称',
      field: 'siteName',
      width: 280,
    },
    {
      title: '二级域名',
      field: 'domain',
      width: 140,
    },
    {
      title: '绑定域名',
      field: 'bindDomain',
      width: 180,
    },
    {
      title: '站点类型',
      field: 'siteType',
      slots: { default: 'siteType' },
      width: 100,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: SiteDrawer,
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

function handleAdd() {
  openDrawer(true);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除网站"${row.siteName}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await siteApi.delete([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}

async function viewDetail(row: any) {
  detailData.value = await siteApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="网站管理">
      <template #toolbar-tools>
        <Button
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleAdd"
        >
          新增网站
        </Button>
      </template>

      <template #siteType="{ row }">
        <Tag v-if="row.siteType === 1" color="blue">企业官网</Tag>
        <Tag v-else-if="row.siteType === 2" color="green">商城</Tag>
        <Tag v-else color="default">其他</Tag>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">正常</Tag>
        <Tag v-else color="error">冻结</Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEye)"
          @click="() => viewDetail(row)"
        >
          详情
        </Button>
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          修改
        </Button>
        <Button type="primary" link danger @click="() => handleDelete(row)">
          删除
        </Button>
      </template>
    </Grid>
    <Drawer />

    <Modal v-model:open="detailModalVisible" title="网站详情" width="800">
      <div v-if="detailData" class="grid grid-cols-2 gap-4">
        <div class="col-span-2 flex gap-4 mb-4">
          <Image
            v-if="detailData.logo"
            :src="detailData.logo"
            width="80"
            height="80"
            fit="cover"
            class="rounded"
          />
          <div>
            <h3 class="font-semibold text-lg">{{ detailData.siteName }}</h3>
            <p class="text-gray-500 text-sm">{{ detailData.description || '-' }}</p>
          </div>
        </div>
        <div><span class="text-gray-500">二级域名：</span>{{ detailData.domain }}</div>
        <div><span class="text-gray-500">绑定域名：</span>{{ detailData.bindDomain || '-' }}</div>
        <div>
          <span class="text-gray-500">站点类型：</span>
          <Tag v-if="detailData.siteType === 1" color="blue">企业官网</Tag>
          <Tag v-else-if="detailData.siteType === 2" color="green">商城</Tag>
          <Tag v-else>其他</Tag>
        </div>
        <div>
          <span class="text-gray-500">状态：</span>
          <Tag v-if="detailData.status === 1" color="success">正常</Tag>
          <Tag v-else color="error">冻结</Tag>
        </div>
        <div><span class="text-gray-500">排序：</span>{{ detailData.sort ?? 0 }}</div>
        <div><span class="text-gray-500">默认站点：</span>{{ detailData.isDefault === 1 ? '是' : '否' }}</div>
        <div><span class="text-gray-500">SEO关键词：</span>{{ detailData.keywords || '-' }}</div>
        <div><span class="text-gray-500">创建时间：</span>{{ detailData.createTime || '-' }}</div>
        <div class="col-span-2"><span class="text-gray-500">备注：</span>{{ detailData.remark || '-' }}</div>
      </div>
    </Modal>
  </Page>
</template>
