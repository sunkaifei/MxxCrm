<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, onMounted, reactive, ref } from 'vue';
import { useRoute } from 'vue-router';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import dayjs from 'dayjs';
import {
  Button,
  DatePicker,
  Drawer,
  Form,
  Input,
  InputNumber,
  message,
  Select,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  getPaymentPlanPageListApi,
  registerPaymentByPlanApi,
} from '#/api';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { $t } from '#/locales';

// 回款计划使用说明步骤数（与 i18n 中 page.sale.paymentPlan.guide.steps 数组对齐）
const guideStepCount = 5;

const accessStore = useAccessStore();

const route = useRoute();

const { canViewAll, canViewSubordinate } = useDataScopeTabs();

const allTabList = [
  { key: 'all', label: '全部回款计划' },
  { key: 'my', label: '我的回款计划' },
  { key: 'subordinate', label: '下属回款计划' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  gridApi.query();
}

// ============ 登记回款（直接核销到当前计划） ============
const paymentMethodOptions = [
  { label: '银行转账', value: 1 },
  { label: '支付宝', value: 2 },
  { label: '微信支付', value: 3 },
  { label: '现金', value: 4 },
  { label: '支票', value: 5 },
  { label: '其他', value: 6 },
];

const registerOpen = ref(false);
const registerLoading = ref(false);
const registerRow = ref<any>(null);

const registerForm = reactive({
  amount: 0,
  paymentDate: dayjs().format('YYYY-MM-DD'),
  paymentMethod: 1,
  payer: '',
  payerAccount: '',
  remark: '',
});

const registerRemaining = computed(() => {
  const row = registerRow.value;
  if (!row) return 0;
  return (
    Number(row.planAmount || 0) - Number(row.receivedAmount || 0)
  );
});

function openRegister(row: any) {
  registerRow.value = row;
  registerForm.amount = registerRemaining.value;
  registerForm.paymentDate = dayjs().format('YYYY-MM-DD');
  registerForm.paymentMethod = 1;
  // 付款方默认带出签约客户名（银行回单付款户名可能不同，允许修改）
  registerForm.payer = row.customerName || '';
  registerForm.payerAccount = '';
  registerForm.remark = '';
  registerOpen.value = true;
}

async function handleRegister() {
  const row = registerRow.value;
  if (!row) return;

  const amount = Number(registerForm.amount);
  if (!amount || amount <= 0) {
    message.warning('请输入回款金额');
    return;
  }
  if (amount > registerRemaining.value) {
    message.warning(
      `回款金额不能超过未收金额 ¥${registerRemaining.value.toFixed(2)}`,
    );
    return;
  }

  registerLoading.value = true;
  try {
    await registerPaymentByPlanApi({
      planId: row.id,
      amount,
      paymentDate: registerForm.paymentDate || undefined,
      paymentMethod: registerForm.paymentMethod,
      payer: registerForm.payer || undefined,
      payerAccount: registerForm.payerAccount || undefined,
      remark: registerForm.remark || undefined,
    });
    message.success('回款登记成功');
    registerOpen.value = false;
    gridApi.query();
  } catch {
    // 错误由全局拦截器提示
  } finally {
    registerLoading.value = false;
  }
}

const statusOptions = [
  { label: '未开始', value: 0 },
  { label: '部分回款', value: 1 },
  { label: '已完成', value: 2 },
  { label: '已逾期', value: 3 },
  { label: '已收讫', value: 4 },
];

const _paymentTypeOptions = [
  { label: '预付款', value: 1 },
  { label: '进度款', value: 2 },
  { label: '到货款', value: 3 },
  { label: '验收款', value: 4 },
  { label: '质保金', value: 5 },
  { label: '尾款', value: 6 },
];
void _paymentTypeOptions;

const statusColorMap: Record<number, string> = {
  0: 'default',
  1: 'orange',
  2: 'green',
  3: 'red',
  4: 'green',
};

const statusLabelMap: Record<number, string> = {
  0: '未开始',
  1: '部分回款',
  2: '已完成',
  3: '已逾期',
  4: '已收讫',
};

const paymentTypeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'orange',
  4: 'purple',
  5: 'geekblue',
  6: 'magenta',
};

