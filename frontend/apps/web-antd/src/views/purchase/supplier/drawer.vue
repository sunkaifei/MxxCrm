<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import type { VbenFormSchema } from '@vben/common-ui';
import { $t } from '#/locales';
import { createSupplierApi, getSupplierInfoApi, updateSupplierApi } from '#/api';
import { message, Tooltip } from 'ant-design-vue';

const isFullscreen = ref(false);
const confirmLoading = ref(false);
const drawerData = ref<{ create: boolean; row?: any }>({ create: true });

const drawerClass = computed(() => [
  'supplier-drawer',
  { 'supplier-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const statusOptions = [
  { label: '正常', value: 1 },
  { label: '停用', value: 2 },
  { label: '待审核', value: 3 },
  { label: '黑名单', value: 4 },
];

const levelOptions = [
  { label: '战略供应商', value: 1 },
  { label: '核心供应商', value: 2 },
  { label: '普通供应商', value: 3 },
  { label: '备选供应商', value: 4 },
];

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
  { label: 'AUD 澳元', value: 7 },
];

const industryOptions = [
  { label: '制造业', value: 'manufacturing' },
  { label: '批发业', value: 'wholesale' },
  { label: '零售业', value: 'retail' },
  { label: '服务业', value: 'service' },
  { label: '科技/电子', value: 'technology' },
  { label: '化工', value: 'chemical' },
  { label: '食品', value: 'food' },
  { label: '纺织/服装', value: 'textile' },
  { label: '建材', value: 'building' },
  { label: '物流/运输', value: 'logistics' },
  { label: '其他', value: 'other' },
];

const countryOptions = [
  { label: '中国', value: 'CN' },
  { label: '美国', value: 'US' },
  { label: '日本', value: 'JP' },
  { label: '德国', value: 'DE' },
  { label: '英国', value: 'GB' },
  { label: '韩国', value: 'KR' },
  { label: '越南', value: 'VN' },
  { label: '印度', value: 'IN' },
  { label: '中国香港', value: 'HK' },
  { label: '中国台湾', value: 'TW' },
  { label: '其他', value: 'other' },
];

const formSchema: VbenFormSchema[] = [
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
    fieldName: 'companyName',
    label: '公司名称',
    rules: 'required',
    componentProps: { placeholder: '请输入供应商公司全称', allowClear: true },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'shortName',
    label: '公司简称',
    componentProps: { placeholder: '请输入公司简称', allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'level',
    label: '供应商等级',
    defaultValue: 3,
    componentProps: { placeholder: '请选择等级', options: levelOptions, allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'industry',
    label: '所属行业',
    componentProps: { placeholder: '请选择行业', options: industryOptions, allowClear: true, showSearch: true, filterOption: true },
  },
  {
    component: 'Select',
    fieldName: 'status',
    label: '供应商状态',
    defaultValue: 1,
    componentProps: { placeholder: '请选择状态', options: statusOptions },
  },
  {
    component: 'Divider',
    fieldName: '_div2',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => '联系信息' }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'contactName',
    label: '联系人',
    rules: 'required',
    componentProps: { placeholder: '请输入联系人姓名', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'contactPhone',
    label: '联系电话',
    componentProps: { placeholder: '请输入联系电话', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'contactEmail',
    label: '电子邮箱',
    componentProps: { placeholder: '请输入邮箱地址', allowClear: true },
  },
  {
    component: 'Select',
    fieldName: 'country',
    label: '国家',
    componentProps: { placeholder: '请选择国家', options: countryOptions, allowClear: true, showSearch: true, filterOption: true },
  },
  {
    component: 'Input',
    fieldName: 'region',
    label: '省/州/地区',
    componentProps: { placeholder: '请输入省/州/地区', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'website',
    label: '公司官网',
    componentProps: { placeholder: 'https://example.com', allowClear: true },
  },
  {
    component: 'Textarea',
    fieldName: 'address',
    label: '详细地址',
    componentProps: { placeholder: '请输入详细地址', allowClear: true, rows: 2 },
    formItemClass: 'col-span-2',
  },
  {
    component: 'Divider',
    fieldName: '_div3',
    hideLabel: true,
    componentProps: { orientation: 'left', plain: true },
    renderComponentContent: () => ({ default: () => '财务与交易' }),
    formItemClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '结算币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择币种', options: currencyOptions, allowClear: true },
  },
  {
    component: 'InputNumber',
    fieldName: 'creditLimit',
    label: '信用额度',
    componentProps: { placeholder: '0.00', min: 0, precision: 2, style: { width: '100%' } },
  },
  {
    component: 'InputNumber',
    fieldName: 'creditDays',
    label: '信用天数',
    componentProps: { placeholder: '0', min: 0, precision: 0, style: { width: '100%' } },
  },
  {
    component: 'Input',
    fieldName: 'paymentTerms',
    label: '付款条款',
    componentProps: { placeholder: '如：月结30天', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'bankName',
    label: '开户银行',
    componentProps: { placeholder: '请输入开户银行名称', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'bankAccount',
    label: '银行账号',
    componentProps: { placeholder: '请输入银行账号', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'taxId',
    label: '税务登记号',
    componentProps: { placeholder: '请输入税号/统一社会信用代码', allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'deliveryTerms',
    label: '交货条款',
    componentProps: { placeholder: '如：FOB上海', allowClear: true },
  },
];

const [MainForm, mainFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: formSchema,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    try {
      const valid = await mainFormApi.validate();
      if (!valid.valid) return;

      confirmLoading.value = true;
      const values = await mainFormApi.getValues();

      const data = {
        ...values,
        isActive: true,
      };

      if (drawerData.value.create) {
        await createSupplierApi(data);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateSupplierApi({ ...data, id: drawerData.value.row.id });
        message.success($t('ui.notification.update_success'));
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      confirmLoading.value = false;
    }
  },
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{ create: boolean; row?: any }>() || { create: true };
      mainFormApi.resetForm();
      confirmLoading.value = false;
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getSupplierInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const num = (v: any) => (v === null || v === undefined ? undefined : Number(v));

    mainFormApi.setValues({
      companyName: data.companyName,
      shortName: data.shortName,
      level: num(data.level) ?? 3,
      industry: data.industry,
      status: num(data.status) ?? 1,
      contactName: data.contactName,
      contactPhone: data.contactPhone,
      contactEmail: data.contactEmail,
      country: data.country,
      region: data.region,
      website: data.website,
      address: data.address,
      currency: num(data.currency) ?? 1,
      creditLimit: data.creditLimit ? Number(data.creditLimit) : undefined,
      creditDays: data.creditDays ? Number(data.creditDays) : undefined,
      bankName: data.bankName,
      bankAccount: data.bankAccount,
      taxId: data.taxId,
      paymentTerms: data.paymentTerms,
      deliveryTerms: data.deliveryTerms,
    });
  } catch (e) {
    console.error('[供应商] 加载详情失败:', e);
  }
}
</script>

<template>
  <Drawer
    :class="drawerClass"
    :title="drawerData.create ? '新建供应商' : '编辑供应商'"
    :confirm-loading="confirmLoading"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <button type="button" class="supplier-drawer__fs-btn" @click="toggleFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </button>
      </Tooltip>
    </template>

    <div class="supplier-drawer__body">
      <MainForm />
    </div>
  </Drawer>
</template>

<style>
.supplier-drawer {
  width: 75vw !important;
}

.supplier-drawer--fullscreen {
  width: 100vw !important;
}

.supplier-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}

.supplier-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

.supplier-drawer__body {
  padding: 0 8px;
  overflow-y: auto;
  height: calc(100vh - 150px);
}

.supplier-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.supplier-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}
</style>
