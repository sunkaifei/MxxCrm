<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { LucideArrowRight } from '@vben/icons';
import { $t } from '#/locales';
import { getCustomerTypeStatsApi, getCustomerSourceStatsApi, getCustomerIndustryStatsApi, getCustomerFunnelApi } from '#/api/core/statistics';

const customerTypeData = ref<any[]>([]);
const customerSourceData = ref<any[]>([]);
const customerIndustryData = ref<any[]>([]);
const funnelData = ref<any[]>([]);

const funnelStageLabels: Record<number, string> = {
  0: $t('page.statistics.funnelStageQualifications'),
  1: $t('page.statistics.funnelStageNeedsAnalysis'),
  2: $t('page.statistics.funnelStageProposal'),
  3: $t('page.statistics.funnelStageNegotiation'),
  4: $t('page.statistics.funnelStageWon'),
  5: $t('page.statistics.funnelStageLost'),
};

const loadData = async () => {
  try {
    const [typeRes, sourceRes, industryRes, funnelRes] = await Promise.all([
      getCustomerTypeStatsApi(),
      getCustomerSourceStatsApi(),
      getCustomerIndustryStatsApi(),
      getCustomerFunnelApi(),
    ]);
    
    if (typeRes.data && typeRes.data.data) {
      customerTypeData.value = typeRes.data.data.map((item: any) => ({
        customerType: item.customer_type,
        totalCount: item.total_count,
        contractCount: item.contract_count,
        conversionRate: item.conversion_rate,
      }));
    }
    
    if (sourceRes.data && sourceRes.data.data) {
      customerSourceData.value = sourceRes.data.data.map((item: any) => ({
        source: item.source,
        totalCount: item.total_count,
        contractCount: item.contract_count,
        conversionRate: item.conversion_rate,
      }));
    }
    
    if (industryRes.data && industryRes.data.data) {
      customerIndustryData.value = industryRes.data.data.map((item: any) => ({
        industry: item.industry,
        totalCount: item.total_count,
        contractCount: item.contract_count,
        conversionRate: item.conversion_rate,
        contractAmount: item.contract_amount,
      }));
    }
    
    if (funnelRes.data && funnelRes.data.data && funnelRes.data.data.funnel) {
      funnelData.value = funnelRes.data.data.funnel.map((item: any) => ({
        stage: funnelStageLabels[item.stage] || item.stage,
        count: item.count,
        rate: item.rate,
      }));
    }
  } catch (e) {
    console.error($t('page.statistics.loadCustomerFailed'), e);
    customerTypeData.value = [];
    customerSourceData.value = [];
    customerIndustryData.value = [];
    funnelData.value = [];
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const typeColumns = [
  { title: $t('page.statistics.customerTypeCol'), dataIndex: 'customerType' },
  { title: $t('page.statistics.totalCount'), dataIndex: 'totalCount', align: 'right' as const },
  { title: $t('page.statistics.contractCount'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'conversionRate', align: 'right' as const, render: (val: number) => `${val}%` },
];

const sourceColumns = [
  { title: $t('page.statistics.source'), dataIndex: 'source' },
  { title: $t('page.statistics.totalCount'), dataIndex: 'totalCount', align: 'right' as const },
  { title: $t('page.statistics.contractCount'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'conversionRate', align: 'right' as const, render: (val: number) => `${val}%` },
];

const industryColumns = [
  { title: $t('page.statistics.industryCol'), dataIndex: 'industry' },
  { title: $t('page.statistics.customerCount'), dataIndex: 'totalCount', align: 'right' as const },
  { title: $t('page.statistics.contractCount'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'conversionRate', align: 'right' as const, render: (val: number) => `${val}%` },
  { title: $t('page.statistics.contractAmount'), dataIndex: 'contractAmount', align: 'right' as const, render: (val: number) => formatCurrency(val) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">{{ $t('page.statistics.customerConversion') }}</h2>
      
      <Card :title="$t('page.statistics.customerFunnel')" class="mb-6">
        <div class="flex items-center justify-center py-8">
          <div class="w-full max-w-md space-y-4">
            <div v-for="(item, idx) in funnelData" :key="item.stage" 
              class="flex items-center gap-4 p-3 rounded"
              :style="{ backgroundColor: '#f0f5ff', width: `${item.rate}%`, marginLeft: `${(100 - item.rate) / 2}%` }">
              <div class="flex-1">{{ item.stage }}</div>
              <div class="font-bold">{{ item.count }}</div>
              <div class="text-sm text-gray-500">{{ item.rate }}%</div>
              <LucideArrowRight v-if="idx < funnelData.length - 1" class="w-4 h-4 text-gray-400" />
            </div>
          </div>
        </div>
      </Card>
      
      <Row :gutter="16">
        <Col :span="8">
          <Card :title="$t('page.statistics.customerType')">
            <Table :columns="typeColumns" :data-source="customerTypeData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="8">
          <Card :title="$t('page.statistics.customerSource')">
            <Table :columns="sourceColumns" :data-source="customerSourceData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="8">
          <Card :title="$t('page.statistics.industry')">
            <Table :columns="industryColumns" :data-source="customerIndustryData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>