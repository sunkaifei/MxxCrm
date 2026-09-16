<script lang="ts" setup>
/**
 * PDF 素材管理（设计文档 §4.2、§26.1）。
 *
 * 素材分类：logo / seal（电子签章）/ background（底图·预印纸）/ font / barcode。
 * 上传走 `/api/system/pdf-asset/upload`，**返回 JSON**（与项目上传接口惯例一致）。
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucidePlus, LucideSearch, LucideTrash2, LucideUpload } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Col,
  Form,
  Image,
  Input,
  Modal,
  Popconfirm,
  Row,
  Select,
  Tag,
  Upload,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deletePdfAssetApi,
  getPdfAssetListApi,
  uploadPdfAssetApi,
} from '#/api/core/system/pdf-designer';
import { $t } from '#/locales';

const CATEGORY_OPTIONS = [
  { label: '公司 Logo', value: 'logo' },
  { label: '电子签章', value: 'seal' },
  { label: '底图 / 预印纸', value: 'background' },
  { label: '字体', value: 'font' },
  { label: '条码图片', value: 'barcode' },
];

const CATEGORY_COLOR: Record<string, string> = {
  logo: 'blue',
  seal: 'red',
  background: 'orange',
  font: 'purple',
  barcode: 'green',
};

const searchForm = ref({ category: undefined as any, name: '' });

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        return await getPdfAssetListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          category: searchForm.value.category || undefined,
          name: searchForm.value.name || undefined,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 70 },
    {
      title: '预览',
      field: 'fileUrl',
      width: 90,
      align: 'center',
      slots: { default: 'preview' },
    },
    { title: '名称', field: 'name', minWidth: 160 },
    {
      title: '分类',
      field: 'category',
      width: 130,
      align: 'center',
      slots: { default: 'category' },
    },
    {
      title: '尺寸',
      field: 'widthPx',
      width: 120,
      align: 'center',
      slots: { default: 'size' },
    },
    {
      title: '文件大小',
      field: 'fileSize',
      width: 110,
      align: 'center',
      slots: { default: 'fileSize' },
    },
    {
      title: 'MD5',
      field: 'md5',
      width: 140,
      slots: { default: 'md5' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 170,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      width: 90,
      align: 'center',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.value = { category: undefined, name: '' };
  gridApi.query();
}

// ==================== 上传 ====================
const uploadVisible = ref(false);
const uploading = ref(false);
const uploadForm = ref({ category: 'logo' as string, name: '' });
const pendingFile = ref<File | null>(null);

function openUpload() {
  uploadForm.value = { category: 'logo', name: '' };
  pendingFile.value = null;
  uploadVisible.value = true;
}

function pickFile(file: File) {
  pendingFile.value = file;
  if (!uploadForm.value.name) uploadForm.value.name = file.name;
  return false;
}

async function submitUpload() {
  if (!pendingFile.value) {
    window.$message.warning('请先选择文件');
    return;
  }
  uploading.value = true;
  try {
    await uploadPdfAssetApi(
      pendingFile.value,
      uploadForm.value.category,
      uploadForm.value.name,
    );
    window.$message.success('上传成功');
    uploadVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    window.$message.error(e?.message ?? '上传失败');
  } finally {
    uploading.value = false;
  }
}

async function handleDelete(row: any) {
  await deletePdfAssetApi([Number(row.id)]);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

function humanSize(n?: number) {
  if (!n) return '-';
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(2)} MB`;
}
</script>

<template>
  <Page>
    <Card :bordered="false" style="margin-bottom: 16px">
      <Form
        :model="searchForm"
        class="pdf-asset-search"
        layout="inline"
        :label-col="{ style: { width: '80px' } }"
      >
        <div class="pdf-asset-search-wrapper">
          <Row :gutter="[16, 12]" style="width: 100%">
            <Col :md="12" :sm="24" :xs="24">
              <Form.Item label="名称" name="name">
                <Input
                  v-model:value="searchForm.name"
                  allow-clear
                  :placeholder="$t('ui.placeholder.input')"
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
            <Col :md="12" :sm="24" :xs="24">
              <Form.Item label="分类" name="category">
                <Select
                  v-model:value="searchForm.category"
                  allow-clear
                  :options="CATEGORY_OPTIONS"
                  :placeholder="$t('ui.placeholder.select')"
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
          </Row>
        </div>
        <div class="mt-3 flex flex-wrap items-center gap-2">
          <Button :icon="h(LucideSearch)" type="default" @click="handleSearch">
            {{ $t('ui.button.search') }}
          </Button>
          <Button type="default" @click="handleReset">
            {{ $t('ui.button.refresh') }}
          </Button>
          <Button :icon="h(LucidePlus)" type="primary" @click="openUpload">
            上传素材
          </Button>
        </div>
      </Form>
    </Card>

    <Grid table-title="PDF 设计素材">
      <template #preview="{ row }">
        <Image
          :height="40"
          :src="row.fileUrl"
          :width="56"
          style="object-fit: contain"
        />
      </template>

      <template #category="{ row }">
        <Tag :color="CATEGORY_COLOR[row.category] || 'default'">
          {{ CATEGORY_OPTIONS.find((c) => c.value === row.category)?.label || row.category }}
        </Tag>
      </template>

      <template #size="{ row }">
        {{ row.widthPx ? `${row.widthPx}×${row.heightPx}` : '-' }}
      </template>

      <template #fileSize="{ row }">
        {{ humanSize(row.fileSize) }}
      </template>

      <template #md5="{ row }">
        <span class="asset-md5">{{ row.md5 ? row.md5.slice(0, 12) : '-' }}</span>
      </template>

      <template #createdAt="{ row }">
        {{ row.createTime ? formatDateTime(row.createTime) : '-' }}
      </template>

      <template #action="{ row }">
        <Popconfirm
          :cancel-text="$t('ui.button.cancel')"
          :ok-text="$t('ui.button.ok')"
          title="确定删除该素材？引用了它的模板将无法出图。"
          @confirm="() => handleDelete(row)"
        >
          <Button danger link :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>

    <Modal
      v-model:open="uploadVisible"
      :confirm-loading="uploading"
      ok-text="上传"
      title="上传素材"
      @ok="submitUpload"
    >
      <div class="asset-upload">
        <div class="asset-upload-row">
          <span class="asset-label">分类</span>
          <Select v-model:value="uploadForm.category" :options="CATEGORY_OPTIONS" />
        </div>
        <div class="asset-upload-row">
          <span class="asset-label">名称</span>
          <Input v-model:value="uploadForm.name" placeholder="留空则使用文件名" />
        </div>
        <div class="asset-upload-row">
          <span class="asset-label">文件</span>
          <Upload
            :before-upload="pickFile"
            :max-count="1"
            :show-upload-list="true"
            accept="image/png,image/jpeg,image/svg+xml"
          >
            <Button :icon="h(LucideUpload)">选择图片</Button>
          </Upload>
        </div>
        <div class="asset-upload-tip">
          建议使用透明背景 PNG。底图（预印纸）请上传整页扫描图或设计稿导出图。
        </div>
      </div>
    </Modal>
  </Page>
</template>

<style scoped>
.pdf-asset-search :deep(.ant-form-item) {
  margin-bottom: 0;
}
.pdf-asset-search :deep(.ant-form-item-control) {
  flex: 1;
}
.pdf-asset-search-wrapper {
  width: 100%;
}
@media (min-width: 768px) {
  .pdf-asset-search-wrapper {
    width: 60%;
  }
}
.asset-md5 {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  color: #8c8c8c;
}
.asset-upload-row {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 12px;
}
.asset-label {
  width: 48px;
  font-size: 12px;
  color: #8c8c8c;
}
.asset-upload-tip {
  margin-top: 6px;
  font-size: 11px;
  line-height: 1.7;
  color: #bfbfbf;
}
</style>
