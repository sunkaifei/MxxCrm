<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { ref } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { Button, Card, Form, Input, Select, Tabs, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  downloadPdfApi,
  getPdfDownloadLogApi,
  getPdfRecordAllApi,
} from '#/api';
import { $t } from '#/locales';

// 当前激活的页签
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

// ========== 搜索表单（参照 pdf-template：外置表单，Grid 不带 formOptions） ==========
const searchForm = ref({
  docType: undefined as string | undefined,
  docNo: '',
  triggerType: undefined as string | undefined,
});

function queryActiveGrid() {
  if (activeTab.value === 'download') {
    dlGridApi.query();
  } else {
    genGridApi.query();
  }
}

function handleSearch() {
  queryActiveGrid();
}

function handleReset() {
  searchForm.value = { docType: undefined, docNo: '', triggerType: undefined };
  queryActiveGrid();
}

// ========== Tab1: 生成记录 ==========
const genGridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        return await getPdfRecordAllApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          docType: searchForm.value.docType,
          docNo: searchForm.value.docNo || undefined,
          triggerType: searchForm.value.triggerType,
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

const [GenGrid, genGridApi] = useVbenVxeGrid({
  gridOptions: genGridOptions,
});

// ========== Tab2: 下载查看记录 ==========
const dlGridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        return await getPdfDownloadLogApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          docType: searchForm.value.docType,
          docNo: searchForm.value.docNo || undefined,
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
});

// Tab切换时重新加载列表（参照 pdf-template：切 Tab 即重查）
function handleTabChange(key: number | string) {
  activeTab.value = String(key);
  queryActiveGrid();
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
    document.body.append(link);
    link.click();
    link.remove();
    window.URL.revokeObjectURL(url);
  } catch {
    // error handled by interceptor
  }
}
</script>

<template>
  <Page>
    <!-- 参照 pdf-template：Card 包裹 Tabs + 外置搜索表单 -->
    <Card :bordered="false" style="margin-bottom: 16px">
      <Tabs
        v-model:active-key="activeTab"
        style="margin-bottom: 16px"
        @change="handleTabChange"
      >
        <Tabs.TabPane
          key="generate"
          :tab="$t('page.system.pdfRecord.tabGenerate')"
        />
        <Tabs.TabPane
          key="download"
          :tab="$t('page.system.pdfRecord.tabDownload')"
        />
      </Tabs>

      <Form :model="searchForm" layout="inline" class="pdf-search-form">
        <Form.Item :label="$t('page.system.pdfRecord.docType')" name="docType">
          <Select
            v-model:value="searchForm.docType"
            :options="docTypeOptions"
            :placeholder="$t('ui.placeholder.all')"
            allow-clear
            style="width: 160px"
          />
        </Form.Item>
        <Form.Item :label="$t('page.system.pdfRecord.docNo')" name="docNo">
          <Input
            v-model:value="searchForm.docNo"
            :placeholder="$t('ui.placeholder.search')"
            allow-clear
            style="width: 200px"
            @press-enter="handleSearch"
          />
        </Form.Item>
        <Form.Item
          v-if="activeTab === 'generate'"
          :label="$t('page.system.pdfRecord.triggerType')"
          name="triggerType"
        >
          <Select
            v-model:value="searchForm.triggerType"
            :options="triggerTypeOptions"
            :placeholder="$t('ui.placeholder.all')"
            allow-clear
            style="width: 140px"
          />
        </Form.Item>
        <Form.Item>
          <Button type="primary" @click="handleSearch">
            {{ $t('ui.button.search') }}
          </Button>
        </Form.Item>
        <Form.Item>
          <Button @click="handleReset">{{ $t('ui.button.refresh') }}</Button>
        </Form.Item>
      </Form>
    </Card>

    <GenGrid
      v-show="activeTab === 'generate'"
      :table-title="$t('page.system.pdfRecord.tabGenerate')"
    >
      <template #docType="{ row }">
        <Tag>{{ docTypeMap[row.docType] || row.docType }}</Tag>
      </template>

      <template #fileSize="{ row }">
        {{ row.fileSize ? `${(row.fileSize / 1024).toFixed(1)} KB` : '-' }}
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
        <span v-else :title="row.errorMsg" class="text-xs text-red-500">
          {{ row.errorMsg }}
        </span>
      </template>
    </GenGrid>

    <DlGrid
      v-show="activeTab === 'download'"
      :table-title="$t('page.system.pdfRecord.tabDownload')"
    >
      <template #docType="{ row }">
        <Tag>{{ docTypeMap[row.docType] || row.docType }}</Tag>
      </template>

      <template #dlAction="{ row }">
        <Button
          type="link"
          size="small"
          @click="
            handleDownload({ id: row.recordId, fileName: row.fileName })
          "
        >
          {{ $t('page.system.pdfRecord.reDownload') }}
        </Button>
      </template>
    </DlGrid>
  </Page>
</template>

<style scoped>
.pdf-search-form :deep(.ant-form-item) {
  margin-right: 16px;
  margin-bottom: 0;
}

/* 默认内容区最小高度 150px：空数据和少数据都生效；数据多时按内容自适应撑开 */
:deep(.vxe-table--body-wrapper) {
  min-height: 150px;
}

:deep(.vxe-grid) {
  overflow: hidden;
}
</style>
