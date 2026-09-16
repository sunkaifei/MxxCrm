<script lang="ts" setup>
import type { ContentModelSaveDTO } from '#/api/core/website/content-model';

import { computed, onMounted, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  addContentModelApi,
  getContentModelDetailApi,
  updateContentModelApi,
} from '#/api';
import { templateApi } from '#/api/core/website/template';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增模型' : '编辑模型',
);

// 最大化 / 还原
const isFullscreen = ref(false);
function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
  drawerApi.setState({ class: isFullscreen.value ? 'w-full' : 'w-[75%]' });
}

/** 12-D：模板下拉（前台 /{modelCode} 列表/详情页用） */
const templateOptions = ref<{ label: string; value: number }[]>([]);
onMounted(async () => {
  try {
    const res: any = await templateApi.list({ page: 1, pageSize: 999, status: 1 });
    const items = res?.items || res?.rows || [];
    templateOptions.value = items.map((t: any) => ({
      label: t.name || `模板#${t.id}`,
      value: Number(t.id),
    }));
  } catch {
    templateOptions.value = [];
  }
});

const switchFields = [
  'hasTitle',
  'hasContent',
  'hasCover',
  'hasAuthor',
  'hasSummary',
  'hasSeo',
  'hasImages',
  'hasAttachment',
] as const;

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  // 大屏两列网格，窄屏单列；col-span-2 的项占满整行
  wrapperClass: 'grid-cols-1 md:grid-cols-2 gap-x-6',
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    // ── 基础信息 ──
    {
      component: 'Input',
      fieldName: 'modelCode',
      label: '模型编码',
      formItemClass: 'cm-field-code',
      help: '作为数据表名的一部分（mxx_model_编码），建议全小写英文；保存后不可修改',
      componentProps: () => ({
        placeholder: '如 case（小写字母/数字/下划线）',
        disabled: !isCreate.value,
      }),
      rules: z
        .string()
        .min(1, { message: '请输入模型编码' })
        .regex(/^[a-zA-Z][a-zA-Z0-9_]*$/, {
          message: '编码须为英文标识符（字母开头，仅含字母、数字、下划线），中文或特殊字符会导致数据表创建失败',
        }),
    },
    {
      component: 'Input',
      fieldName: 'modelName',
      label: '模型名称',
      componentProps: {
        placeholder: '如 案例管理',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入模型名称' }),
    },
    {
      component: 'Input',
      fieldName: 'modelIcon',
      label: '模型图标',
      componentProps: {
        placeholder: '如 ant-design:file-outlined',
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
        placeholder: '越小越靠前',
      },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '描述',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: '模型用途说明（显示在模型列表中）',
        allowClear: true,
        rows: 3,
      },
    },
    // ── 内容能力 ──
    {
      component: 'Switch',
      fieldName: 'hasTitle',
      label: '支持标题',
      defaultValue: true,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasContent',
      label: '支持正文',
      defaultValue: true,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasCover',
      label: '支持封面',
      defaultValue: false,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasAuthor',
      label: '支持作者',
      defaultValue: false,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasSummary',
      label: '支持摘要',
      defaultValue: false,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasSeo',
      label: '支持SEO',
      defaultValue: false,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasImages',
      label: '支持图集',
      defaultValue: false,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    {
      component: 'Switch',
      fieldName: 'hasAttachment',
      label: '支持附件',
      defaultValue: false,
      componentProps: { class: 'cm-switch' },
      formItemClass: 'cm-switch-field',
    },
    // ── 前台展示 ──
    {
      component: 'Select',
      fieldName: 'listTemplateId',
      label: '列表模板',
      componentProps: () => ({
        options: templateOptions.value,
        placeholder: '前台 /{模型编码} 列表页所用模板（可选）',
        allowClear: true,
      }),
      help: '未选择时回退当前站点模板的「模型列表页」页面，再无则用内置极简列表',
    },
    {
      component: 'Select',
      fieldName: 'detailTemplateId',
      label: '详情模板',
      componentProps: () => ({
        options: templateOptions.value,
        placeholder: '前台 /{模型编码}/{id} 详情页所用模板（可选）',
        allowClear: true,
      }),
      help: '未选择时回退当前站点模板的「模型详情页」页面，再无则用内置极简详情',
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      defaultValue: 1,
      formItemClass: 'col-span-2',
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
  class: 'w-[75%]',
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
    // Switch 返回布尔值，转为 0/1
    switchFields.forEach((key) => {
      values[key] = values[key] ? 1 : 0;
    });

    try {
      if (isCreate.value) {
        await addContentModelApi(values as ContentModelSaveDTO);
        message.success('新增成功');
      } else {
        await updateContentModelApi(
          data.value.row.id,
          values as ContentModelSaveDTO,
        );
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      isFullscreen.value = false;
      drawerApi.setState({ class: 'w-[75%]' });
      if (!isCreate.value && data.value?.row?.id) {
        const detail: any = await getContentModelDetailApi(data.value.row.id);
        const row = detail?.data || detail || data.value.row;
        // 数字 0/1 转布尔给 Switch
        const values: Record<string, any> = { ...row };
        switchFields.forEach((key) => {
          values[key] = values[key] === 1;
        });
        baseFormApi.setValues(values);
      } else {
        baseFormApi.setValues({});
      }
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer>
    <template #title>
      <div class="flex w-full items-center justify-between pr-2">
        <span>{{ getTitle }}</span>
        <button
          class="text-sm text-primary transition-colors hover:opacity-80"
          type="button"
          @click="toggleFullscreen"
        >
          {{ isFullscreen ? '还原' : '最大化' }}
        </button>
      </div>
    </template>
    <BaseForm />
  </Drawer>
</template>

<style scoped>
/* 内容能力开关：固定原生尺寸（commonConfig 的 w-full 会把开关拉成胶囊），行距收紧聚成一组 */
:deep(.cm-switch-field) {
  padding: 4px 0;
}

:deep(.cm-switch.ant-switch) {
  flex: 0 0 auto;
  min-width: 44px;
  width: 44px;
}
</style>
