<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Form,
  FormItem,
  Input,
  InputNumber,
  Modal,
  Popconfirm,
  Switch,
  Tag,
  message,
} from 'ant-design-vue';
import { Plus, RefreshCw } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteExpenseTypeApi,
  getExpenseTypeListApi,
  saveExpenseTypeApi,
} from '#/api';
import { $t } from '#/locales';

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'typeName',
      label: '类型名称',
      componentProps: { placeholder: '类型名称', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'typeCode',
      label: '类型编码',
      componentProps: { placeholder: '类型编码', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { value: 1, label: '启用' },
          { value: 0, label: '禁用' },
        ],
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
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        };
        const result: any = await getExpenseTypeListApi(params);
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '类型名称',
      field: 'typeName',
      minWidth: 160,
      slots: { default: 'typeName' },
    },
    {
      title: '类型编码',
      field: 'typeCode',
      width: 150,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
      align: 'center',
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '系统内置',
      field: 'isSystem',
      width: 100,
      slots: { default: 'isSystem' },
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
      slots: { default: 'action' },
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ========== 新增/编辑弹窗 ==========
const modalVisible = ref(false);
const modalLoading = ref(false);
const isEditType = computed(() => !!formData.id);

const formData = reactive<any>({
  id: undefined,
  typeName: '',
  typeCode: '',
  sort: 0,
  status: 1,
  isSystem: false,
});

const formRef = ref();

function resetForm() {
  formData.id = undefined;
  formData.typeName = '';
  formData.typeCode = '';
  formData.sort = 0;
  formData.status = 1;
  formData.isSystem = false;
  formRef.value?.resetFields();
}

function handleCreate() {
  resetForm();
  modalVisible.value = true;
}

function handleEdit(row: any) {
  resetForm();
  formData.id = row.id;
  formData.typeName = row.typeName ?? '';
  formData.typeCode = row.typeCode ?? '';
  formData.sort = row.sort ?? 0;
  formData.status = row.status ?? 1;
  formData.isSystem = row.isSystem ?? false;
  modalVisible.value = true;
}

async function handleSubmit() {
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  if (!formData.typeName?.trim()) {
    message.warning('请输入类型名称');
    return;
  }
  modalLoading.value = true;
  try {
    const payload = {
      ...formData,
      status: formData.status ? 1 : 0,
    };
    await saveExpenseTypeApi(payload);
    message.success(isEditType.value ? '更新成功' : '创建成功');
    modalVisible.value = false;
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  } finally {
    modalLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteExpenseTypeApi([row.id]);
    message.success('删除成功');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '删除失败');
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="费用类型管理">
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" :icon="h(Plus)" @click="handleCreate">
          新增费用类型
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          刷新
        </Button>
      </template>

      <template #typeName="{ row }">
        <span class="font-medium">{{ row.typeName }}</span>
        <Tag v-if="row.isSystem" color="purple" class="ml-2">系统</Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="row.status === 1 ? 'green' : 'red'">
          {{ row.status === 1 ? '启用' : '禁用' }}
        </Tag>
      </template>

      <template #isSystem="{ row }">
        <Tag v-if="row.isSystem" color="purple">是</Tag>
        <span v-else class="text-gray-400">否</span>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          type="link"
          title="编辑"
          @click="handleEdit(row)"
        >编辑</Button>
        <Popconfirm
          v-if="!row.isSystem"
          :title="`确定要删除费用类型「${row.typeName}」吗？`"
          ok-text="删除"
          ok-type="danger"
          cancel-text="取消"
          @confirm="handleDelete(row)"
        >
          <Button type="link" danger title="删除">删除</Button>
        </Popconfirm>
        <span v-else class="text-gray-300 cursor-not-allowed" title="系统内置类型不可删除">
          删除
        </span>
      </template>
    </Grid>

    <!-- 新增/编辑弹窗 -->
    <Modal
      v-model:open="modalVisible"
      :title="isEditType ? '编辑费用类型' : '新增费用类型'"
      :confirm-loading="modalLoading"
      :mask-closable="false"
      :destroy-on-close="true"
      ok-text="保存"
      cancel-text="取消"
      @ok="handleSubmit"
    >
      <Form
        ref="formRef"
        :model="formData"
        :label-col="{ span: 5 }"
        :wrapper-col="{ span: 18 }"
      >
        <FormItem
          name="typeName"
          label="类型名称"
          :rules="[{ required: true, message: '请输入类型名称' }]"
        >
          <Input
            v-model:value="formData.typeName"
            placeholder="请输入类型名称"
            allow-clear
          />
        </FormItem>
        <FormItem
          name="typeCode"
          label="类型编码"
          :rules="[{ required: true, message: '请输入类型编码' }]"
        >
          <Input
            v-model:value="formData.typeCode"
            placeholder="请输入类型编码"
            allow-clear
          />
        </FormItem>
        <FormItem name="sort" label="排序">
          <InputNumber
            v-model:value="formData.sort"
            :min="0"
            style="width: 100%"
            placeholder="排序值（越小越靠前）"
          />
        </FormItem>
        <FormItem name="status" label="状态">
          <Switch
            :checked="formData.status === 1"
            checked-children="启用"
            un-checked-children="禁用"
            @change="(checked: boolean) => (formData.status = checked ? 1 : 0)"
          />
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>
