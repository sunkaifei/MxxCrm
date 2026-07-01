<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideCheck, LucideX } from '@vben/icons';
import { Button, Tag, Modal, Image, message } from 'ant-design-vue';
import { goodsAuditApi } from '#/api';

const auditModalVisible = ref(false);
const detailModalVisible = ref(false);
const currentGoods = ref<any>(null);
const detailData = ref<any>(null);
const auditFormValues = ref({
  status: 1,
  auditRemark: '',
});

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '审核状态',
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
        return await goodsAuditApi.list({
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
      width: 150,
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
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
    },
    {
      title: '申请时间',
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

function openAuditModal(row: any) {
  currentGoods.value = row;
  auditFormValues.value = {
    status: 1,
    auditRemark: '',
  };
  auditModalVisible.value = true;
}

async function handleAudit(status: number) {
  if (!currentGoods.value) return;

  try {
    await goodsAuditApi.audit({
      id: currentGoods.value.id,
      status,
      auditRemark: auditFormValues.value.auditRemark,
    });
    message.success(status === 1 ? '审核通过，商品已上架' : '审核驳回');
    auditModalVisible.value = false;
    gridApi.query();
  } catch (e) {
    console.error(e);
  }
}

async function viewDetail(row: any) {
  detailData.value = await goodsAuditApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="商品审核">
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
        <template v-if="row.status === 0">
          <Button
            type="primary"
            link
            :icon="h(LucideCheck)"
            @click="() => openAuditModal(row)"
          >
            审核
          </Button>
        </template>
      </template>
    </Grid>

    <Modal
      v-model:open="auditModalVisible"
      title="审核商品"
      width="500px"
      :footer="null"
    >
      <div v-if="currentGoods" class="space-y-4">
        <div class="flex gap-4">
          <Image
            v-if="currentGoods.primaryImage"
            :src="currentGoods.primaryImage"
            width="100"
            height="100"
            fit="cover"
          />
          <div class="flex-1">
            <h3 class="font-semibold">{{ currentGoods.title }}</h3>
            <p class="text-gray-500 text-sm">
              店铺: {{ currentGoods.shopName }}
            </p>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2"
            >审核结果</label
          >
          <div class="flex gap-4">
            <Button
              type="primary"
              size="large"
              :icon="h(LucideCheck)"
              @click="handleAudit(1)"
            >
              通过上架
            </Button>
            <Button
              type="default"
              size="large"
              :icon="h(LucideX)"
              @click="handleAudit(3)"
            >
              驳回
            </Button>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2"
            >审核备注</label
          >
          <textarea
            v-model="auditFormValues.auditRemark"
            class="w-full border border-gray-300 rounded-lg p-3 resize-none"
            rows="3"
            placeholder="请输入审核备注（驳回时必填）"
          ></textarea>
        </div>
      </div>
    </Modal>

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
              <span>库存: {{ detailData.stockTotal }}</span>
              <span>已售: {{ detailData.soldNum }}</span>
            </div>
          </div>
        </div>
        <div v-if="detailData.auditRemark" class="p-3 bg-gray-50 rounded">
          <span class="text-sm text-gray-700">审核备注:</span>
          <p class="text-gray-500">{{ detailData.auditRemark }}</p>
        </div>
      </div>
    </Modal>
  </Page>
</template>
