<script lang="ts" setup>
import { ref, h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideSettings, LucidePlus, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Input, message, Modal, Popconfirm, Select, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  deleteSkuTemplateApi,
  getSkuTemplateInfoApi,
  getSkuTemplateListApi,
  saveSkuTemplateApi,
  saveTemplateSpecsApi,
  updateSkuTemplateApi,
} from '#/api';
import type { SkuTemplateListVO, SkuTemplateDetailVO, SpecSaveItem, SpecValueSaveItem } from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

/** 商品类型选项 */
const productTypeOptions = [
  { label: '实物产品', value: 'physical' },
  { label: '数字产品', value: 'digital' },
  { label: '服务', value: 'service' },
];

function getProductTypeLabel(val?: string): string {
  return productTypeOptions.find((o) => o.value === val)?.label || val || '-';
}

/** ============= 模板列表 ============= */

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '模板名称',
      componentProps: {
        placeholder: '搜索模板名称',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'productType',
      label: '商品类型',
      componentProps: {
        placeholder: '全部类型',
        allowClear: true,
        options: productTypeOptions,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getSkuTemplateListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          productType: formValues.productType,
        });
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 70 },
    { title: '模板名称', field: 'templateName', minWidth: 160 },
    {
      title: '商品类型',
      field: 'productType',
      width: 120,
      slots: { default: 'productType' },
    },
    { title: '规格数', field: 'specCount', width: 80 },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
      width: 170,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

/** ============= 新增/编辑模板弹窗 ============= */

const showTemplateModal = ref(false);
const editingTemplate = ref<{ id?: number; templateName: string; productType?: string; description?: string }>({
  templateName: '',
});

function openCreateTemplate() {
  editingTemplate.value = { templateName: '' };
  showTemplateModal.value = true;
}

function openEditTemplate(row: SkuTemplateListVO) {
  editingTemplate.value = {
    id: Number(row.id),
    templateName: row.templateName,
    productType: row.productType,
    description: row.description,
  };
  showTemplateModal.value = true;
}

async function handleSaveTemplate() {
  if (!editingTemplate.value.templateName.trim()) {
    message.warning('请输入模板名称');
    return;
  }
  try {
    if (editingTemplate.value.id) {
      await updateSkuTemplateApi({
        id: String(editingTemplate.value.id),
        templateName: editingTemplate.value.templateName,
        productType: editingTemplate.value.productType,
        description: editingTemplate.value.description,
      });
      message.success('模板更新成功');
    } else {
      await saveSkuTemplateApi({
        templateName: editingTemplate.value.templateName,
        productType: editingTemplate.value.productType,
        description: editingTemplate.value.description,
      });
      message.success('模板创建成功');
    }
    showTemplateModal.value = false;
    gridApi.query();
  } catch {
    message.error('操作失败');
  }
}

async function handleDeleteTemplate(id: number) {
  try {
    await deleteSkuTemplateApi(id);
    message.success('模板已删除');
    gridApi.query();
  } catch {
    message.error('删除失败');
  }
}

/** ============= 规格配置抽屉 ============= */

const specDrawerLoading = ref(false);
const selectedTemplate = ref<SkuTemplateDetailVO | null>(null);
const specList = ref<Array<{ name: string; values: string[] }>>([]);

async function openSpecDrawer(row: SkuTemplateListVO) {
  const id = Number(row.id);
  specDrawerLoading.value = true;
  specList.value = [];
  selectedTemplate.value = null;
  drawerApi.open();

  try {
    const detail = (await getSkuTemplateInfoApi(id)) as any;
    const data: SkuTemplateDetailVO = detail?.data ?? detail;
    selectedTemplate.value = data;
    if (data?.specs && Array.isArray(data.specs)) {
      specList.value = data.specs.map((s: any) => ({
        name: s.name || '',
        values: (s.values || []).map((v: any) => v.value).filter(Boolean),
      }));
    }
  } catch {
    message.error('加载模板数据失败');
  } finally {
    specDrawerLoading.value = false;
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleSaveSpecs();
  },
  onOpenChange(isOpen) {
    if (!isOpen) {
      selectedTemplate.value = null;
    }
  },
});

