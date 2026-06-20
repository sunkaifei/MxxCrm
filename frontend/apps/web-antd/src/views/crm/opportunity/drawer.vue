<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createOpportunityApi, updateOpportunityApi } from '#/api';

const data = ref();

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.opportunity.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.opportunity.title') }),
);

// 商机阶段 - 对齐后端 OpportunityStage 枚举
const stageOptions = [
  { label: '资格审查', value: 'qualification', color: 'blue' },
  { label: '需求分析', value: 'needs_analysis', color: 'cyan' },
  { label: '方案报价', value: 'proposal', color: 'gold' },
  { label: '商务谈判', value: 'negotiation', color: 'orange' },
  { label: '已成交', value: 'won', color: 'green' },
  { label: '已输单', value: 'lost', color: 'red' },
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
      defaultValue: 'qualification',
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
      component: 'InputNumber',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: { placeholder: '关联客户ID', min: 0, class: 'w-full' },
    },
    {
      component: 'InputNumber',
      fieldName: 'contactId',
      label: '联系人ID',
      componentProps: { placeholder: '关联联系人ID', min: 0, class: 'w-full' },
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
    // 编辑模式下不提交只读字段
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
      baseFormApi.setValues(row);
      setLoading(false);
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