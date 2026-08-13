<script lang="ts" setup>
import { ref } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { $t } from '#/locales';
import { getContractRankingApi, getContractTypeDistributionApi, getContractStatusAnalysisApi } from '#/api/core/statistics';
import TimeFilter from '../components/time-filter.vue';

const rankingData = ref<any[]>([]);
const typeDistributionData = ref<any[]>([]);
const statusAnalysisData = ref<any[]>([]);
const timeParams = ref<{ start_date?: string; end_date?: string; year?: number }>({});

const loadData = async () => {
  try {
    const [rankingRes, typeRes, statusRes] = await Promise.all([
      getContractRankingApi({ ...timeParams.value, order_by: 'amount', limit: 10 }),
      getContractTypeDistributionApi(timeParams.value),
      getContractStatusAnalysisApi(timeParams.value),
    ]);

    const rankingList = Array.isArray(rankingRes) ? rankingRes : (rankingRes as any)?.data ?? [];
    rankingData.value = rankingList.map((item: any) => ({
      rank: item.rank,
      targetName: item.target_name,
      contractCount: item.contract_count,
      contractAmount: Number(item.contract_amount) || 0,
      paymentAmount: Number(item.payment_amount) || 0,
      paymentRate: Number(item.payment_rate) || 0,
    }));

    const typeList = Array.isArray(typeRes) ? typeRes : (typeRes as any)?.data ?? [];
    typeDistributionData.value = typeList.map((item: any) => ({
      contractType: item.contract_type,
      contractCount: item.contract_count,
      contractAmount: Number(item.contract_amount) || 0,
      percentage: Number(item.percentage) || 0,
    }));

    const statusList = Array.isArray(statusRes) ? statusRes : (statusRes as any)?.data ?? [];
    statusAnalysisData.value = statusList.map((item: any) => ({
      status: item.status_name,
      contractCount: item.contract_count,
      contractAmount: Number(item.contract_amount) || 0,
      percentage: Number(item.percentage) || 0,
    }));
  } catch (e) {
    console.error($t('page.statistics.loadContractFailed'), e);
  }
};

function handleTimeChange(params: { start_date?: string; end_date?: string; year?: number }) {
  timeParams.value = params;
  loadData();
}

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const rankingColumns = [
  { title: $t('page.statistics.rank'), dataIndex: 'rank', width: 60 },
  { title: $t('page.statistics.customerName'), dataIndex: 'targetName' },
  { title: $t('page.statistics.contractCountCol'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.contractAmount'), dataIndex: 'contractAmount', align: 'right' as const, customRender: ({ text }: any) => formatCurrency(text) },
  { title: $t('page.statistics.paidAmount'), dataIndex: 'paymentAmount', align: 'right' as const, customRender: ({ text }: any) => formatCurrency(text) },
  { title: $t('page.statistics.paymentRate'), dataIndex: 'paymentRate', align: 'right' as const, customRender: ({ text }: any) => `${Number(text).toFixed(2)}%` },
];

const typeColumns = [
  { title: $t('page.statistics.contractTypeCol'), dataIndex: 'contractType' },
  { title: $t('page.statistics.count'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.amount'), dataIndex: 'contractAmount', align: 'right' as const, customRender: ({ text }: any) => formatCurrency(text) },
  { title: $t('page.statistics.percentage'), dataIndex: 'percentage', align: 'right' as const, customRender: ({ text }: any) => `${Number(text).toFixed(2)}%` },
];

const statusColumns = [
  { title: $t('page.statistics.status'), dataIndex: 'status' },
  { title: $t('page.statistics.count'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.amount'), dataIndex: 'contractAmount', align: 'right' as const, customRender: ({ text }: any) => formatCurrency(text) },
  { title: $t('page.statistics.percentage'), dataIndex: 'percentage', align: 'right' as const, customRender: ({ text }: any) => `${Number(text).toFixed(2)}%` },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">{{ $t('page.statistics.contractRanking') }}</h2>

      <TimeFilter @change="handleTimeChange" />

      <Card :title="$t('page.statistics.contractRankingTitle')" class="mb-6">
        <Table :columns="rankingColumns" :data-source="rankingData" :pagination="false" />
      </Card>

      <Row :gutter="16" class="mt-4">
        <Col :span="12">
          <Card :title="$t('page.statistics.contractTypeDist')">
            <Table :columns="typeColumns" :data-source="typeDistributionData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="12">
          <Card :title="$t('page.statistics.contractStatus')">
            <Table :columns="statusColumns" :data-source="statusAnalysisData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>
