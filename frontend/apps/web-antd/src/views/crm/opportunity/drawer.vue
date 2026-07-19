<script lang="ts" setup>
import { computed, nextTick, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createOpportunityApi, updateOpportunityApi, getCustomerListApi, getCustomerContactsApi, getOpportunityInfoApi } from '#/api';
import { getUserListApi } from '#/api/core/system/user';

const data = ref();
const isMaximized = ref(false);

const drawerClass = computed(() =>
  isMaximized.value ? 'w-[95vw]' : 'w-[75vw]',
);

// 当前选中的客户公司名（用于 ApiSelect 编辑回显）
const currentCompanyName = ref<string>('');
const currentCustomerId = ref<number | null>(null);

// 当前选中客户下的联系人列表
const contactOptions = ref<{ label: string; value: number }[]>([]);

// 负责人选项
const userOptions = ref<any[]>([]);

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

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.opportunity.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.opportunity.title') }),
);

// 商机阶段 - 数值对齐后端（1-5）
const stageOptions = [
  { label: '初步沟通', value: 1, color: 'blue' },
  { label: '需求确认', value: 2, color: 'cyan' },
  { label: '方案沟通', value: 3, color: 'gold' },
  { label: '已报价', value: 4, color: 'orange' },
  { label: '成交/丢单', value: 5, color: 'green' },
];

// 商机来源 - 对齐后端 LeadSource 枚举（数字值）
const sourceOptions = [
  { label: '官网', value: 1 },
  { label: '展会', value: 2 },
  { label: '社交媒体', value: 3 },
  { label: '客户转介', value: 4 },
  { label: '陌生拜访', value: 5 },
  { label: '海关数据', value: 6 },
  { label: '邮件营销', value: 7 },
  { label: '阿里国际站', value: 8 },
  { label: 'Amazon', value: 9 },
  { label: 'TikTok', value: 10 },
  { label: '微信', value: 11 },
  { label: '其他', value: 12 },
];

