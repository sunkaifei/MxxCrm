<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Card, Col, Progress, Row, Table } from 'ant-design-vue';

import {
  getPaymentCompletionApi,
  getPaymentMonthlyTrendApi,
  getPaymentRankingApi,
  getPaymentStatusAnalysisApi,
} from '#/api/core/statistics';
import { $t } from '#/locales';

import TimeFilter from '../components/time-filter.vue';

const completionData = ref<any>({});
const monthlyTrendData = ref<any[]>([]);
const statusAnalysisData = ref<any[]>([]);
const rankingData = ref<any[]>([]);
const timeParams = ref<{
  end_date?: string;
  start_date?: string;
  year?: number;
}>({});

const loadData = async () => {
  try {
    const [completionRes, trendRes, statusRes, rankingRes] = await Promise.all([
      getPaymentCompletionApi(timeParams.value),
      // 月度趋势为全年 12 个月视图，按筛选范围的年份展示
      getPaymentMonthlyTrendApi({ year: timeParams.value.year }),
      getPaymentStatusAnalysisApi(timeParams.value),
      getPaymentRankingApi({
        ...timeParams.value,
        order_by: 'payment_amount',
        limit: 10,
      }),
    ]);

    const cd = (completionRes as any)?.data ?? (completionRes as any) ?? {};
    completionData.value = {
      year: cd.year,
      totalContractAmount: Number(cd.total_contract_amount) || 0,
      totalPaymentAmount: Number(cd.total_payment_amount) || 0,
      completionRate: Number(cd.completion_rate) || 0,
      overdueAmount: Number(cd.overdue_amount) || 0,
      overdueRate: Number(cd.overdue_rate) || 0,
      unpaidAmount: Number(cd.unpaid_amount) || 0,
      unpaidRate: Number(cd.unpaid_rate) || 0,
    };

    const td = (trendRes as any)?.data ?? (trendRes as any) ?? {};
    monthlyTrendData.value = (td.months ?? []).map((item: any) => ({
      month: item.month,
      contractAmount: Number(item.contract_amount) || 0,
      paymentAmount: Number(item.payment_amount) || 0,
      completionRate: Number(item.completion_rate) || 0,
      overdueAmount: Number(item.overdue_amount) || 0,
    }));

    const statusList = Array.isArray(statusRes)
      ? statusRes
      : ((statusRes as any)?.data ?? []);
    statusAnalysisData.value = statusList.map((item: any) => ({
      status: item.status_name,
      contractCount: item.contract_count,
      contractAmount: Number(item.contract_amount) || 0,
      paidAmount: Number(item.paid_amount) || 0,
      percentage: Number(item.percentage) || 0,
    }));

    const rankingList = Array.isArray(rankingRes)
      ? rankingRes
      : ((rankingRes as any)?.data ?? []);
    rankingData.value = rankingList.map((item: any) => ({
      rank: item.rank,
      targetName: item.target_name,
      contractAmount: Number(item.contract_amount) || 0,
      paymentAmount: Number(item.payment_amount) || 0,
      completionRate: Number(item.completion_rate) || 0,
      overdueAmount: Number(item.overdue_amount) || 0,
    }));
  } catch (error) {
    console.error($t('page.statistics.loadPaymentFailed'), error);
  }
};

function handleTimeChange(params: {
  end_date?: string;
  start_date?: string;
  year?: number;
}) {
  timeParams.value = params;
  loadData();
}

