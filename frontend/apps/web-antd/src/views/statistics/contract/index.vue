<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
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
    console.error('加载合同统计数据失败', e);
    rankingData.value = [];
    typeDistributionData.value = [];
    statusAnalysisData.value = [];
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}万`;
}

const rankingColumns = [
  { title: '排名', dataIndex: 'rank', width: 60 },
  { title: '客户名称', dataIndex: 'targetName' },
  { title: '合同数', dataIndex: 'contractCount', align: 'right' },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '已回款', dataIndex: 'paymentAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '回款率', dataIndex: 'paymentRate', align: 'right', render: (val: number) => `${val}%` },
];

const typeColumns = [
  { title: '合同类型', dataIndex: 'contractType' },
  { title: '数量', dataIndex: 'contractCount', align: 'right' },
  { title: '金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '占比', dataIndex: 'percentage', align: 'right', render: (val: number) => `${val}%` },
];

const statusColumns = [
  { title: '状态', dataIndex: 'status' },
  { title: '数量', dataIndex: 'contractCount', align: 'right' },
  { title: '金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '占比', dataIndex: 'percentage', align: 'right', render: (val: number) => `${val}%` },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">合同排行分析</h2>
      
      <Card title="合同排行" class="mb-6">
        <Table :columns="rankingColumns" :data-source="rankingData" :pagination="false" />
      </Card>
      
      <Row :gutter="16">
        <Col :span="12">
          <Card title="合同类型分布">
            <Table :columns="typeColumns" :data-source="typeDistributionData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="12">
          <Card title="合同状态分析">
            <Table :columns="statusColumns" :data-source="statusAnalysisData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>