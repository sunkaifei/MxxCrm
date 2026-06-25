<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createContractApi, updateContractApi } from '#/api';
import { getCustomerListApi } from '#/api/core/crm/customer';
import { getOpportunityListApi } from '#/api/core/crm/opportunity';
import { getUserListApi } from '#/api/core/system/user';

const data = ref();
const loading = ref(false);
const isMaximized = ref(false);

const customerOptions = ref<any[]>([]);
const opportunityOptions = ref<any[]>([]);
const userOptions = ref<any[]>([]);

const drawerClass = computed(() =>
  isMaximized.value ? 'w-[95vw]' : 'w-[75vw]',
);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.contract.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.contract.title') }),
);

const contractStatusList = computed(() => [
  { value: 1, label: '草稿' },
  { value: 2, label: '已签署' },
  { value: 3, label: '执行中' },
  { value: 4, label: '已完成' },
  { value: 5, label: '已终止' },
]);

const contractTypeList = computed(() => [
  { value: 1, label: '销售合同' },
  { value: 2, label: '采购合同' },
  { value: 3, label: '服务合同' },
  { value: 4, label: '合作协议' },
  { value: 5, label: '其他' },
]);

const currencyList = computed(() => [
  { value: 1, label: 'CNY - 人民币' },
  { value: 2, label: 'USD - 美元' },
  { value: 3, label: 'EUR - 欧元' },
  { value: 4, label: 'GBP - 英镑' },
  { value: 5, label: 'JPY - 日元' },
  { value: 6, label: 'HKD - 港币' },
  { value: 7, label: 'AUD - 澳元' },
]);

async function loadCustomerOptions() {
  try {
    const result = await getCustomerListApi({ page: 1, pageSize: 1000 });
    if (result.data && result.data.items) {
      customerOptions.value = result.data.items.map((item: any) => ({
        value: item.id,
        label: item.companyName || item.name,
      }));
    }
  } catch (e) {
    console.error('Failed to load customer options:', e);
  }
}

async function loadOpportunityOptions(customerId?: number) {
  try {
    const params: any = { page: 1, pageSize: 1000 };
    if (customerId) {
      params.customerId = customerId;
    }
    const result = await getOpportunityListApi(params);
    if (result.data && result.data.items) {
      opportunityOptions.value = result.data.items.map((item: any) => ({
        value: item.id,
        label: item.title || item.name,
      }));
    }
  } catch (e) {
    console.error('Failed to load opportunity options:', e);
  }
}

