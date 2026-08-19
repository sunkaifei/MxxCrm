<script lang="ts" setup>
import type { UploadFile } from 'ant-design-vue';

import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref, watch } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import { message, TabPane, Tabs, Tooltip } from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { uploadFileApi } from '#/api/core/attachment/file';
import { getContractListApi } from '#/api/core/crm/contract';
import { getCustomerListApi } from '#/api/core/crm/customer';
import { getOrderListApi } from '#/api/core/sale/order';
import {
  createPaymentApi,
  getPaymentInfoApi,
  updatePaymentApi,
} from '#/api/core/sale/payment';
import { getUserListApi } from '#/api/core/system/user';

const props = withDefaults(defineProps<{ create?: boolean; row?: any }>(), {
  create: true,
  row: () => ({}),
});

const isEdit = computed(() => !props.create);
const activeTab = ref('basic');
const isFullscreen = ref(false);
const displayAmount = ref(0);

// 关联实体下拉数据
const contractOptions = ref<any[]>([]);
const orderOptions = ref<any[]>([]);
const customerOptions = ref<any[]>([]);
const userOptions = ref<any[]>([]);

// 选中的附件文件（手动上传，保存时再调用上传接口）
const attachmentFileList = ref<UploadFile[]>([]);
const originalAttachment = ref<string>('');

