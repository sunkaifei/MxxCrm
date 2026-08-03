<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import {
  categoryApi,
  getTemplateDataListByTemplateApi,
  siteApi,
} from '#/api';
import { message } from 'ant-design-vue';
import type { CategorySaveDTO } from '#/api/core/shop/category';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增分类' : '编辑分类'));

// 列表/详情模板选项（按当前站点 templateId 加载）
const listTemplateOptions = ref<Array<{ label: string; value: number }>>([]);
const detailTemplateOptions = ref<Array<{ label: string; value: number }>>([]);

async function loadTemplateOptions() {
  try {
    const site: any = await siteApi.getCurrent();
    const templateId = site?.templateId;
    if (!templateId) {
      listTemplateOptions.value = [];
      detailTemplateOptions.value = [];
      return;
    }
    const resp: any = await getTemplateDataListByTemplateApi(
      Number(templateId),
    );
    const list = resp?.data ?? resp ?? [];
    const items = Array.isArray(list) ? list : [];
    listTemplateOptions.value = items
      .filter((item: any) => item.typeId === 2)
      .map((item: any) => ({ label: item.name, value: item.id }));
    detailTemplateOptions.value = items
      .filter((item: any) => item.typeId === 3)
      .map((item: any) => ({ label: item.name, value: item.id }));
  } catch (e) {
    console.error('[分类] 加载模板选项失败:', e);
    listTemplateOptions.value = [];
    detailTemplateOptions.value = [];
  }
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
      fieldName: 'name',
      label: '分类名称',
      componentProps: {
        placeholder: '请输入分类名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入分类名称' }),
    },
    {
      component: 'Input',
      fieldName: 'shortUrl',
      label: '短链接',
      componentProps: {
        placeholder: '请输入短链接',
        allowClear: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sortOrder',
      label: '排序值',
      defaultValue: 0,
      componentProps: {
        min: 0,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'isShow',
      label: '是否显示',
      defaultValue: 1,
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: [
          { label: '显示', value: 1 },
          { label: '隐藏', value: 0 },
        ],
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'pageType',
      label: '页面模式',
      defaultValue: 2,
      componentProps: {
        optionType: 'button',
        options: [
          { label: '封面模式', value: 1 },
          { label: '列表模式', value: 2 },
        ],
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'contentType',
      label: '内容类型',
      defaultValue: 1,
      componentProps: {
        optionType: 'button',
        options: [
          { label: '文章', value: 1 },
          { label: '自定义链接', value: 3 },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'linkUrl',
      label: '自定义链接URL',
      componentProps: {
        placeholder: '请输入链接URL',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'bannerImage',
      label: '栏目Banner',
      componentProps: {
        placeholder: '请输入Banner图片URL',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'listTemplateDataId',
      label: '列表模板',
      componentProps: {
        placeholder: '默认',
        options: listTemplateOptions,
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Select',
      fieldName: 'detailTemplateDataId',
      label: '详情模板',
      componentProps: {
        placeholder: '默认',
        options: detailTemplateOptions,
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          (option?.label ?? '').toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '栏目简介',
      componentProps: {
        placeholder: '请输入栏目简介',
        allowClear: true,
        rows: 3,
      },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
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
    const parentId = data.value?.parentId ?? 0;
    values.parentId = parentId;

    try {
      if (data.value?.create) {
        await categoryApi.save(values as CategorySaveDTO);
      } else {
        await categoryApi.update({
          ...values,
          id: data.value.row.id,
        } as CategorySaveDTO);
      }

      message.success(data.value?.create ? '新增成功' : '更新成功');
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      loadTemplateOptions();
      baseFormApi.setValues(data.value?.row || {});
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
  </Drawer>
</template>