async function loadUserOptions() {
  try {
    const result = await getUserListApi({ page: 1, pageSize: 1000 });
    if (result.data && result.data.items) {
      userOptions.value = result.data.items.map((item: any) => ({
        value: item.id,
        label: item.realName || item.userName,
      }));
    }
  } catch (e) {
    console.error('Failed to load user options:', e);
  }
}

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Divider',
      fieldName: '_div1',
      hideLabel: true,
      formItemClass: 'col-span-2',
      componentProps: {
        orientation: 'left',
        plain: true,
      },
      renderComponentContent: () => ({ default: () => '基本信息' }),
    },
    {
      component: 'Select',
      fieldName: 'customerId',
      label: '客户',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label.toLowerCase().includes(input.toLowerCase()),
        options: customerOptions,
        onChange: async (value: any) => {
          if (value) {
            await loadOpportunityOptions(value);
          } else {
            await loadOpportunityOptions();
          }
          baseFormApi.setValues({ opportunityId: undefined });
        },
      },
    },
    {
      component: 'Select',
      fieldName: 'opportunityId',
      label: '关联商机',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label.toLowerCase().includes(input.toLowerCase()),
        options: opportunityOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'contractNo',
      label: '合同编号',
      rules: 'required',
      componentProps: {
        placeholder: '系统自动生成',
        disabled: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '合同标题',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'contractType',
      label: '合同类型',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: contractTypeList,
      },
    },

    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      formItemClass: 'col-span-2',
      componentProps: {
        orientation: 'left',
        plain: true,
      },
      renderComponentContent: () => ({ default: () => '财务信息' }),
    },
    {
      component: 'InputNumber',
      fieldName: 'amount',
      label: '合同金额（不含税）',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        min: 0,
        precision: 2,
        style: { width: '100%' },
        onChange: async () => {
          await calculateTotalAmount();
        },
      },
    },
    {
      component: 'Select',
      fieldName: 'currency',
      label: '币种',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        options: currencyList,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'taxAmount',
      label: '税额',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        min: 0,
        precision: 2,
        style: { width: '100%' },
        onChange: async () => {
          await calculateTotalAmount();
        },
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'totalAmount',
      label: '总金额（含税）',
      rules: 'required',
      componentProps: {
        placeholder: '自动计算',
        disabled: true,
        style: { width: '100%' },
      },
    },

    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      formItemClass: 'col-span-2',
      componentProps: {
        orientation: 'left',
        plain: true,
      },
      renderComponentContent: () => ({ default: () => '日期与条款' }),
    },
    {
      component: 'DatePicker',
      fieldName: 'startDate',
      label: '合同开始日期',
      rules: 'required',
      componentProps: {
        placeholder: '选择日期',
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'endDate',
      label: '合同结束日期',
      rules: 'required',
      componentProps: {
        placeholder: '选择日期',
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'signDate',
      label: '签署日期',
      componentProps: {
        placeholder: '选择日期',
      },
    },
    {
      component: 'Textarea',
      fieldName: 'paymentTerms',
      label: '付款条款',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: '请输入付款条款',
        allowClear: true,
        rows: 2,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'deliveryTerms',
      label: '交付条款',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: '请输入交付条款',
        allowClear: true,
        rows: 2,
      },
    },

    {
      component: 'Divider',
      fieldName: '_div4',
      hideLabel: true,
      formItemClass: 'col-span-2',
      componentProps: {
        orientation: 'left',
        plain: true,
      },
      renderComponentContent: () => ({ default: () => '负责人与文件' }),
    },
    {
      component: 'Select',
      fieldName: 'assignedTo',
      label: '负责人',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label.toLowerCase().includes(input.toLowerCase()),
        options: userOptions,
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
        options: contractStatusList,
      },
    },
    {
      component: 'Upload',
      fieldName: 'contractFile',
      label: '合同文件',
      formItemClass: 'col-span-2',
      componentProps: {
        accept: '.pdf,.doc,.docx',
        maxCount: 1,
        showUploadList: true,
        listType: 'text',
        beforeUpload: () => false,
      },
    },
    {
      component: 'Upload',
      fieldName: 'contractImages',
      label: '合同扫描件',
      formItemClass: 'col-span-2',
      componentProps: {
        accept: 'image/*',
        maxCount: 9,
        multiple: true,
        showUploadList: true,
        listType: 'picture-card',
        beforeUpload: () => false,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'remark',
      label: '备注',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
        rows: 2,
      },
    },
  ],
});

async function calculateTotalAmount() {
  const values = await baseFormApi.getValues();
  const amount = Number(values.amount) || 0;
  const taxAmount = Number(values.taxAmount) || 0;
  const total = amount + taxAmount;
  await baseFormApi.setValues({ totalAmount: total });
}

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
      await (data.value?.create
        ? createContractApi(values)
        : updateContractApi({ ...values, id: data.value.row.id }));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};

      await Promise.all([
        loadCustomerOptions(),
        loadUserOptions(),
      ]);

      if (row.customerId) {
        await loadOpportunityOptions(row.customerId);
      } else {
        await loadOpportunityOptions();
      }

      if (data.value?.create) {
        const now = new Date();
        const year = now.getFullYear();
        const month = String(now.getMonth() + 1).padStart(2, '0');
        const day = String(now.getDate()).padStart(2, '0');
        const random = String(Math.floor(Math.random() * 1000)).padStart(3, '0');
        row.contractNo = `CON-${year}${month}${day}-${random}`;
        row.status = 1;
      }

      baseFormApi.setValues(row);
      setLoading(false);
    }
  },
});

function setLoading(loadingState: boolean) {
  loading.value = loadingState;
  drawerApi.setState({ loading: loadingState });
}

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}
</script>

<template>
  <Drawer :title="getTitle" :class="drawerClass">
    <template #extra>
      <button
        type="button"
        class="w-8 h-8 flex items-center justify-center text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-md transition-colors"
        @click="toggleMaximize"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
        </svg>
      </button>
    </template>
    <BaseForm />
  </Drawer>
</template>
