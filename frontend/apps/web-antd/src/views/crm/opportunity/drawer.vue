<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createOpportunityApi, updateOpportunityApi, getCustomerListApi, getCustomerContactsApi } from '#/api';

const data = ref();

// 当前选中的客户公司名（用于 ApiSelect 编辑回显）
const currentCompanyName = ref<string>('');
const currentCustomerId = ref<number | null>(null);

// 当前选中客户下的联系人列表
const contactOptions = ref<{ label: string; value: number }[]>([]);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.opportunity.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.opportunity.title') }),
);

// 商机阶段 - 数值对齐后端
const stageOptions = [
  { label: '资格审查', value: 0, color: 'blue' },
  { label: '需求分析', value: 1, color: 'cyan' },
  { label: '方案报价', value: 2, color: 'gold' },
  { label: '商务谈判', value: 3, color: 'orange' },
  { label: '已成交', value: 4, color: 'green' },
  { label: '已输单', value: 5, color: 'red' },
];

// 商机来源 - 对齐后端 LeadSource 枚举
const sourceOptions = [
  { label: '官网', value: 'website' },
  { label: '展会', value: 'exhibition' },
  { label: '社交媒体', value: 'social' },
  { label: '客户转介', value: 'referral' },
  { label: '陌生拜访', value: 'cold_call' },
  { label: '海关数据', value: 'customs' },
  { label: '邮件营销', value: 'email' },
  { label: '阿里国际站', value: 'alibaba' },
  { label: 'Amazon', value: 'amazon' },
  { label: 'TikTok', value: 'tiktok' },
  { label: '微信', value: 'wechat' },
  { label: '其他', value: 'other' },
];

// 币种 - 对齐后端 CurrencyCode 枚举
const currencyOptions = [
  { label: 'CNY 人民币', value: 'CNY' },
  { label: 'USD 美元', value: 'USD' },
  { label: 'EUR 欧元', value: 'EUR' },
  { label: 'GBP 英镑', value: 'GBP' },
  { label: 'JPY 日元', value: 'JPY' },
  { label: 'HKD 港币', value: 'HKD' },
  { label: 'AUD 澳元', value: 'AUD' },
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
      component: 'Input',
      fieldName: 'opportunityNo',
      label: '商机编号',
      componentProps: { placeholder: '保存后自动生成', disabled: true },
    },
    {
      component: 'Select',
      fieldName: 'stage',
      label: '销售阶段',
      defaultValue: 0,
      rules: 'required',
      componentProps: {
        placeholder: '请选择销售阶段',
        allowClear: false,
        options: stageOptions,
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
      label: '商机金额',
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
      defaultValue: 'CNY',
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
    {
      component: 'InputNumber',
      fieldName: 'leadId',
      label: '线索ID',
      componentProps: { placeholder: '关联线索ID', min: 0, class: 'w-full' },
    },
    {
      component: 'InputNumber',
      fieldName: 'assignedTo',
      label: '负责人ID',
      componentProps: { placeholder: '负责人ID', min: 0, class: 'w-full' },
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
    const values = await baseFormApi.getValues();
    if (!data.value?.create) {
      delete values.opportunityNo;
    }
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
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      setLoading(false);

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
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle" :width="680">
    <BaseForm />
  </Drawer>
</template>
