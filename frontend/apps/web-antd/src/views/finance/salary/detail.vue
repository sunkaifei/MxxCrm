<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  Form,
  FormItem,
  InputNumber,
  message,
  Modal,
  Table,
  Tag,
} from 'ant-design-vue';

import {
  approveSalaryApi,
  getSalaryDetailApi,
  paySalaryApi,
  updateSalaryApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

const route = useRoute();
const router = useRouter();

const loading = ref(false);
const detail = ref<any>(null);

const statusMap: Record<number, { color: string; label: string }> = {
  0: { label: $t('page.finance.salary.status.pending'), color: 'blue' },
  1: { label: $t('page.finance.salary.status.approved'), color: 'orange' },
  2: { label: $t('page.finance.salary.status.paid'), color: 'green' },
};

const adjustVisible = ref(false);
const adjustLoading = ref(false);
const adjustForm = reactive({
  performanceBonus: 0,
  deduction: 0,
});

function formatMoney(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `¥${Number(val).toLocaleString()}`;
}

const salaryId = computed(() => Number(route.params.id));

const commissionColumns = computed(() => [
  {
    title: $t('page.finance.salary.detail.column.contractName'),
    dataIndex: 'contractName',
  },
  {
    title: $t('page.finance.salary.detail.column.contractAmount'),
    dataIndex: 'contractAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.detail.column.paymentAmount'),
    dataIndex: 'paymentAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.detail.column.commissionRate'),
    dataIndex: 'commissionRate',
    customRender: ({ text }: any) =>
      text === null || text === undefined ? '-' : `${(text * 100).toFixed(2)}%`,
  },
  {
    title: $t('page.finance.salary.detail.column.commissionAmount'),
    dataIndex: 'commissionAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.detail.column.ruleName'),
    dataIndex: 'ruleName',
  },
]);

async function loadDetail() {
  loading.value = true;
  try {
    const res: any = await getSalaryDetailApi(salaryId.value);
    detail.value = res?.data ?? res;
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.loadFailed'),
    );
  } finally {
    loading.value = false;
  }
}

function openAdjust() {
  adjustForm.performanceBonus = detail.value?.performanceBonus ?? 0;
  adjustForm.deduction = detail.value?.deduction ?? 0;
  adjustVisible.value = true;
}

async function handleAdjustSubmit() {
  adjustLoading.value = true;
  try {
    await updateSalaryApi({
      id: salaryId.value,
      performanceBonus: adjustForm.performanceBonus,
      deduction: adjustForm.deduction,
    });
    message.success($t('page.finance.salary.message.adjustSuccess'));
    adjustVisible.value = false;
    await loadDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.adjustFailed'),
    );
  } finally {
    adjustLoading.value = false;
  }
}

async function handleApprove() {
  try {
    await approveSalaryApi(salaryId.value);
    message.success($t('page.finance.salary.message.approveSuccess'));
    await loadDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.approveFailed'),
    );
  }
}

async function handlePay() {
  try {
    await paySalaryApi(salaryId.value);
    message.success($t('page.finance.salary.message.paySuccess'));
    await loadDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.payFailed'),
    );
  }
}

function goBack() {
  void router.push('/finance/salary');
}

onMounted(() => {
  if (salaryId.value) {
    loadDetail();
  }
});
</script>

<template>
  <Page auto-content-height>
    <div class="mb-4 flex items-center justify-between">
      <Button @click="goBack">{{ $t('page.finance.common.back') }}</Button>
      <div class="flex gap-2">
        <Button v-if="detail?.status === 0" type="primary" @click="openAdjust">
          {{ $t('page.finance.salary.button.adjust') }}
        </Button>
        <Button
          v-if="detail?.status === 0"
          type="primary"
          @click="handleApprove"
        >
          {{ $t('page.finance.salary.detail.approveButton') }}
        </Button>
        <Button v-if="detail?.status === 1" type="primary" @click="handlePay">
          {{ $t('page.finance.salary.detail.payButton') }}
        </Button>
      </div>
    </div>

    <Card
      :title="$t('page.finance.salary.detail.salaryInfo')"
      class="mb-4"
      :loading="loading"
    >
      <Descriptions v-if="detail" :column="3" bordered>
        <DescriptionsItem
          :label="$t('page.finance.salary.column.employeeName')"
        >
          {{ detail.employeeName }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.finance.salary.column.department')">
          {{ detail.deptName }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.finance.salary.column.yearMonth')">
          {{ detail.year }}{{ $t('page.finance.common.year') }}{{ detail.month
          }}{{ $t('page.finance.common.month') }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.finance.salary.column.baseSalary')">
          {{ formatMoney(detail.baseSalary) }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.finance.salary.column.commissionAmount')"
        >
          {{ formatMoney(detail.commissionAmount) }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.finance.salary.column.teamCommissionAmount')"
        >
          {{ formatMoney(detail.teamCommissionAmount) }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.finance.salary.column.bonusAmount')">
          {{ formatMoney(detail.bonusAmount) }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.finance.salary.column.allocatedCommission')"
        >
          {{ formatMoney(detail.allocatedCommission) }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.finance.salary.column.deferredCommission')"
        >
          {{ formatMoney(detail.deferredCommission) }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.finance.salary.column.performanceBonus')"
        >
          {{ formatMoney(detail.performanceBonus) }}
        </DescriptionsItem>
        <DescriptionsItem
          :label="$t('page.finance.salary.detail.deductionAmount')"
        >
          {{ formatMoney(detail.deduction) }}
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.finance.salary.column.totalSalary')">
          <span class="font-medium text-blue-600">
            {{ formatMoney(detail.totalAmount) }}
          </span>
        </DescriptionsItem>
        <DescriptionsItem :label="$t('page.finance.common.status')">
          <Tag :color="statusMap[detail.status]?.color || 'default'">
            {{ statusMap[detail.status]?.label || detail.status }}
          </Tag>
        </DescriptionsItem>
      </Descriptions>
    </Card>

    <Card
      :title="$t('page.finance.salary.detail.commissionDetail')"
      :loading="loading"
    >
      <Table
        :data-source="detail?.commissionDetails || []"
        :columns="commissionColumns"
        :pagination="false"
        row-key="id"
        size="middle"
      />
    </Card>

    <Modal
      v-model:open="adjustVisible"
      :title="$t('page.finance.salary.detail.adjustPerformance')"
      :confirm-loading="adjustLoading"
      @ok="handleAdjustSubmit"
    >
      <Form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }" class="py-4">
        <FormItem :label="$t('page.finance.salary.detail.performanceBonus')">
          <InputNumber
            v-model:value="adjustForm.performanceBonus"
            :min="0"
            :precision="2"
            style="width: 100%"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salary.detail.deductionAmount')">
          <InputNumber
            v-model:value="adjustForm.deduction"
            :min="0"
            :precision="2"
            style="width: 100%"
          />
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>
