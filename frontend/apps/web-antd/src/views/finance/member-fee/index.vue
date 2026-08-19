<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Popconfirm,
  Select,
  Tag,
} from 'ant-design-vue';
import { Plus } from 'lucide-vue-next';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createMemberFeeApi,
  deleteMemberFeeApi,
  getMemberFeeListApi,
  updateMemberFeeApi,
} from '#/api';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { UserPickerModal } from '#/components/UserPickerModal';
import { $t } from '#/locales';

// 会员费使用说明步骤数（与 i18n 中 page.finance.memberFee.guide.steps 数组对齐）
const guideStepCount = 5;

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  schema: [
    {
      component: 'InputNumber',
      fieldName: 'userId',
      label: $t('page.finance.memberFee.column.userId'),
      componentProps: {
        placeholder: $t('page.finance.memberFee.placeholder.userId'),
      },
    },
    {
      component: 'Select',
      fieldName: 'memberType',
      label: $t('page.finance.memberFee.column.memberType'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: [
          { value: 1, label: $t('page.finance.memberFee.memberType.monthly') },
          { value: 2, label: $t('page.finance.memberFee.memberType.yearly') },
          { value: 3, label: $t('page.finance.memberFee.memberType.lifetime') },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('page.finance.memberFee.column.status'),
      componentProps: {
        placeholder: $t('page.finance.common.all'),
        allowClear: true,
        options: [
          { value: 0, label: $t('page.finance.memberFee.status.inactive') },
          { value: 1, label: $t('page.finance.memberFee.status.active') },
          { value: 2, label: $t('page.finance.memberFee.status.expired') },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
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
        const result: any = await getMemberFeeListApi(params);
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
    { type: 'seq', width: 60, title: $t('page.finance.common.seq') },
    { field: 'id', title: 'ID', width: 80 },
    {
      field: 'userId',
      title: $t('page.finance.memberFee.column.userId'),
      width: 100,
    },
    {
      field: 'memberType',
      title: $t('page.finance.memberFee.column.memberType'),
      width: 120,
      slots: { default: 'memberType' },
    },
    {
      field: 'amount',
      title: $t('page.finance.memberFee.column.amount'),
      width: 120,
      align: 'right',
      formatter: ({ cellValue }) => `¥${Number(cellValue || 0).toFixed(2)}`,
    },
    {
      field: 'validStartTime',
      title: $t('page.finance.memberFee.column.validStartTime'),
      width: 160,
    },
    {
      field: 'validEndTime',
      title: $t('page.finance.memberFee.column.validEndTime'),
      width: 160,
    },
    {
      field: 'status',
      title: $t('page.finance.memberFee.column.status'),
      width: 100,
      slots: { default: 'status' },
    },
    {
      field: 'paymentRecordId',
      title: $t('page.finance.memberFee.column.paymentRecordId'),
      width: 140,
    },
    {
      field: 'remark',
      title: $t('page.finance.memberFee.column.remark'),
      minWidth: 120,
    },
    {
      field: 'createTime',
      title: $t('page.finance.memberFee.column.createTime'),
      width: 160,
    },
    {
      field: 'action',
      title: $t('page.finance.common.action'),
      width: 160,
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ formOptions, gridOptions });

// ===== Drawer 表单 =====
const drawerVisible = ref(false);
const drawerMode = ref<'create' | 'edit'>('create');
const drawerLoading = ref(false);
const formRef = ref();
const formData = reactive({
  id: 0,
  userId: undefined as number | undefined,
  memberType: 1,
  amount: 0,
  validStartTime: '',
  validEndTime: '',
  status: 1,
  paymentRecordId: undefined as number | undefined,
  remark: '',
});

function resetForm() {
  formData.id = 0;
  formData.userId = undefined;
  formData.memberType = 1;
  formData.amount = 0;
  formData.validStartTime = '';
  formData.validEndTime = '';
  formData.status = 1;
  formData.paymentRecordId = undefined;
  formData.remark = '';
}

function openCreate() {
  resetForm();
  drawerMode.value = 'create';
  drawerVisible.value = true;
}

function openEdit(row: any) {
  formData.id = row.id;
  formData.userId = row.userId;
  formData.memberType = row.memberType ?? 1;
  formData.amount = row.amount;
  formData.validStartTime = row.validStartTime ?? '';
  formData.validEndTime = row.validEndTime ?? '';
  formData.status = row.status ?? 1;
  formData.paymentRecordId = row.paymentRecordId;
  formData.remark = row.remark ?? '';
  drawerMode.value = 'edit';
  drawerVisible.value = true;
}

async function handleSubmit() {
  try {
    await formRef.value?.validate();
  } catch {
    return;
  }

  if (!formData.userId) {
    message.warning($t('page.finance.memberFee.message.userIdRequired'));
    return;
  }

  const payload: any = {
    userId: formData.userId,
    memberType: formData.memberType,
    amount: formData.amount,
    validStartTime: formData.validStartTime || undefined,
    validEndTime: formData.validEndTime || undefined,
    status: formData.status,
    paymentRecordId: formData.paymentRecordId || undefined,
    remark: formData.remark || undefined,
  };

  drawerLoading.value = true;
  try {
    if (drawerMode.value === 'create') {
      await createMemberFeeApi(payload);
      message.success($t('page.finance.memberFee.message.createSuccess'));
    } else {
      await updateMemberFeeApi(formData.id, payload);
      message.success($t('page.finance.memberFee.message.updateSuccess'));
    }
    drawerVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.memberFee.message.saveFailed'),
    );
  } finally {
    drawerLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteMemberFeeApi(row.id);
    message.success($t('page.finance.memberFee.message.deleteSuccess'));
    gridApi.query();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.memberFee.message.deleteFailed'),
    );
  }
}

const memberTypeMap: Record<number, { color: string; text: string }> = {
  1: { color: 'blue', text: $t('page.finance.memberFee.memberType.monthly') },
  2: { color: 'cyan', text: $t('page.finance.memberFee.memberType.yearly') },
  3: {
    color: 'purple',
    text: $t('page.finance.memberFee.memberType.lifetime'),
  },
};
const statusMap: Record<number, { color: string; text: string }> = {
  0: { color: 'default', text: $t('page.finance.memberFee.status.inactive') },
  1: { color: 'green', text: $t('page.finance.memberFee.status.active') },
  2: { color: 'red', text: $t('page.finance.memberFee.status.expired') },
};
</script>

<template>
  <Page :title="$t('page.finance.memberFee.title')">
    <PageUsageGuide
      :title="$t('page.finance.memberFee.guide.title')"
      :brief="$t('page.finance.memberFee.guide.brief')"
      :expand-text="$t('page.finance.memberFee.guide.expand')"
      :collapse-text="$t('page.finance.memberFee.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.memberFee.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.memberFee.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid>
      <template #toolbar-tools>
        <Button type="primary" class="mr-2" @click="openCreate">
          <template #icon><Plus /></template>
          {{ $t('page.finance.memberFee.button.add') }}
        </Button>
      </template>
      <template #memberType="{ row }">
        <Tag :color="memberTypeMap[row.memberType]?.color || 'default'">
          {{ memberTypeMap[row.memberType]?.text || row.memberType }}
        </Tag>
      </template>
      <template #status="{ row }">
        <Tag :color="statusMap[row.status]?.color || 'default'">
          {{ statusMap[row.status]?.text || row.status }}
        </Tag>
      </template>
      <template #action="{ row }">
        <Button type="link" size="small" @click="openEdit(row)">
          {{ $t('page.finance.common.edit') }}
        </Button>
        <Popconfirm
          :title="$t('page.finance.memberFee.message.deleteConfirm')"
          @confirm="handleDelete(row)"
        >
          <Button type="link" size="small" danger>
            {{ $t('page.finance.common.delete') }}
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <Drawer
      v-model:open="drawerVisible"
      :title="
        drawerMode === 'create'
          ? $t('page.finance.memberFee.drawer.titleCreate')
          : $t('page.finance.memberFee.drawer.titleEdit')
      "
      width="480"
      :confirm-loading="drawerLoading"
      @ok="handleSubmit"
    >
      <Form ref="formRef" layout="vertical" class="pt-4">
        <FormItem
          :label="$t('page.finance.memberFee.column.userId')"
          name="userId"
          required
        >
          <UserPickerModal v-model:value="formData.userId" />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.memberType')"
          name="memberType"
        >
          <Select
            v-model:value="formData.memberType"
            :options="[
              {
                value: 1,
                label: $t('page.finance.memberFee.memberType.monthly'),
              },
              {
                value: 2,
                label: $t('page.finance.memberFee.memberType.yearly'),
              },
              {
                value: 3,
                label: $t('page.finance.memberFee.memberType.lifetime'),
              },
            ]"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.amount')"
          name="amount"
          required
        >
          <InputNumber
            v-model:value="formData.amount"
            :min="0"
            :precision="2"
            :placeholder="$t('page.finance.memberFee.placeholder.amount')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.validStartTime')"
          name="validStartTime"
        >
          <Input
            v-model:value="formData.validStartTime"
            :placeholder="
              $t('page.finance.memberFee.placeholder.validStartTime')
            "
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.validEndTime')"
          name="validEndTime"
        >
          <Input
            v-model:value="formData.validEndTime"
            :placeholder="$t('page.finance.memberFee.placeholder.validEndTime')"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.status')"
          name="status"
        >
          <Select
            v-model:value="formData.status"
            :options="[
              { value: 0, label: $t('page.finance.memberFee.status.inactive') },
              { value: 1, label: $t('page.finance.memberFee.status.active') },
              { value: 2, label: $t('page.finance.memberFee.status.expired') },
            ]"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.paymentRecordId')"
          name="paymentRecordId"
        >
          <InputNumber
            v-model:value="formData.paymentRecordId"
            :min="1"
            :placeholder="
              $t('page.finance.memberFee.placeholder.paymentRecordId')
            "
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.memberFee.column.remark')"
          name="remark"
        >
          <Input.TextArea
            v-model:value="formData.remark"
            :rows="2"
            :placeholder="$t('page.finance.memberFee.placeholder.remark')"
          />
        </FormItem>
      </Form>
    </Drawer>
  </Page>
</template>
