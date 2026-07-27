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
// 客户字段锁定：订单有关联商机或报价单（来源于上游）时锁定客户字段
const isCustomerLocked = ref(false);
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

function onQuotationSelect(item: any) {
  quotationInfo.value = { id: item.id, title: item.title, quotationNo: item.quotationNo };
  basicFormApi.setValues({ quotationId: item.id });
  // 自动填充客户信息
  if (item.customerName) {
    basicFormApi.setValues({ customerName: item.customerName });
  }
  // 重新加载买方财务信息
  if (item.customerId) {
    void loadBuyerFinancialInfo(item.customerId, item.customerName);
  }
}

function clearQuotation() {
  quotationInfo.value = {};
  basicFormApi.setValues({ quotationId: undefined });
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
    componentProps: () => ({
      placeholder: '请输入客户名称',
      disabled: isCustomerLocked.value,
    }),
  },
  {
    component: 'Input',
    fieldName: 'contactName',
    label: '联系人姓名',
    componentProps: { placeholder: '请输入联系人姓名' },
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
  },
  {
    component: 'DatePicker',
    fieldName: 'paymentDueDate',
    label: '付款截止日期',
    componentProps: { placeholder: '请选择', style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
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
    // 客户字段锁定：来源于上游（有关联商机或报价单）时锁定
    isCustomerLocked.value = !!data.opportunityId || !!data.quotationId;
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

    // 销售流程模式校验：未关联报价单时，校验企业配置允许跳过 + 必填商机
    if (!quotationInfo.value.id && !basicValues.opportunityId) {
      if (flowMode.value === 'A') {
        message.error('当前为标准流程模式，订单必须关联报价单');
        activeTab.value = 'basic';
        return;
      }
      if (flowMode.value === 'B') {
        message.error('当前为简易流程模式，订单必须关联商机（请通过商机"转订单"入口创建）');
        activeTab.value = 'basic';
        return;
      }
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
      isCustomerLocked.value = false;
      activeTab.value = 'basic';
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
        // 从报价单创建订单时锁定客户字段
        isCustomerLocked.value = true;
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
        <BasicForm />
        <!-- 报价单选择（模式 A 或 both 时显示，模式 B 隐藏） -->
        <div v-if="showQuotationSelect" class="flex items-center gap-2 mt-2 px-1">
          <span class="text-sm text-gray-500 shrink-0" style="width: 82px">报价单：</span>
          <div class="flex-1">
            <a
              v-if="quotationInfo.id"
              class="text-blue-600 cursor-pointer"
              @click="quotationModalVisible = true"
            >
              {{ quotationInfo.title || quotationInfo.quotationNo || `报价单 #${quotationInfo.id}` }}
            </a>
            <a
              v-else
              class="text-blue-600 cursor-pointer"
              @click="quotationModalVisible = true"
            >
              选择报价单
            </a>
          </div>
          <Button v-if="quotationInfo.id" type="link" size="small" danger @click="clearQuotation">清除</Button>
        </div>
        <!-- 模式 B 提示：未关联商机时给出提示 -->
        <div v-if="flowMode === 'B' && !quotationInfo.id" class="flex items-center gap-2 mt-2 px-1">
          <span class="text-sm text-gray-500 shrink-0" style="width: 82px">关联商机：</span>
          <div class="flex-1">
            <span class="text-sm text-orange-500">
              当前为简易流程模式，请通过商机列表"转订单"入口创建订单，或先选择商机。
            </span>
          </div>
        </div>
        <!-- 合同信息（只读，仅编辑时显示） -->
        <div v-if="contractInfo.id" class="flex items-center gap-2 mt-2 px-1">
          <span class="text-sm text-gray-500 shrink-0" style="width: 82px">关联合同：</span>
          <div class="flex-1">
            <span class="text-sm">{{ contractInfo.title || contractInfo.contractNo || `合同 #${contractInfo.id}` }}</span>
          </div>
        </div>
      </TabPane>
      <TabPane key="items" tab="商品明细">
        <!-- 空状态 -->
        <div v-if="items.length === 0" class="py-12 text-center">
          <div class="mb-4 text-gray-400">暂无商品，请添加产品到订单</div>
          <Button type="primary" @click="openProductModal">添加产品</Button>
        </div>

        <!-- 商品列表 -->
        <template v-else>
          <div class="mb-3 flex justify-between items-center">
            <span class="text-sm text-gray-500">共 {{ items.length }} 项</span>
            <Button type="primary" size="small" @click="openProductModal">继续添加</Button>
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
                  <span class="text-xs text-gray-400">{{ record.productCode || '' }}</span>
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
                  @change="() => updateLineAmount(index)"
                />
              </template>
              <template v-else-if="column.key === 'amount'">
                <span class="font-medium">{{ (record.amount || 0).toFixed(2) }}</span>
              </template>
              <template v-else-if="column.key === 'action'">
                <Button type="link" danger size="small" @click="removeItem(index)">删除</Button>
              </template>
            </template>
          </Table>
        </template>

        <!-- 金额汇总（始终显示） -->
        <div class="mt-4 flex flex-col items-end gap-2 pr-4">
          <div class="flex items-center gap-2">
            <span class="w-32 text-right text-gray-500">商品金额合计：</span>
            <span class="w-32 text-right">{{ productAmount.toFixed(2) }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">整单折扣：</span>
            <InputNumber
              v-model:value="discountAmount"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">运费：</span>
            <InputNumber
              v-model:value="shippingFee"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">税额：</span>
            <InputNumber
              v-model:value="taxAmount"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 text-right text-gray-500">其他费用：</span>
            <InputNumber
              v-model:value="otherFee"
              :min="0"
              :precision="2"
              class="w-32"
            />
          </div>
          <div class="flex items-center gap-2 border-t pt-2">
            <span class="w-24 text-right font-medium">订单总金额：</span>
            <span class="w-32 text-right text-lg font-bold text-red-500">
              {{ totalAmount.toFixed(2) }}
            </span>
          </div>
        </div>

        <!-- 产品选择弹窗 -->
        <ProductSelectModal
          v-model:visible="productModalVisible"
          @select="onProductSelect"
        />

        <!-- 报价单选择弹窗 -->
        <QuotationSelectModal
          v-model:visible="quotationModalVisible"
          @select="onQuotationSelect"
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
                  <span class="fin-label">开户行</span>
                  <Input
                    v-model:value="buyerInfo.bankName"
                    class="fin-input"
                    placeholder="开户行名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label">账户名称</span>
                  <Input
                    v-model:value="buyerInfo.accountName"
                    class="fin-input"
                    placeholder="账户名称"
                    :bordered="false"
                  />
                </div>
                <div class="fin-item">
                  <span class="fin-label">银行账号</span>
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
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}

.sale-order-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
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
  background: linear-gradient(135deg, #f0f7ff 0%, #e6f4ff 100%);
  border: 1px solid #bae0ff;
  box-shadow: 0 2px 8px rgba(24, 144, 255, 0.08);
}

.buyer-card:hover {
  box-shadow: 0 6px 20px rgba(24, 144, 255, 0.15);
}

.seller-card {
  background: linear-gradient(135deg, #f6ffed 0%, #f0ffe8 100%);
  border: 1px solid #b7eb8f;
  box-shadow: 0 2px 8px rgba(82, 196, 26, 0.08);
}

.seller-card:hover {
  box-shadow: 0 6px 20px rgba(82, 196, 26, 0.15);
}

.financial-card-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 18px;
  padding-bottom: 16px;
  border-bottom: 1px dashed rgba(0, 0, 0, 0.08);
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
  background: linear-gradient(135deg, #1890ff, #40a9ff);
  color: #fff;
  box-shadow: 0 4px 12px rgba(24, 144, 255, 0.3);
}

.seller-icon {
  background: linear-gradient(135deg, #52c41a, #73d13d);
  color: #fff;
  box-shadow: 0 4px 12px rgba(82, 196, 26, 0.3);
}

.financial-card-title .title-main {
  font-size: 16px;
  font-weight: 600;
  color: #262626;
  line-height: 1.3;
}

.financial-card-title .title-sub {
  font-size: 12px;
  color: #8c8c8c;
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
  color: #8c8c8c;
  line-height: 1.6;
  padding-top: 5px;
}

.fin-value {
  flex: 1;
  font-size: 13px;
  color: #262626;
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
  color: #262626;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  transition: all 0.2s;
}

.fin-input.ant-input:hover {
  background: rgba(0, 0, 0, 0.03);
  border-color: #d9d9d9;
}

.fin-input.ant-input:focus {
  background: #fff;
  border-color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.1);
}

.fin-input.mono.ant-input {
  font-family: 'SF Mono', 'Menlo', 'Monaco', 'Consolas', monospace;
  letter-spacing: 0.3px;
}

.fin-input.ant-input::placeholder {
  color: #bfbfbf;
  font-weight: 400;
}

.financial-arrow {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #bfbfbf;
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
  background: #fff;
  border-radius: 12px;
  padding: 20px 24px;
  border: 1px solid #f0f0f0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  margin-bottom: 20px;
}

.payment-summary-title {
  font-size: 15px;
  font-weight: 600;
  color: #262626;
  margin-bottom: 16px;
  padding-left: 10px;
  border-left: 3px solid #1890ff;
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
  background: #fafafa;
  border-radius: 8px;
  transition: background 0.2s;
}

.pay-item:hover {
  background: #f5f5f5;
}

.pay-item.amount-item {
  background: linear-gradient(135deg, #fafafa 0%, #f5f5f5 100%);
}

.pay-item.highlight {
  background: linear-gradient(135deg, #fff2e8 0%, #ffe7ba 100%);
  border: 1px solid #ffd591;
}

.pay-label {
  font-size: 12px;
  color: #8c8c8c;
}

.pay-value {
  font-size: 14px;
  font-weight: 600;
  color: #262626;
}

.amount-total {
  font-size: 16px;
  color: #262626;
}

.amount-paid {
  font-size: 16px;
  color: #52c41a;
}

.amount-unpaid {
  font-size: 18px;
  color: #fa8c16;
  font-weight: 700;
}

.payment-form-section {
  background: #fff;
  border-radius: 12px;
  padding: 20px 24px;
  border: 1px solid #f0f0f0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
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
