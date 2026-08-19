<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideFilePenLine,
  LucidePlus,
  LucideSearch,
  LucideTrash2,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Col,
  Drawer,
  Form,
  Input,
  Popconfirm,
  Row,
  Select,
  Spin,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { demoPdfApi } from '#/api/core/system/pdf';
import {
  bathDeletePdfTemplateApi,
  getPdfTemplateListApi,
  setDefaultPdfTemplateApi,
} from '#/api/core/system/pdf-template';
import { $t } from '#/locales';
import { statusList } from '#/store';

import EditorDrawer from './editor.vue';

// 单据类型选项
const docTypeOptions = [
  { label: $t('page.system.pdfTemplate.docTypeQuotation'), value: 'quotation' },
  { label: $t('page.system.pdfTemplate.docTypeOrder'), value: 'order' },
  { label: $t('page.system.pdfTemplate.docTypeContract'), value: 'contract' },
];

// 单据类型 -> 标签颜色
const docTypeColorMap: Record<string, string> = {
  quotation: 'blue',
  order: 'green',
  contract: 'orange',
};

// 方向选项
const orientationOptions = [
  { label: $t('page.system.pdfTemplate.portrait'), value: 'portrait' },
  { label: $t('page.system.pdfTemplate.landscape'), value: 'landscape' },
];

// 当前激活的单据类型 Tab
const activeTab = ref('quotation');

// 搜索表单
const searchForm = ref({
  name: '',
  status: undefined as number | undefined,
});

// 表格配置
const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  pagerConfig: {},
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        return await getPdfTemplateListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          name: searchForm.value.name || undefined,
          docType: activeTab.value,
          status: searchForm.value.status,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 70 },
    {
      title: $t('page.system.pdfTemplate.name'),
      field: 'name',
      minWidth: 160,
      slots: { default: 'tplName' },
    },
    {
      title: $t('page.system.pdfTemplate.code'),
      field: 'code',
      width: 140,
    },
    {
      title: $t('page.system.pdfTemplate.docType'),
      field: 'docType',
      width: 100,
      align: 'center',
      slots: { default: 'docType' },
    },
    {
      title: $t('page.system.pdfTemplate.paperSize'),
      field: 'paperSize',
      width: 100,
      align: 'center',
    },
    {
      title: $t('page.system.pdfTemplate.orientation'),
      field: 'orientation',
      width: 90,
      align: 'center',
      slots: { default: 'orientation' },
    },
    {
      title: $t('page.system.pdfTemplate.isDefault'),
      field: 'isDefault',
      width: 90,
      align: 'center',
      slots: { default: 'isDefault' },
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      width: 90,
      align: 'center',
      slots: { default: 'status' },
    },
    {
      title: $t('page.system.pdfTemplate.sort'),
      field: 'sort',
      width: 80,
      align: 'center',
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      width: 180,
      align: 'center',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

// 切换 Tab 时重新加载列表
function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  gridApi.query();
}

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.value = { name: '', status: undefined };
  gridApi.query();
}

// ==================== 编辑抽屉 ====================
const editorVisible = ref(false);
const editorData = ref<any>(null);

function handleCreate() {
  editorData.value = null;
  editorVisible.value = true;
}

function handleEdit(row: any) {
  editorData.value = row;
  editorVisible.value = true;
}

function handleEditorSaved() {
  gridApi.query();
}

// 设为默认
async function handleSetDefault(row: any) {
  row.pending = true;
  try {
    await setDefaultPdfTemplateApi(row.id);
    window.$message.success($t('ui.notification.update_success'));
    gridApi.query();
  } finally {
    row.pending = false;
  }
}

