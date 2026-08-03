<script lang="ts" setup>
import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Descriptions,
  DescriptionsItem,
  Modal,
  Popconfirm,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { deleteLogApi, getLogListApi } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

// 业务类型 / 操作状态 颜色与文案映射
const businessTypeColorMap: Record<number, string> = {
  0: 'default',
  1: 'success',
  2: 'blue',
  3: 'red',
};

const businessTypeTextKey: Record<number, string> = {
  0: 'page.system.log.businessTypeOther',
  1: 'page.system.log.businessTypeInsert',
  2: 'page.system.log.businessTypeUpdate',
  3: 'page.system.log.businessTypeDelete',
};

const statusColorMap: Record<number, string> = {
  0: 'success',
  1: 'red',
};

const statusTextKey: Record<number, string> = {
  0: 'page.system.log.statusNormal',
  1: 'page.system.log.statusException',
};

// 详情弹窗
const detailVisible = ref(false);
const detailData = ref<Record<string, any>>({});

// 长文本展示样式
const longTextStyle = {
  maxHeight: '300px',
  overflow: 'auto',
  whiteSpace: 'pre-wrap' as const,
  wordBreak: 'break-all' as const,
  margin: 0,
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'title',
      label: $t('page.system.log.moduleTitle'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'operName',
      label: $t('page.system.log.operName'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'businessType',
      label: $t('page.system.log.businessType'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: $t('page.system.log.businessTypeOther'), value: 0 },
          { label: $t('page.system.log.businessTypeInsert'), value: 1 },
          { label: $t('page.system.log.businessTypeUpdate'), value: 2 },
          { label: $t('page.system.log.businessTypeDelete'), value: 3 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.system.log.status'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: $t('page.system.log.statusNormal'), value: 0 },
          { label: $t('page.system.log.statusException'), value: 1 },
        ],
      },
    },
    {
      component: 'RangePicker',
      fieldName: 'dateRange',
      label: $t('page.system.log.createTime'),
      componentProps: {
        valueFormat: 'YYYY-MM-DD HH:mm:ss',
        showTime: true,
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
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const { dateRange, ...rest } = formValues || {};
        const [beginTime, endTime] = Array.isArray(dateRange)
          ? dateRange
          : [];
        return await getLogListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          beginTime,
          endTime,
          ...rest,
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
    {
      title: $t('page.system.log.moduleTitle'),
      field: 'title',
      minWidth: 120,
    },
    {
      title: $t('page.system.log.businessType'),
      field: 'businessType',
      width: 100,
      slots: { default: 'businessType' },
    },
    {
      title: $t('page.system.log.operName'),
      field: 'operName',
      width: 120,
    },
    {
      title: $t('page.system.log.requestMethod'),
      field: 'requestMethod',
      width: 100,
    },
    {
      title: $t('page.system.log.operUrl'),
      field: 'operUrl',
      minWidth: 220,
      showOverflow: true,
    },
    {
      title: $t('page.system.log.operIp'),
      field: 'operIp',
      width: 140,
    },
    {
      title: $t('page.system.log.status'),
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    {
      title: $t('page.system.log.statusCode'),
      field: 'statusCode',
      width: 90,
    },
    {
      title: $t('page.system.log.elapsed'),
      field: 'elapsed',
      width: 110,
      slots: { default: 'elapsed' },
    },
    {
      title: $t('page.system.log.createTime'),
      field: 'createTime',
      width: 170,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 130,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteLogApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleViewDetail(row: any) {
  detailData.value = row;
  detailVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #businessType="{ row }">
        <Tag :color="businessTypeColorMap[row.businessType] || 'default'">
          {{
            $t(
              businessTypeTextKey[row.businessType] ||
                'page.system.log.businessTypeOther',
            )
          }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status] || 'default'">
          {{
            $t(statusTextKey[row.status] || 'page.system.log.statusNormal')
          }}
        </Tag>
      </template>

      <template #elapsed="{ row }">
        <span>{{ row.elapsed }} ms</span>
      </template>

      <template #action="{ row }">
        <Button type="link" size="small" @click="handleViewDetail(row)">
          {{ $t('page.system.log.viewDetail') }}
        </Button>
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.log.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('system:log:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>

    <Modal
      v-model:open="detailVisible"
      :title="$t('page.system.log.viewDetail')"
      :width="800"
      :footer="null"
    >
      <Descriptions
        :column="2"
        bordered
        size="small"
        :label-style="{ width: '120px' }"
      >
        <DescriptionsItem :label="$t('page.system.log.moduleTitle')">
          {{ detailData.title }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.businessType')">
          {{
            $t(
              businessTypeTextKey[detailData.businessType] ||
                'page.system.log.businessTypeOther',
            )
          }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.operName')">
          {{ detailData.operName }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.deptName')">
          {{ detailData.deptName }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.system.log.requestMethod')"
          :span="2"
        >
          {{ detailData.requestMethod }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.method')" :span="2">
          {{ detailData.method }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.operUrl')" :span="2">
          {{ detailData.operUrl }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.operIp')">
          {{ detailData.operIp }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.operLocation')">
          {{ detailData.operLocation }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.status')">
          {{
            $t(
              statusTextKey[detailData.status] ||
                'page.system.log.statusNormal',
            )
          }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.statusCode')">
          {{ detailData.statusCode }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.elapsed')" :span="2">
          {{ detailData.elapsed }} ms
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.errorMsg')" :span="2">
          {{ detailData.errorMsg }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.operParam')" :span="2">
          <pre :style="longTextStyle">{{ detailData.operParam }}</pre>
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.jsonResult')" :span="2">
          <pre :style="longTextStyle">{{ detailData.jsonResult }}</pre>
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.system.log.createTime')" :span="2">
          {{ formatDateTime(detailData.createTime) }}
        </DescriptionsItem>
      </Descriptions>
    </Modal>
  </Page>
</template>
