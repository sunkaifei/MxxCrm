<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Progress, Table } from 'ant-design-vue';
import { getPaymentCompletionApi, getPaymentMonthlyTrendApi, getPaymentStatusAnalysisApi, getPaymentRankingApi } from '#/api/core/statistics';

const completionData = ref<any>({});
const monthlyTrendData = ref<any[]>([]);
const statusAnalysisData = ref<any[]>([]);
const rankingData = ref<any[]>([]);

const loadData = async () => {
  try {
    const [completionRes, trendRes, statusRes, rankingRes] = await Promise.all([
      getPaymentCompletionApi(),
      getPaymentMonthlyTrendApi(),
      getPaymentStatusAnalysisApi(),
      getPaymentRankingApi({ order_by: 'payment_amount', limit: 10 }),
    ]);
    
    if (completionRes.data && completionRes.data.data) {
      completionData.value = {
        year: completionRes.data.data.year,
        totalContractAmount: completionRes.data.data.total_contract_amount,
        totalPaymentAmount: completionRes.data.data.total_payment_amount,
        completionRate: completionRes.data.data.completion_rate,
        overdueAmount: completionRes.data.data.overdue_amount,
        overdueRate: completionRes.data.data.overdue_rate,
        unpaidAmount: completionRes.data.data.unpaid_amount,
        unpaidRate: completionRes.data.data.unpaid_rate,
      };
    }
    
    if (trendRes.data && trendRes.data.data && trendRes.data.data.months) {
      monthlyTrendData.value = trendRes.data.data.months.map((item: any) => ({
        month: item.month,
        contractAmount: item.contract_amount,
        paymentAmount: item.payment_amount,
        completionRate: item.completion_rate,
        overdueAmount: item.overdue_amount,
      }));
    }
    
    if (statusRes.data && statusRes.data.data) {
      statusAnalysisData.value = statusRes.data.data.map((item: any) => ({
        status: item.status_name,
        contractCount: item.contract_count,
        contractAmount: item.contract_amount,
        paidAmount: item.paid_amount,
        percentage: item.percentage,
      }));
    }
    
    if (rankingRes.data && rankingRes.data.data) {
      rankingData.value = rankingRes.data.data.map((item: any) => ({
        rank: item.rank,
        targetName: item.target_name,
        contractAmount: item.contract_amount,
        paymentAmount: item.payment_amount,
        completionRate: item.completion_rate,
        overdueAmount: item.overdue_amount,
      }));
    }
  } catch (e) {
    console.error('加载回款分析数据失败', e);
    completionData.value = {};
    monthlyTrendData.value = [];
    statusAnalysisData.value = [];
    rankingData.value = [];
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}万`;
}

const trendColumns = [
  { title: '月份', dataIndex: 'month', render: (val: number) => `${val}月` },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '已回款', dataIndex: 'paymentAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '回款率', dataIndex: 'completionRate', align: 'center', render: (val: number) => `${val}%` },
  { title: '逾期金额', dataIndex: 'overdueAmount', align: 'right', render: (val: number) => formatCurrency(val) },
];

const statusColumns = [
  { title: '状态', dataIndex: 'status' },
  { title: '合同数', dataIndex: 'contractCount', align: 'right' },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '已回款', dataIndex: 'paidAmount', align: 'right', render: (val: number) => val ? formatCurrency(val) : '-' },
  { title: '占比', dataIndex: 'percentage', align: 'right', render: (val: number) => `${val}%` },
];

const rankingColumns = [
  { title: '排名', dataIndex: 'rank', width: 60 },
  { title: '客户名称', dataIndex: 'targetName' },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '已回款', dataIndex: 'paymentAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '回款率', dataIndex: 'completionRate', align: 'right', render: (val: number) => `${val}%` },
  { title: '逾期金额', dataIndex: 'overdueAmount', align: 'right', render: (val: number) => formatCurrency(val) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">回款分析</h2>
      
      <Row :gutter="16" class="mb-6">
        <Col :span="12">
          <Card title="回款完成率" :extra="`${completionData.completionRate}%`">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm text-gray-500">合同总额</div>
                <div class="text-xl font-bold">{{ formatCurrency(completionData.totalContractAmount) }}</div>
              </div>
              <div class="text-right">
                <div class="text-sm text-gray-500">已回款</div>
                <div class="text-xl font-bold text-green-600">{{ formatCurrency(completionData.totalPaymentAmount) }}</div>
              </div>
            </div>
            <Progress :percent="completionData.completionRate" :stroke-width="12" class="mt-3" strokeColor="#52c41a" />
          </Card>
        </Col>
        <Col :span="12">
          <Card title="回款状态">
            <div class="space-y-3">
              <div class="flex justify-between">
                <span>未回款</span>
                <span class="text-red-500">{{ formatCurrency(completionData.unpaidAmount) }} ({{ completionData.unpaidRate }}%)</span>
              </div>
              <div class="flex justify-between">
                <span>逾期金额</span>
                <span class="text-orange-500">{{ formatCurrency(completionData.overdueAmount) }} ({{ completionData.overdueRate }}%)</span>
              </div>
            </div>
          </Card>
        </Col>
      </Row>
      
      <Card title="月度回款趋势" class="mb-6">
        <Table :columns="trendColumns" :data-source="monthlyTrendData" :pagination="false" />
      </Card>
      
      <Row :gutter="16">
        <Col :span="12">
          <Card title="回款状态分析">
            <Table :columns="statusColumns" :data-source="statusAnalysisData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="12">
          <Card title="回款排行">
            <Table :columns="rankingColumns" :data-source="rankingData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>