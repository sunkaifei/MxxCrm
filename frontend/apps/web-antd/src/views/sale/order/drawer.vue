<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref, watch, reactive } from 'vue';
import { useRoute } from 'vue-router';

import { useVbenForm } from '@vben/common-ui';

import {
  Button,
  Input,
  InputNumber,
  Select,
  Tabs,
  TabPane,
  Table,
  Tooltip,
  message,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import ProductSelectModal from '../components/ProductSelectModal.vue';
import QuotationSelectModal from '../components/QuotationSelectModal.vue';
import OpportunitySelectModal from '../../crm/components/OpportunitySelectModal.vue';
import ContactSelectModal from '../../crm/components/ContactSelectModal.vue';
import {
  createOrderApi,
  getContractInfoApi,
  getOrderInfoApi,
  updateOrderApi,
  getCustomerFinancialApi,
  getCompanyInfoApi,
  getCompanyAccountListApi,
  getSalesFlowModeApi,
  type BankAccount,
  type SalesFlowMode,
} from '#/api';
import { getQuotationInfoApi } from '#/api/core/sale/quotation';

const props = withDefaults(
  defineProps<{ fromQuotation?: any }>(),
  { fromQuotation: () => null },
);

const route = useRoute();

// drawerData 在 onOpenChange 中手动赋值，避免引用尚未定义的 drawerApi
const drawerData = ref<{ create: boolean; row: any }>({ create: true, row: {} });

const isEdit = computed(() => !drawerData.value.create);
const activeTab = ref('basic');
const items = ref<any[]>([]);
const shippingFee = ref(0);
const taxAmount = ref(0);
const discountAmount = ref(0);
const otherFee = ref(0);
const isFullscreen = ref(false);
const submitting = ref(false);

// 销售流程模式：A=仅标准(必填报价单) B=仅简易(隐藏报价单/必填商机) both=两种都允许
const flowMode = ref<SalesFlowMode>('both');
// 是否显示"选择报价单"入口
const showQuotationSelect = computed(
  () => flowMode.value === 'A' || flowMode.value === 'both',
);
// both 模式下用户选择的销售路径：standard=标准(报价单) simple=简易(商机)
const salesMode = ref<'standard' | 'simple'>('standard');
// both 模式下是否显示模式选择器
const showModeSelector = computed(
  () => flowMode.value === 'both' && !drawerData.value.create ? false : flowMode.value === 'both',
);
// 是否显示报价单选择入口
const showQuotationEntry = computed(() => {
  if (flowMode.value === 'A') return true;
  if (flowMode.value === 'B') return false;
  // both 模式
  if (isEdit.value) return !!quotationInfo.value.id;
  return salesMode.value === 'standard';
});
// 是否显示商机选择入口
const showOpportunityEntry = computed(() => {
  if (flowMode.value === 'A') return false;
  if (flowMode.value === 'B') return true;
  // both 模式
  if (isEdit.value) return !!opportunityInfo.value.id && !quotationInfo.value.id;
  return salesMode.value === 'simple';
});
// 客户名称字段始终禁用，由商机/报价单自动带入
// 加载销售流程模式
const loadFlowMode = async () => {
  try {
    flowMode.value = await getSalesFlowModeApi();
  } catch {
    flowMode.value = 'both';
  }
};
loadFlowMode();

// 报价单选择
const quotationModalVisible = ref(false);
const quotationInfo = ref<{ id?: number; title?: string; quotationNo?: string }>({});

// 商机选择
const opportunityModalVisible = ref(false);
const opportunityInfo = ref<{ id?: number; name?: string; customerName?: string }>({});

// 联系人选择
const contactModalVisible = ref(false);
const contactInfo = ref<{ id?: number; name?: string }>({});
// 当前客户ID（用于联系人弹窗按客户过滤）
const contactFilterCustomerId = ref<number | undefined>(undefined);
// 报价单模式：产品明细一比一来源于报价单，不可增删改
const isQuotationMode = computed(() => !!quotationInfo.value.id);

function onOpportunitySelect(item: any) {
  // 记录切换前的客户ID，用于判断客户是否变化
  const previousCustomerId = contactFilterCustomerId.value;
  opportunityInfo.value = { id: item.id, name: item.opportunityName || item.title || '' };
  basicFormApi.setValues({ opportunityId: item.id });
  // 自动填充客户信息
  if (item.customerName) {
    basicFormApi.setValues({ customerName: item.customerName });
  }
  if (item.customerId) {
    basicFormApi.setValues({ customerId: item.customerId });
    contactFilterCustomerId.value = item.customerId;
    void loadBuyerFinancialInfo(item.customerId, item.customerName);
  }
  // 客户变更时重置联系人，避免残留上一个客户的联系人
  if (item.customerId !== previousCustomerId) {
    clearContact();
  }
  // 继承商机关联的联系人信息（客户未变或新商机自带联系人时填充）
  if (item.contactId) {
    contactInfo.value = { id: item.contactId, name: item.contactName || '' };
    basicFormApi.setValues({ contactId: item.contactId, contactName: item.contactName || '' });
  }
  // 选择商机后清除报价单
  clearQuotation();
}

function onContactSelect(item: any) {
  contactInfo.value = { id: item.id, name: item.name || '' };
  basicFormApi.setValues({ contactId: item.id, contactName: item.name || '' });
}

function clearContact() {
  contactInfo.value = {};
  basicFormApi.setValues({ contactId: undefined, contactName: undefined });
}

function openContactModal() {
  // 联系人必须根据客户ID过滤，未选择商机/报价单时提示用户
  if (!contactFilterCustomerId.value) {
    message.warning('请先选择商机或报价单，客户信息会自动带入');
    return;
  }
  contactModalVisible.value = true;
}

function clearOpportunity() {
  opportunityInfo.value = {};
  basicFormApi.setValues({ opportunityId: undefined });
}

function selectSalesMode(mode: 'standard' | 'simple') {
  if (isEdit.value) return;
  salesMode.value = mode;
  // 切换模式时清空另一方的选择
  if (mode === 'standard') {
    clearOpportunity();
  } else {
    clearQuotation();
  }
}

// 统一来源字段：根据模式显示"关联报价单"或"关联商机"
const sourceLabel = computed(() => {
  if (showOpportunityEntry.value) return '关联商机';
  return '关联报价单';
});
const sourceValue = computed(() => {
  if (showOpportunityEntry.value) {
    return opportunityInfo.value.name || '';
  }
  return quotationInfo.value.title || quotationInfo.value.quotationNo || '';
});
const sourceFilled = computed(() => {
  if (showOpportunityEntry.value) return !!opportunityInfo.value.id;
  return !!quotationInfo.value.id;
});
function openSourceModal() {
  if (showOpportunityEntry.value) {
    opportunityModalVisible.value = true;
  } else {
    quotationModalVisible.value = true;
  }
}
function clearSource() {
  if (showOpportunityEntry.value) {
    clearOpportunity();
  } else {
    clearQuotation();
  }
}

// 合同信息（只读）
const contractInfo = ref<{ id?: number; title?: string; contractNo?: string }>({});

// 财务信息 - 收款方（我方企业，可编辑）
const sellerInfo = reactive({
  companyName: '',
  bankName: '',
  accountName: '',
  accountNumber: '',
});

// 财务信息 - 付款方（客户，可编辑）
const buyerInfo = reactive({
  companyName: '',
  accountName: '',
  bankName: '',
  accountNumber: '',
});

// 账户列表（用于下拉选择）
const sellerAccountList = ref<any[]>([]);
const buyerAccountList = ref<any[]>([]);

const orderDetail = ref<any>({});
const financialLoading = ref(false);

// 产品选择弹窗
const productModalVisible = ref(false);

function openProductModal() {
  productModalVisible.value = true;
}

async function onQuotationSelect(item: any) {
  // 记录切换前的客户ID，用于判断客户是否变化
  const previousCustomerId = contactFilterCustomerId.value;
  quotationInfo.value = { id: item.id, title: item.title, quotationNo: item.quotationNo };
  basicFormApi.setValues({ quotationId: item.id });
  // 自动填充客户信息
  if (item.customerName) {
    basicFormApi.setValues({ customerName: item.customerName });
  }
  // 重新加载买方财务信息
  if (item.customerId) {
    basicFormApi.setValues({ customerId: item.customerId });
    contactFilterCustomerId.value = item.customerId;
    void loadBuyerFinancialInfo(item.customerId, item.customerName);
  }
  // 客户变更时重置联系人，避免残留上一个客户的联系人
  if (item.customerId !== previousCustomerId) {
    clearContact();
  }
  // 拉取报价单详情，一比一复制产品明细（含完整SKU信息）
  try {
    const resp: any = await getQuotationInfoApi(item.id);
    const detail = resp?.data ?? resp ?? {};
    // 继承报价单关联的联系人信息
    if (detail.contactId) {
      contactInfo.value = { id: detail.contactId, name: detail.contactName || '' };
      basicFormApi.setValues({ contactId: detail.contactId, contactName: detail.contactName || '' });
    }
    // 复制报价单产品明细到订单（一比一，含SKU/规格/单位/数量/单价/折扣/税率）
    items.value = Array.isArray(detail.items)
      ? detail.items.map((it: any) => ({
          productId: it.productId,
          productName: it.productName || '',
          productCode: it.productCode || '',
          spec: it.spec || '',
          unit: it.unit || '',
          productType: 1,
          quantity: Number(it.quantity) || 1,
          unitPrice: Number(it.unitPrice) || 0,
          discountRate: Number(it.discountRate) || 100,
          taxRate: Number(it.taxRate) || 0,
          amount: Number(it.subtotal) || calcLineAmount(it),
          remark: it.remark || '',
        }))
      : [];
    // 同步报价单的金额字段
    discountAmount.value = Number(detail.discountAmount) || 0;
    taxAmount.value = Number(detail.taxAmount) || 0;
    shippingFee.value = 0;
    otherFee.value = 0;
  } catch (e) {
    console.error('[订单] 加载报价单详情失败:', e);
    items.value = [];
  }
}

function clearQuotation() {
  // 清除报价单时同步清空其带来的产品明细和金额
  const wasQuotationMode = !!quotationInfo.value.id;
  quotationInfo.value = {};
  basicFormApi.setValues({ quotationId: undefined });
  if (wasQuotationMode) {
    items.value = [];
    discountAmount.value = 0;
    taxAmount.value = 0;
    shippingFee.value = 0;
    otherFee.value = 0;
  }
}

function onProductSelect(selectedItems: any[]) {
  selectedItems.forEach((item) => {
    items.value.push({
      productId: item.productId,
      productName: item.productName,
      productCode: item.productCode || '',
      spec: item.spec || '',
      unit: item.unit || '',
      productType: item.productType ?? 1,
      quantity: 1,
      unitPrice: item.unitPrice || 0,
      discountRate: 100,
      taxRate: 0,
      amount: 0,
    });
    updateLineAmount(items.value.length - 1);
  });
}

const drawerClass = computed(() => [
  'sale-order-drawer',
  { 'sale-order-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const shippingMethodOptions = [
  { label: '快递', value: 1 },
  { label: '物流', value: 2 },
  { label: '自提', value: 3 },
  { label: '送货上门', value: 4 },
];

const paymentMethodOptions = [
  { label: '银行转账', value: 1 },
  { label: '支付宝', value: 2 },
  { label: '微信支付', value: 3 },
  { label: '现金', value: 4 },
  { label: '支票', value: 5 },
  { label: '其他', value: 6 },
];

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '订单标题',
    rules: 'required',
    componentProps: { placeholder: '如：XX公司采购订单' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'DatePicker',
    fieldName: 'orderDate',
    label: '下单日期',
    rules: 'required',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'Input',
    fieldName: 'customerName',
    label: '客户名称',
    rules: 'required',
    componentProps: () => ({
      placeholder: '由商机/报价单自动带入',
      disabled: true,
    }),
  },
  {
    component: 'Input',
    fieldName: 'contactName',
    label: '联系人',
    rules: 'required',
    componentProps: () => ({
      placeholder: '点击选择联系人（必填）',
      readOnly: true,
      style: 'cursor: pointer',
      onClick: () => openContactModal(),
      suffix: '点击选择',
    }),
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'DatePicker',
    fieldName: 'deliveryDate',
    label: '预计交付日期',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '备注信息', rows: 3, showCount: true, maxlength: 500 },
    wrapperClass: 'col-span-2',
  },
];

const shippingFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'receiverName',
    label: '收货人',
    componentProps: { placeholder: '请输入收货人姓名' },
  },
  {
    component: 'Input',
    fieldName: 'receiverPhone',
    label: '收货人电话',
    componentProps: { placeholder: '请输入联系电话' },
  },
  {
    component: 'Textarea',
    fieldName: 'shippingAddress',
    label: '收货地址',
    componentProps: { placeholder: '请输入收货地址', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'billingAddress',
    label: '账单地址',
    componentProps: { placeholder: '请输入账单地址', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'shippingMethod',
    label: '配送方式',
    componentProps: { placeholder: '请选择', options: shippingMethodOptions, allowClear: true },
  },
  {
    component: 'Input',
    fieldName: 'trackingNo',
    label: '物流单号',
    componentProps: { placeholder: '请输入物流单号' },
  },
];

const paymentFormSchema: VbenFormSchema[] = [
  {
    component: 'Select',
    fieldName: 'paymentMethod',
    label: '支付方式',
    componentProps: { placeholder: '请选择', options: paymentMethodOptions, allowClear: true },
    rules: 'required',
  },
  {
    component: 'DatePicker',
    fieldName: 'paymentDueDate',
    label: '付款截止日期',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
    rules: 'required',
  },
  {
    component: 'InputNumber',
    fieldName: 'paidAmount',
    label: '已付金额',
    componentProps: { placeholder: '已付金额', style: 'width:100%', precision: 2, disabled: true },
  },
  {
    component: 'InputNumber',
    fieldName: 'unpaidAmount',
    label: '未付金额',
    componentProps: { placeholder: '未付金额', style: 'width:100%', precision: 2, disabled: true },
  },
];

const [BasicForm, basicFormApi] = useVbenForm({
  schema: basicFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [ShippingForm, shippingFormApi] = useVbenForm({
  schema: shippingFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [PaymentForm, paymentFormApi] = useVbenForm({
  schema: paymentFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

function calcLineAmount(item: any): number {
  const qty = Number(item.quantity) || 0;
  const price = Number(item.unitPrice) || 0;
  const disc = Number(item.discountRate) || 100;
  const tax = Number(item.taxRate) || 0;
  const gross = qty * price;
  const discountAmt = gross * (100 - disc) / 100;
  const afterDisc = gross - discountAmt;
  const taxAmt = afterDisc * tax / 100;
  const lineAmt = afterDisc + taxAmt;
  return Math.round(lineAmt * 100) / 100;
}

function updateLineAmount(index: number) {
  items.value[index].amount = calcLineAmount(items.value[index]);
}

const productAmount = computed(() => {
  return items.value.reduce((sum, item) => sum + (Number(item.amount) || 0), 0);
});

const totalAmount = computed(() => {
  const prod = productAmount.value;
  const ship = Number(shippingFee.value) || 0;
  const tax = Number(taxAmount.value) || 0;
  const other = Number(otherFee.value) || 0;
  const disc = Number(discountAmount.value) || 0;
  return Math.round((prod - disc + ship + tax + other) * 100) / 100;
});

const unpaidAmountComp = computed(() => {
  const paid = Number(drawerData.value.row?.paidAmount) || 0;
  return Math.round((totalAmount.value - paid) * 100) / 100;
});

watch(
  [productAmount, shippingFee, taxAmount, discountAmount, otherFee, totalAmount],
  () => {
    paymentFormApi.setValues({
      paidAmount: Number(drawerData.value.row?.paidAmount) || 0,
      unpaidAmount: unpaidAmountComp.value,
    });
  },
  { immediate: true },
);

const itemColumns = [
  { title: '#', width: 45, key: 'seq', customRender: ({ index }: any) => index + 1, align: 'center' },
  { title: '产品信息', dataIndex: 'productName', key: 'product', width: 240 },
  { title: '规格', dataIndex: 'spec', key: 'spec', width: 110 },
  { title: '单位', dataIndex: 'unit', key: 'unit', width: 55, align: 'center' },
  { title: '数量', dataIndex: 'quantity', key: 'quantity', width: 80 },
  { title: '单价', dataIndex: 'unitPrice', key: 'unitPrice', width: 95, align: 'right' },
  { title: '折扣率(%)', dataIndex: 'discountRate', key: 'discountRate', width: 75 },
  { title: '税率(%)', dataIndex: 'taxRate', key: 'taxRate', width: 75 },
  { title: '金额', dataIndex: 'amount', key: 'amount', width: 105, align: 'right' },
  { title: '操作', key: 'action', width: 55, align: 'center' },
];

function addItem() {
  openProductModal();
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}

async function loadSellerFinancialInfo() {
  try {
    const [companyInfo, accountList]: any = await Promise.all([
      getCompanyInfoApi(),
      getCompanyAccountListApi(),
    ]);
    sellerAccountList.value = Array.isArray(accountList) ? accountList : [];
    const defaultAccount = sellerAccountList.value.find((a: any) => a.isDefault === 1) || sellerAccountList.value[0];
    sellerInfo.companyName = companyInfo?.companyName || companyInfo?.companyAbbr || '';
    sellerInfo.bankName = defaultAccount?.bankName || '';
    sellerInfo.accountName = defaultAccount?.accountName || '';
    sellerInfo.accountNumber = defaultAccount?.accountNumber || '';
  } catch (e) {
    console.error('[财务信息] 加载收款方信息失败:', e);
  }
}

async function loadBuyerFinancialInfo(customerId: number, customerName?: string) {
  if (!customerId) {
    buyerInfo.companyName = customerName || '';
    buyerInfo.accountName = '';
    buyerInfo.bankName = '';
    buyerInfo.accountNumber = '';
    buyerAccountList.value = [];
    return;
  }
  try {
    const resp: any = await getCustomerFinancialApi(customerId);
    const data = resp?.data ?? resp ?? null;
    const bankAccounts: BankAccount[] = data?.bankAccounts || [];
    // 为没有 id 的账户生成稳定 id（用于下拉选择）
    buyerAccountList.value = bankAccounts.map((a: any, i: number) => ({ ...a, _idx: i }));
    const defaultAccount = bankAccounts.find((a) => a.isDefault) || bankAccounts[0];
    buyerInfo.companyName = customerName || '';
    buyerInfo.accountName = defaultAccount?.accountName || '';
    buyerInfo.bankName = defaultAccount?.bankName || '';
    buyerInfo.accountNumber = defaultAccount?.accountNumber || '';
  } catch (e) {
    console.error('[财务信息] 加载付款方信息失败:', e);
    buyerInfo.companyName = customerName || '';
    buyerInfo.accountName = '';
    buyerInfo.bankName = '';
    buyerInfo.accountNumber = '';
    buyerAccountList.value = [];
  }
}

async function loadFinancialInfo(customerId: number, customerName?: string) {
  financialLoading.value = true;
  try {
    await Promise.all([
      loadSellerFinancialInfo(),
      loadBuyerFinancialInfo(customerId, customerName),
    ]);
  } finally {
    financialLoading.value = false;
  }
}

// 选择收款方账户
function onSellerAccountChange(value: any) {
  const account = sellerAccountList.value.find((a: any) => a.id === value);
  if (account) {
    sellerInfo.bankName = account.bankName || '';
    sellerInfo.accountName = account.accountName || '';
    sellerInfo.accountNumber = account.accountNumber || '';
  }
}

// 选择付款方账户
function onBuyerAccountChange(value: any) {
  const account = buyerAccountList.value.find((a: any) => a._idx === value);
  if (account) {
    buyerInfo.accountName = account.accountName || '';
    buyerInfo.bankName = account.bankName || '';
    buyerInfo.accountNumber = account.accountNumber || '';
  }
}

async function loadOrderDetail(orderId: number) {
  try {
    const resp: any = await getOrderInfoApi(orderId);
    const data = resp?.data ?? resp ?? {};
    orderDetail.value = data;
    basicFormApi.setValues({
      title: data.title,
      orderDate: data.orderDate,
      customerId: data.customerId,
      customerName: data.customerName,
      contactId: data.contactId,
      contactName: data.contactName,
      ownerUserId: data.ownerUserId,
      deptId: data.deptId,
      opportunityId: data.opportunityId,
      currency: data.currency ?? 1,
      deliveryDate: data.deliveryDate,
      remark: data.remark,
    });
    // 报价单信息
    quotationInfo.value = data.quotationId
      ? { id: data.quotationId, title: data.quotationTitle || data.quotationNo || '' }
      : {};
    // 商机信息
    opportunityInfo.value = data.opportunityId
      ? { id: data.opportunityId, name: data.opportunityName || '' }
      : {};
    // both 模式下恢复 salesMode
    if (flowMode.value === 'both') {
      salesMode.value = data.quotationId ? 'standard' : 'simple';
    }
    // 联系人信息恢复
    contactInfo.value = data.contactId
      ? { id: data.contactId, name: data.contactName || '' }
      : {};
    contactFilterCustomerId.value = data.customerId;
    // 合同信息（只读）
    contractInfo.value = data.contractId
      ? { id: data.contractId, title: data.contractTitle || data.contractNo || '' }
      : {};
    shippingFormApi.setValues({
      receiverName: data.receiverName,
      receiverPhone: data.receiverPhone,
      shippingAddress: data.shippingAddress,
      billingAddress: data.billingAddress,
      shippingMethod: data.shippingMethod,
      trackingNo: data.trackingNo,
    });
    paymentFormApi.setValues({
      paymentMethod: data.paymentMethod,
      paymentDueDate: data.paymentDueDate,
      paidAmount: data.paidAmount ?? 0,
      unpaidAmount: data.unpaidAmount ?? 0,
    });
    // 加载财务信息（双方账户信息），然后用订单保存的值覆盖
    await loadFinancialInfo(data.customerId, data.customerName);
    if (data.buyerCompanyName) buyerInfo.companyName = data.buyerCompanyName;
    if (data.buyerAccountName) buyerInfo.accountName = data.buyerAccountName;
    if (data.buyerBankName) buyerInfo.bankName = data.buyerBankName;
    if (data.buyerAccountNumber) buyerInfo.accountNumber = data.buyerAccountNumber;
    if (data.sellerCompanyName) sellerInfo.companyName = data.sellerCompanyName;
    if (data.sellerBankName) sellerInfo.bankName = data.sellerBankName;
    if (data.sellerAccountName) sellerInfo.accountName = data.sellerAccountName;
    if (data.sellerAccountNumber) sellerInfo.accountNumber = data.sellerAccountNumber;
    items.value = Array.isArray(data.items)
      ? data.items.map((it: any) => ({
          id: it.id,
          productId: it.productId,
          productName: it.productName || '',
          productCode: it.productCode || '',
          spec: it.spec || it.sku || '',
          unit: it.unit || '',
          productType: it.productType ?? 1,
          quantity: Number(it.quantity) || 1,
          unitPrice: Number(it.unitPrice) || 0,
          discountRate: Number(it.discountRate) ?? Number(it.discount_rate) ?? 100,
          taxRate: Number(it.taxRate) ?? Number(it.tax_rate) ?? 0,
          amount: Number(it.amount) || 0,
        }))
      : [];
    shippingFee.value = Number(data.shippingFee) ?? 0;
    taxAmount.value = Number(data.taxAmount) ?? 0;
    discountAmount.value = Number(data.discountAmount) ?? 0;
    otherFee.value = Number(data.otherFee) ?? 0;
  } catch (e) {
    console.error('[订单] 加载详情失败:', e);
    items.value = [];
    shippingFee.value = 0;
    taxAmount.value = 0;
    discountAmount.value = 0;
    otherFee.value = 0;
  }
}

// 数据加载统一在 onOpenChange 中通过 loadOrderDetail 处理，无需额外 watch

async function handleSubmit() {
  console.log('[订单提交] ========== 开始提交流程 ==========');
  console.log('[订单提交] isEdit:', isEdit.value, 'rowId:', drawerData.value.row?.id);

  try {
    // 1. 表单验证
    let validResult;
    try {
      validResult = await basicFormApi.validate();
      console.log('[订单提交] 基本表单验证结果:', validResult);
    } catch (e) {
      console.error('[订单提交] 基本表单验证异常:', e);
      activeTab.value = 'basic';
      message.warning('请完善基本信息');
      return;
    }
    if (!validResult?.valid) {
      console.warn('[订单提交] 基本表单验证失败');
      activeTab.value = 'basic';
      message.warning('请完善必填项');
      return;
    }

    // 2. 商品明细检查
    console.log('[订单提交] 商品明细数量:', items.value.length);
    if (items.value.length === 0) {
      message.error('请至少添加一条商品明细');
      activeTab.value = 'items';
      return;
    }

    // 3. 收集数据
    const basicValues = await basicFormApi.getValues();
    const shippingValues = await shippingFormApi.getValues();
    const paymentValues = await paymentFormApi.getValues();
    console.log('[订单提交] 表单数据收集完成:', { basicValues, shippingValues, paymentValues });

    // 销售流程模式校验
    if (!quotationInfo.value.id && !basicValues.opportunityId) {
      if (flowMode.value === 'A') {
        message.error('当前为标准流程模式，订单必须关联报价单');
        activeTab.value = 'basic';
        return;
      }
      if (flowMode.value === 'B') {
        message.error('当前为简易流程模式，订单必须关联商机');
        activeTab.value = 'basic';
        return;
      }
      // both 模式：标准和简易都没选
      if (flowMode.value === 'both') {
        message.error('请选择报价单或商机来创建订单');
        activeTab.value = 'basic';
        return;
      }
    }

    // 4. 财务信息校验：支付账户名称、开户行、开户账号、支付方式、支付截止日期不能为空
    if (!buyerInfo.accountName || !buyerInfo.accountName.trim()) {
      message.error('请填写支付账户名称');
      activeTab.value = 'payment';
      return;
    }
    if (!buyerInfo.bankName || !buyerInfo.bankName.trim()) {
      message.error('请填写开户行');
      activeTab.value = 'payment';
      return;
    }
    if (!buyerInfo.accountNumber || !buyerInfo.accountNumber.trim()) {
      message.error('请填写开户账号');
      activeTab.value = 'payment';
      return;
    }
    if (!paymentValues.paymentMethod) {
      message.error('请选择支付方式');
      activeTab.value = 'payment';
      return;
    }
    if (!paymentValues.paymentDueDate) {
      message.error('请选择支付截止日期');
      activeTab.value = 'payment';
      return;
    }

    const submitItems = items.value.map((item, idx) => ({
      ...item,
      sort: idx,
      productType: Number(item.productType) || 1,
      quantity: Number(item.quantity) || 1,
      unitPrice: Number(item.unitPrice) || 0,
      discountRate: Number(item.discountRate) || 100,
      taxRate: Number(item.taxRate) || 0,
      amount: calcLineAmount(item),
    }));

    const data = {
      ...basicValues,
      ...shippingValues,
      // 显式带上 quotationId（来自选择报价单弹窗，schema 中无此字段）
      quotationId: quotationInfo.value.id || undefined,
      // 显式带上 contactId（来自联系人选择，schema 中无此字段）
      contactId: contactInfo.value.id || undefined,
      contactName: contactInfo.value.name || undefined,
      paymentMethod: paymentValues.paymentMethod,
      paymentDueDate: paymentValues.paymentDueDate,
      items: submitItems,
      shippingFee: Number(shippingFee.value) || 0,
      taxAmount: Number(taxAmount.value) || 0,
      discountAmount: Number(discountAmount.value) || 0,
      otherFee: Number(otherFee.value) || 0,
      productAmount: productAmount.value,
      totalAmount: totalAmount.value,
      // 双方财务信息
      buyerCompanyName: buyerInfo.companyName || undefined,
      buyerAccountName: buyerInfo.accountName || undefined,
      buyerBankName: buyerInfo.bankName || undefined,
      buyerAccountNumber: buyerInfo.accountNumber || undefined,
      sellerCompanyName: sellerInfo.companyName || undefined,
      sellerBankName: sellerInfo.bankName || undefined,
      sellerAccountName: sellerInfo.accountName || undefined,
      sellerAccountNumber: sellerInfo.accountNumber || undefined,
    };

    submitting.value = true;
    const submitData = isEdit.value
      ? { ...data, id: drawerData.value.row.id }
      : data;
    console.log('[订单提交] 提交数据:', submitData);

    if (isEdit.value) {
      await updateOrderApi(submitData);
      message.success('更新成功');
    } else {
      await createOrderApi(submitData);
      message.success('创建成功');
    }
    closeDrawer();
  } catch (e) {
    console.error('[订单提交] 提交失败:', e);
    message.error('操作失败');
  } finally {
    submitting.value = false;
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onConfirm: handleSubmit,
  onOpenChange(isOpen) {
    if (isOpen) {
      // 同步 drawerApi.getData() 到 drawerData ref
      const data = drawerApi.getData() as { create?: boolean; row?: any };
      drawerData.value = { create: data?.create ?? true, row: data?.row ?? {} };
      isFullscreen.value = false;
      activeTab.value = 'basic';
      // 重置商机信息
      opportunityInfo.value = {};
      // 重置联系人信息
      contactInfo.value = {};
      contactFilterCustomerId.value = undefined;
      salesMode.value = 'standard';
      if (props.fromQuotation) {
        // 从报价单创建订单，预填充报价单信息
        basicFormApi.resetForm();
        shippingFormApi.resetForm();
        paymentFormApi.resetForm();
        items.value = [];
        shippingFee.value = 0;
        taxAmount.value = 0;
        discountAmount.value = 0;
        otherFee.value = 0;
        quotationInfo.value = {};
        contractInfo.value = {};
        const q = props.fromQuotation;
        basicFormApi.setValues({
          title: q.title ? `${q.title}-订单` : '',
          orderDate: new Date().toISOString().slice(0, 10),
          customerId: q.customerId,
          customerName: q.customerName,
          contactId: q.contactId,
          contactName: q.contactName,
          opportunityId: q.opportunityId,
          currency: q.currency ?? 1,
          deliveryDate: q.deliveryDate,
          remark: q.remark,
        });
        quotationInfo.value = { id: q.id, title: q.title, quotationNo: q.quotationNo };
        // 继承报价单联系人信息
        contactFilterCustomerId.value = q.customerId;
        contactInfo.value = q.contactId
          ? { id: q.contactId, name: q.contactName || '' }
          : {};
        // 复制报价单产品明细到订单
        items.value = Array.isArray(q.items)
          ? q.items.map((it: any) => ({
              productId: it.productId,
              productName: it.productName || '',
              productCode: it.productCode || '',
              spec: it.spec || '',
              unit: it.unit || '',
              productType: 1,
              quantity: Number(it.quantity) || 1,
              unitPrice: Number(it.unitPrice) || 0,
              discountRate: Number(it.discountRate) || 100,
              taxRate: Number(it.taxRate) || 0,
              amount: Number(it.subtotal) || calcLineAmount(it),
              remark: it.remark || '',
            }))
          : [];
        discountAmount.value = Number(q.discountAmount) || 0;
        taxAmount.value = Number(q.taxAmount) || 0;
        shippingFee.value = 0;
        otherFee.value = 0;
        // 加载财务信息
        void loadFinancialInfo(q.customerId, q.customerName);
      } else if (!drawerData.value.create && drawerData.value.row?.id) {
        // 编辑模式：直接加载订单详情（不先 reset，避免清除 watch 已加载的数据）
        void loadOrderDetail(drawerData.value.row.id);
      } else {
        basicFormApi.resetForm();
        shippingFormApi.resetForm();
        paymentFormApi.resetForm();
        items.value = [];
        shippingFee.value = 0;
        taxAmount.value = 0;
        discountAmount.value = 0;
        otherFee.value = 0;
        quotationInfo.value = {};
        contractInfo.value = {};
        basicFormApi.setValues({
          currency: 1,
          orderDate: new Date().toISOString().slice(0, 10),
        });
        const contractIdFromRoute = route.query.contractId;
        if (contractIdFromRoute) {
          void loadContractInfo(Number(contractIdFromRoute));
        }
        // 加载收款方财务信息（我方企业）
        void loadSellerFinancialInfo();
      }
    }
  },
});

async function loadContractInfo(contractId: number) {
  try {
    const info: any = await getContractInfoApi(contractId);
    const data = info || {};
    basicFormApi.setValues({
      title: data.title ? `${data.title}-订单` : '',
      customerId: data.customerId,
      opportunityId: data.opportunityId,
      ownerUserId: data.assignedTo,
      contractId,
      currency: data.currency ?? 1,
      orderDate: new Date().toISOString().slice(0, 10),
    });
  } catch {
    // 加载合同信息失败时忽略，不影响新建订单流程
  }
}

function closeDrawer() {
  drawerApi.close();
  drawerApi.setData({ needRefresh: true });
}

watch(submitting, (val) => {
  drawerApi.setState({ confirmLoading: val });
});

// 也可以直接从路由参数传递合同信息
</script>

<template>
  <Drawer
    :title="isEdit ? '修改订单信息' : '新建销售订单'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button type="button" class="sale-order-drawer__fs-btn" @click="toggleFullscreen">
          <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 3H5a2 2 0 0 0-2 2v3" />
            <path d="M21 8V5a2 2 0 0 0-2-2h-3" />
            <path d="M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M16 21h3a2 2 0 0 0 2-2v-3" />
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 3v3a2 2 0 0 1-2 2H3" />
            <path d="M21 8h-3a2 2 0 0 1-2-2V3" />
            <path d="M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M16 21v-3a2 2 0 0 1 2-2h3" />
          </svg>
        </button>
      </Tooltip>
    </template>
    <Tabs v-model:activeKey="activeTab">
      <TabPane key="basic" tab="基本信息">
        <!-- 销售模式选择器（仅 both 模式 + 新建时显示） -->
        <div v-if="showModeSelector" class="sales-mode-selector">
          <div
            class="mode-card"
            :class="{ 'mode-card--active': salesMode === 'standard' }"
            @click="selectSalesMode('standard')"
          >
            <div class="mode-card__icon mode-card__icon--standard">
              <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
                <polyline points="10 9 9 9 8 9"/>
              </svg>
            </div>
            <div class="mode-card__content">
              <div class="mode-card__title">标准模式</div>
              <div class="mode-card__desc">从报价单转订单，完整流程</div>
            </div>
            <div class="mode-card__check" v-if="salesMode === 'standard'">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </div>
          </div>

          <div class="mode-card__divider">
            <span>或</span>
          </div>

          <div
            class="mode-card"
            :class="{ 'mode-card--active': salesMode === 'simple' }"
            @click="selectSalesMode('simple')"
          >
            <div class="mode-card__icon mode-card__icon--simple">
              <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                <polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
            </div>
            <div class="mode-card__content">
              <div class="mode-card__title">简易模式</div>
              <div class="mode-card__desc">从商机直接转订单，快捷开单</div>
            </div>
            <div class="mode-card__check" v-if="salesMode === 'simple'">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </div>
          </div>
        </div>

        <!-- 来源选择字段（必填，在客户表单前） -->
        <div class="source-field-row">
          <label class="source-field-label">
            <span class="source-field-required">*</span>{{ sourceLabel }}
          </label>
          <div class="source-field-control">
            <div
              class="source-field-input"
              :class="{ 'source-field-input--filled': sourceFilled }"
              @click="openSourceModal"
            >
              <span v-if="sourceFilled" class="source-field-value">{{ sourceValue }}</span>
              <span v-else class="source-field-placeholder">请选择{{ sourceLabel }}</span>
              <div class="source-field-actions" v-if="sourceFilled && !isEdit">
                <button type="button" class="source-field-btn" @click.stop="clearSource">
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"/>
                    <line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              </div>
              <div class="source-field-arrow" v-else>
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </div>
            </div>
          </div>
        </div>

        <BasicForm />

        <!-- 合同信息（只读，仅编辑时显示） -->
        <div v-if="contractInfo.id" class="source-select-row">
          <span class="source-select-label">关联合同：</span>
          <div class="flex-1">
            <span class="text-sm">{{ contractInfo.title || contractInfo.contractNo || `合同 #${contractInfo.id}` }}</span>
          </div>
        </div>

        <!-- 报价单选择弹窗 -->
        <QuotationSelectModal
          v-model:visible="quotationModalVisible"
          @select="onQuotationSelect"
        />

        <!-- 商机选择弹窗 -->
        <OpportunitySelectModal
          v-model:visible="opportunityModalVisible"
          @select="onOpportunitySelect"
        />

        <!-- 联系人选择弹窗 -->
        <ContactSelectModal
          v-model:visible="contactModalVisible"
          :customer-id="contactFilterCustomerId"
          @select="onContactSelect"
        />
      </TabPane>
      <TabPane key="items" tab="商品明细">
        <!-- 报价单模式提示 -->
        <div v-if="isQuotationMode" class="mb-3 px-3 py-2 rounded text-xs" style="background: hsl(var(--primary) / 0.06); color: hsl(var(--primary)); border: 1px solid hsl(var(--primary) / 0.2);">
          当前为报价单模式，产品明细一比一来源于报价单，不可增删改
        </div>

        <!-- 空状态 -->
        <div v-if="items.length === 0" class="py-12 text-center">
          <div class="mb-4" style="color: hsl(var(--muted-foreground))">暂无商品，请添加产品到订单</div>
          <Button v-if="!isQuotationMode" type="primary" @click="openProductModal">添加产品</Button>
        </div>

        <!-- 商品列表 -->
        <template v-else>
          <div class="mb-3 flex justify-between items-center">
            <span class="text-sm" style="color: hsl(var(--muted-foreground))">共 {{ items.length }} 项</span>
            <Button v-if="!isQuotationMode" type="primary" size="small" @click="openProductModal">继续添加</Button>
          </div>
          <Table
            :columns="itemColumns"
            :data-source="items"
            :pagination="false"
            size="small"
            :scroll="{ x: 1150 }"
            row-key="(_: any, index: number) => index"
            bordered
          >
            <template #bodyCell="{ column, record, index }">
              <template v-if="column.key === 'product'">
                <div class="flex flex-col">
                  <span class="font-medium">{{ record.productName || '-' }}</span>
                  <span class="text-xs" style="color: hsl(var(--muted-foreground))">{{ record.productCode || '' }}</span>
                </div>
              </template>
              <template v-else-if="column.key === 'spec'">
                <span class="text-sm">{{ record.spec || '-' }}</span>
              </template>
              <template v-else-if="column.key === 'unit'">
                <span class="text-center">{{ record.unit || '-' }}</span>
              </template>
              <template v-else-if="column.key === 'quantity'">
                <InputNumber
                  v-model:value="record.quantity"
                  :min="1"
                  :precision="0"
                  style="width: 80px"
                  size="small"
                  :disabled="isQuotationMode"
                  @change="() => updateLineAmount(index)"
                />
              </template>
              <template v-else-if="column.key === 'unitPrice'">
                <InputNumber
                  v-model:value="record.unitPrice"
                  :min="0"
                  :precision="2"
                  style="width: 90px"
                  size="small"
                  :disabled="isQuotationMode"
                  @change="() => updateLineAmount(index)"
                />
              </template>
              <template v-else-if="column.key === 'discountRate'">
                <InputNumber
                  v-model:value="record.discountRate"
                  :min="0"
                  :max="100"
                  :precision="1"
                  style="width: 70px"
                  size="small"
                  :disabled="isQuotationMode"
                  @change="() => updateLineAmount(index)"
                />
              </template>
              <template v-else-if="column.key === 'taxRate'">
                <InputNumber
                  v-model:value="record.taxRate"
                  :min="0"
                  :max="100"
                  :precision="1"
                  style="width: 70px"
                  size="small"
                  :disabled="isQuotationMode"
                  @change="() => updateLineAmount(index)"
                />
              </template>
              <template v-else-if="column.key === 'amount'">
                <span class="font-medium">{{ (record.amount || 0).toFixed(2) }}</span>
              </template>
              <template v-else-if="column.key === 'action'">
                <Button v-if="!isQuotationMode" type="link" danger size="small" @click="removeItem(index)">删除</Button>
                <span v-else style="color: hsl(var(--muted-foreground))">—</span>
              </template>
            </template>
          </Table>
        </template>

        <!-- 金额汇总（始终显示） -->
        <div class="mt-4 flex flex-col items-end gap-2 pr-4">
          <div class="flex items-center gap-2">
            <span class="w-32 text-right" style="color: hsl(var(--muted-foreground))">商品金额合计：</span>
            <span class="w-32 text-right">{{ productAmount.toFixed(2) }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right" style="color: hsl(var(--muted-foreground))">整单折扣：</span>
            <InputNumber
              v-model:value="discountAmount"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right" style="color: hsl(var(--muted-foreground))">运费：</span>
            <InputNumber
              v-model:value="shippingFee"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right" style="color: hsl(var(--muted-foreground))">税额：</span>
            <InputNumber
              v-model:value="taxAmount"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right" style="color: hsl(var(--muted-foreground))">其他费用：</span>
            <InputNumber
              v-model:value="otherFee"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2 border-t pt-2">
            <span class="w-24 text-right font-medium">订单总金额：</span>
            <span class="w-32 text-right text-lg font-bold" style="color: hsl(0 84% 60%)">
              {{ totalAmount.toFixed(2) }}
            </span>
          </div>
        </div>

        <!-- 产品选择弹窗 -->
        <ProductSelectModal
          v-model:visible="productModalVisible"
          @select="onProductSelect"
        />
      </TabPane>
      <TabPane key="shipping" tab="收货信息">
        <ShippingForm />
      </TabPane>
      <TabPane key="payment" tab="财务信息">
        <div class="financial-section">
          <div class="financial-row">
            <div class="financial-card buyer-card">
              <div class="financial-card-header">
                <div class="financial-card-icon buyer-icon">
                  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"></path>
                    <circle cx="9" cy="7" r="4"></circle>
                    <path d="M22 21v-2a4 4 0 0 0-3-3.87"></path>
                    <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
                  </svg>
                </div>
                <div class="financial-card-title">
                  <div class="title-main">付款方</div>
                  <div class="title-sub">买方 · 客户</div>
                </div>
                <Select
                  v-if="buyerAccountList.length > 0"
                  class="financial-account-select"
                  placeholder="选择已有账户"
                  allow-clear
                  size="small"
                  :options="buyerAccountList.map((a) => ({ label: `${a.bankName || '—'} · ${(a.accountNumber || '').slice(-4) || '—'}`, value: a._idx }))"
                  @change="onBuyerAccountChange"
                />
              </div>
              <div class="financial-card-body">
                <div class="fin-item">
                  <span class="fin-label">企业名称</span>
                  <Input
                    v-model:value="buyerInfo.companyName"
                    class="fin-input"
                    placeholder="付款方企业名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label"><span class="fin-required">*</span>开户行</span>
                  <Input
                    v-model:value="buyerInfo.bankName"
                    class="fin-input"
                    placeholder="开户行名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label"><span class="fin-required">*</span>账户名称</span>
                  <Input
                    v-model:value="buyerInfo.accountName"
                    class="fin-input"
                    placeholder="账户名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label"><span class="fin-required">*</span>银行账号</span>
                  <Input
                    v-model:value="buyerInfo.accountNumber"
                    class="fin-input mono"
                    placeholder="银行账号"
                    :bordered="false"
                  />
                </div>
              </div>
            </div>

            <div class="financial-arrow">
              <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="5" y1="12" x2="19" y2="12"></line>
                <polyline points="12 5 19 12 12 19"></polyline>
              </svg>
            </div>

            <div class="financial-card seller-card">
              <div class="financial-card-header">
                <div class="financial-card-icon seller-icon">
                  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                    <polyline points="9 22 9 12 15 12 15 22"></polyline>
                  </svg>
                </div>
                <div class="financial-card-title">
                  <div class="title-main">收款方</div>
                  <div class="title-sub">卖方 · 我方企业</div>
                </div>
                <Select
                  v-if="sellerAccountList.length > 0"
                  class="financial-account-select"
                  placeholder="选择已有账户"
                  allow-clear
                  size="small"
                  :options="sellerAccountList.map((a) => ({ label: `${a.bankName || '—'} · ${(a.accountNumber || '').slice(-4) || '—'}`, value: a.id }))"
                  @change="onSellerAccountChange"
                />
              </div>
              <div class="financial-card-body">
                <div class="fin-item">
                  <span class="fin-label">企业名称</span>
                  <Input
                    v-model:value="sellerInfo.companyName"
                    class="fin-input"
                    placeholder="收款方企业名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label">开户行</span>
                  <Input
                    v-model:value="sellerInfo.bankName"
                    class="fin-input"
                    placeholder="开户行名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label">账户名称</span>
                  <Input
                    v-model:value="sellerInfo.accountName"
                    class="fin-input"
                    placeholder="账户名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label">银行账号</span>
                  <Input
                    v-model:value="sellerInfo.accountNumber"
                    class="fin-input mono"
                    placeholder="银行账号"
                    :bordered="false"
                  />
                </div>
              </div>
            </div>
          </div>

          <div class="payment-summary-card">
            <div class="payment-summary-title">支付信息</div>
            <div class="payment-summary-grid">
              <div class="pay-item">
                <span class="pay-label">支付方式</span>
                <span class="pay-value">
                  {{ paymentMethodOptions.find((o) => o.value === orderDetail?.paymentMethod)?.label || '—' }}
                </span>
              </div>
              <div class="pay-item">
                <span class="pay-label">付款截止日期</span>
                <span class="pay-value">{{ orderDetail?.paymentDueDate || '—' }}</span>
              </div>
              <div class="pay-item amount-item">
                <span class="pay-label">订单总金额</span>
                <span class="pay-value amount-total">{{ totalAmount.toFixed(2) }}</span>
              </div>
              <div class="pay-item amount-item">
                <span class="pay-label">已支付金额</span>
                <span class="pay-value amount-paid">{{ (Number(orderDetail?.paidAmount) || 0).toFixed(2) }}</span>
              </div>
              <div class="pay-item amount-item highlight">
                <span class="pay-label">待支付金额</span>
                <span class="pay-value amount-unpaid">{{ unpaidAmountComp.toFixed(2) }}</span>
              </div>
            </div>
          </div>

          <div class="payment-form-section">
            <PaymentForm />
          </div>
        </div>
      </TabPane>
    </Tabs>
  </Drawer>
</template>

<style>
/* ========== 销售模式选择器 ========== */
.sales-mode-selector {
  display: flex;
  align-items: stretch;
  gap: 0;
  margin-bottom: 20px;
  padding: 4px;
  background: hsl(var(--muted));
  border-radius: 14px;
  border: 1px solid hsl(var(--border));
}

.mode-card {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 18px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  border: 2px solid transparent;
  background: transparent;
}

.mode-card:hover {
  background: hsl(var(--background) / 0.7);
}

.mode-card--active {
  background: hsl(var(--background));
  border-color: hsl(var(--primary));
  box-shadow: 0 4px 14px hsl(var(--primary) / 0.12);
}

.mode-card--active:hover {
  background: hsl(var(--background));
}

.mode-card__icon {
  width: 46px;
  height: 46px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.3s ease;
}

.mode-card__icon--standard {
  background: hsl(217 92% 90%);
  color: hsl(217 91% 45%);
}

.mode-card__icon--simple {
  background: hsl(38 92% 86%);
  color: hsl(32 95% 44%);
}

.mode-card--active .mode-card__icon--standard {
  background: linear-gradient(135deg, hsl(217 91% 60%), hsl(217 91% 45%));
  color: #fff;
  box-shadow: 0 4px 12px hsl(217 91% 60% / 0.35);
}

.mode-card--active .mode-card__icon--simple {
  background: linear-gradient(135deg, hsl(38 92% 50%), hsl(32 95% 44%));
  color: #fff;
  box-shadow: 0 4px 12px hsl(38 92% 50% / 0.35);
}

.mode-card__content {
  flex: 1;
  min-width: 0;
}

.mode-card__title {
  font-size: 15px;
  font-weight: 600;
  color: hsl(var(--foreground));
  line-height: 1.4;
}

.mode-card__desc {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-top: 3px;
  line-height: 1.4;
}

.mode-card__check {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: hsl(var(--primary));
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  animation: checkPop 0.3s ease;
}

.mode-card--active .mode-card__check {
  background: hsl(var(--primary));
}

@keyframes checkPop {
  0% { transform: scale(0); }
  50% { transform: scale(1.2); }
  100% { transform: scale(1); }
}

.mode-card__divider {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 8px;
  color: hsl(var(--muted-foreground));
  font-size: 13px;
  font-weight: 500;
  flex-shrink: 0;
}

/* ========== 来源选择行 ========== */
.source-select-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding: 0 4px;
}

.source-select-label {
  flex-shrink: 0;
  width: 82px;
  font-size: 14px;
  color: hsl(var(--muted-foreground));
  text-align: right;
}

/* ========== 来源选择字段（表单样式） ========== */
.source-field-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 18px;
}

.source-field-label {
  flex-shrink: 0;
  width: 80px;
  text-align: right;
  font-size: 14px;
  color: hsl(var(--foreground));
  line-height: 32px;
  white-space: nowrap;
}

.source-field-required {
  color: hsl(0 84% 60%);
  margin-right: 4px;
}

.source-field-control {
  flex: 1;
  min-width: 0;
}

.source-field-input {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 11px;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: hsl(var(--background));
}

.source-field-input:hover {
  border-color: hsl(var(--primary) / 0.6);
}

.source-field-input--filled {
  border-color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.06);
}

.source-field-input--filled:hover {
  border-color: hsl(var(--primary));
}

.source-field-value {
  flex: 1;
  font-size: 14px;
  color: hsl(var(--primary));
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-field-placeholder {
  flex: 1;
  font-size: 14px;
  color: hsl(var(--muted-foreground) / 0.6);
}

.source-field-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.source-field-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 50%;
  background: hsl(var(--muted-foreground) / 0.15);
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  transition: all 0.2s;
  padding: 0;
}

.source-field-btn:hover {
  background: hsl(0 84% 60%);
  color: #fff;
}

.source-field-arrow {
  flex-shrink: 0;
  color: hsl(var(--muted-foreground) / 0.6);
  display: flex;
  align-items: center;
}

.sale-order-drawer {
  width: 75vw !important;
}

.sale-order-drawer--fullscreen {
  width: 100vw !important;
}

.sale-order-drawer__fs-btn {
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
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  transition: all 0.2s;
}

.sale-order-drawer__fs-btn:hover {
  color: hsl(var(--primary));
  background-color: hsl(var(--accent));
}

/* ========== 财务信息样式 ========== */
.financial-section {
  padding: 8px 4px;
}

.financial-row {
  display: flex;
  align-items: stretch;
  gap: 16px;
  margin-bottom: 20px;
}

.financial-card {
  flex: 1;
  border-radius: 12px;
  padding: 20px;
  position: relative;
  overflow: hidden;
  transition: transform 0.25s ease, box-shadow 0.25s ease;
}

.financial-card:hover {
  transform: translateY(-2px);
}

.buyer-card {
  background: hsl(210 100% 96% / 0.5);
  border: 1px solid hsl(210 100% 80% / 0.4);
  box-shadow: 0 2px 8px hsl(210 100% 50% / 0.08);
}

.buyer-card:hover {
  box-shadow: 0 6px 20px hsl(210 100% 50% / 0.15);
}

.seller-card {
  background: hsl(120 60% 95% / 0.5);
  border: 1px solid hsl(120 60% 70% / 0.4);
  box-shadow: 0 2px 8px hsl(120 60% 40% / 0.08);
}

.seller-card:hover {
  box-shadow: 0 6px 20px hsl(120 60% 40% / 0.15);
}

.financial-card-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 18px;
  padding-bottom: 16px;
  border-bottom: 1px dashed hsl(var(--border));
}

.financial-account-select {
  margin-left: auto;
  width: 220px;
}

.financial-card-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.buyer-icon {
  background: linear-gradient(135deg, hsl(210 100% 45%), hsl(210 100% 55%));
  color: #fff;
  box-shadow: 0 4px 12px hsl(210 100% 50% / 0.3);
}

.seller-icon {
  background: linear-gradient(135deg, hsl(120 60% 35%), hsl(120 60% 45%));
  color: #fff;
  box-shadow: 0 4px 12px hsl(120 60% 40% / 0.3);
}

.financial-card-title .title-main {
  font-size: 16px;
  font-weight: 600;
  color: hsl(var(--foreground));
  line-height: 1.3;
}

.financial-card-title .title-sub {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  margin-top: 2px;
}

.financial-card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.fin-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.fin-label {
  flex-shrink: 0;
  width: 80px;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  line-height: 1.6;
  padding-top: 5px;
}

.fin-required {
  color: hsl(0 84% 60%);
  margin-right: 2px;
  font-weight: 600;
}

.fin-value {
  flex: 1;
  font-size: 13px;
  color: hsl(var(--foreground));
  font-weight: 500;
  line-height: 1.6;
  word-break: break-all;
}

.fin-value.mono {
  font-family: 'SF Mono', 'Menlo', 'Monaco', 'Consolas', monospace;
  font-size: 13px;
  letter-spacing: 0.3px;
}

/* 无边框可编辑 Input */
.fin-input.ant-input {
  flex: 1;
  padding: 4px 8px;
  margin: 0 -8px;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  transition: all 0.2s;
}

.fin-input.ant-input:hover {
  background: hsl(var(--accent) / 0.5);
  border-color: hsl(var(--border));
}

.fin-input.ant-input:focus {
  background: hsl(var(--background));
  border-color: hsl(var(--primary));
  box-shadow: 0 0 0 2px hsl(var(--primary) / 0.1);
}

.fin-input.mono.ant-input {
  font-family: 'SF Mono', 'Menlo', 'Monaco', 'Consolas', monospace;
  letter-spacing: 0.3px;
}

.fin-input.ant-input::placeholder {
  color: hsl(var(--muted-foreground) / 0.6);
  font-weight: 400;
}

.financial-arrow {
  display: flex;
  align-items: center;
  justify-content: center;
  color: hsl(var(--muted-foreground) / 0.6);
  padding: 0 4px;
  flex-shrink: 0;
  animation: arrowPulse 2s ease-in-out infinite;
}

@keyframes arrowPulse {
  0%, 100% { opacity: 0.5; transform: translateX(0); }
  50% { opacity: 1; transform: translateX(4px); }
}

/* 支付汇总卡片 */
.payment-summary-card {
  background: hsl(var(--background));
  border-radius: 12px;
  padding: 20px 24px;
  border: 1px solid hsl(var(--border) / 0.5);
  box-shadow: 0 2px 8px hsl(0 0% 0% / 0.04);
  margin-bottom: 20px;
}

.payment-summary-title {
  font-size: 15px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 16px;
  padding-left: 10px;
  border-left: 3px solid hsl(var(--primary));
}

.payment-summary-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
}

.pay-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 16px;
  background: hsl(var(--muted));
  border-radius: 8px;
  transition: background 0.2s;
}

.pay-item:hover {
  background: hsl(var(--accent));
}

.pay-item.amount-item {
  background: hsl(var(--muted));
}

.pay-item.highlight {
  background: hsl(30 100% 92% / 0.5);
  border: 1px solid hsl(30 100% 75% / 0.4);
}

.pay-label {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.pay-value {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.amount-total {
  font-size: 16px;
  color: hsl(var(--foreground));
}

.amount-paid {
  font-size: 16px;
  color: hsl(120 60% 40%);
}

.amount-unpaid {
  font-size: 18px;
  color: hsl(30 90% 50%);
  font-weight: 700;
}

.payment-form-section {
  background: hsl(var(--background));
  border-radius: 12px;
  padding: 20px 24px;
  border: 1px solid hsl(var(--border) / 0.5);
  box-shadow: 0 2px 8px hsl(0 0% 0% / 0.04);
}

@media (max-width: 1200px) {
  .financial-row {
    flex-direction: column;
  }
  .financial-arrow {
    transform: rotate(90deg);
    padding: 4px 0;
  }
  .payment-summary-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 768px) {
  .payment-summary-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
