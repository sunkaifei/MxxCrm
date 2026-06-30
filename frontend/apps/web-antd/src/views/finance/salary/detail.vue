<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  Form,
  FormItem,
  InputNumber,
  Modal,
  Table,
  Tag,
  message,
} from 'ant-design-vue';
import { useRoute, useRouter } from 'vue-router';

import {
  approveSalaryApi,
  getSalaryDetailApi,
  paySalaryApi,
  updateSalaryApi,
} from '#/api/core/finance';

const route = useRoute();
const router = useRouter();

const loading = ref(false);
const detail = ref<any>(null);

const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: '待审核', color: 'blue' },
  1: { label: '已审核', color: 'orange' },
  2: { label: '已发放', color: 'green' },
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
  { title: '合同名称', dataIndex: 'contractName' },
  {
    title: '合同金额',
    dataIndex: 'contractAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: '回款金额',
    dataIndex: 'paymentAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: '提成比例',
    dataIndex: 'commissionRate',
    customRender: ({ text }: any) =>
      text === null || text === undefined ? '-' : `${(text * 100).toFixed(2)}%`,
  },
  {
    title: '提成金额',
    dataIndex: 'commissionAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  { title: '适用规则', dataIndex: 'ruleName' },
]);

async function loadDetail() {
  loading.value = true;
  try {
    const res: any = await getSalaryDetailApi(salaryId.value);
    detail.value = res?.data ?? res;
  } catch (e: any) {
    message.error(e?.message || '加载详情失败');
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
    message.success('调整成功');
    adjustVisible.value = false;
    await loadDetail();
  } catch (e: any) {
    message.error(e?.message || '调整失败');
  } finally {
    adjustLoading.value = false;
  }
}

async function handleApprove() {
  try {
    await approveSalaryApi(salaryId.value);
    message.success('审核成功');
    await loadDetail();
  } catch (e: any) {
    message.error(e?.message || '审核失败');
  }
}

async function handlePay() {
  try {
    await paySalaryApi(salaryId.value);
    message.success('发放成功');
    await loadDetail();
  } catch (e: any) {
    message.error(e?.message || '发放失败');
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
      <Button @click="goBack">返回列表</Button>
      <div class="flex gap-2">
        <Button
          v-if="detail?.status === 0"
          type="primary"
          @click="openAdjust"
        >
          手动调整
        </Button>
        <Button
          v-if="detail?.status === 0"
          type="primary"
          @click="handleApprove"
        >
          审核
        </Button>
        <Button
          v-if="detail?.status === 1"
          type="primary"
          @click="handlePay"
        >
          发放
        </Button>
      </div>
    </div>

    <Card title="工资信息" class="mb-4" :loading="loading">
      <Descriptions v-if="detail" :column="3" bordered>
        <DescriptionsItem label="员工姓名">
          {{ detail.employeeName }}
        </DescriptionsItem>
        <DescriptionsItem label="部门">
          {{ detail.deptName }}
        </DescriptionsItem>
        <DescriptionsItem label="年月">
          {{ detail.year }}年{{ detail.month }}月
        </DescriptionsItem>
        <DescriptionsItem label="底薪">
          {{ formatMoney(detail.baseSalary) }}
        </DescriptionsItem>
        <DescriptionsItem label="提成金额">
          {{ formatMoney(detail.commissionAmount) }}
        </DescriptionsItem>
        <DescriptionsItem label="绩效奖金">
          {{ formatMoney(detail.performanceBonus) }}
        </DescriptionsItem>
        <DescriptionsItem label="扣款金额">
          {{ formatMoney(detail.deduction) }}
        </DescriptionsItem>
        <DescriptionsItem label="应发工资">
          <span class="font-medium text-blue-600">
            {{ formatMoney(detail.totalAmount) }}
          </span>
        </DescriptionsItem>
        <DescriptionsItem label="状态">
          <Tag :color="statusMap[detail.status]?.color || 'default'">
            {{ statusMap[detail.status]?.label || detail.status }}
          </Tag>
        </DescriptionsItem>
      </Descriptions>
    </Card>

    <Card title="提成明细" :loading="loading">
      <Table
        :data-source="detail?.commissionDetails || []"
        :columns="commissionColumns"
        :pagination="false"
        row-key="id"
        size="middle"
      >
      </Table>
    </Card>

    <Modal
      v-model:open="adjustVisible"
      title="手动调整绩效/扣款"
      :confirm-loading="adjustLoading"
      @ok="handleAdjustSubmit"
    >
      <Form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }" class="py-4">
        <FormItem label="绩效奖金">
          <InputNumber
            v-model:value="adjustForm.performanceBonus"
            :min="0"
            :precision="2"
            style="width: 100%"
          />
        </FormItem>
        <FormItem label="扣款金额">
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
