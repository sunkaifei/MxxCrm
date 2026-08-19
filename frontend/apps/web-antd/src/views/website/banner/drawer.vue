<script lang="ts" setup>
import type { UploadFile } from 'ant-design-vue';

import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message, Upload } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { addBannerApi, updateBannerApi } from '#/api';
import { uploadFileApi } from '#/api/core/attachment/file';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增Banner' : '修改Banner'));

const positionOptions = [
  { label: '首页顶部', value: 'home_top' },
  { label: '首页中部', value: 'home_middle' },
  { label: '首页底部', value: 'home_bottom' },
  { label: '侧栏顶部', value: 'sidebar_top' },
  { label: '侧栏底部', value: 'sidebar_bottom' },
  { label: '分类顶部', value: 'category_top' },
];

const targetOptions = [
  { label: '当前窗口（_self）', value: '_self' },
  { label: '新窗口（_blank）', value: '_blank' },
];

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'title',
      label: '标题',
      componentProps: {
        placeholder: '请输入标题',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入标题' }),
    },
    {
      component: 'Input',
      fieldName: 'imageUrl',
      label: '图片URL',
      componentProps: {
        style: 'display: none',
      },
    },
    {
      component: 'Input',
      fieldName: 'linkUrl',
      label: '链接URL',
      componentProps: {
        placeholder: '请输入链接URL（含http://）',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'altText',
      label: 'ALT文本',
      componentProps: {
        placeholder: '请输入图片ALT文本',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'position',
      label: '位置',
      defaultValue: 'home_top',
      componentProps: {
        options: positionOptions,
        placeholder: '请选择位置',
      },
      rules: z.string().min(1, { message: '请选择位置' }),
    },
    {
      component: 'Select',
      fieldName: 'target',
      label: '打开方式',
      defaultValue: '_blank',
      componentProps: {
        options: targetOptions,
        placeholder: '请选择打开方式',
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: {
        min: 0,
        style: 'width: 100%',
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'startTime',
      label: '开始时间',
      componentProps: {
        showTime: true,
        format: 'YYYY-MM-DD HH:mm:ss',
        placeholder: '请选择开始时间',
        style: 'width: 100%',
        allowClear: true,
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'endTime',
      label: '结束时间',
      componentProps: {
        showTime: true,
        format: 'YYYY-MM-DD HH:mm:ss',
        placeholder: '请选择结束时间',
        style: 'width: 100%',
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      label: '状态',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: '启用', value: 1 },
          { label: '禁用', value: 0 },
        ],
      },
    },
  ],
});

// 图片上传
const imageFileList = ref<UploadFile[]>([]);
const imageUrl = ref('');

async function handleImageUpload(file: File) {
  try {
    const res: any = await uploadFileApi(file, 'banner');
    const url = res?.data?.url || res?.url;
    if (url) {
      imageUrl.value = url;
      imageFileList.value = [
        { uid: '-1', name: 'banner', status: 'done' as const, url },
      ];
      message.success('上传成功');
    }
    return false;
  } catch {
    message.error('上传失败');
    return false;
  }
}

function handleImageRemove() {
  imageFileList.value = [];
  imageUrl.value = '';
}

function syncImageFileList(url: string) {
  imageFileList.value = url
    ? [{ uid: '-1', name: 'banner', status: 'done' as const, url }]
    : [];
}

function formatTime(value: any): string | undefined {
  if (!value) return undefined;
  if (typeof value === 'string') return value;
  if (typeof value?.format === 'function')
    return value.format('YYYY-MM-DD HH:mm:ss');
  return undefined;
}

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[60%] max-w-[100vw]',
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) {
      return;
    }

    setLoading(true);

    const values = await baseFormApi.getValues();
    values.imageUrl = imageUrl.value;
    values.startTime = formatTime(values.startTime);
    values.endTime = formatTime(values.endTime);

    try {
      if (isCreate.value) {
        await addBannerApi(values);
        message.success('新增成功');
      } else {
        await updateBannerApi(data.value.row.id, values);
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};
      baseFormApi.setValues(row);
      imageUrl.value = row.imageUrl || '';
      syncImageFileList(imageUrl.value);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />

    <div class="px-4 pb-4">
      <div class="text-sm font-medium mb-2">Banner 图片</div>
      <Upload
        :file-list="imageFileList"
        :before-upload="
          (file: File) => {
            handleImageUpload(file);
            return false;
          }
        "
        :remove="handleImageRemove"
        list-type="picture-card"
        accept="image/*"
      >
        <div v-if="imageFileList.length === 0">
          <div class="text-2xl leading-none mb-1">+</div>
          <div class="text-xs">上传图片</div>
        </div>
      </Upload>
      <div class="text-xs text-gray-400 mt-1">
        建议尺寸 1920x500，支持 JPG/PNG/GIF
      </div>
    </div>
  </Drawer>
</template>

<style>
@media (max-width: 767px) {
  .vben-drawer .ant-drawer-content-wrapper {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>