function formatCurrency(val: number) {
  return `¥${(val / 10_000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const trendColumns = [
  {
    title: $t('page.statistics.month'),
    dataIndex: 'month',
    customRender: ({ text }: any) =>
      `${text}${$t('page.statistics.monthUnit')}`,
  },
  {
    title: $t('page.statistics.contractAmount'),
    dataIndex: 'contractAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.paidAmount'),
    dataIndex: 'paymentAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.paymentRate'),
    dataIndex: 'completionRate',
    align: 'center' as const,
    customRender: ({ text }: any) => `${Number(text).toFixed(2)}%`,
  },
  {
    title: $t('page.statistics.overdueAmount'),
    dataIndex: 'overdueAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
];

const statusColumns = [
  { title: $t('page.statistics.status'), dataIndex: 'status' },
  {
    title: $t('page.statistics.contractCountCol'),
    dataIndex: 'contractCount',
    align: 'right' as const,
  },
  {
    title: $t('page.statistics.contractAmount'),
    dataIndex: 'contractAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.paidAmount'),
    dataIndex: 'paidAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => (text ? formatCurrency(text) : '-'),
  },
  {
    title: $t('page.statistics.percentage'),
    dataIndex: 'percentage',
    align: 'right' as const,
    customRender: ({ text }: any) => `${Number(text).toFixed(2)}%`,
  },
];

const rankingColumns = [
  { title: $t('page.statistics.rank'), dataIndex: 'rank', width: 60 },
  { title: $t('page.statistics.customerName'), dataIndex: 'targetName' },
  {
    title: $t('page.statistics.contractAmount'),
    dataIndex: 'contractAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.paidAmount'),
    dataIndex: 'paymentAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.paymentRate'),
    dataIndex: 'completionRate',
    align: 'right' as const,
    customRender: ({ text }: any) => `${Number(text).toFixed(2)}%`,
  },
  {
    title: $t('page.statistics.overdueAmount'),
    dataIndex: 'overdueAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">
        {{ $t('page.statistics.paymentAnalysis') }}
      </h2>

      <TimeFilter @change="handleTimeChange" />

      <Row :gutter="16" class="mb-6">
        <Col :span="12">
          <Card
            :title="$t('page.statistics.paymentCompletion')"
            :extra="`${Number(completionData.completionRate || 0).toFixed(2)}%`"
          >
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm text-gray-500">
                  {{ $t('page.statistics.totalContractAmount') }}
                </div>
                <div class="text-xl font-bold">
                  {{ formatCurrency(completionData.totalContractAmount || 0) }}
                </div>
              </div>
              <div class="text-right">
                <div class="text-sm text-gray-500">
                  {{ $t('page.statistics.totalPaymentAmount') }}
                </div>
                <div class="text-xl font-bold text-green-600">
                  {{ formatCurrency(completionData.totalPaymentAmount || 0) }}
                </div>
              </div>
            </div>
            <Progress
              :percent="completionData.completionRate || 0"
              :stroke-width="12"
              class="mt-3"
              stroke-color="#52c41a"
            />
          </Card>
        </Col>
        <Col :span="12">
          <Card :title="$t('page.statistics.paymentStatus')">
            <div class="space-y-3">
              <div class="flex justify-between">
                <span>{{ $t('page.statistics.unpaid') }}</span>
                <span class="text-red-500"
                  >{{ formatCurrency(completionData.unpaidAmount || 0) }} ({{
                    Number(completionData.unpaidRate || 0).toFixed(2)
                  }}%)</span
                >
              </div>
              <div class="flex justify-between">
                <span>{{ $t('page.statistics.overdueAmount') }}</span>
                <span class="text-orange-500"
                  >{{ formatCurrency(completionData.overdueAmount || 0) }} ({{
                    Number(completionData.overdueRate || 0).toFixed(2)
                  }}%)</span
                >
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Card :title="$t('page.statistics.monthlyTrend')" class="mb-6">
        <Table
          :columns="trendColumns"
          :data-source="monthlyTrendData"
          :pagination="false"
        />
      </Card>

      <Row :gutter="16" class="mt-4">
        <Col :span="12">
          <Card :title="$t('page.statistics.paymentStatus')">
            <Table
              :columns="statusColumns"
              :data-source="statusAnalysisData"
              :pagination="false"
              size="small"
            />
          </Card>
        </Col>
        <Col :span="12">
          <Card :title="$t('page.statistics.paymentRanking')">
            <Table
              :columns="rankingColumns"
              :data-source="rankingData"
              :pagination="false"
              size="small"
            />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>
