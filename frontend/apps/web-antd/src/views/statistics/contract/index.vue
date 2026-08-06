<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { $t } from '#/locales';
import { getContractRankingApi, getContractTypeDistributionApi, getContractStatusAnalysisApi } from '#/api/core/statistics';

const rankingData = ref<any[]>([]);
const typeDistributionData = ref<any[]>([]);
const statusAnalysisData = ref<any[]>([]);

const loadData = async () => {
  try {
    const [rankingRes, typeRes, statusRes] = await Promise.all([
      getContractRankingApi({ order_by: 'amount', limit: 10 }),
      getContractTypeDistributionApi(),
      getContractStatusAnalysisApi(),
    ]);
    
    if (rankingRes.data && rankingRes.data.data) {
      rankingData.value = rankingRes.data.data.map((item: any) => ({
        rank: item.rank,
        targetName: item.target_name,
        contractCount: item.contract_count,
        contractAmount: item.contract_amount,
        paymentAmount: item.payment_amount,
        paymentRate: item.payment_rate,
      }));
    }
    
    if (typeRes.data && typeRes.data.data) {
      typeDistributionData.value = typeRes.data.data.map((item: any) => ({
        contractType: item.contract_type,
        contractCount: item.contract_count,
        contractAmount: item.contract_amount,
        percentage: item.percentage,
      }));
    }
    
    if (statusRes.data && statusRes.data.data) {
      statusAnalysisData.value = statusRes.data.data.map((item: any) => ({
        status: item.status_name,
        contractCount: item.contract_count,
        contractAmount: item.contract_amount,
        percentage: item.percentage,
      }));
    }
  } catch (e) {
    console.error($t('page.statistics.loadContractFailed'), e);
    rankingData.value = [];
    typeDistributionData.value = [];
    statusAnalysisData.value = [];
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const rankingColumns = [
  { title: $t('page.statistics.rank'), dataIndex: 'rank', width: 60 },
  { title: $t('page.statistics.customerName'), dataIndex: 'targetName' },
  { title: $t('page.statistics.contractCountCol'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.contractAmount'), dataIndex: 'contractAmount', align: 'right' as const, render: (val: number) => formatCurrency(val) },
  { title: $t('page.statistics.paidAmount'), dataIndex: 'paymentAmount', align: 'right' as const, render: (val: number) => formatCurrency(val) },
  { title: $t('page.statistics.paymentRate'), dataIndex: 'paymentRate', align: 'right' as const, render: (val: number) => `${val}%` },
];

const typeColumns = [
  { title: $t('page.statistics.contractTypeCol'), dataIndex: 'contractType' },
  { title: $t('page.statistics.count'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.amount'), dataIndex: 'contractAmount', align: 'right' as const, render: (val: number) => formatCurrency(val) },
  { title: $t('page.statistics.percentage'), dataIndex: 'percentage', align: 'right' as const, render: (val: number) => `${val}%` },
];

const statusColumns = [
  { title: $t('page.statistics.status'), dataIndex: 'status' },
  { title: $t('page.statistics.count'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.amount'), dataIndex: 'contractAmount', align: 'right' as const, render: (val: number) => formatCurrency(val) },
  { title: $t('page.statistics.percentage'), dataIndex: 'percentage', align: 'right' as const, render: (val: number) => `${val}%` },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">{{ $t('page.statistics.contractRanking') }}</h2>
      
      <Card :title="$t('page.statistics.contractRankingTitle')" class="mb-6">
        <Table :columns="rankingColumns" :data-source="rankingData" :pagination="false" />
      </Card>
      
      <Row :gutter="16">
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