const paymentTypeLabelMap: Record<number, string> = {
  1: '预付款',
  2: '进度款',
  3: '到货款',
  4: '验收款',
  5: '质保金',
  6: '尾款',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '合同编号/期次名称', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: statusOptions,
      },
    },
    {
      component: 'Input',
      fieldName: 'contractId',
      label: '合同ID',
      componentProps: { placeholder: '合同ID', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'customerId',
      label: '客户ID',
      componentProps: { placeholder: '客户ID', allowClear: true },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (formValues.keywords) params.keywords = formValues.keywords;
        if (formValues.status) params.status = formValues.status;
        if (formValues.contractId) params.contractId = formValues.contractId;
        if (formValues.customerId) params.customerId = formValues.customerId;
        return await getPaymentPlanPageListApi(params);
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '合同编号', field: 'contractNo', width: 160 },
    { title: '客户名称', field: 'customerName', minWidth: 140 },
    { title: '期次名称', field: 'stageName', width: 140 },
    {
      title: '款项类型',
      field: 'paymentType',
      width: 110,
      slots: { default: 'paymentType' },
    },
    {
      title: '计划金额',
      field: 'planAmount',
      width: 130,
      slots: { default: 'planAmount' },
    },
    {
      title: '已收金额',
      field: 'receivedAmount',
      width: 130,
      slots: { default: 'receivedAmount' },
    },
    {
      title: '未收金额',
      field: 'unreceivedAmount',
      width: 130,
      slots: { default: 'unreceivedAmount' },
    },
    { title: '计划日期', field: 'planDate', width: 120 },
    { title: '实际日期', field: 'actualDate', width: 120 },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    { title: '备注', field: 'remark', minWidth: 150 },
    {
      title: $t('ui.table.action'),
      field: 'action',
      width: 110,
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 从工作台待办「查看详细」跳转：按合同定位回款计划
onMounted(() => {
  const cid = route.query.contractId;
  if (!cid) return;
  // 有全部数据权限时切到「全部回款计划」，确保能看到该合同下所有计划（待办回款提醒不区分归属）
  if (route.query.tab === 'all' && canViewAll.value) {
    activeTab.value = 'all';
  }
  const contractId = Array.isArray(cid) ? cid[0] : cid;
  gridApi.formApi.setValues({ contractId: String(contractId) });
  gridApi.query();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.sale.paymentPlan.guide.title')"
      :brief="$t('page.sale.paymentPlan.guide.brief')"
      :expand-text="$t('page.sale.paymentPlan.guide.expand')"
      :collapse-text="$t('page.sale.paymentPlan.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.sale.paymentPlan.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.sale.paymentPlan.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Grid table-title="回款计划列表">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>

      <template #paymentType="{ row }">
        <Tag
          v-if="row.paymentType != null"
          :color="paymentTypeColorMap[row.paymentType]"
        >
          {{ paymentTypeLabelMap[row.paymentType] || row.paymentType }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #planAmount="{ row }">
        <span class="text-right block">
          ¥{{
            Number(row.planAmount || 0).toLocaleString('zh-CN', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })
          }}
        </span>
      </template>

      <template #receivedAmount="{ row }">
        <span class="text-right block">
          ¥{{
            Number(row.receivedAmount || 0).toLocaleString('zh-CN', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })
          }}
        </span>
      </template>

      <template #unreceivedAmount="{ row }">
        <span
          class="text-right block"
          :class="
            Number(row.planAmount || 0) - Number(row.receivedAmount || 0) > 0
              ? 'text-orange-600 font-medium'
              : 'text-gray-400'
          "
        >
          ¥{{
            (
              Number(row.planAmount || 0) - Number(row.receivedAmount || 0)
            ).toLocaleString('zh-CN', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })
          }}
        </span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status != null" :color="statusColorMap[row.status]">
          {{ statusLabelMap[row.status] || row.status }}
        </Tag>
        <span v-else class="text-gray-300">-</span>
      </template>

      <template #action="{ row }">
        <Button
          v-if="
            accessStore.hasAccessCode('sale:payment:save') &&
            (row.status === 0 || row.status === 1) &&
            Number(row.planAmount || 0) - Number(row.receivedAmount || 0) > 0
          "
          type="link"
          size="small"
          @click="() => openRegister(row)"
        >
          登记回款
        </Button>
      </template>
    </Grid>

    <!-- 登记回款抽屉：直接在计划上登记回款并核销 -->
    <Drawer
      :open="registerOpen"
      title="登记回款"
      :width="480"
      @close="registerOpen = false"
    >
      <div
        v-if="registerRow"
        class="mb-4 rounded-lg bg-blue-50 px-3 py-2 text-sm text-blue-600"
      >
        期次「{{ registerRow.stageName }}」· 计划金额
        ¥{{
          Number(registerRow.planAmount || 0).toLocaleString('zh-CN', {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
          })
        }}
        · 未收
        ¥{{
          registerRemaining.toLocaleString('zh-CN', {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
          })
        }}
      </div>

      <Form :model="registerForm" layout="vertical">
        <Form.Item label="本次回款金额" required>
          <InputNumber
            v-model:value="registerForm.amount"
            :min="0.01"
            :max="registerRemaining"
            :precision="2"
            style="width: 100%"
            placeholder="请输入回款金额"
          />
        </Form.Item>
        <Form.Item label="回款日期">
          <DatePicker
            v-model:value="registerForm.paymentDate"
            value-format="YYYY-MM-DD"
            style="width: 100%"
            :allow-clear="false"
          />
        </Form.Item>
        <Form.Item label="支付方式">
          <Select
            v-model:value="registerForm.paymentMethod"
            :options="paymentMethodOptions"
            style="width: 100%"
          />
        </Form.Item>
        <Form.Item label="付款方名称">
          <Input
            v-model:value="registerForm.payer"
            placeholder="默认带出客户名，可改为实际付款单位"
            allow-clear
          />
        </Form.Item>
        <Form.Item label="付款方账号">
          <Input
            v-model:value="registerForm.payerAccount"
            placeholder="请输入付款方银行账号（选填）"
            allow-clear
          />
        </Form.Item>
        <Form.Item label="备注">
          <Input.TextArea
            v-model:value="registerForm.remark"
            :rows="3"
            :maxlength="200"
            placeholder="选填"
            show-count
          />
        </Form.Item>
      </Form>
      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button @click="registerOpen = false">取消</Button>
          <Button
            type="primary"
            :loading="registerLoading"
            @click="handleRegister"
          >
            确认登记
          </Button>
        </div>
      </template>
    </Drawer>
  </Page>
</template>
