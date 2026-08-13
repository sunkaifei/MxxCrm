<script lang="ts" setup>
import { ref } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { LucideArrowRight } from '@vben/icons';
import { $t } from '#/locales';
import { getCustomerTypeStatsApi, getCustomerSourceStatsApi, getCustomerIndustryStatsApi, getCustomerFunnelApi } from '#/api/core/statistics';
import TimeFilter from '../components/time-filter.vue';

const customerTypeData = ref<any[]>([]);
const customerSourceData = ref<any[]>([]);
const customerIndustryData = ref<any[]>([]);
const funnelData = ref<any[]>([]);
const timeParams = ref<{ start_date?: string; end_date?: string; year?: number }>({});

const loadData = async () => {
  try {
    const [typeRes, sourceRes, industryRes, funnelRes] = await Promise.all([
      getCustomerTypeStatsApi(timeParams.value),
      getCustomerSourceStatsApi(timeParams.value),
      getCustomerIndustryStatsApi(timeParams.value),
      getCustomerFunnelApi(timeParams.value),
    ]);

    // requestClient.get 返回 { code, data, msg }，data 字段即为后端实际数据
    const typeList = Array.isArray(typeRes) ? typeRes : (typeRes as any)?.data ?? [];
    customerTypeData.value = typeList.map((item: any) => ({
      customerType: item.customer_type,
      totalCount: item.total_count,
      contractCount: item.contract_count,
      conversionRate: Number(item.conversion_rate) || 0,
    }));

    const sourceList = Array.isArray(sourceRes) ? sourceRes : (sourceRes as any)?.data ?? [];
    customerSourceData.value = sourceList.map((item: any) => ({
      source: item.source,
      totalCount: item.total_count,
      contractCount: item.contract_count,
      conversionRate: Number(item.conversion_rate) || 0,
    }));

    const industryList = Array.isArray(industryRes) ? industryRes : (industryRes as any)?.data ?? [];
    customerIndustryData.value = industryList.map((item: any) => ({
      industry: item.industry,
      totalCount: item.total_count,
      contractCount: item.contract_count,
      conversionRate: Number(item.conversion_rate) || 0,
      contractAmount: Number(item.contract_amount) || 0,
    }));

    // funnel API 返回 { total_leads, total_customers, total_opportunities, total_contracts, ... }
    const fd = (funnelRes as any)?.data ?? (funnelRes as any) ?? {};
    const totalLeads = Number(fd.total_leads) || 0;
    const totalCustomers = Number(fd.total_customers) || 0;
    const totalOpps = Number(fd.total_opportunities) || 0;
    const totalContracts = Number(fd.total_contracts) || 0;
    const maxVal = Math.max(totalLeads, totalCustomers, totalOpps, totalContracts, 1);
    funnelData.value = [
      { stage: $t('page.statistics.funnelStageNeedsAnalysis'), count: totalLeads, rate: totalLeads > 0 ? (totalLeads / maxVal * 100) : 0 },
      { stage: $t('page.statistics.funnelStageProposal'), count: totalCustomers, rate: totalCustomers > 0 ? (totalCustomers / maxVal * 100) : 0 },
      { stage: $t('page.statistics.funnelStageNegotiation'), count: totalOpps, rate: totalOpps > 0 ? (totalOpps / maxVal * 100) : 0 },
      { stage: $t('page.statistics.funnelStageWon'), count: totalContracts, rate: totalContracts > 0 ? (totalContracts / maxVal * 100) : 0 },
    ];
  } catch (e) {
    console.error($t('page.statistics.loadCustomerFailed'), e);
  }
};

function handleTimeChange(params: { start_date?: string; end_date?: string; year?: number }) {
  timeParams.value = params;
  loadData();
}

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const typeColumns = [
  { title: $t('page.statistics.customerTypeCol'), dataIndex: 'customerType' },
  { title: $t('page.statistics.totalCount'), dataIndex: 'totalCount', align: 'right' as const },
  { title: $t('page.statistics.contractCount'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'conversionRate', align: 'right' as const, customRender: ({ text }: any) => `${Number(text).toFixed(2)}%` },
];

const sourceColumns = [
  { title: $t('page.statistics.source'), dataIndex: 'source' },
  { title: $t('page.statistics.totalCount'), dataIndex: 'totalCount', align: 'right' as const },
  { title: $t('page.statistics.contractCount'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'conversionRate', align: 'right' as const, customRender: ({ text }: any) => `${Number(text).toFixed(2)}%` },
];

const industryColumns = [
  { title: $t('page.statistics.industryCol'), dataIndex: 'industry' },
  { title: $t('page.statistics.customerCount'), dataIndex: 'totalCount', align: 'right' as const },
  { title: $t('page.statistics.contractCount'), dataIndex: 'contractCount', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'conversionRate', align: 'right' as const, customRender: ({ text }: any) => `${Number(text).toFixed(2)}%` },
  { title: $t('page.statistics.contractAmount'), dataIndex: 'contractAmount', align: 'right' as const, customRender: ({ text }: any) => formatCurrency(text) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">{{ $t('page.statistics.customerConversion') }}</h2>

      <TimeFilter @change="handleTimeChange" />

      <Card :title="$t('page.statistics.customerFunnel')" class="mb-6">
        <div class="flex items-center justify-center py-8">
          <div class="w-full max-w-md space-y-4">
            <div v-for="(item, idx) in funnelData" :key="idx"
              class="flex items-center gap-4 p-3 rounded"
              :style="{ backgroundColor: '#f0f5ff', width: `${item.rate}%`, marginLeft: `${(100 - item.rate) / 2}%` }">
              <div class="flex-1">{{ item.stage }}</div>
              <div class="font-bold">{{ item.count }}</div>
              <div class="text-sm text-gray-500">{{ Number(item.rate).toFixed(2) }}%</div>
              <LucideArrowRight v-if="idx < funnelData.length - 1" class="w-4 h-4 text-gray-400" />
            </div>
          </div>
        </div>
      </Card>

      <Row :gutter="16" class="mt-4">
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
