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
  message,
  Modal,
  Popconfirm,
  Switch,
  Tag,
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
      label: $t('page.finance.expenseType.column.typeName'),
      componentProps: {
        placeholder: $t('page.finance.expenseType.drawer.typeNamePlaceholder'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'typeCode',
      label: $t('page.finance.expenseType.column.typeCode'),
      componentProps: {
        placeholder: $t('page.finance.expenseType.column.typeCode'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.expenseType.column.status'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: [
          { value: 1, label: $t('page.finance.expenseType.status.enabled') },
          { value: 0, label: $t('page.finance.expenseType.status.disabled') },
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
  cellConfig: { isHover: true } as any,
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
      title: $t('page.finance.expenseType.column.typeName'),
      field: 'typeName',
      minWidth: 160,
      slots: { default: 'typeName' },
    },
    {
      title: $t('page.finance.expenseType.column.typeCode'),
      field: 'typeCode',
      width: 150,
    },
    {
      title: $t('page.finance.expenseType.column.sort'),
      field: 'sort',
      width: 80,
      align: 'center',
    },
    {
      title: $t('page.finance.expenseType.column.status'),
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: $t('page.finance.expenseType.column.isSystem'),
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
    message.warning($t('page.finance.expenseType.drawer.typeNameRequired'));
    return;
  }
  modalLoading.value = true;
  try {
    const payload = {
      ...formData,
      status: formData.status ? 1 : 0,
    };
    await saveExpenseTypeApi(payload);
    message.success(
      isEditType.value
        ? $t('page.finance.expenseType.message.updateSuccess')
        : $t('page.finance.expenseType.message.createSuccess'),
    );
    modalVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(error?.message || $t('page.finance.common.failed'));
  } finally {
    modalLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteExpenseTypeApi([row.id]);
    message.success($t('page.finance.common.deleteSuccess'));
    gridApi.query();
  } catch (error: any) {
    message.error(error?.message || $t('page.finance.common.deleteFailed'));
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.finance.expenseType.manageTitle')">
      <template #toolbar-tools>
        <Button
          type="primary"
          class="mr-2"
          :icon="h(Plus)"
          @click="handleCreate"
        >
          {{ $t('page.finance.expenseType.drawer.titleCreate') }}
        </Button>
        <Button class="mr-2" :icon="h(RefreshCw)" @click="gridApi.query()">
          {{ $t('page.finance.common.refresh') }}
        </Button>
      </template>

      <template #typeName="{ row }">
        <span class="font-medium">{{ row.typeName }}</span>
        <Tag v-if="row.isSystem" color="purple" class="ml-2">
          {{ $t('page.finance.expenseType.tag.system') }}
        </Tag>
      </template>

      <template #status="{ row }">
        <Tag :color="row.status === 1 ? 'green' : 'red'">
          {{
            row.status === 1
              ? $t('page.finance.expenseType.status.enabled')
              : $t('page.finance.expenseType.status.disabled')
          }}
        </Tag>
      </template>

      <template #isSystem="{ row }">
        <Tag v-if="row.isSystem" color="purple">
          {{ $t('page.finance.common.yes') }}
        </Tag>
        <span v-else class="text-gray-400">{{
          $t('page.finance.common.no')
        }}</span>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          type="link"
          :title="$t('page.finance.common.edit')"
          @click="handleEdit(row)"
        >
          {{ $t('page.finance.common.edit') }}
        </Button>
        <Popconfirm
          v-if="!row.isSystem"
          :title="
            $t('page.finance.expenseType.drawer.deleteConfirm', {
              name: row.typeName,
            })
          "
          :ok-text="$t('page.finance.common.delete')"
          ok-type="danger"
          :cancel-text="$t('page.finance.common.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button type="link" danger :title="$t('page.finance.common.delete')">
            {{ $t('page.finance.common.delete') }}
          </Button>
        </Popconfirm>
        <span
          v-else
          class="text-gray-300 cursor-not-allowed"
          :title="$t('page.finance.expenseType.message.systemBuiltinNoDelete')"
        >
          {{ $t('page.finance.common.delete') }}
        </span>
      </template>
    </Grid>

    <!-- 新增/编辑弹窗 -->
    <Modal
      v-model:open="modalVisible"
      :title="
        isEditType
          ? $t('page.finance.expenseType.drawer.titleEdit')
          : $t('page.finance.expenseType.drawer.titleCreate')
      "
      :confirm-loading="modalLoading"
      :mask-closable="false"
      :destroy-on-close="true"
      :ok-text="$t('page.finance.common.save')"
      :cancel-text="$t('page.finance.common.cancel')"
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
          :label="$t('page.finance.expenseType.column.typeName')"
          :rules="[
            {
              required: true,
              message: $t('page.finance.expenseType.drawer.typeNameRequired'),
            },
          ]"
        >
          <Input
            v-model:value="formData.typeName"
            :placeholder="
              $t('page.finance.expenseType.drawer.typeNameRequired')
            "
            allow-clear
          />
        </FormItem>
        <FormItem
          name="typeCode"
          :label="$t('page.finance.expenseType.column.typeCode')"
          :rules="[
            {
              required: true,
              message: $t('page.finance.expenseType.drawer.typeCodeRequired'),
            },
          ]"
        >
          <Input
            v-model:value="formData.typeCode"
            :placeholder="
              $t('page.finance.expenseType.drawer.typeCodeRequired')
            "
            allow-clear
          />
        </FormItem>
        <FormItem
          name="sort"
          :label="$t('page.finance.expenseType.column.sort')"
        >
          <InputNumber
            v-model:value="formData.sort"
            :min="0"
            style="width: 100%"
            :placeholder="$t('page.finance.expenseType.drawer.sortPlaceholder')"
          />
        </FormItem>
        <FormItem
          name="status"
          :label="$t('page.finance.expenseType.column.status')"
        >
          <Switch
            :checked="formData.status === 1"
            :checked-children="$t('page.finance.expenseType.status.enabled')"
            :un-checked-children="
              $t('page.finance.expenseType.status.disabled')
            "
            @change="
              (checked: boolean | string | number) => {
                formData.status = checked ? 1 : 0;
              }
            "
          />
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>
