<script lang="ts" setup>
import { computed, ref, watch } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message, Upload } from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import { addBlockApi, updateBlockApi } from '#/api';
import { uploadFileApi } from '#/api/core/attachment/file';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增区块' : '修改区块'));

const blockTypeOptions = [
  { label: '文本', value: 1 },
  { label: 'HTML', value: 2 },
  { label: '图片', value: 3 },
  { label: '链接', value: 4 },
];

// 当前区块类型，用于动态切换内容字段展示
const currentBlockType = ref<number>(1);

function applyContentSchema(type: number) {
  baseFormApi.updateSchema([
    {
      fieldName: 'content',
      component: type === 2 ? 'Textarea' : 'Input',
      componentProps:
        type === 2
          ? { placeholder: '请输入HTML内容', allowClear: true, rows: 6 }
          : type === 4
            ? { placeholder: '请输入链接URL（含http://）', allowClear: true }
            : { placeholder: '请输入文本内容', allowClear: true },
    },
  ]);
}

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
      fieldName: 'blockCode',
      label: '区块编码',
      componentProps: {
        placeholder: '请输入区块编码（唯一标识）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入区块编码' }),
    },
    {
      component: 'Input',
      fieldName: 'blockName',
      label: '区块名称',
      componentProps: {
        placeholder: '请输入区块名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入区块名称' }),
    },
    {
      component: 'Select',
      fieldName: 'blockType',
      label: '区块类型',
      defaultValue: 1,
      componentProps: {
        options: blockTypeOptions,
        placeholder: '请选择区块类型',
        onChange: (value: number) => {
          currentBlockType.value = value;
        },
      },
      rules: z.number({ required_error: '请选择区块类型' }),
    },
    {
      component: 'Input',
      fieldName: 'content',
      label: '内容',
      componentProps: {
        placeholder: '请输入文本内容',
        allowClear: true,
      },
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

// 监听区块类型变化，动态切换内容字段的渲染组件
watch(currentBlockType, (type) => {
  applyContentSchema(type);
});

// 图片上传
const imageFileList = ref<UploadFile[]>([]);
const imageUrl = ref('');

async function handleImageUpload(file: File) {
  try {
    const res: any = await uploadFileApi(file, 'block');
    const url = res?.data?.url || res?.url;
    if (url) {
      imageUrl.value = url;
      imageFileList.value = [
        { uid: '-1', name: 'block', status: 'done' as const, url },
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
  if (url) {
    imageFileList.value = [
      { uid: '-1', name: 'block', status: 'done' as const, url },
    ];
  } else {
    imageFileList.value = [];
  }
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
    // 类型非图片时清空 imageUrl；类型非链接时清空 linkUrl
    if (values.blockType !== 3) {
      values.imageUrl = '';
    }
    if (values.blockType !== 4) {
      values.linkUrl = '';
    }

    try {
      if (isCreate.value) {
        await addBlockApi(values);
        message.success('新增成功');
      } else {
        await updateBlockApi(data.value.row.id, values);
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
      currentBlockType.value = row.blockType ?? 1;
      applyContentSchema(currentBlockType.value);
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

    <div v-if="currentBlockType === 3" class="px-4 pb-4">
      <div class="text-sm font-medium mb-2">区块图片</div>
      <Upload
        :file-list="imageFileList"
        :before-upload="(file: File) => { handleImageUpload(file); return false; }"
        :remove="handleImageRemove"
        list-type="picture-card"
        accept="image/*"
      >
        <div v-if="imageFileList.length < 1">
          <div class="text-2xl leading-none mb-1">+</div>
          <div class="text-xs">上传图片</div>
        </div>
      </Upload>
      <div class="text-xs text-gray-400 mt-1">仅当区块类型为"图片"时生效</div>
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
