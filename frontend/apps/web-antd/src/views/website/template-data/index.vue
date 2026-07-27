<script lang="ts" setup>
import { computed, h, onMounted, ref } from 'vue';

import { Page, useVbenDrawer, z } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  addTemplateDataApi,
  deleteTemplateDataApi,
  getTemplateDataListApi,
  updateTemplateDataApi,
  templateApi,
} from '#/api';
import type { TemplateDataVO } from '#/api/core/website/template-data';
import { $t } from '#/locales';
import { statusList } from '#/store';

const accessStore = useAccessStore();

const typeOptions = [
  { value: 1, label: '首页' },
  { value: 2, label: '列表页' },
  { value: 3, label: '内容页' },
  { value: 4, label: '栏目封面' },
  { value: 5, label: '报价页' },
  { value: 6, label: '专题' },
];

const typeColors: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'green',
  4: 'purple',
  5: 'orange',
  6: 'red',
};

const templateOptions = ref<{ label: string; value: number }[]>([]);

async function loadTemplateOptions() {
  try {
    const res: any = await templateApi.list({ page: 1, pageSize: 999, status: 1 });
    const list = res?.items || [];
    templateOptions.value = list.map((t: any) => ({
      label: t.name,
      value: Number(t.id),
    }));
  } catch {
    templateOptions.value = [];
  }
}

onMounted(() => {
  loadTemplateOptions();
});

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'templateId',
      label: '所属模板',
      componentProps: {
        options: templateOptions,
        placeholder: '请选择模板',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label?.toLowerCase().includes(input.toLowerCase()),
      },
    },
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '名称',
      componentProps: {
        placeholder: '请输入名称',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'typeId',
      label: '类型',
      componentProps: {
        options: typeOptions,
        placeholder: '请选择类型',
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getTemplateDataListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          typeId: formValues.typeId,
        });
      },
    },
  },

  columns: [
    {
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: 'ID',
      field: 'id',
      width: 80,
    },
    {
      title: '所属模板',
      field: 'templateId',
      width: 150,
      slots: { default: 'templateName' },
    },
    {
      title: '名称',
      field: 'name',
    },
    {
      title: '类型',
      field: 'typeId',
      slots: { default: 'typeId' },
      width: 100,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: '创建时间',
      field: 'createTime',
      slots: { default: 'createdAt' },
      width: 160,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// --- Drawer ---
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerTitle = computed(() =>
  drawerData.value.create ? '新增模板数据' : '编辑模板数据',
);

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Select',
      fieldName: 'templateId',
      label: '所属模板',
      componentProps: {
        options: templateOptions,
        placeholder: '请选择所属模板',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label?.toLowerCase().includes(input.toLowerCase()),
      },
      rules: z.string().min(1, { message: '请选择所属模板' }),
    },
    {
      component: 'Input',
      fieldName: 'name',
      label: '名称',
      componentProps: {
        placeholder: '请输入名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入名称' }),
    },
    {
      component: 'Select',
      fieldName: 'typeId',
      label: '类型',
      componentProps: {
        options: typeOptions,
        placeholder: '请选择类型',
      },
      rules: z.string().min(1, { message: '请选择类型' }),
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      componentProps: {
        placeholder: '请输入排序',
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: 1,
      label: '状态',
      rules: 'selectRequired',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: statusList,
      },
    },
    {
      component: 'Input',
      fieldName: 'temptext',
      label: '模板内容',
      componentProps: {
        type: 'textarea',
        autosize: { minRows: 6, maxRows: 16 },
        placeholder: '请输入HTML模板内容',
        allowClear: true,
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

    try {
      if (drawerData.value.create) {
        await addTemplateDataApi(values);
      } else {
        await updateTemplateDataApi(drawerData.value.row.id, values);
      }

      message.success(
        drawerData.value.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.close();
      gridApi.query();
    } catch {
      // 错误由全局拦截器处理
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      drawerData.value = drawerApi.getData<{ create: boolean; row?: any }>();
      baseFormApi.setValues(drawerData.value?.row ?? {});
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteTemplateDataApi([row.id]);

    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

function getTypeLabel(typeId: number): string {
  const option = typeOptions.find((o) => o.value === typeId);
  return option ? option.label : '未知';
}

function getTemplateName(templateId: number): string {
  const option = templateOptions.value.find((o) => o.value === templateId);
  return option ? option.label : '未知模板';
}
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('template:data:add')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新增模板数据
        </Button>
      </template>

      <template #templateName="{ row }">
        <Tag color="blue">
          {{ getTemplateName(row.templateId) }}
        </Tag>
      </template>

      <template #typeId="{ row }">
        <Tag :color="typeColors[row.typeId] ?? 'default'">
          {{ getTypeLabel(row.typeId) }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #status="{ row }">
        <Tag :color="row.status === 1 ? 'success' : 'default'">
          {{ row.status === 1 ? '启用' : '禁用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('template:data:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="handleEdit(row)"
        />
        <Popconfirm
          title="确定删除该模板吗？"
          ok-text="确定"
          cancel-text="取消"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('template:data:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>

    <Drawer :title="drawerTitle">
      <BaseForm />
    </Drawer>
  </Page>
</template>
