<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message, Upload } from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import { templateApi } from '#/api';
import { uploadFileApi } from '#/api/core/attachment/file';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增模板' : '编辑模板'));

// 分类选项
const categoryOptions = ref<{ label: string; value: number }[]>([]);

async function loadCategoryOptions() {
  try {
    const { requestClient } = await import('#/api/request');
    const res: any = await requestClient.get('/api/system/template_category/options');
    const rawList = Array.isArray(res) ? res : res?.data || [];
    function flatten(list: any[], result: { label: string; value: number }[]) {
      for (const item of list) {
        result.push({ label: item.name || '', value: Number(item.id) });
        if (item.children?.length) flatten(item.children, result);
      }
    }
    flatten(rawList, categoryOptions.value);
  } catch {
    categoryOptions.value = [];
  }
}

onMounted(() => {
  loadCategoryOptions();
});

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
      fieldName: 'name',
      label: '模板名称',
      componentProps: {
        placeholder: '请输入模板名称（唯一标识）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入模板名称' }),
    },
    {
      component: 'Input',
      fieldName: 'templateFolder',
      label: '文件夹名称',
      componentProps: {
        placeholder: '模板文件存储目录',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'categoryId',
      label: '所属分类',
      componentProps: {
        placeholder: '请选择分类',
        allowClear: true,
        options: categoryOptions,
        showSearch: true,
        filterOption: (input: string, option: any) => option.label?.toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Input',
      fieldName: 'previewPic',
      label: '预览图',
    },
    {
      component: 'Input',
      fieldName: 'previewUrl',
      label: '演示网址',
      componentProps: {
        placeholder: '模板在线演示地址',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: '备注说明',
      componentProps: {
        type: 'textarea',
        autosize: { minRows: 3, maxRows: 6 },
        placeholder: '模板简介、功能说明等',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: { min: 0 },
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

const [Drawer, drawerApi] = useVbenDrawer({
  width: '60%',
  drawerStyle: { maxWidth: '100vw' },
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    const values = await baseFormApi.getValues();
    // 合并预览图URL
    values.previewPic = previewPicUrl.value;
    try {
      if (isCreate.value) {
        await templateApi.add(values);
        message.success('新增成功');
      } else {
        await templateApi.update(data.value.row.id, values);
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
      previewPicUrl.value = row.previewPic || '';
      syncFileList(previewPicUrl.value);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

// 预览图上传
const fileList = ref<UploadFile[]>([]);
const previewPicUrl = ref('');

async function handleUpload(file: File) {
  try {
    const res: any = await uploadFileApi(file, 'template');
    const url = res?.data?.url || res?.url;
    if (url) {
      previewPicUrl.value = url;
      syncFileList(url);
      message.success('上传成功');
    }
    return false;
  } catch {
    message.error('上传失败');
    return false;
  }
}

function handleRemove() {
  fileList.value = [];
  previewPicUrl.value = '';
}

function syncFileList(url: string) {
  if (url) {
    fileList.value = [{ uid: '-1', name: 'preview', status: 'done', url }];
  } else {
    fileList.value = [];
  }
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
    <div class="px-4 pb-4">
      <div class="text-sm font-medium mb-2">预览图上传</div>
      <Upload
        :file-list="fileList"
        :before-upload="(file: File) => { handleUpload(file); return false; }"
        :remove="handleRemove"
        list-type="picture-card"
        accept="image/*"
      >
        <div v-if="fileList.length < 1">
          <div class="text-2xl leading-none mb-1">+</div>
          <div class="text-xs">上传预览图</div>
        </div>
      </Upload>
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
