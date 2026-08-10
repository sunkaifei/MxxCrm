<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, nextTick, ref } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import {
  Button,
  InputNumber,
  Table,
  Tabs,
  TabPane,
  Timeline,
  TimelineItem,
  Tooltip,
  message,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  createQuotationApi,
  getQuotationInfoApi,
  updateQuotationApi,
  getOpportunityInfoApi,
  getWarehouseListApi,
} from '#/api';
import {
  getCustomerListApi,
  getContactListApi,
} from '#/api';
import ProductSelectModal from '../components/ProductSelectModal.vue';
import OpportunitySelectModal from '../../crm/components/OpportunitySelectModal.vue';

const drawerData = ref<{ create?: boolean; row?: any; needRefresh?: boolean; fromOpportunity?: any }>({ create: true });
const isEdit = computed(() => !drawerData.value.create);
const isReadOnly = ref(false);
const activeTab = ref('basic');
const isFullscreen = ref(false);

const drawerClass = computed(() => [
  'sale-quotation-drawer',
  { 'sale-quotation-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// ============ 下拉搜索数据 ============
const customerOptions = ref<any[]>([]);
const contactOptions = ref<any[]>([]);
const customerLoading = ref(false);
const contactLoading = ref(false);

async function searchCustomers(keyword: string) {
  customerLoading.value = true;
  try {
    const res = await getCustomerListApi({ page: 1, pageSize: 50, keywords: keyword });
    customerOptions.value = (res?.list || res?.items || res || []).map((c: any) => ({
      label: c.customerName || c.name || c.companyName,
      value: c.id,
      raw: c,
    }));
  } finally {
    customerLoading.value = false;
  }
}

async function searchContacts(keyword: string) {
  // 联系人严格按当前绑定客户过滤，无客户则不加载
  const custId = boundCustomerId.value;
  if (!custId) {
    contactOptions.value = [];
    return;
  }
  contactLoading.value = true;
  try {
    const params: any = { page: 1, pageSize: 50, customerId: custId };
    if (keyword) params.keywords = keyword;
    const res = await getContactListApi(params);
    contactOptions.value = (res?.list || res?.items || res || []).map((c: any) => ({
      label: c.contactName || c.name,
      value: c.id,
      raw: c,
    }));
  } finally {
    contactLoading.value = false;
  }
}

// 下拉打开时自动加载当前客户下的联系人（避免显示旧数据或其他企业联系人）
async function onContactDropdownVisibleChange(open: boolean) {
  if (!open) return;
  if (!boundCustomerId.value) {
    contactOptions.value = [];
    return;
  }
  // 已有选项且全部属于当前客户时不重复加载
  await searchContacts('');
}

// 按指定客户ID加载联系人选项（商机选中后自动调用）
async function loadContactsByCustomer(customerId: number | undefined) {
  if (!customerId) {
    contactOptions.value = [];
    return;
  }
  contactLoading.value = true;
  try {
    const res = await getContactListApi({ page: 1, pageSize: 50, customerId });
    contactOptions.value = (res?.list || res?.items || res || []).map((c: any) => ({
      label: c.contactName || c.name,
      value: c.id,
      raw: c,
    }));
  } finally {
    contactLoading.value = false;
  }
}

// ============ 发货仓库（可选，选品时参考库存） ============
const warehouseOptions = ref<any[]>([]);
// 选品弹窗使用的仓库ID（从表单读取，可能为空）
const currentWarehouseId = ref<number | undefined>(undefined);

async function loadWarehouseList() {
  try {
    const res: any = await getWarehouseListApi({ page: 1, pageSize: 100 });
    const data = res?.data ?? res ?? {};
    const rawList = data.list || data.items || data.rows || [];
    warehouseOptions.value = rawList.map((w: any) => ({
      label: w.warehouseName || w.name,
      value: w.id,
    }));
  } catch (e) {
    console.error('[报价单] 加载仓库列表失败:', e);
    warehouseOptions.value = [];
  }
}

// ============ 产品选择弹窗 ============
const productModalVisible = ref(false);

// ============ 商机选择弹窗 ============
const opportunitySelectVisible = ref(false);
// 当前选中的商机ID（表单只展示标题，ID单独存储用于提交）
const selectedOpportunityId = ref<number | undefined>(undefined);

function openOpportunitySelect() {
  if (isReadOnly.value) return;
  opportunitySelectVisible.value = true;
}

// 商机弹窗选择回调（row 为列表完整数据）
async function handleSelectOpportunity(row: any) {
  if (!row) return;
  opportunitySelectVisible.value = false;
  const oppId = row.id ? Number(row.id) : undefined;
  const oppTitle = row.title || row.opportunityName || row.name || '';

  selectedOpportunityId.value = oppId;
  await nextTick();
  basicFormApi.setValues({ opportunityTitle: oppTitle });

  // 复用 onOpportunityChange 逻辑：通过详情接口获取客户和联系人
  await onOpportunityChange(oppId, { raw: row });
}

async function openProductModal() {
  // 选品前读取当前选中的发货仓库（可能为空），用于弹窗内展示参考库存
  const values = await basicFormApi.getValues();
  currentWarehouseId.value = values.warehouseId ? Number(values.warehouseId) : undefined;
  productModalVisible.value = true;
}

function onProductSelect(items: any[]) {
  items.forEach((item) => {
    quotationItems.value.push({
      key: ++itemKeyCounter,
      productId: item.productId,
      productName: item.productName,
      productCode: item.productCode,
      spec: item.spec || '',
      unit: item.unit || '',
      weight: Number(item.weight || 0),
      unitPrice: item.unitPrice || 0,
      quantity: 1,
      discountRate: 0,
      discountAmount: 0,
      taxRate: 0,
      taxAmount: 0,
      subtotal: 0,
    });
  });
  recalculateAll();
}

function onCustomerChange(_val: any, option: any) {
  if (option?.raw) {
    const patch: any = {
      customerName: option.raw.customerName || option.raw.name || '',
    };
    // 自动带出客户负责人作为报价单负责人
    if (option.raw.assignedTo) {
      patch.ownerUserId = option.raw.assignedTo;
    }
    basicFormApi.setValues(patch);
  }
}

function onContactChange(_val: any, option: any) {
  if (option?.raw) {
    basicFormApi.setValues({
      contactName: option.raw.contactName || option.raw.name || '',
    });
  }
}

// 当前绑定客户ID（用于锁定客户字段和加载联系人）
const boundCustomerId = ref<number | undefined>(undefined);

async function onOpportunityChange(val: any, option: any) {
  if (!val) {
    // 清空商机时清空继承字段
    boundCustomerId.value = undefined;
    selectedOpportunityId.value = undefined;
    basicFormApi.setValues({
      opportunityTitle: '',
      customerId: undefined,
      customerName: '',
      contactId: undefined,
      contactName: '',
      ownerUserId: undefined,
    });
    contactOptions.value = [];
    return;
  }

  // 调用详情接口获取客户和联系人信息
  try {
    const res: any = await getOpportunityInfoApi(val);
    const detail = res?.data ?? res;
    const custId = detail?.customerId ? Number(detail.customerId) : undefined;
    const custName = detail?.customerName || '';
    const contId = detail?.contactId ? Number(detail.contactId) : undefined;
    const contName = detail?.contactName || '';
    // 商机负责人作为报价单负责人
    const ownerId = detail?.assignedTo ? Number(detail.assignedTo) : undefined;

    boundCustomerId.value = custId;

    // 预填客户下拉并锁定
    if (custId) {
      customerOptions.value = [{ label: custName, value: custId }];
    }
    // 加载该客户下的联系人选项
    await loadContactsByCustomer(custId);
    // 联系人若存在于选项中则预选，否则补充一条
    if (contId) {
      const exists = contactOptions.value.some((o) => o.value === contId);
      if (!exists && contName) {
        contactOptions.value = [{ label: contName, value: contId }, ...contactOptions.value];
      }
    }

    await nextTick();
    basicFormApi.setValues({
      customerId: custId,
      customerName: custName,
      contactId: contId,
      contactName: contName,
      ownerUserId: ownerId,
    });
  } catch (e) {
    console.error('[报价单] 加载商机详情失败:', e);
  }
}

// ============ 商品明细 ============
interface QuotationItem {
  key: number;
  productId?: number;
  productName?: string;
  productCode?: string;
  spec?: string;
  unit: string;
  weight: number;
  quantity: number;
  unitPrice: number;
  discountRate: number;
  discountAmount: number;
  taxRate: number;
  taxAmount: number;
  subtotal: number;
  remark?: string;
}

let itemKeyCounter = 0;
const quotationItems = ref<QuotationItem[]>([]);

function removeItem(index: number) {
  quotationItems.value.splice(index, 1);
  recalculateAll();
}

function recalculateItem(index: number) {
  const item = quotationItems.value[index];
  if (!item) return;
  const gross = item.quantity * item.unitPrice;
  item.discountAmount = Number((gross * item.discountRate / 100).toFixed(2));
  const afterDiscount = gross - item.discountAmount;
  item.taxAmount = Number((afterDiscount * item.taxRate / 100).toFixed(2));
  item.subtotal = Number((afterDiscount + item.taxAmount).toFixed(2));
}

function recalculateAll() {
  quotationItems.value.forEach((_, i) => recalculateItem(i));
}

// 整体折扣率（默认100%）
const overallDiscountRate = ref(100);

const summary = computed(() => {
  const total = quotationItems.value.reduce((sum, item) => sum + item.quantity * item.unitPrice, 0);
  const itemDiscount = quotationItems.value.reduce((sum, item) => sum + item.discountAmount, 0);
  const afterItemDiscount = total - itemDiscount;
  const overallDiscount = Number((afterItemDiscount * (1 - overallDiscountRate.value / 100)).toFixed(2));
  const afterDiscount = afterItemDiscount - overallDiscount;
  const tax = quotationItems.value.reduce((sum, item) => sum + item.taxAmount, 0);
  const grand = afterDiscount + tax;
  const totalWeight = quotationItems.value.reduce((sum, item) => sum + item.weight * item.quantity, 0);
  return { total, itemDiscount, overallDiscount, tax, grand, totalWeight };
});

const itemColumns = [
  { title: '#', width: 45, key: 'seq', customRender: ({ index }: any) => index + 1, align: 'center' as const },
  { title: '产品信息', dataIndex: 'productName', key: 'product', width: 240 },
  { title: '规格', dataIndex: 'spec', key: 'spec', width: 110 },
  { title: '单位', dataIndex: 'unit', key: 'unit', width: 55, align: 'center' as const },
  { title: '单价(只读)', dataIndex: 'unitPrice', key: 'unitPrice', width: 95, align: 'right' as const },
  { title: '数量', dataIndex: 'quantity', key: 'quantity', width: 80 },
  { title: '单重(kg)', dataIndex: 'weight', key: 'weight', width: 80, align: 'right' as const },
  { title: '折扣(%)', dataIndex: 'discountRate', key: 'discountRate', width: 75 },
  { title: '折扣额', dataIndex: 'discountAmount', key: 'discountAmount', width: 85, align: 'right' as const },
  { title: '小计', dataIndex: 'subtotal', key: 'subtotal', width: 105, align: 'right' as const },
  { title: '操作', key: 'action', width: 55, align: 'center' as const },
];

// ============ 审批记录 ============
const approvalList = ref<any[]>([]);

const approvalTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '提交审批', color: 'blue' },
  2: { label: '审批通过', color: 'green' },
  3: { label: '审批驳回', color: 'red' },
  4: { label: '修改重报', color: 'orange' },
};

// ============ 表单定义 ============
const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '报价单标题',
    rules: 'required',
    componentProps: { placeholder: '如：ABC集团年度续约报价' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'opportunityTitle',
    label: '商机',
    rules: 'required',
    componentProps: () => ({
      placeholder: '点击右侧按钮选择商机（必填）',
      readOnly: true,
      disabled: isReadOnly.value,
      // 点击输入框也可以打开弹窗
      onClick: () => {
        if (!isReadOnly.value) openOpportunitySelect();
      },
      suffix: '点击选择',
    }),
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'customerId',
    label: '客户',
    rules: 'required',
    componentProps: () => ({
      showSearch: true,
      filterOption: false,
      placeholder: '由商机自动带出，不可修改',
      options: customerOptions,
      loading: customerLoading,
      onSearch: searchCustomers,
      onChange: onCustomerChange,
      // 客户由商机继承，始终锁定不可修改
      disabled: true,
    }),
  },
  {
    component: 'Select',
    fieldName: 'contactId',
    label: '联系人',
    rules: 'required',
    componentProps: () => ({
      showSearch: true,
      filterOption: false,
      placeholder: '请选择当前客户下的联系人',
      options: contactOptions,
      loading: contactLoading,
      onSearch: searchContacts,
      onChange: onContactChange,
      onDropdownVisibleChange: onContactDropdownVisibleChange,
      disabled: isReadOnly.value || !boundCustomerId.value,
    }),
  },
  {
    component: 'Select',
    fieldName: 'warehouseId',
    label: '发货仓库',
    componentProps: () => ({
      placeholder: '可选，选择后选品时展示库存',
      allowClear: true,
      options: warehouseOptions,
      disabled: isReadOnly.value,
    }),
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { options: currencyOptions },
  },
  {
    component: 'DatePicker',
    fieldName: 'quotationDate',
    label: '报价日期',
    componentProps: { style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'DatePicker',
    fieldName: 'validUntil',
    label: '有效期至',
    componentProps: { style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'Input',
    fieldName: 'remark',
    label: '备注',
    componentProps: { placeholder: '备注信息' },
    wrapperClass: 'col-span-2',
  },
];

const tradeFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'paymentTerms',
    label: '付款条件',
    componentProps: { placeholder: '如：T/T 30% advance, 70% before shipment' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'deliveryTerms',
    label: '交货条款',
    componentProps: { placeholder: '如：FOB Shenzhen' },
  },
  {
    component: 'DatePicker',
    fieldName: 'deliveryDate',
    label: '交货日期',
    componentProps: { style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
  },
  {
    component: 'Input',
    fieldName: 'portOfLoading',
    label: '装运港',
    componentProps: { placeholder: '如：深圳' },
  },
  {
    component: 'Input',
    fieldName: 'portOfDestination',
    label: '目的港',
    componentProps: { placeholder: '如：洛杉矶' },
  },
  {
    component: 'Textarea',
    fieldName: 'bankInfo',
    label: '银行信息',
    componentProps: { placeholder: 'Beneficiary / Account No / SWIFT / Bank', rows: 3 },
    wrapperClass: 'col-span-2',
  },
];

const [BasicForm, basicFormApi] = useVbenForm({
  schema: basicFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [TradeForm, tradeFormApi] = useVbenForm({
  schema: tradeFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

// ============ 数据加载 ============
// 从商机初始化报价单表单（一键转报价单）
// 入参 opp 可为商机列表 row 或商机详情 detail
async function initFromOpportunity(opp: any) {
  if (!opp) return;
  const oppId = opp.id ? Number(opp.id) : undefined;
  const oppTitle = opp.title || opp.opportunityName || '';
  selectedOpportunityId.value = oppId;

  // 默认值
  basicFormApi.setValues({
    currency: Number(opp.currency ?? 1),
    quotationDate: new Date().toISOString().slice(0, 10),
    opportunityTitle: oppTitle,
  });

  // 若列表 row 已含客户和联系人信息，直接预填；否则调用详情接口获取
  let custId = opp.customerId ? Number(opp.customerId) : undefined;
  let custName = opp.customerName || '';
  let contId = opp.contactId ? Number(opp.contactId) : undefined;
  let contName = opp.contactName || '';
  let ownerId = opp.assignedTo ? Number(opp.assignedTo) : undefined;

  if ((!custId || !contId || !ownerId) && oppId) {
    try {
      const res: any = await getOpportunityInfoApi(oppId);
      const detail = res?.data ?? res;
      if (!custId && detail?.customerId) custId = Number(detail.customerId);
      if (!custName) custName = detail?.customerName || '';
      if (!contId && detail?.contactId) contId = Number(detail.contactId);
      if (!contName) contName = detail?.contactName || '';
      if (!ownerId && detail?.assignedTo) ownerId = Number(detail.assignedTo);
    } catch (e) {
      console.error('[报价单] 加载商机详情失败:', e);
    }
  }

  boundCustomerId.value = custId;

  // 预填客户下拉并锁定
  if (custId && custName) {
    customerOptions.value = [{ label: custName, value: custId }];
  }
  // 加载该客户下的联系人选项
  await loadContactsByCustomer(custId);
  // 联系人若不在选项中则补充一条
  if (contId && contName) {
    const exists = contactOptions.value.some((o: any) => o.value === contId);
    if (!exists) {
      contactOptions.value = [{ label: contName, value: contId }, ...contactOptions.value];
    }
  }

  await nextTick();
  basicFormApi.setValues({
    opportunityTitle: oppTitle,
    customerId: custId,
    customerName: custName,
    contactId: contId,
    contactName: contName,
    ownerUserId: ownerId,
  });
}

async function loadDetail(id: number) {
  try {
    const info = await getQuotationInfoApi(id);
    const data = info?.data ?? info;
    // 审批中(approvalStatus=2)时表单只读
    isReadOnly.value = Number(data.approvalStatus) === 2;
    basicFormApi.setState({ commonConfig: { componentProps: { disabled: isReadOnly.value } } });
    tradeFormApi.setState({ commonConfig: { componentProps: { disabled: isReadOnly.value } } });
    const custId = data.customerId ? Number(data.customerId) : undefined;
    const contId = data.contactId ? Number(data.contactId) : undefined;
    const oppId = data.opportunityId ? Number(data.opportunityId) : undefined;

    // 客户由商机继承，绑定客户ID用于锁定和加载联系人
    boundCustomerId.value = custId;
    // 商机ID单独存储（表单只展示标题）
    selectedOpportunityId.value = oppId;

    // 预填客户下拉（必须在setValues之前，否则Select无法匹配label）
    if (custId && data.customerName) {
      customerOptions.value = [{ label: data.customerName, value: custId }];
    }
    // 按当前客户加载联系人选项（编辑模式回显）
    await loadContactsByCustomer(custId);
    // 联系人若不在选项中则补充一条以保证回显
    if (contId && data.contactName) {
      const exists = contactOptions.value.some((o: any) => o.value === contId);
      if (!exists) {
        contactOptions.value = [{ label: data.contactName, value: contId }, ...contactOptions.value];
      }
    }
    // 等待下拉选项渲染完成后再设置表单值，确保Select能匹配到label
    await nextTick();
    // 负责人：新建时由客户/商机的 assignedTo 带出；编辑时回显已有负责人，状态由审批流控制，不在表单中展示
    basicFormApi.setValues({
      title: data.title,
      customerId: custId,
      contactId: contId,
      customerName: data.customerName,
      contactName: data.contactName,
      opportunityTitle: data.opportunityTitle,
      currency: Number(data.currency ?? 1),
      quotationDate: data.quotationDate,
      validUntil: data.validUntil,
      warehouseId: data.warehouseId ? Number(data.warehouseId) : undefined,
      remark: data.remark,
      ownerUserId: data.ownerUserId != null ? Number(data.ownerUserId) : undefined,
    });
    tradeFormApi.setValues({
      paymentTerms: data.paymentTerms,
      deliveryTerms: data.deliveryTerms,
      deliveryDate: data.deliveryDate,
      portOfLoading: data.portOfLoading,
      portOfDestination: data.portOfDestination,
      bankInfo: data.bankInfo,
    });
    // 加载商品明细
    if (data.items && Array.isArray(data.items)) {
      quotationItems.value = data.items.map((item: any) => ({
        key: ++itemKeyCounter,
        productId: Number(item.productId),
        productName: item.productName,
        productCode: item.productCode,
        spec: item.spec,
        unit: item.unit || '',
        weight: Number(item.weight || 0),
        quantity: Number(item.quantity || 1),
        unitPrice: Number(item.unitPrice || 0),
        discountRate: Number(item.discountRate || 0),
        discountAmount: Number(item.discountAmount || 0),
        taxRate: Number(item.taxRate || 0),
        taxAmount: Number(item.taxAmount || 0),
        subtotal: Number(item.subtotal || 0),
        remark: item.remark,
      }));
    }
    // 加载审批记录
    if (data.approvals && Array.isArray(data.approvals)) {
      approvalList.value = data.approvals;
    }
  } catch (e) {
    console.error('[报价单] 加载详情失败:', e);
  }
}

// ============ 提交 ============
async function handleSubmit() {
  console.log('[报价单提交] ========== 开始提交流程 ==========');
  console.log('[报价单提交] 1. 开始验证基本表单...');
  let validResult;
  try {
    validResult = await basicFormApi.validate();
    console.log('[报价单提交] 2. 基本表单验证结果:', validResult);
  } catch (e) {
    console.error('[报价单提交] 2. 基本表单验证异常:', e);
    activeTab.value = 'basic';
    message.warning('请完善基本信息');
    return false;
  }
  if (!validResult?.valid) {
    console.warn('[报价单提交] 2.1 基本表单验证失败，valid:', validResult?.valid);
    activeTab.value = 'basic';
    message.warning('请完善必填项');
    return false;
  }
  console.log('[报价单提交] 3. 检查商品明细数量:', quotationItems.value.length);
  if (quotationItems.value.length === 0) {
    console.warn('[报价单提交] 3.1 商品明细为空');
    activeTab.value = 'items';
    message.warning('请添加至少一个产品');
    return false;
  }
  // 校验每行必须选择产品（productId 不能为空）
  const emptyProductIndex = quotationItems.value.findIndex((it: any) => !it.productId);
  if (emptyProductIndex >= 0) {
    activeTab.value = 'items';
    message.warning(`第 ${emptyProductIndex + 1} 行未选择产品，请选择产品或删除该行`);
    return false;
  }
  console.log('[报价单提交] 4. 获取表单值...');
  let values, tradeValues;
  try {
    values = await basicFormApi.getValues();
    console.log('[报价单提交] 4.1 基本表单值:', values);
  } catch (e) {
    console.error('[报价单提交] 4.1 获取基本表单值失败:', e);
    message.error('获取表单数据失败');
    return false;
  }
  try {
    tradeValues = await Promise.race([
      tradeFormApi.getValues(),
      new Promise((resolve) => setTimeout(() => resolve({}), 300)),
    ]);
    console.log('[报价单提交] 4.2 交易条款值:', tradeValues);
  } catch (e) {
    console.warn('[报价单提交] 4.2 获取交易条款值失败（可能未切换到交易条款Tab）:', e);
    tradeValues = {};
  }
  console.log('[报价单提交] 4.3 商品明细:', quotationItems.value);
  
  const toNumber = (v: any) => (v === null || v === undefined || v === '') ? undefined : Number(v);

  const findOptionLabel = (opts: any[], val: any) => {
    const found = opts.find((o: any) => String(o.value) === String(val));
    return found?.label || '';
  };

  const custId = toNumber(values.customerId);
  const contId = toNumber(values.contactId);
  // 商机ID从独立ref获取（表单只展示标题）
  const oppId = selectedOpportunityId.value;

  // 商机必填校验
  if (!oppId) {
    activeTab.value = 'basic';
    message.warning('请选择商机');
    return false;
  }

  let data;
  try {
    data = {
      ...values,
      ...(tradeValues as any),
      customerId: custId,
      contactId: contId,
      opportunityId: oppId,
      currency: toNumber(values.currency),
      customerName: findOptionLabel(customerOptions.value, values.customerId),
      contactName: findOptionLabel(contactOptions.value, values.contactId),
      opportunityTitle: values.opportunityTitle,
      items: quotationItems.value.map((item, index) => ({
        productId: toNumber(item.productId),
        productName: item.productName,
        productCode: item.productCode,
        spec: item.spec,
        unit: item.unit,
        weight: Number(item.weight || 0),
        quantity: Number(item.quantity || 1),
        unitPrice: Number(item.unitPrice || 0),
        discountRate: Number(item.discountRate || 0),
        discountAmount: Number(item.discountAmount || 0),
        taxRate: Number(item.taxRate || 0),
        taxAmount: Number(item.taxAmount || 0),
        subtotal: Number(item.subtotal || 0),
        sort: index,
        remark: item.remark,
      })),
      totalAmount: Number(summary.value.total || 0),
      discountAmount: Number(summary.value.itemDiscount + summary.value.overallDiscount || 0),
      taxAmount: Number(summary.value.tax || 0),
      grandTotal: Number(summary.value.grand || 0),
    };
    console.log('[报价单提交] 5. 构造提交数据:', JSON.stringify(data));
  } catch (e) {
    console.error('[报价单提交] 5. 构造提交数据失败:', e);
    message.error('构造提交数据失败');
    return false;
  }

  console.log('[报价单提交] 6. 调用API，isEdit:', isEdit.value);
  
  drawerApi.setState({ confirmLoading: true });
  try {
    if (isEdit.value) {
      console.log('[报价单提交] 6.1 调用更新API...');
      await updateQuotationApi({ ...data, id: drawerData.value.row.id });
      console.log('[报价单提交] 6.2 更新API调用成功');
      message.success('更新成功');
    } else {
      console.log('[报价单提交] 6.1 调用创建API...');
      await createQuotationApi(data);
      console.log('[报价单提交] 6.2 创建API调用成功');
      message.success('创建成功');
    }
    console.log('[报价单提交] 7. 提交成功！');
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch (e) {
    console.error('[报价单提交] 7. 提交失败:', e);
    console.error('[报价单提交] 7.1 错误详情:', (e as any)?.response?.data || (e as any)?.message || e);
    message.error('操作失败');
  } finally {
    drawerApi.setState({ confirmLoading: false });
  }
  console.log('[报价单提交] ========== 提交流程结束 ==========');
  return false;
}

const [Drawer, drawerApi] = useVbenDrawer({
  async onOpenChange(isOpen) {
    if (!isOpen) return;
    drawerData.value = drawerApi.getData<{ create?: boolean; row?: any; fromOpportunity?: any }>() || { create: true };
    isFullscreen.value = false;
    isReadOnly.value = false;
    boundCustomerId.value = undefined;
    selectedOpportunityId.value = undefined;
    activeTab.value = 'basic';
    quotationItems.value = [];
    approvalList.value = [];
    overallDiscountRate.value = 100;
    customerOptions.value = [];
    contactOptions.value = [];
    basicFormApi.resetForm();
    tradeFormApi.resetForm();
    // 加载发货仓库列表（供选品时参考库存，可选）
    await loadWarehouseList();

    if (!drawerData.value.create && drawerData.value.row?.id) {
      // 编辑模式：加载报价单详情
      loadDetail(drawerData.value.row.id);
    } else if (drawerData.value.fromOpportunity) {
      // 从商机转入：预填商机、客户、联系人
      await initFromOpportunity(drawerData.value.fromOpportunity);
    } else {
      basicFormApi.setValues({
        currency: 1,
        quotationDate: new Date().toISOString().slice(0, 10),
      });
    }
  },
  onConfirm: handleSubmit,
});
</script>

<template>
  <Drawer
    :title="isEdit ? (isReadOnly ? '查看报价单' : '编辑报价单') : '新建报价单'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
    :show-footer="!isReadOnly"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-quotation-drawer__fs-btn"
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

    <Tabs v-model:activeKey="activeTab">
      <TabPane key="basic" tab="基本信息">
        <BasicForm />
      </TabPane>

      <TabPane key="items" tab="商品明细">
        <!-- 空状态 -->
        <div v-if="quotationItems.length === 0" class="py-12 text-center">
          <div class="mb-4 q-text-hint">暂无商品，请添加产品到报价单</div>
          <Button v-if="!isReadOnly" type="primary" @click="openProductModal">添加产品</Button>
        </div>

        <!-- 商品列表（阿里巴巴采购单样式） -->
        <template v-else>
          <div class="mb-3 flex justify-between items-center">
            <span class="text-sm q-text-secondary">共 {{ quotationItems.length }} 项</span>
            <Button v-if="!isReadOnly" type="primary" size="small" @click="openProductModal">继续添加</Button>
          </div>
          <Table
            :columns="itemColumns"
            :data-source="quotationItems"
            :pagination="false"
            size="small"
            :scroll="{ x: 1150 }"
            row-key="key"
            bordered
          >
            <template #bodyCell="{ column, record, index }">
              <template v-if="column.key === 'product'">
                <div class="flex flex-col">
                  <span class="font-medium">{{ record.productName || '-' }}</span>
                  <span class="text-xs q-text-hint">{{ record.productCode || '' }}</span>
                </div>
              </template>
              <template v-else-if="column.key === 'spec'">
                <span class="text-sm">{{ record.spec || '-' }}</span>
              </template>
              <template v-else-if="column.key === 'unit'">
                <span class="text-center">{{ record.unit || '-' }}</span>
              </template>
              <template v-else-if="column.key === 'unitPrice'">
                <span class="text-right q-text-secondary">¥{{ Number(record.unitPrice || 0).toFixed(2) }}</span>
              </template>
              <template v-else-if="column.key === 'weight'">
                <span class="text-right q-text-hint">{{ record.weight != null ? Number(record.weight).toFixed(3) : '-' }}</span>
              </template>
              <template v-else-if="column.key === 'quantity'">
                <InputNumber
                  v-if="!isReadOnly"
                  v-model:value="record.quantity"
                  :min="1"
                  :precision="2"
                  style="width: 80px"
                  size="small"
                  @change="() => recalculateItem(index)"
                />
                <span v-else class="text-right">{{ record.quantity }}</span>
              </template>
              <template v-else-if="column.key === 'discountRate'">
                <InputNumber
                  v-if="!isReadOnly"
                  v-model:value="record.discountRate"
                  :min="0"
                  :max="100"
                  :precision="1"
                  style="width: 70px"
                  size="small"
                  @change="() => recalculateItem(index)"
                />
                <span v-else class="text-right">{{ record.discountRate }}%</span>
              </template>
              <template v-else-if="column.key === 'discountAmount'">
                <span class="text-right q-text-danger">{{ Number(record.discountAmount || 0).toFixed(2) }}</span>
              </template>
              <template v-else-if="column.key === 'subtotal'">
                <span class="text-right font-medium q-text-primary">{{ Number(record.subtotal || 0).toFixed(2) }}</span>
              </template>
              <template v-else-if="column.key === 'action'">
                <Button v-if="!isReadOnly" type="link" danger size="small" @click="removeItem(index)">删除</Button>
                <span v-else class="text-gray-300">-</span>
              </template>
            </template>
          </Table>

          <!-- 金额汇总 -->
          <div class="quotation-summary mt-4 flex justify-end">
            <div class="quotation-summary__card w-80 rounded-lg p-4">
              <div class="quotation-summary__row">
                <span class="quotation-summary__label">商品总额</span>
                <span class="quotation-summary__value">¥ {{ summary.total.toFixed(2) }}</span>
              </div>
              <div class="quotation-summary__row quotation-summary__row--center">
                <span class="quotation-summary__label">整体折扣</span>
                <div class="flex items-center gap-2">
                  <InputNumber
                    v-if="!isReadOnly"
                    v-model:value="overallDiscountRate"
                    :min="0"
                    :max="100"
                    :precision="1"
                    style="width: 80px"
                    size="small"
                  />
                  <span v-else class="text-sm">{{ overallDiscountRate }}%</span>
                  <span class="quotation-summary__hint">%</span>
                  <span class="quotation-summary__value quotation-summary__value--danger w-20 text-right">- ¥ {{ summary.overallDiscount.toFixed(2) }}</span>
                </div>
              </div>
              <div class="quotation-summary__row">
                <span class="quotation-summary__label">税额合计</span>
                <span class="quotation-summary__value quotation-summary__value--warning">+ ¥ {{ summary.tax.toFixed(2) }}</span>
              </div>
              <div class="quotation-summary__divider"></div>
              <div class="quotation-summary__row quotation-summary__row--grand">
                <span class="quotation-summary__grand-label">报价总计</span>
                <span class="quotation-summary__grand-value">¥ {{ summary.grand.toFixed(2) }}</span>
              </div>
              <div class="quotation-summary__row quotation-summary__row--weight">
                <span class="quotation-summary__label">总重量</span>
                <span class="quotation-summary__value font-medium">{{ summary.totalWeight.toFixed(3) }} kg</span>
              </div>
            </div>
          </div>
        </template>

        <!-- 产品选择弹窗 -->
        <ProductSelectModal
          v-model:visible="productModalVisible"
          :warehouse-id="currentWarehouseId"
          @select="onProductSelect"
        />
      </TabPane>

      <TabPane key="approval" tab="审批记录">
        <div v-if="approvalList.length === 0" class="py-8 text-center q-text-hint">
          暂无审批记录
        </div>
        <Timeline v-else>
          <TimelineItem
            v-for="item in approvalList"
            :key="item.id"
            :color="approvalTypeMap[item.approvalType]?.color || 'gray'"
          >
            <div class="flex items-center gap-2">
              <span class="font-medium">
                {{ approvalTypeMap[item.approvalType]?.label || '未知' }}
              </span>
              <span class="text-xs q-text-hint">
                {{ item.createTime }}
              </span>
            </div>
            <div class="mt-1 text-sm q-text-secondary">
              审批人：{{ item.approverName || '-' }}
            </div>
            <div v-if="item.originalAmount" class="text-sm q-text-secondary">
              原报价：¥{{ Number(item.originalAmount).toFixed(2) }}
              <span v-if="item.adjustedAmount">
                → 调整后：¥{{ Number(item.adjustedAmount).toFixed(2) }}
              </span>
            </div>
            <div v-if="item.approvalRemark" class="mt-1 text-sm">
              意见：{{ item.approvalRemark }}
            </div>
          </TimelineItem>
        </Timeline>
      </TabPane>

      <TabPane key="trade" tab="交易条款">
        <TradeForm />
      </TabPane>
    </Tabs>

    <!-- 商机选择弹窗 -->
    <OpportunitySelectModal
      v-model:visible="opportunitySelectVisible"
      @select="handleSelectOpportunity"
    />
  </Drawer>
</template>

<style>
.sale-quotation-drawer {
  width: 75vw !important;
}

.sale-quotation-drawer--fullscreen {
  width: 100vw !important;
}

.sale-quotation-drawer__fs-btn {
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

.sale-quotation-drawer__fs-btn:hover {
  color: hsl(var(--primary));
  background-color: hsl(var(--muted));
}

/* 语义化文本色 —— 自动跟随明/暗主题 */
.q-text-primary   { color: hsl(var(--primary)); }
.q-text-danger    { color: hsl(var(--destructive)); }
.q-text-warning   { color: hsl(var(--warning)); }
.q-text-success   { color: hsl(var(--success)); }
.q-text-secondary { color: hsl(var(--muted-foreground)); }
.q-text-hint      { color: hsl(var(--muted-foreground) / 70%); }

/* ========== 金额汇总卡片 ========== */
.quotation-summary__card {
  background: hsl(var(--muted) / 50%);
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius, 8px);
}

.quotation-summary__row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  line-height: 28px;
}

.quotation-summary__row--center {
  align-items: center;
}

.quotation-summary__label {
  color: hsl(var(--muted-foreground));
}

.quotation-summary__hint {
  color: hsl(var(--muted-foreground) / 60%);
  font-size: 12px;
}

.quotation-summary__value {
  color: hsl(var(--foreground));
  font-variant-numeric: tabular-nums;
}

.quotation-summary__value--danger {
  color: hsl(var(--destructive));
}

.quotation-summary__value--warning {
  color: hsl(var(--warning));
}

.quotation-summary__divider {
  height: 1px;
  margin: 8px 0;
  background: hsl(var(--border));
}

.quotation-summary__row--grand {
  padding: 6px 0 2px;
  align-items: baseline;
}

.quotation-summary__grand-label {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.quotation-summary__grand-value {
  font-size: 22px;
  font-weight: 700;
  color: hsl(var(--primary));
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.5px;
}

.quotation-summary__row--weight {
  margin-top: 2px;
  padding-top: 4px;
  border-top: 1px dashed hsl(var(--border));
}
</style>