// 删除
async function handleDelete(row: any) {
  row.pending = true;
  try {
    await bathDeletePdfTemplateApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// ==================== 演示预览 ====================
const previewVisible = ref(false);
const previewUrl = ref('');
const previewLoading = ref(false);

async function handlePreview(row: any) {
  previewLoading.value = true;
  previewVisible.value = true;
  previewUrl.value = '';
  try {
    const blob: any = await demoPdfApi(row.id);
    const pdfBlob =
      blob instanceof Blob
        ? blob
        : new Blob([blob], { type: 'application/pdf' });
    previewUrl.value = window.URL.createObjectURL(pdfBlob);
  } catch {
    window.$message.error('演示PDF生成失败');
    previewVisible.value = false;
  } finally {
    previewLoading.value = false;
  }
}

function handlePreviewClose() {
  if (previewUrl.value) {
    window.URL.revokeObjectURL(previewUrl.value);
    previewUrl.value = '';
  }
  previewVisible.value = false;
}
</script>

<template>
  <Page>
    <Card :bordered="false" style="margin-bottom: 16px">
      <Tabs
        v-model:active-key="activeTab"
        style="margin-bottom: 16px"
        @change="handleTabChange"
      >
        <Tabs.TabPane
          v-for="opt in docTypeOptions"
          :key="opt.value"
          :tab="opt.label"
        />
      </Tabs>

      <Form
        :model="searchForm"
        layout="inline"
        :label-col="{ style: { width: '80px' } }"
        class="pdf-search-form"
      >
        <div class="pdf-search-form-wrapper">
          <Row :gutter="[16, 12]" style="width: 100%">
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item
                :label="$t('page.system.pdfTemplate.name')"
                name="name"
              >
                <Input
                  v-model:value="searchForm.name"
                  :placeholder="$t('ui.placeholder.input')"
                  allow-clear
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item :label="$t('ui.table.status')" name="status">
                <Select
                  v-model:value="searchForm.status"
                  :options="statusList"
                  :placeholder="$t('ui.placeholder.select')"
                  allow-clear
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
          </Row>
        </div>

        <div class="mt-3 flex flex-wrap items-center gap-2">
          <Button type="default" :icon="h(LucideSearch)" @click="handleSearch">
            {{ $t('ui.button.search') }}
          </Button>
          <Button type="default" @click="handleReset">
            {{ $t('ui.button.refresh') }}
          </Button>
          <Button type="primary" :icon="h(LucidePlus)" @click="handleCreate">
            {{ $t('page.system.pdfTemplate.button.create') }}
          </Button>
        </div>
      </Form>
    </Card>

    <Grid :table-title="$t('page.system.pdfTemplate.title')">
      <template #tplName="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => handlePreview(row)"
        >
          {{ row.name }}
        </a>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #docType="{ row }">
        <Tag :color="docTypeColorMap[row.docType] || 'default'">
          {{
            docTypeOptions.find((o) => o.value === row.docType)?.label ||
            row.docType
          }}
        </Tag>
      </template>

      <template #orientation="{ row }">
        {{
          orientationOptions.find((o) => o.value === row.orientation)?.label ||
          row.orientation
        }}
      </template>

      <template #isDefault="{ row }">
        <Tag :color="row.isDefault ? 'gold' : 'default'">
          {{ row.isDefault ? $t('page.system.pdfTemplate.default') : '-' }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="row.status === 1 ? 'success' : 'default'">
          {{
            row.status === 1 ? $t('ui.switch.active') : $t('ui.switch.inactive')
          }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="!row.isDefault"
          type="primary"
          link
          :loading="row.pending"
          @click="() => handleSetDefault(row)"
        >
          {{ $t('page.system.pdfTemplate.button.setDefault') }}
        </Button>
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.system.pdfTemplate.module'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <Button danger link :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>

    <!-- 模板编辑抽屉 -->
    <EditorDrawer
      v-model:visible="editorVisible"
      :data="editorData"
      :default-doc-type="activeTab"
      @saved="handleEditorSaved"
    />

    <!-- 演示预览抽屉 -->
    <Drawer
      v-model:open="previewVisible"
      title="模板演示预览"
      :width="800"
      placement="right"
      :destroy-on-close="true"
      :footer="null"
      @close="handlePreviewClose"
    >
      <Spin :spinning="previewLoading" tip="正在生成演示PDF...">
        <iframe
          v-if="previewUrl"
          :src="previewUrl"
          class="w-full"
          style="height: calc(100vh - 160px); border: none"
        ></iframe>
      </Spin>
    </Drawer>
  </Page>
</template>

<style scoped>
.pdf-search-form :deep(.ant-form-item) {
  margin-bottom: 0;
}

.pdf-search-form :deep(.ant-form-item-control) {
  flex: 1;
}

.pdf-search-form-wrapper {
  width: 100%;
}

@media (min-width: 768px) {
  .pdf-search-form-wrapper {
    width: 60%;
  }
}

:deep(.vxe-table--empty-block) {
  min-height: 150px;
}

:deep(.vxe-grid) {
  overflow: hidden;
}
</style>
