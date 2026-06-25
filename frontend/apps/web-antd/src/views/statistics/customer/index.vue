<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { LucideUsers, LucidePieChart, LucideArrowRight } from '@vben/icons';
import { getCustomerTypeStatsApi, getCustomerSourceStatsApi, getCustomerIndustryStatsApi, getCustomerFunnelApi } from '#/api/core/statistics';

const customerTypeData = ref<any[]>([]);
const customerSourceData = ref<any[]>([]);
const customerIndustryData = ref<any[]>([]);
const funnelData = ref<any[]>([]);

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
        stage: item.stage,
        count: item.count,
        rate: item.rate,
      }));
    }
  } catch (e) {
    console.error('加载客户转化数据失败', e);
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
  return `¥${(val / 10000).toFixed(1)}万`;
}

const typeColumns = [
  { title: '客户类型', dataIndex: 'customerType' },
  { title: '总数', dataIndex: 'totalCount', align: 'right' },
  { title: '成交数', dataIndex: 'contractCount', align: 'right' },
  { title: '转化率', dataIndex: 'conversionRate', align: 'right', render: (val: number) => `${val}%` },
];

const sourceColumns = [
  { title: '来源渠道', dataIndex: 'source' },
  { title: '总数', dataIndex: 'totalCount', align: 'right' },
  { title: '成交数', dataIndex: 'contractCount', align: 'right' },
  { title: '转化率', dataIndex: 'conversionRate', align: 'right', render: (val: number) => `${val}%` },
];

const industryColumns = [
  { title: '行业', dataIndex: 'industry' },
  { title: '客户数', dataIndex: 'totalCount', align: 'right' },
  { title: '成交数', dataIndex: 'contractCount', align: 'right' },
  { title: '转化率', dataIndex: 'conversionRate', align: 'right', render: (val: number) => `${val}%` },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">客户转化分析</h2>
      
      <Card title="客户转化漏斗" class="mb-6">
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
          <Card title="客户类型分布">
            <Table :columns="typeColumns" :data-source="customerTypeData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="8">
          <Card title="客户来源分析">
            <Table :columns="sourceColumns" :data-source="customerSourceData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="8">
          <Card title="行业分布">
            <Table :columns="industryColumns" :data-source="customerIndustryData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>