/** 添加规格行 */
function addSpecRow() {
  specList.value.push({ name: '', values: [] });
}

/** 删除规格行 */
function removeSpecRow(idx: number) {
  specList.value.splice(idx, 1);
}

/** 添加规格值 */
function addSpecValue(specIdx: number) {
  specList.value[specIdx].values.push('');
}

/** 删除规格值 */
function removeSpecValue(specIdx: number, valIdx: number) {
  specList.value[specIdx].values.splice(valIdx, 1);
}

/** 失焦时清理空值 */
function handleBlurSpecValue(specIdx: number, valIdx: number) {
  const val = specList.value[specIdx].values[valIdx];
  if (val && typeof val === 'string') {
    const trimmed = val.trim();
    if (trimmed === '') {
      removeSpecValue(specIdx, valIdx);
    } else {
      specList.value[specIdx].values[valIdx] = trimmed;
    }
  }
}

/** 保存规格 */
async function handleSaveSpecs() {
  if (!selectedTemplate.value?.id) return;

  const validSpecs = specList.value
      .filter((s) => s.name.trim())
      .map((s) => ({
        name: s.name.trim(),
        sortOrder: 0,
        values: s.values
          .map((v) => v.trim())
          .filter(Boolean)
          .map((v, idx) => ({ value: v, sortOrder: idx })),
      }));

    if (validSpecs.length === 0) {
      message.warning('请至少定义一个有效规格（名称+规格值）');
      return;
    }

    try {
      const resp = await saveTemplateSpecsApi({
        templateId: Number(selectedTemplate.value.id),
        specs: validSpecs,
      });
      console.log('save specs response:', resp);
      message.success('规格保存成功');
      gridApi.query();
      drawerApi.close();
    } catch (e: any) {
      message.error(`规格保存失败: ${e?.message || ''}`);
    }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.product.sku.title')">
      <!-- 自定义工具栏 -->
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('product:product:save')"
          type="primary"
          :icon="h(LucidePlus)"
          @click="openCreateTemplate"
        >
          新增模板
        </Button>
      </template>

      <template #productType="{ row }">
        <Tag>{{ getProductTypeLabel(row.productType) }}</Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt ?? row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('product:product:view')"
          type="link"
          :icon="h(LucideSettings)"
          @click="openSpecDrawer(row)"
        >
          规格配置
        </Button>
        <Button type="link" @click="openEditTemplate(row)">编辑</Button>
        <Popconfirm title="确定删除此模板？" @confirm="handleDeleteTemplate(Number(row.id))">
          <Button type="link" danger :icon="h(LucideTrash2)">删除</Button>
        </Popconfirm>
      </template>
    </Grid>

    <!-- 新增/编辑模板弹窗 -->
    <Modal
      v-model:open="showTemplateModal"
      :title="editingTemplate?.id ? '编辑模板' : '新增模板'"
      @ok="handleSaveTemplate"
      @cancel="showTemplateModal = false"
    >
      <div class="flex flex-col gap-4 py-2">
        <div>
          <label class="text-sm font-medium text-gray-700">模板名称 <span class="text-red-500">*</span></label>
          <Input
            v-model:value="editingTemplate.templateName"
            placeholder="请输入模板名称，如：服装SKU模板"
            class="mt-1"
          />
        </div>
        <div>
          <label class="text-sm font-medium text-gray-700">商品类型</label>
          <Select
            v-model:value="editingTemplate.productType"
            placeholder="请选择（可选）"
            :options="productTypeOptions"
            class="mt-1 w-full"
            allow-clear
          />
        </div>
        <div>
          <label class="text-sm font-medium text-gray-700">描述</label>
          <Input
            v-model:value="editingTemplate.description"
            placeholder="模板描述（可选）"
            class="mt-1"
          />
        </div>
      </div>
    </Modal>

    <!-- 规格配置抽屉 -->
    <Drawer
      :title="selectedTemplate ? `规格配置 - ${selectedTemplate.templateName}` : '规格配置'"
      class="sku-drawer"
      :loading="specDrawerLoading"
    >

      <div v-if="selectedTemplate" class="sku-container">
        <!-- 模板信息卡片 -->
        <div class="info-card mb-4">
          <div class="flex items-center justify-between">
            <div>
              <span class="text-lg font-semibold">{{ selectedTemplate.templateName }}</span>
              <Tag class="ml-2">{{ getProductTypeLabel(selectedTemplate.productType) }}</Tag>
            </div>
          </div>
          <div class="text-sm text-gray-400 mt-1">
            {{ selectedTemplate.description || '暂无描述' }}
          </div>
        </div>

        <!-- 规格定义区域 -->
        <div class="section-card">
          <div class="section-header">
            <div class="section-title">
              <span class="step-badge">1</span>
              <span>规格定义</span>
            </div>
            <Button size="small" type="dashed" @click="addSpecRow">
              + 添加规格
            </Button>
          </div>

          <div v-if="specList.length === 0" class="empty-tip">
            暂无规格定义，点击「添加规格」开始配置
          </div>

          <div v-for="(spec, sIdx) in specList" :key="sIdx" class="spec-item">
            <div class="spec-row">
              <div class="flex items-center gap-2 flex-1">
                <span class="spec-label">规格 {{ sIdx + 1 }}</span>
                <Input
                  v-model:value="spec.name"
                  placeholder="规格名称，如：颜色"
                  style="width: 140px"
                  size="small"
                />
                <span class="text-xs text-gray-400">值:</span>
                <div class="flex flex-wrap gap-1 flex-1">
                  <span
                    v-for="(val, vIdx) in spec.values"
                    :key="vIdx"
                    class="spec-value-tag"
                  >
                    <Input
                      v-model:value="spec.values[vIdx]"
                      placeholder="值"
                      size="small"
                      style="width: 70px"
                      @blur="handleBlurSpecValue(sIdx, vIdx)"
                      @keydown.enter.prevent="addSpecValue(sIdx)"
                    />
                    <Button
                      type="text"
                      size="small"
                      danger
                      @click="removeSpecValue(sIdx, vIdx)"
                    >
                      ×
                    </Button>
                  </span>
                  <Button type="link" size="small" @click="addSpecValue(sIdx)">
                    + 添加值
                  </Button>
                </div>
              </div>
              <Button type="text" danger size="small" @click="removeSpecRow(sIdx)">
                删除
              </Button>
            </div>
          </div>

          <div v-if="specList.length > 0" class="mt-3 text-xs text-gray-400">
            提示：定义好规格后点击「保存规格」，可在添加产品时选择此模板快速应用规格设置
          </div>
        </div>
      </div>

      <div v-else class="flex items-center justify-center h-40 text-gray-400">
        {{ specDrawerLoading ? '加载中...' : '请选择一个模板' }}
      </div>
    </Drawer>
  </Page>
</template>

<style scoped>
.sku-drawer :deep(.ant-drawer-body) {
  padding: 16px;
  overflow-y: auto;
}

.sku-container {
  min-height: 200px;
}

.info-card {
  background: linear-gradient(135deg, #f0f5ff 0%, #e6f7ff 100%);
  border: 1px solid #91d5ff;
  border-radius: 8px;
  padding: 16px;
}

.section-card {
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  padding: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 14px;
}

.step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: #1890ff;
  color: #fff;
  border-radius: 50%;
  font-size: 13px;
  font-weight: 600;
}

.empty-tip {
  text-align: center;
  padding: 32px 0;
  color: #bfbfbf;
  font-size: 13px;
}

.spec-item {
  margin-bottom: 12px;
  padding: 12px;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 6px;
}

.spec-item:last-child {
  margin-bottom: 0;
}

.spec-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.spec-label {
  font-size: 12px;
  font-weight: 500;
  color: #666;
  white-space: nowrap;
  line-height: 24px;
  min-width: 50px;
}

.spec-value-tag {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  background: #fff;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  padding: 1px 4px;
}
</style>
