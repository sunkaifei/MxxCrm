<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message, Upload } from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import { linksApi, siteApi } from '#/api';
import { uploadFileApi } from '#/api/core/attachment/file';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增友情链接' : '修改友情链接'));

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
      fieldName: 'linkName',
      label: '链接名称',
      componentProps: {
        placeholder: '请输入链接名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入链接名称' }),
    },
    {
      component: 'Input',
      fieldName: 'linkUrl',
      label: '链接地址',
      componentProps: {
        placeholder: '请输入链接地址（含http://）',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入链接地址' }),
    },
    {
      component: 'RadioGroup',
      fieldName: 'linkType',
      label: '链接类型',
      defaultValue: 0,
      componentProps: {
        options: [
          { label: '文字链接', value: 0 },
          { label: 'Logo链接', value: 1 },
        ],
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      label: '状态',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: '显示', value: 1 },
          { label: '隐藏', value: 0 },
        ],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: {
        min: 0,
      },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[80%] max-w-[100vw]',
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
    // 将 linkLogo 值合并到提交数据中
    values.linkLogo = linkLogoUrl.value;

    try {
      if (isCreate.value) {
        // 单站模式：自动注入当前站点 ID
        const site: any = await siteApi.getCurrent();
        values.websiteId = site?.id;
        await linksApi.add(values);
        message.success('新增成功');
      } else {
        await linksApi.update(data.value.row.id, values);
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
      // 同步 Logo URL
      linkLogoUrl.value = row.linkLogo || '';
      syncLogoFileList(linkLogoUrl.value);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

// Logo 图片上传
const logoFileList = ref<UploadFile[]>([]);
const linkLogoUrl = ref('');

async function handleLogoUpload(file: File) {
  try {
    const res: any = await uploadFileApi(file, 'links');
    const url = res?.data?.url || res?.url;
    if (url) {
      linkLogoUrl.value = url;
      syncLogoFileList(url);
      message.success('Logo上传成功');
    }
    return false;
  } catch {
    message.error('Logo上传失败');
    return false;
  }
}

function handleLogoRemove() {
  logoFileList.value = [];
  linkLogoUrl.value = '';
}

function syncLogoFileList(url: string) {
  if (url) {
    logoFileList.value = [
      {
        uid: '-1',
        name: 'logo',
        status: 'done',
        url: url,
      },
    ];
  } else {
    logoFileList.value = [];
  }
}

// function handleLogoPreview(file: UploadFile) {
//   // 使用 ant-design-vue 的 image preview
// }
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />

    <div class="px-4 pb-4">
      <div class="text-sm font-medium mb-2">Logo 图片</div>
      <Upload
        :file-list="logoFileList"
        :before-upload="(file: File) => { handleLogoUpload(file); return false; }"
        :remove="handleLogoRemove"
        list-type="picture-card"
        accept="image/*"
      >
        <div v-if="logoFileList.length < 1">
          <div class="text-2xl leading-none mb-1">+</div>
          <div class="text-xs">上传Logo</div>
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
