<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, h, ref } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import { message, Tooltip } from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { createInvoiceApi, getInvoiceInfoApi, updateInvoiceApi } from '#/api';
import { getContractInfoApi } from '#/api/core/crm/contract';
import { getOrderInfoApi } from '#/api/core/sale/order';
import { getUserListApi } from '#/api/core/system/user';
import ContractSelectModal from '../../crm/components/ContractSelectModal.vue';

// connectedComponent 模式下父级 setData 的数据不映射为 props，需通过 drawerApi.getData() 获取
const drawerData = ref<{ create?: boolean; row?: any }>({});

const isEdit = computed(() => !drawerData.value.create);
const isFullscreen = ref(false);

const drawerClass = computed(() => [
  'sale-invoice-drawer',
  { 'sale-invoice-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const invoiceTypeOptions = [
  { label: '增值税专用发票', value: 1 },
  { label: '增值税普通发票', value: 2 },
  { label: '形式发票(PI)', value: 3 },
  { label: '商业发票(CI)', value: 4 },
];

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const userOptions = ref<any[]>([]);

// 已选合同（新建/改选时赋值）与编辑回显的原始关联ID
const selectedContract = ref<any>(null);
const editSource = ref<{
  contractId?: any;
  orderId?: any;
  customerId?: any;
  ownerUserId?: any;
}>({});
const contractSelectVisible = ref(false);

// 后端枚举可能以字符串变体名返回（如 "CNY"），归一化为数字
const currencyNumMap: Record<string, number> = {
  cny: 1,
  usd: 2,
  eur: 3,
  gbp: 4,
  jpy: 5,
  hkd: 6,
  aud: 7,
};
function toCurrencyNum(val: any): number {
  if (typeof val === 'number') return val;
  const num = Number(val);
  if (!Number.isNaN(num) && num > 0) return num;
  return currencyNumMap[String(val).toLowerCase()] ?? 1;
}
function formatContractLabel(row: any): string {
  if (!row) return '';
  const title = row.title || (row.id ? `合同 #${row.id}` : '');
  if (!title) return '';
  return row.contractNo ? `${title}（${row.contractNo}）` : title;
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
  } catch (error) {
    console.error('Failed to load user options:', error);
  }
}

const formSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '发票标题',
    rules: 'required',
    componentProps: { placeholder: '请输入发票标题' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'invoiceType',
    label: '发票类型',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: invoiceTypeOptions },
  },
  {
    component: 'DatePicker',
    fieldName: 'invoiceDate',
    label: '开票日期',
    rules: 'required',
    componentProps: {
      placeholder: '请选择',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
    },
  },
  {
    component: 'Input',
    fieldName: 'contractDisplay',
    label: '关联合同',
    rules: 'required',
    componentProps: {
      placeholder: '点击选择合同',
      readonly: true,
      style: 'width:100%;cursor:pointer',
      onClick: () => (contractSelectVisible.value = true),
    },
    renderComponentContent: () => ({
      suffix: () =>
        h(
          'span',
          {
            class: 'text-xs text-blue-600',
            style: 'cursor:pointer;white-space:nowrap;user-select:none',
            onClick: (e: Event) => {
              e.stopPropagation();
              contractSelectVisible.value = true;
            },
          },
          '选择',
        ),
    }),
  },
  {
    component: 'Input',
    fieldName: 'orderDisplay',
    label: '关联订单',
    componentProps: {
      placeholder: '选择合同后自动带出',
      disabled: true,
      style: 'width:100%',
    },
  },
  {
    component: 'Input',
    fieldName: 'customerName',
    label: '客户名称',
    componentProps: {
      placeholder: '选择合同后自动带出',
      disabled: true,
      style: 'width:100%',
    },
  },
  {
    component: 'Input',
    fieldName: 'ownerDisplay',
    label: '负责人',
    componentProps: {
      placeholder: '选择合同后自动带出',
      disabled: true,
      style: 'width:100%',
    },
  },
  {
    component: 'DatePicker',
    fieldName: 'dueDate',
    label: '到期日',
    componentProps: {
      placeholder: '请选择',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'amount',
    label: '金额',
    rules: 'required',
    componentProps: {
      placeholder: '请输入金额',
      style: 'width:100%',
      precision: 2,
      min: 0,
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'taxRate',
    label: '税率',
    componentProps: {
      placeholder: '请输入税率',
      style: 'width:100%',
      precision: 2,
      min: 0,
      max: 100,
      addonAfter: '%',
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'taxAmount',
    label: '税额',
    componentProps: {
      placeholder: '自动计算',
      style: 'width:100%',
      precision: 2,
      min: 0,
    },
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'Input',
    fieldName: 'buyerName',
    label: '购买方名称',
    componentProps: { placeholder: '请输入购买方名称' },
  },
  {
    component: 'Input',
    fieldName: 'buyerTaxNo',
    label: '购买方税号',
    componentProps: { placeholder: '请输入购买方税号' },
  },
  {
    component: 'Input',
    fieldName: 'buyerBank',
    label: '开户行',
    componentProps: { placeholder: '请输入开户行' },
  },
  {
    component: 'Input',
    fieldName: 'taxNo',
    label: '销方税号',
    componentProps: { placeholder: '请输入销方税号' },
  },
  {
    component: 'Textarea',
    fieldName: 'buyerAddress',
    label: '购买方地址',
    componentProps: { placeholder: '请输入购买方地址、电话', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: {
      placeholder: '备注信息',
      rows: 3,
      showCount: true,
      maxlength: 500,
    },
    wrapperClass: 'col-span-2',
  },
];

const [Form, formApi] = useVbenForm({
  schema: formSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

// 编辑回显：拉取发票详情填充表单，并解析关联合同/订单/负责人的显示名称
async function fillForm(row: any) {
  try {
    // 确保负责人选项已加载，避免回显降级为 "员工 #id"
    if (!userOptions.value.length) {
      await loadUserOptions();
    }
    const info = await getInvoiceInfoApi(row.id);
    const data = info || row;
    editSource.value = {
      contractId: data.contractId ?? undefined,
      orderId: data.orderId ?? undefined,
      customerId: data.customerId ?? undefined,
      ownerUserId:
        data.ownerUserId === null || data.ownerUserId === undefined
          ? undefined
          : Number(data.ownerUserId),
    };
    // 解析合同显示名（标题+编号）
    let contractDisplay = '';
    if (data.contractId) {
      try {
        const contract = await getContractInfoApi(data.contractId);
        contractDisplay = formatContractLabel(contract);
      } catch {
        contractDisplay = `合同 #${data.contractId}`;
      }
    }
    // 解析订单号显示
    let orderDisplay = '';
    if (data.orderId) {
      try {
        const order = await getOrderInfoApi(data.orderId);
        orderDisplay = order?.orderNo || `订单 #${data.orderId}`;
      } catch {
        orderDisplay = `订单 #${data.orderId}`;
      }
    }
    // 解析负责人姓名
    const ownerDisplay =
      userOptions.value.find((u: any) => u.value === editSource.value.ownerUserId)
        ?.label ??
      (editSource.value.ownerUserId
        ? `员工 #${editSource.value.ownerUserId}`
        : '');
    formApi.setValues({
      title: data.title,
      invoiceType: data.invoiceType ?? 1,
      invoiceDate: data.invoiceDate,
      contractDisplay,
      orderDisplay,
      customerName: data.customerName,
      ownerDisplay,
      dueDate: data.dueDate,
      amount: data.amount ?? 0,
      taxRate: data.taxRate ?? 0,
      taxAmount: data.taxAmount ?? 0,
      currency: data.currency ?? 1,
      taxNo: data.taxNo,
      buyerName: data.buyerName,
      buyerTaxNo: data.buyerTaxNo,
      buyerBank: data.buyerBank,
      buyerAddress: data.buyerAddress,
      remark: data.remark,
    });
  } catch {
    formApi.setValues(row);
  }
}

// 选择合同后：自动关闭弹窗，带出订单号/客户/负责人，并默认合同金额与币种
async function handleContractSelect(row: any) {
  contractSelectVisible.value = false;
  selectedContract.value = row;
  let orderDisplay = row.orderNo || '';
  if (row.orderId && !orderDisplay) {
    try {
      const order = await getOrderInfoApi(row.orderId);
      orderDisplay = order?.orderNo || '';
    } catch {
      orderDisplay = '';
    }
  }
  const amount = Number(row.totalAmount ?? row.amount ?? 0);
  formApi.setValues({
    contractDisplay: formatContractLabel(row),
    orderDisplay,
    customerName: row.customerName || '',
    ownerDisplay: row.assignedToName || '',
    ...(amount > 0 ? { amount } : {}),
    currency: toCurrencyNum(row.currency ?? 1),
  });
}

async function handleSubmit() {
  const { valid, values } = await formApi.validate();
  if (!valid || !values) return;
  try {
    // 剔除仅用于展示的字段，按已选合同（或编辑原关联）组装真实关联数据
    const {
      contractDisplay: _cd,
      orderDisplay: _od,
      ownerDisplay: _wd,
      ...rest
    } = values;
    const picked = selectedContract.value ?? {};
    const src = editSource.value;
    const toId = (v: any) =>
      v === null || v === undefined || v === '' ? undefined : Number(v);
    const data = {
      ...rest,
      contractId: toId(picked.id ?? src.contractId),
      orderId: toId(picked.orderId ?? src.orderId),
      customerId: toId(picked.customerId ?? src.customerId),
      customerName: picked.customerName ?? rest.customerName,
      ownerUserId: toId(picked.assignedTo ?? src.ownerUserId),
    };
    if (isEdit.value) {
      await updateInvoiceApi({
        ...data,
        id: drawerData.value.row?.id,
      });
      message.success('更新成功');
    } else {
      await createInvoiceApi(data);
      message.success('创建成功');
    }
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch {
    message.error('操作失败');
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleSubmit();
  },
  onOpenChange(isOpen) {
    if (!isOpen) return;
    const data = drawerApi.getData() as { create?: boolean; row?: any };
    drawerData.value = { create: data?.create ?? true, row: data?.row ?? {} };
    isFullscreen.value = false;
    formApi.resetForm();
    selectedContract.value = null;
    editSource.value = {};
    loadUserOptions();
    if (isEdit.value && drawerData.value.row?.id) {
      fillForm(drawerData.value.row);
    } else {
      formApi.setValues({
        invoiceType: 1,
        currency: 1,
        invoiceDate: new Date().toISOString().slice(0, 10),
      });
    }
  },
});
</script>

<template>
  <Drawer
    :title="isEdit ? '编辑发票' : '新建发票'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-invoice-drawer__fs-btn"
          @click="toggleFullscreen"
        >
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 3H5a2 2 0 0 0-2 2v3" />
            <path d="M21 8V5a2 2 0 0 0-2-2h-3" />
            <path d="M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M16 21h3a2 2 0 0 0 2-2v-3" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 3v3a2 2 0 0 1-2 2H3" />
            <path d="M21 8h-3a2 2 0 0 1-2-2V3" />
            <path d="M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M16 21v-3a2 2 0 0 1 2-2h3" />
          </svg>
        </button>
      </Tooltip>
    </template>
    <Form />
    <ContractSelectModal
      v-model:visible="contractSelectVisible"
      @select="handleContractSelect"
    />
  </Drawer>
</template>

<style>
.sale-invoice-drawer {
  width: 75vw !important;
}

.sale-invoice-drawer--fullscreen {
  width: 100vw !important;
}

.sale-invoice-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  color: rgb(0 0 0 / 45%);
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 4px;
  transition: all 0.2s;
}

.sale-invoice-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}
</style>
