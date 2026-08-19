<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Button,
  DatePicker,
  Input,
  message,
  Modal,
  Popconfirm,
  Select,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';
import dayjs from 'dayjs';

import { useVbenForm } from '#/adapter/form';
import {
  createSupplierApi,
  createSupplierBrandApi,
  deleteSupplierBrandApi,
  getAllBrandsApi,
  getBrandsBySupplierApi,
  getSupplierInfoApi,
  updateSupplierApi,
} from '#/api';
import { $t } from '#/locales';

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
    componentProps: {
      placeholder: '请选择等级',
      options: levelOptions,
      allowClear: true,
    },
  },
  {
    component: 'Select',
    fieldName: 'industry',
    label: '所属行业',
    componentProps: {
      placeholder: '请选择行业',
      options: industryOptions,
      allowClear: true,
      showSearch: true,
      filterOption: true,
    },
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
    componentProps: {
      placeholder: '请选择国家',
      options: countryOptions,
      allowClear: true,
      showSearch: true,
      filterOption: true,
    },
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
    componentProps: {
      placeholder: '请输入详细地址',
      allowClear: true,
      rows: 2,
    },
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
    componentProps: {
      placeholder: '请选择币种',
      options: currencyOptions,
      allowClear: true,
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'creditLimit',
    label: '信用额度',
    componentProps: {
      placeholder: '0.00',
      min: 0,
      precision: 2,
      style: { width: '100%' },
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'creditDays',
    label: '信用天数',
    componentProps: {
      placeholder: '0',
      min: 0,
      precision: 0,
      style: { width: '100%' },
    },
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
    componentProps: {
      placeholder: '请输入税号/统一社会信用代码',
      allowClear: true,
    },
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
      drawerData.value = drawerApi.getData<{
        create: boolean;
        row?: any;
      }>() || { create: true };
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
    const num = (v: any) =>
      v === null || v === undefined ? undefined : Number(v);

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

    // 加载代理品牌
    loadSupplierBrands(id);
  } catch (error) {
    console.error('[供应商] 加载详情失败:', error);
  }
}

// ========== 代理品牌管理 ==========
const supplierBrands = ref<any[]>([]);
const allBrandOptions = ref<Array<{ label: string; value: number }>>([]);

const authorizedOptions = [
  { label: '未授权', value: 0 },
  { label: '已授权', value: 1 },
  { label: '已过期', value: 2 },
];

const authorizedLabelMap: Record<number, string> = {
  0: '未授权',
  1: '已授权',
  2: '已过期',
};

const authorizedColorMap: Record<number, string> = {
  0: 'default',
  1: 'green',
  2: 'red',
};

const brandColumns = [
  {
    title: '品牌名称',
    dataIndex: 'brandName',
    key: 'brandName',
    minWidth: 140,
  },
  {
    title: '授权状态',
    dataIndex: 'isAuthorized',
    key: 'isAuthorized',
    width: 100,
  },
  {
    title: '授权编号',
    dataIndex: 'authorizationNo',
    key: 'authorizationNo',
    width: 140,
  },
  {
    title: '授权开始',
    dataIndex: 'authorizationStart',
    key: 'authorizationStart',
    width: 120,
  },
  {
    title: '授权结束',
    dataIndex: 'authorizationEnd',
    key: 'authorizationEnd',
    width: 120,
  },
  { title: '操作', key: 'action', width: 80 },
];

function getBrandName(brandId: any): string {
  const brand = allBrandOptions.value.find((b) => b.value === Number(brandId));
  return brand?.label || `品牌#${brandId}`;
}

async function loadAllBrands() {
  if (allBrandOptions.value.length > 0) return;
  try {
    const res = await getAllBrandsApi();
    const list = (res as any) || [];
    if (Array.isArray(list)) {
      allBrandOptions.value = list.map((b: any) => ({
        value: Number(b.id),
        label: b.name,
      }));
    }
  } catch {
    // ignore
  }
}

async function loadSupplierBrands(supplierId: number) {
  await loadAllBrands();
  try {
    const res = await getBrandsBySupplierApi({ supplierId });
    const list = Array.isArray(res) ? res : ((res as any)?.records ?? []);
    supplierBrands.value = list.map((item: any) => ({
      ...item,
      brandName: getBrandName(item.brandId),
    }));
  } catch {
    supplierBrands.value = [];
  }
}

// 添加品牌关联
const addBrandVisible = ref(false);
const addBrandLoading = ref(false);
const addBrandForm = ref({
  brandId: undefined as number | undefined,
  isAuthorized: 1,
  authorizationNo: '',
  authorizationStart: undefined as dayjs.Dayjs | undefined,
  authorizationEnd: undefined as dayjs.Dayjs | undefined,
});

function openAddBrandModal() {
  addBrandForm.value = {
    brandId: undefined,
    isAuthorized: 1,
    authorizationNo: '',
    authorizationStart: undefined,
    authorizationEnd: undefined,
  };
  loadAllBrands();
  addBrandVisible.value = true;
}

async function handleAddBrand() {
  if (!addBrandForm.value.brandId) {
    message.warning('请选择品牌');
    return;
  }
  if (!drawerData.value.row?.id) {
    message.warning('请先保存供应商信息');
    return;
  }
  addBrandLoading.value = true;
  try {
    await createSupplierBrandApi({
      supplierId: drawerData.value.row.id,
      brandId: addBrandForm.value.brandId,
      isAuthorized: addBrandForm.value.isAuthorized,
      authorizationNo: addBrandForm.value.authorizationNo || undefined,
      authorizationStart:
        addBrandForm.value.authorizationStart?.format('YYYY-MM-DD'),
      authorizationEnd:
        addBrandForm.value.authorizationEnd?.format('YYYY-MM-DD'),
    });
    message.success('添加品牌关联成功');
    addBrandVisible.value = false;
    loadSupplierBrands(drawerData.value.row.id);
  } finally {
    addBrandLoading.value = false;
  }
}

async function handleDeleteBrand(record: any) {
  try {
    await deleteSupplierBrandApi([record.id]);
    message.success('已删除品牌关联');
    if (drawerData.value.row?.id) {
      loadSupplierBrands(drawerData.value.row.id);
    }
  } catch {
    // ignore
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
        <button
          type="button"
          class="supplier-drawer__fs-btn"
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
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
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

      <template v-if="!drawerData.create">
        <div class="supplier-brand-section">
          <div class="flex items-center justify-between mb-3">
            <h4 class="text-base font-semibold text-gray-800 m-0">代理品牌</h4>
            <Button type="primary" size="small" @click="openAddBrandModal">
              添加品牌
            </Button>
          </div>
          <Table
            :columns="brandColumns"
            :data-source="supplierBrands"
            :pagination="false"
            :row-key="(record: any) => record.id"
            bordered
            size="small"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'isAuthorized'">
                <Tag
                  :color="authorizedColorMap[record.isAuthorized] || 'default'"
                >
                  {{ authorizedLabelMap[record.isAuthorized] || '未知' }}
                </Tag>
              </template>
              <template v-if="column.key === 'action'">
                <Popconfirm
                  title="确认删除该品牌关联？"
                  ok-text="确定"
                  cancel-text="取消"
                  @confirm="() => handleDeleteBrand(record)"
                >
                  <Button type="link" danger size="small">删除</Button>
                </Popconfirm>
              </template>
            </template>
          </Table>
        </div>
      </template>
    </div>

    <Modal
      v-model:open="addBrandVisible"
      title="添加代理品牌"
      :confirm-loading="addBrandLoading"
      ok-text="确定"
      cancel-text="取消"
      @ok="handleAddBrand"
    >
      <div class="py-2 space-y-4">
        <div class="flex items-center">
          <label class="w-28 text-right pr-3 text-sm text-gray-700">品牌</label>
          <Select
            v-model:value="addBrandForm.brandId"
            placeholder="请选择品牌"
            :options="allBrandOptions"
            show-search
            :filter-option="
              (input: string, option: any) =>
                option.label?.toLowerCase().includes(input.toLowerCase())
            "
            style="flex: 1"
          />
        </div>
        <div class="flex items-center">
          <label class="w-28 text-right pr-3 text-sm text-gray-700"
            >授权状态</label
          >
          <Select
            v-model:value="addBrandForm.isAuthorized"
            :options="authorizedOptions"
            style="flex: 1"
          />
        </div>
        <div class="flex items-center">
          <label class="w-28 text-right pr-3 text-sm text-gray-700"
            >授权编号</label
          >
          <Input
            v-model:value="addBrandForm.authorizationNo"
            placeholder="请输入授权编号"
            allow-clear
            style="flex: 1"
          />
        </div>
        <div class="flex items-center">
          <label class="w-28 text-right pr-3 text-sm text-gray-700"
            >授权开始</label
          >
          <DatePicker
            v-model:value="addBrandForm.authorizationStart"
            placeholder="请选择日期"
            style="flex: 1"
          />
        </div>
        <div class="flex items-center">
          <label class="w-28 text-right pr-3 text-sm text-gray-700"
            >授权结束</label
          >
          <DatePicker
            v-model:value="addBrandForm.authorizationEnd"
            placeholder="请选择日期"
            style="flex: 1"
          />
        </div>
      </div>
    </Modal>
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
  color: rgb(0 0 0 / 45%);
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 4px;
  transition: all 0.2s;
}

.supplier-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}

.supplier-drawer__body {
  height: calc(100vh - 150px);
  padding: 0 8px;
  overflow-y: auto;
}

.supplier-drawer__body .ant-divider {
  margin: 12px 0 8px;
}

.supplier-drawer__body .ant-divider-inner-text {
  font-size: 13px;
  font-weight: 600;
  color: #1890ff;
}

.supplier-brand-section {
  padding-top: 16px;
  margin-top: 24px;
  border-top: 1px solid #f0f0f0;
}
</style>
