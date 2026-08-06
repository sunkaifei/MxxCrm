<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import type { VbenFormSchema } from '@vben/common-ui';
import { $t } from '#/locales';
import { createBrandApi, getBrandInfoApi, updateBrandApi } from '#/api';
import { message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'brand-drawer',
  { 'brand-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const statusOptions = [
  { label: '正常', value: 0 },
  { label: '停用', value: 1 },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'name',
    label: '品牌名称',
    rules: 'required',
    componentProps: { placeholder: '请输入品牌名称', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'englishName',
    label: '品牌英文名',
    componentProps: { placeholder: '请输入品牌英文名', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'logo',
    label: 'Logo',
    componentProps: { placeholder: '请输入Logo图片URL', allowClear: true },
  },
  {
    component: 'Textarea',
    fieldName: 'description',
    label: '品牌介绍',
    componentProps: { placeholder: '请输入品牌介绍', rows: 3, allowClear: true },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'originCountry',
    label: '品牌原产国',
    componentProps: { placeholder: '请输入原产国', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'website',
    label: '官网',
    componentProps: { placeholder: 'https://example.com', allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'status',
    label: '状态',
    defaultValue: 0,
    componentProps: { placeholder: '请选择状态', options: statusOptions },
  },
  {
    component: 'InputNumber',
    fieldName: 'sortOrder',
    label: '排序',
    defaultValue: 0,
    componentProps: { placeholder: '排序号', min: 0, precision: 0, style: { width: '100%' } },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '请输入备注', rows: 2, allowClear: true },
    formItemClass: 'col-span-2',
  },
];

const [MainForm, mainFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: formSchema,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    try {
      const valid = await mainFormApi.validate();
      if (!valid.valid) return;

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();

      if (drawerData.value.create) {
        await createBrandApi(values);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateBrandApi({ ...values, id: drawerData.value.row.id });
        message.success($t('ui.notification.update_success'));
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      confirmLoading.value = false;
    }
  },
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{ create: boolean; row?: any }>() || { create: true };
      mainFormApi.resetForm();
      confirmLoading.value = false;
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getBrandInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) => (v === null || v === undefined ? undefined : Number(v));

    mainFormApi.setValues({
      name: data.name,
      englishName: data.englishName,
      logo: data.logo,
      description: data.description,
      originCountry: data.originCountry,
      website: data.website,
      status: num(data.status) ?? 0,
      sortOrder: num(data.sortOrder) ?? 0,
      remark: data.remark,
    });
  } catch (e) {
    console.error('[品牌] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建品牌' : '编辑品牌'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button type="button" class="brand-drawer__fs-btn" @click="toggleFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="brand-drawer__body">
      <MainForm />
    </div>
  </Drawer>
</template>

<style>
.brand-drawer {
  width: 75vw !important;
}

.brand-drawer--fullscreen {
  width: 100vw !important;
}

.brand-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}

.brand-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.brand-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}
</style>