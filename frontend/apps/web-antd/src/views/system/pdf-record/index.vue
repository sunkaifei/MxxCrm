<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { Button, TabPane, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  downloadPdfApi,
  getPdfDownloadLogApi,
  getPdfRecordAllApi,
} from '#/api';
import { $t } from '#/locales';

const activeTab = ref('generate');

const docTypeOptions = [
  { label: $t('page.system.pdfRecord.docTypeQuotation'), value: 'quotation' },
  { label: $t('page.system.pdfRecord.docTypeOrder'), value: 'order' },
  { label: $t('page.system.pdfRecord.docTypeContract'), value: 'contract' },
];

const docTypeMap: Record<string, string> = {
  quotation: $t('page.system.pdfRecord.docTypeQuotation'),
  order: $t('page.system.pdfRecord.docTypeOrder'),
  contract: $t('page.system.pdfRecord.docTypeContract'),
};

const triggerTypeOptions = [
  { label: $t('page.system.pdfRecord.triggerManual'), value: 'manual' },
  { label: $t('page.system.pdfRecord.triggerAuto'), value: 'auto' },
];

// ========== Tab1: 生成记录 ==========
const genFormOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'docType',
      label: $t('page.system.pdfRecord.docType'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: docTypeOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'docNo',
      label: $t('page.system.pdfRecord.docNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.search'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'triggerType',
      label: $t('page.system.pdfRecord.triggerType'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: triggerTypeOptions,
      },
    },
  ],
};

const genGridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  height: 'auto',
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getPdfRecordAllApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.system.pdfRecord.docType'),
      field: 'docType',
      width: 100,
      slots: { default: 'docType' },
    },
    { title: $t('page.system.pdfRecord.docNo'), field: 'docNo', width: 160 },
    {
      title: $t('page.system.pdfRecord.templateName'),
      field: 'templateName',
      width: 160,
    },
    {
      title: $t('page.system.pdfRecord.fileName'),
      field: 'fileName',
      minWidth: 200,
    },
    {
      title: $t('page.system.pdfRecord.fileSize'),
      field: 'fileSize',
      width: 100,
      slots: { default: 'fileSize' },
    },
    {
      title: $t('page.system.pdfRecord.triggerType'),
      field: 'triggerType',
      width: 90,
      slots: { default: 'triggerType' },
    },
    {
      title: $t('page.system.pdfRecord.status'),
      field: 'status',
      width: 80,
      slots: { default: 'status' },
    },
    {
      title: $t('page.system.pdfRecord.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 100,
    },
  ],
};

const [GenGrid] = useVbenVxeGrid({ gridOptions: genGridOptions, formOptions: genFormOptions });

// ========== Tab2: 下载查看记录 ==========
const dlFormOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'docType',
      label: $t('page.system.pdfRecord.docType'),
      componentProps: {
        placeholder: $t('ui.placeholder.all'),
        allowClear: true,
        options: docTypeOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'docNo',
      label: $t('page.system.pdfRecord.docNo'),
      componentProps: {
        placeholder: $t('ui.placeholder.search'),
        allowClear: true,
      },
    },
  ],
};

const dlGridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  height: 'auto',
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getPdfDownloadLogApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.system.pdfRecord.docType'),
      field: 'docType',
      width: 100,
      slots: { default: 'docType' },
    },
    { title: $t('page.system.pdfRecord.docNo'), field: 'docNo', width: 160 },
    {
      title: $t('page.system.pdfRecord.fileName'),
      field: 'fileName',
      minWidth: 200,
    },
    {
      title: $t('page.system.pdfRecord.downloadUser'),
      field: 'operatorName',
      width: 120,
    },
    {
      title: $t('page.system.pdfRecord.downloadIp'),
      field: 'ipAddress',
      width: 130,
    },
    {
      title: $t('page.system.pdfRecord.downloadTime'),
      field: 'createTime',
      width: 170,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'dlAction' },
      width: 100,
    },
  ],
};

const [DlGrid, dlGridApi] = useVbenVxeGrid({
  gridOptions: dlGridOptions,
  formOptions: dlFormOptions,
});

// Tab切换时加载数据
function handleTabChange(key: string) {
  if (key === 'download') {
    dlGridApi.query();
  }
}

async function handleDownload(row: any) {
  if (row.status === 0) {
    window.$message.warning($t('page.system.pdfRecord.downloadFailed'));
    return;
  }
  try {
    const blob: any = await downloadPdfApi(row.id);
    const pdfBlob =
      blob instanceof Blob
        ? blob
        : new Blob([blob], { type: 'application/pdf' });
    const url = window.URL.createObjectURL(pdfBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = row.fileName || 'document.pdf';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);
  } catch {
    // error handled by interceptor
  }
}
</script>

<template>
  <Page auto-content-height>
    <Tabs v-model:active-key="activeTab" @change="handleTabChange">
      <TabPane
        key="generate"
        :tab="$t('page.system.pdfRecord.tabGenerate')"
      >
        <GenGrid>
          <template #docType="{ row }">
            <Tag>{{ docTypeMap[row.docType] || row.docType }}</Tag>
          </template>

          <template #fileSize="{ row }">
            {{ row.fileSize ? (row.fileSize / 1024).toFixed(1) + ' KB' : '-' }}
          </template>

          <template #triggerType="{ row }">
            <Tag :color="row.triggerType === 'auto' ? 'blue' : 'orange'">
              {{
                row.triggerType === 'auto'
                  ? $t('page.system.pdfRecord.triggerAuto')
                  : $t('page.system.pdfRecord.triggerManual')
              }}
            </Tag>
          </template>

          <template #status="{ row }">
            <Tag :color="row.status === 1 ? 'green' : 'red'">
              {{
                row.status === 1
                  ? $t('page.system.pdfRecord.statusSuccess')
                  : $t('page.system.pdfRecord.statusFailed')
              }}
            </Tag>
          </template>

          <template #createTime="{ row }">
            {{ formatDateTime(row.createTime) }}
          </template>

          <template #action="{ row }">
            <Button
              v-if="row.status === 1"
              type="link"
              size="small"
              @click="handleDownload(row)"
            >
              {{ $t('page.system.pdfRecord.download') }}
            </Button>
            <span
              v-else
              :title="row.errorMsg"
              class="text-xs text-red-500"
            >
              {{ row.errorMsg }}
            </span>
          </template>
        </GenGrid>
      </TabPane>

      <TabPane
        key="download"
        :tab="$t('page.system.pdfRecord.tabDownload')"
      >
        <DlGrid>
          <template #docType="{ row }">
            <Tag>{{ docTypeMap[row.docType] || row.docType }}</Tag>
          </template>

          <template #dlAction="{ row }">
            <Button
              type="link"
              size="small"
              @click="handleDownload({ id: row.recordId, fileName: row.fileName })"
            >
              {{ $t('page.system.pdfRecord.reDownload') }}
            </Button>
          </template>
        </DlGrid>
      </TabPane>
    </Tabs>
  </Page>
</template>
