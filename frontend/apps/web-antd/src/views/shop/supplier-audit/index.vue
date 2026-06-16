<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideCheck, LucideX } from '@vben/icons';
import { Button, Tag, Modal, message, Descriptions } from 'ant-design-vue';
import { supplierAuditApi } from '#/api';

const auditModalVisible = ref(false);
const detailModalVisible = ref(false);
const currentApply = ref<any>(null);
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
      label: '申请状态',
      componentProps: {
        options: [
          { label: '待审核', value: 0 },
          { label: '已通过', value: 1 },
          { label: '已驳回', value: 2 },
        ],
        placeholder: '请选择状态',
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
        return await supplierAuditApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status,
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
      title: '店铺简介',
      field: 'shopDesc',
      width: 200,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
    },
    {
      title: '审核时间',
      field: 'auditTime',
      width: 160,
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
  currentApply.value = row;
  auditFormValues.value = {
    status: 1,
    auditRemark: '',
  };
  auditModalVisible.value = true;
}

async function handleAudit(status: number) {
  if (!currentApply.value) return;

  try {
    await supplierAuditApi.audit({
      id: currentApply.value.id,
      status,
      auditRemark: auditFormValues.value.auditRemark,
    });
    message.success(status === 1 ? '审核通过' : '审核驳回');
    auditModalVisible.value = false;
    gridApi.query();
  } catch (e) {
    console.error(e);
  }
}

function viewDetail(row: any) {
  detailData.value = row;
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="供货商入驻审核">
      <template #status="{ row }">
        <Tag
          :color="
            row.status === 0
              ? 'warning'
              : row.status === 1
                ? 'success'
                : 'error'
          "
        >
          {{
            row.status === 0 ? '待审核' : row.status === 1 ? '已通过' : '已驳回'
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
          查看
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
      title="审核申请"
      width="500px"
      :footer="null"
    >
      <div v-if="currentApply" class="space-y-4">
        <div class="text-gray-600">店铺名称: {{ currentApply.shopName }}</div>
        <div class="text-gray-600">联系人: {{ currentApply.contactName }}</div>
        <div class="text-gray-600">
          联系电话: {{ currentApply.contactPhone }}
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
              通过
            </Button>
            <Button
              type="default"
              size="large"
              :icon="h(LucideX)"
              @click="handleAudit(2)"
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
            placeholder="请输入审核备注"
          ></textarea>
        </div>
      </div>
    </Modal>

    <Modal v-model:open="detailModalVisible" title="申请详情" width="600">
      <Descriptions :column="2" bordered>
        <Descriptions.Item label="店铺名称">{{
          detailData?.shopName || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="联系人">{{
          detailData?.contactName || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="联系电话">{{
          detailData?.contactPhone || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="店铺简介" :span="2">{{
          detailData?.shopDesc || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="审核备注" :span="2">{{
          detailData?.auditRemark || '-'
        }}</Descriptions.Item>
      </Descriptions>
    </Modal>
  </Page>
</template>
