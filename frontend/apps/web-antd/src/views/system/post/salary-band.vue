<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, onMounted } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm, z } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, message, Popconfirm, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createSalaryBandApi,
  deleteSalaryBandApi,
  getPostOptionsApi,
  getSalaryBandListApi,
  updateSalaryBandApi,
} from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

const accessStore = useAccessStore();

// ===== 岗位下拉（带宽按岗位维度维护） =====
const postOptions = ref<{ label: string; value: number }[]>([]);
const postLoading = ref(false);
async function loadPostOptions() {
  postLoading.value = true;
  try {
    const res: any = await getPostOptionsApi();
    postOptions.value = (res?.data ?? res ?? [])
      .filter((p: any) => p.value !== undefined && p.value !== null)
      .map((p: any) => ({ label: p.label || `岗位${p.value}`, value: Number(p.value) }));
  } catch {
    postOptions.value = [];
  } finally {
    postLoading.value = false;
  }
}
onMounted(loadPostOptions);

// ===== 筛选表单 =====
const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'postId',
      label: $t('page.system.post.postName'),
      componentProps: {
        options: postOptions,
        loading: postLoading,
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('ui.table.status'),
      componentProps: {
        options: statusList,
        placeholder: $t('ui.placeholder.select'),
      },
    },
  ],
};

// ===== 列表 =====
const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getSalaryBandListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          postId: formValues.postId,
          status: formValues.status,
        });
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
      title: $t('page.system.post.postName'),
      field: 'postName',
      minWidth: 140,
    },
    {
      title: '带宽区间（元/月）',
      field: 'band',
      minWidth: 180,
      slots: { default: 'band' },
    },
    {
      title: $t('ui.table.remark'),
      field: 'remark',
      minWidth: 160,
      showOverflow: true,
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 120,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ===== 状态切换 =====
async function handleStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 0;
  try {
    await updateSalaryBandApi(row.id, row);

    message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// ===== 新增/编辑抽屉 =====
const data = ref<{ create: boolean; row?: any } | null>(null);
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value ? '新增薪资带宽' : '编辑薪资带宽',
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
      fieldName: 'postId',
      label: $t('page.system.post.postName'),
      componentProps: {
        options: postOptions,
        loading: postLoading,
        placeholder: '请选择岗位',
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        disabled: !isCreate.value,
      },
      rules: z
        .number({ required_error: '请选择岗位' })
        .min(1, { message: '请选择岗位' }),
    },
    {
      component: 'InputNumber',
      fieldName: 'minSalary',
      label: '带宽下限（元/月）',
      componentProps: {
        placeholder: '如 4500',
        min: 0,
        precision: 2,
        style: { width: '100%' },
      },
      // 列表接口返回的金额是字符串（Decimal 序列化），统一转数字再校验
      rules: z.preprocess(
        (v) =>
          v === null || v === undefined || v === ''
            ? undefined
            : Number(v),
        z.number({ required_error: '请输入带宽下限' }),
      ),
    },
    {
      component: 'InputNumber',
      fieldName: 'maxSalary',
      label: '带宽上限（元/月）',
      componentProps: {
        placeholder: '如 8000',
        min: 0,
        precision: 2,
        style: { width: '100%' },
      },
      rules: z.preprocess(
        (v) =>
          v === null || v === undefined || v === ''
            ? undefined
            : Number(v),
        z.number({ required_error: '请输入带宽上限' }),
      ),
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: $t('ui.table.remark'),
      componentProps: {
        type: 'textarea',
        autosize: true,
        rows: 4,
        placeholder: '如：销售岗位薪酬范围参考，供入职定薪审批评估',
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: 1,
      label: $t('ui.table.status'),
      rules: 'selectRequired',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: statusList,
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
    // 带宽校验：上限不能小于下限
    if (
      values.minSalary !== undefined &&
      values.maxSalary !== undefined &&
      Number(values.maxSalary) < Number(values.minSalary)
    ) {
      message.warning('带宽上限不能小于下限');
      setLoading(false);
      return;
    }

    try {
      await (data.value?.create
        ? createSalaryBandApi(values)
        : updateSalaryBandApi(data.value?.row.id, values));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } catch {
      // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
    } finally {
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<{ create: boolean; row?: any }>();
      // 先重置表单，避免上次编辑数据残留（如 remark 等空字段不被 setValues 覆盖）
      await baseFormApi.resetForm();
      baseFormApi.setValues({
        ...(data.value?.row || {}),
        // 新增时岗位必须手动选择，不预填任何值
        postId: data.value?.create
          ? undefined
          : Number(data.value?.row?.postId) || undefined,
        minSalary: data.value?.row?.minSalary ?? undefined,
        maxSalary: data.value?.row?.maxSalary ?? undefined,
      });
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
    await deleteSalaryBandApi([row.id]);

    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <div class="h-full">
    <Grid>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('system:post:save')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新增带宽
        </Button>
      </template>

      <template #band="{ row }">
        <span class="font-medium">
          <span v-if="row.minSalary !== null && row.minSalary !== undefined">
            {{ row.minSalary }}
          </span>
          <span class="mx-1 text-gray-400">~</span>
          <span v-if="row.maxSalary !== null && row.maxSalary !== undefined">
            {{ row.maxSalary }}
          </span>
          <span v-if="row.minSalary === undefined && row.maxSalary === undefined" class="text-gray-400">—</span>
        </span>
      </template>

      <template #status="{ row }">
        <Switch
          v-model:checked="row.status"
          :checked-value="1"
          :disabled="!accessStore.hasAccessCode('system:post:update')"
          :loading="row.pending"
          :un-checked-value="0"
          @change="(checked: any) => handleStatusChanged(row, checked)"
        />
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('system:post:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: '薪资带宽',
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('system:post:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer :title="getTitle">
      <BaseForm />
    </Drawer>
  </div>
</template>
