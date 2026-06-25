<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideArrowDownToLine } from '@vben/icons';
import { Button, Tag, Image, Modal, message } from 'ant-design-vue';
import { goodsApi } from '#/api';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '商品状态',
      componentProps: {
        options: [
          { label: '待审核', value: 0 },
          { label: '已上架', value: 1 },
          { label: '已下架', value: 2 },
          { label: '审核驳回', value: 3 },
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
        placeholder: '请输入商品名称',
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
        return await goodsApi.list({
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
      title: '商品图片',
      field: 'primaryImage',
      slots: { default: 'image' },
      width: 100,
    },
    {
      title: '商品名称',
      field: 'title',
    },
    {
      title: '所属店铺',
      field: 'shopName',
    },
    {
      title: '价格区间',
      field: 'priceRange',
      slots: { default: 'price' },
      width: 180,
    },
    {
      title: '库存',
      field: 'stockTotal',
      width: 80,
    },
    {
      title: '已售',
      field: 'soldNum',
      width: 80,
    },
    {
      title: '佣金',
      field: 'isCommission',
      slots: { default: 'commission' },
      width: 80,
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

async function handleOffline(row: any) {
  row.pending = true;
  try {
    await goodsApi.offline(row.id);
    message.success('商品已下架');
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const detailModalVisible = ref(false);
const detailData = ref<any>(null);

async function viewDetail(row: any) {
  detailData.value = await goodsApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="商品管理">
      <template #image="{ row }">
        <Image
          v-if="row.primaryImage"
          :src="row.primaryImage"
          width="60"
          height="60"
          fit="cover"
        />
        <span v-else class="text-gray-400 text-sm">-</span>
      </template>

      <template #price="{ row }">
        <span class="text-red-500"
          >¥{{ (row.minSalePrice / 100).toFixed(2) }}</span
        >
        <span class="mx-1">~</span>
        <span class="text-red-500"
          >¥{{ (row.maxSalePrice / 100).toFixed(2) }}</span
        >
        <span class="text-gray-400 ml-2"
          >(划线价 ¥{{ (row.minLinePrice / 100).toFixed(2) }})</span
        >
      </template>

      <template #commission="{ row }">
        <Tag :color="row.isCommission === 1 ? 'success' : 'default'">
          {{ row.isCommission === 1 ? '参与' : '不参与' }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag
          :color="
            row.status === 0
              ? 'warning'
              : row.status === 1
                ? 'success'
                : row.status === 2
                  ? 'default'
                  : 'error'
          "
        >
          {{
            row.status === 0
              ? '待审核'
              : row.status === 1
                ? '已上架'
                : row.status === 2
                  ? '已下架'
                  : '审核驳回'
          }}
        </Tag>
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
        <template v-if="row.status === 1">
          <Button
            type="default"
            link
            :icon="h(LucideArrowDownToLine)"
            @click="() => handleOffline(row)"
          >
            下架
          </Button>
        </template>
      </template>
    </Grid>

    <Modal v-model:open="detailModalVisible" title="商品详情" width="800">
      <div v-if="detailData" class="space-y-4">
        <div class="flex gap-4">
          <Image :src="detailData.primaryImage" width="200" />
          <div class="flex-1 space-y-2">
            <h3 class="font-semibold text-lg">{{ detailData.title }}</h3>
            <p class="text-gray-500">{{ detailData.subtitle || '-' }}</p>
            <div class="text-red-500 font-bold">
              ¥{{ (detailData.minSalePrice / 100).toFixed(2) }} - ¥{{
                (detailData.maxSalePrice / 100).toFixed(2)
              }}
            </div>
            <div class="flex gap-4 text-sm text-gray-600">
              <span
                >划线价: ¥{{ (detailData.minLinePrice / 100).toFixed(2) }} - ¥{{
                  (detailData.maxLinePrice / 100).toFixed(2)
                }}</span
              >
            </div>
            <div class="flex gap-4 text-sm text-gray-600">
              <span>库存: {{ detailData.stockTotal }}</span>
              <span>已售: {{ detailData.soldNum }}</span>
            </div>
            <div class="flex gap-4 text-sm text-gray-600">
              <span
                >参与佣金:
                {{ detailData.isCommission === 1 ? '是' : '否' }}</span
              >
            </div>
          </div>
        </div>
      </div>
    </Modal>
  </Page>
</template>