const drawerClass = computed(() => [
  'sale-payment-drawer',
  { 'sale-payment-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const paymentMethodOptions = [
  { label: '银行转账', value: 1 },
  { label: '支付宝', value: 2 },
  { label: '微信支付', value: 3 },
  { label: '现金', value: 4 },
  { label: '支票', value: 5 },
  { label: '其他', value: 6 },
];

const currencyOptions = [
  { label: 'CNY 人民币', value: 1 },
  { label: 'USD 美元', value: 2 },
  { label: 'EUR 欧元', value: 3 },
  { label: 'GBP 英镑', value: 4 },
  { label: 'JPY 日元', value: 5 },
  { label: 'HKD 港币', value: 6 },
];

const statusOptions = [
  { label: '待确认', value: 1, color: 'orange' },
  { label: '已确认', value: 2, color: 'green' },
  { label: '已驳回', value: 3, color: 'red' },
  { label: '已取消', value: 4, color: 'default' },
];

async function loadContractOptions() {
  try {
    const result: any = await getContractListApi({ page: 1, pageSize: 1000 });
    const list = result?.data?.items || result?.items || result?.list || [];
    contractOptions.value = list.map((item: any) => ({
      value: item.id,
      label: item.contractNo || item.title || `合同#${item.id}`,
    }));
  } catch (error) {
    console.error('加载合同选项失败:', error);
  }
}

async function loadOrderOptions() {
  try {
    const result: any = await getOrderListApi({ page: 1, pageSize: 1000 });
    const list = result?.data?.items || result?.items || result?.list || [];
    orderOptions.value = list.map((item: any) => ({
      value: item.id,
      label: item.orderNo || `订单#${item.id}`,
    }));
  } catch (error) {
    console.error('加载订单选项失败:', error);
  }
}

async function loadCustomerOptions() {
  try {
    const result: any = await getCustomerListApi({ page: 1, pageSize: 1000 });
    const list = result?.data?.items || result?.items || result?.list || [];
    customerOptions.value = list.map((item: any) => ({
      value: item.id,
      label:
        item.companyName || item.customerName || item.name || `客户#${item.id}`,
    }));
  } catch (error) {
    console.error('加载客户选项失败:', error);
  }
}

async function loadUserOptions() {
  try {
    const result: any = await getUserListApi({ page: 1, pageSize: 1000 });
    const list = result?.data?.items || result?.items || result?.list || [];
    userOptions.value = list.map((item: any) => ({
      value: item.id,
      label: item.realName || item.userName || `用户#${item.id}`,
    }));
  } catch (error) {
    console.error('加载用户选项失败:', error);
  }
}

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'paymentNo',
    label: '回款编号',
    componentProps: { placeholder: '保存后自动生成', disabled: true },
  },
  {
    component: 'DatePicker',
    fieldName: 'paymentDate',
    label: '到账日期',
    rules: 'required',
    componentProps: {
      placeholder: '请选择实际到账日期',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
    },
  },
  {
    component: 'Select',
    fieldName: 'contractId',
    label: '关联合同',
    componentProps: {
      placeholder: '请选择合同',
      showSearch: true,
      allowClear: true,
      filterOption: (input: string, option: any) =>
        String(option?.label ?? '')
          .toLowerCase()
          .includes(input.toLowerCase()),
      options: contractOptions,
      onChange: (_value: any) => {
        // 选择合同后可触发联动（如自动加载该合同的计划），这里保持简单
      },
    },
  },
  {
    component: 'Select',
    fieldName: 'orderId',
    label: '关联订单',
    componentProps: {
      placeholder: '请选择订单',
      showSearch: true,
      allowClear: true,
      filterOption: (input: string, option: any) =>
        String(option?.label ?? '')
          .toLowerCase()
          .includes(input.toLowerCase()),
      options: orderOptions,
    },
  },
  {
    component: 'Select',
    fieldName: 'customerId',
    label: '客户',
    rules: 'required',
    componentProps: {
      placeholder: '请选择客户',
      showSearch: true,
      allowClear: true,
      filterOption: (input: string, option: any) =>
        String(option?.label ?? '')
          .toLowerCase()
          .includes(input.toLowerCase()),
      options: customerOptions,
      onChange: (_value: any, option: any) => {
        if (option?.label) {
          basicFormApi.setValues({ customerName: option.label });
        }
      },
    },
  },
  {
    component: 'Input',
    fieldName: 'customerName',
    label: '客户名称',
    componentProps: { placeholder: '选择客户后自动带出，可手动修改' },
  },
  {
    component: 'Select',
    fieldName: 'currency',
    label: '币种',
    defaultValue: 1,
    componentProps: { placeholder: '请选择', options: currencyOptions },
  },
  {
    component: 'InputNumber',
    fieldName: 'amount',
    label: '回款金额',
    rules: 'required',
    componentProps: {
      placeholder: '请输入回款金额',
      style: 'width:100%',
      precision: 2,
      min: 0,
      prefix: '¥',
      onChange: (val: any) => {
        displayAmount.value = Number(val) || 0;
      },
    },
  },
  {
    component: 'Select',
    fieldName: 'paymentMethod',
    label: '支付方式',
    defaultValue: 1,
    rules: 'required',
    componentProps: {
      placeholder: '请选择支付方式',
      options: paymentMethodOptions,
    },
  },
  {
    component: 'Select',
    fieldName: 'status',
    label: '回款状态',
    defaultValue: 1,
    componentProps: { placeholder: '请选择状态', options: statusOptions },
  },
];

const paymentFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'payer',
    label: '付款方',
    componentProps: { placeholder: '请输入付款方名称（如公司名称）' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'payerAccount',
    label: '付款方账号',
    componentProps: { placeholder: '请输入付款方银行账号' },
  },
  {
    component: 'Input',
    fieldName: 'bankFlowNo',
    label: '银行流水号',
    componentProps: { placeholder: '请输入银行流水号' },
  },
  {
    component: 'Upload',
    fieldName: 'attachment',
    label: '回单附件',
    wrapperClass: 'col-span-2',
    componentProps: {
      accept: '.pdf,.jpg,.jpeg,.png',
      maxCount: 1,
      showUploadList: true,
      listType: 'picture-card',
      beforeUpload: () => false, // 手动上传，保存时再调用上传接口
      onChange: (info: any) => {
        attachmentFileList.value = info.fileList || [];
      },
    },
  },
];

