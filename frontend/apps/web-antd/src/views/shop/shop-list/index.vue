<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideFilePenLine } from '@vben/icons';
import {
  Button,
  Tag,
  Image,
  Modal,
  Switch,
  message,
  Descriptions,
} from 'ant-design-vue';
import ShopDrawer from './drawer.vue';
import { shopApi } from '#/api';

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
      label: '店铺状态',
      componentProps: {
        options: [
          { label: '待审核', value: 0 },
          { label: '已开通', value: 1 },
          { label: '已驳回', value: 2 },
          { label: '已冻结', value: 3 },
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
        placeholder: '请输入店铺名称',
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
  rowConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await shopApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status,
          keyword: formValues.keyword,
        });
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
      title: '店铺Logo',
      field: 'shopLogo',
      slots: { default: 'logo' },
      width: 80,
    },
    {
      title: '店铺名称',
      field: 'shopName',
    },
    {
      title: '联系人',
      field: 'contactName',
    },
    {
      title: '联系电话',
      field: 'contactPhone',
    },
    {
      title: '佣金比例',
      field: 'commissionRate',
      slots: { default: 'rate' },
      width: 100,
    },
    {
      title: '可结算余额',
      field: 'balance',
      slots: { default: 'balance' },
      width: 120,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
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
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ShopDrawer,
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

async function handleStatusChanged(
  row: any,
  checked: boolean | string | number,
) {
  row.pending = true;
  try {
    const isChecked = checked === true || checked === 'true' || checked === 1;
    await shopApi.update({
      id: row.id,
      status: isChecked ? 1 : 3,
    });
    message.success(isChecked ? '店铺已解冻' : '店铺已冻结');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function viewDetail(row: any) {
  detailData.value = await shopApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="店铺管理">
      <template #logo="{ row }">
        <Image
          v-if="row.shopLogo"
          :src="row.shopLogo"
          width="50"
          height="50"
          fit="cover"
        />
        <span v-else class="text-gray-400 text-sm">-</span>
      </template>

      <template #rate="{ row }">
        <span class="text-blue-500">{{ row.commissionRate }}%</span>
      </template>

      <template #balance="{ row }">
        <span class="text-red-500">¥{{ (row.balance / 100).toFixed(2) }}</span>
      </template>

      <template #status="{ row }">
        <template v-if="row.status === 0">
          <Tag color="warning">待审核</Tag>
        </template>
        <template v-else-if="row.status === 1">
          <Tag color="success">已开通</Tag>
        </template>
        <template v-else-if="row.status === 2">
          <Tag color="error">已驳回</Tag>
        </template>
        <template v-else>
          <Tag color="default">已冻结</Tag>
        </template>
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
          编辑
        </Button>
        <Switch
          v-if="row.status === 1 || row.status === 3"
          :checked="row.status === 1"
          :loading="row.pending"
          @change="(checked) => handleStatusChanged(row, checked)"
        />
      </template>
    </Grid>
    <Drawer />

    <Modal v-model:open="detailModalVisible" title="店铺详情" width="600">
      <div class="flex gap-4 mb-4">
        <Image
          v-if="detailData?.shopLogo"
          :src="detailData.shopLogo"
          width="100"
          height="100"
        />
        <div class="flex-1">
          <h3 class="font-semibold text-lg">{{ detailData?.shopName }}</h3>
          <p class="text-gray-500">{{ detailData?.shopDesc || '-' }}</p>
        </div>
      </div>
      <Descriptions :column="2" bordered>
        <Descriptions.Item label="联系人">{{
          detailData?.contactName || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="联系电话">{{
          detailData?.contactPhone || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="佣金比例"
          >{{ detailData?.commissionRate || 0 }}%</Descriptions.Item
        >
        <Descriptions.Item label="结算周期">{{
          detailData?.settlementCycle === 1 ? '月结' : '其他'
        }}</Descriptions.Item>
        <Descriptions.Item label="可结算余额" :span="2">
          <span class="text-red-500 font-bold"
            >¥{{ ((detailData?.balance || 0) / 100).toFixed(2) }}</span
          >
        </Descriptions.Item>
      </Descriptions>
    </Modal>
  </Page>
</template>