// 币种 - 对齐后端 CurrencyCode 枚举（数字值：1=人民币, 2=美元, 3=欧元, 4=英镑, 5=日元, 6=港币, 7=澳元）
const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
  { label: 'AUD 澳元', value: 7 },
];

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: [
    // 基本信息
    {
      component: 'Divider',
      fieldName: '_div1',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '基本信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '商机名称',
      rules: 'required',
      componentProps: { placeholder: '请输入商机名称/标题', allowClear: true, maxlength: 100 },
    },
    {
      component: 'Select',
      fieldName: 'stage',
      label: '销售阶段',
      defaultValue: 1,
      rules: 'required',
      componentProps: {
        placeholder: '请选择销售阶段',
        allowClear: false,
        options: stageOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'assignedTo',
      label: '负责人',
      componentProps: {
        placeholder: '请选择负责人',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label.toLowerCase().includes(input.toLowerCase()),
        options: userOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '商机来源',
      componentProps: {
        placeholder: '请选择来源',
        allowClear: true,
        options: sourceOptions,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'probability',
      label: '赢单概率',
      defaultValue: 10,
      componentProps: {
        placeholder: '0-100',
        min: 0,
        max: 100,
        class: 'w-full',
        addonAfter: '%',
      },
    },
    // 财务信息
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '金额与日期' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'InputNumber',
      fieldName: 'amount',
      label: '预算金额',
      rules: 'required',
      componentProps: {
        placeholder: '请输入商机金额',
        min: 0,
        class: 'w-full',
        precision: 2,
      },
    },
    {
      component: 'Select',
      fieldName: 'currency',
      label: '币种',
      defaultValue: 1,
      rules: 'required',
      componentProps: { options: currencyOptions, allowClear: false },
    },
    {
      component: 'DatePicker',
      fieldName: 'expectedCloseDate',
      label: '预计成交日期',
      componentProps: { placeholder: '请选择预计成交日期', class: 'w-full', allowClear: true, valueFormat: 'YYYY-MM-DD' },
    },
    // 关联信息
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '关联信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'ApiSelect',
      fieldName: 'customerId',
      label: '所属企业',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: '搜索并选择客户',
        allowClear: true,
        showSearch: true,
        filterOption: false,
        remote: true,
        params: { companyName: '' },
        api: async (params: any) => {
          const res: any = await getCustomerListApi({
            page: 1,
            pageSize: 20,
            ...(params?.companyName ? { companyName: params.companyName } : {}),
          });
          const items = res?.items || [];
          // 编辑回显：如果当前公司不在列表中，手动加入
          if (currentCompanyName.value && currentCustomerId.value) {
            const exists = items.some((item: any) => String(item.id) === String(currentCustomerId.value));
            if (!exists) {
              items.unshift({ id: String(currentCustomerId.value), companyName: currentCompanyName.value });
            }
          }
          return items;
        },
        labelField: 'companyName',
        valueField: 'id',
        onSearch(keyword: string) {
          baseFormApi.updateSchema('customerId', {
            componentProps: { params: { companyName: keyword } },
          });
        },
        immediate: true,
        // 客户变化时加载联系人
        onChange: async (value: any) => {
          contactOptions.value = [];
          baseFormApi.setValues({ contactId: undefined });
          if (value) {
            currentCustomerId.value = Number(value);
            try {
              const res: any = await getCustomerContactsApi(Number(value));
              const items: any[] = res?.data?.current || [];
              contactOptions.value = items.map((c: any) => ({
                label: c.name || c.contactName || '',
                value: Number(c.id || c.contactId),
              }));
            } catch { /* ignore */ }
          } else {
            currentCustomerId.value = null;
          }
        },
      },
    },
    {
      component: 'Select',
      fieldName: 'contactId',
      label: '联系人',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: '请先选择所属企业',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) => {
          return (option?.label ?? '').toLowerCase().includes(input.toLowerCase());
        },
        options: contactOptions,
        onFocus: () => {
          // 使用 ref 而非 getValues()（getValues 返回 Promise，无法同步读取）
          if (!currentCustomerId.value) {
            message.warning('请先选择所属企业');
          }
        },
      },
    },
    // 描述
    {
      component: 'Divider',
      fieldName: '_div4',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '商机描述' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '商机描述',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '详细描述商机背景、客户需求、价值主张等', rows: 4, allowClear: true, maxlength: 2000, showCount: true },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() { drawerApi.close(); },

  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    const rawValues = await baseFormApi.getValues();
    const values = {
      ...rawValues,
      customerId: rawValues.customerId != null ? Number(rawValues.customerId) : undefined,
      contactId: rawValues.contactId != null ? Number(rawValues.contactId) : undefined,
      assignedTo: rawValues.assignedTo != null ? Number(rawValues.assignedTo) : undefined,
    };
    try {
      await (data.value?.create
        ? createOpportunityApi(values)
        : updateOpportunityApi({ ...values, id: data.value.row.id }));
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

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      setLoading(false);

      // 加载负责人选项
      loadUserOptions();

      // 重置表单
      baseFormApi.resetForm();

      // 设置客户回显信息
      currentCompanyName.value = row.companyName || '';
      currentCustomerId.value = row.customerId || null;

      // 只设置 row 中存在的字段，customerId 转为 string 匹配 ApiSelect 选项
      const definedValues = Object.fromEntries(
        Object.entries(row).filter(([_, v]) => v !== undefined && v !== null),
      );
      baseFormApi.setValues({
        ...definedValues,
        customerId: row.customerId != null ? String(row.customerId) : undefined,
      });

      // 如果有 customerId，加载联系人列表
      if (row.customerId) {
        const cid = Number(row.customerId);
        currentCustomerId.value = cid;
        getCustomerContactsApi(cid)
          .then((res: any) => {
            const items: any[] = res?.data?.current || [];
            contactOptions.value = items.map((c: any) => ({
              label: c.name || c.contactName || '',
              value: Number(c.id || c.contactId),
            }));
          })
          .catch(() => {});
      } else {
        contactOptions.value = [];
      }

      // 编辑模式：从详情 API 回填完整数据（source、description、contactId 等列表 API 未返回的字段）
      if (!data.value?.create && row?.id) {
        getOpportunityInfoApi(Number(row.id))
          .then(async (detail: any) => {
            const d = detail?.data || detail || {};
            // 确保下拉选项渲染后再设置表单值，避免 Select 无法匹配 label
            await nextTick();
            baseFormApi.setValues({
              title: d.title,
              stage: d.stage != null ? Number(d.stage) : undefined,
              source: d.source != null ? Number(d.source) : undefined,
              probability: d.probability != null ? Number(d.probability) : undefined,
              amount: d.amount != null ? Number(d.amount) : undefined,
              currency: d.currency != null ? Number(d.currency) : undefined,
              expectedCloseDate: d.expectedCloseDate,
              customerId: d.customerId != null ? String(d.customerId) : undefined,
              contactId: d.contactId != null ? Number(d.contactId) : undefined,
              assignedTo: d.assignedTo != null ? Number(d.assignedTo) : undefined,
              description: d.description,
            });
          })
          .catch(() => {});
      }
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
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