const otherFormSchema: VbenFormSchema[] = [
  {
    component: 'Select',
    fieldName: 'ownerUserId',
    label: '负责人',
    componentProps: {
      placeholder: '请选择负责人',
      allowClear: true,
      showSearch: true,
      filterOption: (input: string, option: any) =>
        String(option?.label ?? '')
          .toLowerCase()
          .includes(input.toLowerCase()),
      options: userOptions,
    },
  },
  {
    component: 'InputNumber',
    fieldName: 'deptId',
    label: '部门ID',
    componentProps: { placeholder: '部门ID', style: 'width:100%', min: 1 },
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: {
      placeholder: '备注信息',
      rows: 4,
      showCount: true,
      maxlength: 500,
    },
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

const [PaymentForm, paymentFormApi] = useVbenForm({
  schema: paymentFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const [OtherForm, otherFormApi] = useVbenForm({
  schema: otherFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

watch(
  () => props.row,
  async (val) => {
    if (val && !props.create) {
      try {
        const info = await getPaymentInfoApi(val.id);
        const data = info || val;
        basicFormApi.setValues({
          paymentNo: data.paymentNo,
          paymentDate: data.paymentDate,
          contractId: data.contractId,
          orderId: data.orderId,
          customerId: data.customerId,
          customerName: data.customerName,
          currency: data.currency ?? 1,
          amount: data.amount ?? 0,
          paymentMethod: data.paymentMethod ?? 1,
          status: data.status ?? 1,
        });
        displayAmount.value = Number(data.amount) || 0;
        paymentFormApi.setValues({
          payer: data.payer,
          payerAccount: data.payerAccount,
          bankFlowNo: data.bankFlowNo,
          attachment: undefined,
        });
        originalAttachment.value = data.attachment || '';
        // 回显已有附件（仅展示 URL，不重建 UploadFile 列表以避免复杂度）
        attachmentFileList.value = [];
        otherFormApi.setValues({
          ownerUserId:
            data.ownerUserId === null || data.ownerUserId === undefined
              ? undefined
              : Number(data.ownerUserId),
          deptId: data.deptId,
          remark: data.remark,
        });
      } catch {
        basicFormApi.setValues(val);
      }
    }
  },
  { immediate: true },
);

async function uploadAttachment(): Promise<string> {
  // 如果用户选择了新文件，先上传获取URL
  if (attachmentFileList.value.length > 0) {
    const fileItem = attachmentFileList.value[0];
    // 来自 ant Upload 的 originFileObj 才是真实 File
    const rawFile: File | undefined =
      (fileItem as any).originFileObj || (fileItem as any).file || fileItem;
    if (rawFile instanceof File) {
      try {
        const res: any = await uploadFileApi(rawFile, 'payment');
        // uploadFileApi 返回 JSON: { code, msg, data: { id, url, ... } }
        const url = res?.data?.url || res?.url;
        if (url) {
          return url;
        }
        message.warning('附件上传成功但未返回 URL，附件字段将留空');
        return '';
      } catch (error) {
        console.error('附件上传失败:', error);
        message.error('附件上传失败');
        throw error;
      }
    }
  }
  // 没有选择新文件时，沿用原附件URL
  return originalAttachment.value;
}

async function handleSubmit() {
  const validResult = await basicFormApi.validate();
  if (!validResult.valid) {
    activeTab.value = 'basic';
    return false;
  }
  const basicValues = await basicFormApi.getValues();
  const paymentValues = await paymentFormApi.getValues();
  const otherValues = await otherFormApi.getValues();

  drawerApi.setState({ confirmLoading: true });
  try {
    // 处理附件上传
    const attachmentUrl = await uploadAttachment();

    // 移除 attachment 字段（避免表单原值干扰），用上传后的 URL 替换
    const { attachment: _attachment, ...restPaymentValues } = paymentValues;
    void _attachment;

    const data = {
      ...basicValues,
      ...restPaymentValues,
      attachment: attachmentUrl || undefined,
      ...otherValues,
      ownerUserId:
        otherValues.ownerUserId === null ||
        otherValues.ownerUserId === undefined
          ? undefined
          : Number(otherValues.ownerUserId),
    };

    if (isEdit.value) {
      await updatePaymentApi({ ...data, id: props.row.id });
      message.success('更新成功');
    } else {
      await createPaymentApi(data);
      message.success('登记成功');
    }
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch {
    message.error('操作失败');
  } finally {
    drawerApi.setState({ confirmLoading: false });
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      displayAmount.value = 0;
      attachmentFileList.value = [];
      originalAttachment.value = '';
      basicFormApi.resetForm();
      paymentFormApi.resetForm();
      otherFormApi.resetForm();

      // 加载下拉数据
      Promise.all([
        loadContractOptions(),
        loadOrderOptions(),
        loadCustomerOptions(),
        loadUserOptions(),
      ]);

      if (!props.create && props.row) {
        basicFormApi.setValues({
          currency: 1,
          paymentMethod: 1,
          status: 1,
          ...props.row,
        });
        paymentFormApi.setValues(props.row);
        otherFormApi.setValues({
          ownerUserId:
            props.row?.ownerUserId === null ||
            props.row?.ownerUserId === undefined
              ? undefined
              : Number(props.row.ownerUserId),
          deptId: props.row?.deptId,
          remark: props.row?.remark,
        });
        originalAttachment.value = props.row?.attachment || '';
      } else {
        basicFormApi.setValues({
          currency: 1,
          paymentMethod: 1,
          status: 1,
          paymentDate: new Date().toISOString().slice(0, 10),
        });
      }
    }
  },
  onCancel() {
    drawerApi.close();
    drawerApi.setData({ needRefresh: true });
  },
  onConfirm: handleSubmit,
});
</script>

<template>
  <Drawer
    :title="isEdit ? '编辑回款记录' : '登记回款'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-payment-drawer__fs-btn"
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

    <Tabs v-model:active-key="activeTab">
      <TabPane key="basic" tab="基本信息">
        <BasicForm />
      </TabPane>
      <TabPane key="payment" tab="收款信息">
        <PaymentForm />
        <div
          v-if="originalAttachment"
          class="mt-2 text-xs text-gray-500 break-all"
        >
          当前附件：
          <a
            :href="originalAttachment"
            target="_blank"
            class="text-blue-500 hover:underline"
          >
            {{ originalAttachment }}
          </a>
        </div>
        <div class="mt-4 rounded-lg bg-gray-50 p-4">
          <div class="mb-2 text-sm text-gray-500">回款提示</div>
          <div class="text-xs text-gray-400 leading-6">
            1. 回款登记后状态默认为"待确认"，财务确认后变为"已确认"<br />
            2. 银行流水号用于财务对账，建议填写<br />
            3. 回款确认后，可通过"核销"操作将回款金额分配到合同的回款计划<br />
            4. 回单附件支持上传银行回单截图或PDF文件（手动上传，保存时自动提交）
          </div>
        </div>
      </TabPane>
      <TabPane key="other" tab="其他信息">
        <OtherForm />
      </TabPane>
    </Tabs>

    <div class="mt-6 border-t pt-4">
      <div
        class="flex items-center justify-between rounded-lg bg-blue-50 px-4 py-3"
      >
        <span class="text-sm text-gray-600">本次回款金额</span>
        <span class="text-xl font-bold text-blue-600">
          ¥{{
            displayAmount.toLocaleString('zh-CN', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })
          }}
        </span>
      </div>
    </div>
  </Drawer>
</template>

<style>
.sale-payment-drawer {
  width: 75vw !important;
}

.sale-payment-drawer--fullscreen {
  width: 100vw !important;
}

.sale-payment-drawer__fs-btn {
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

.sale-payment-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}
</style>
