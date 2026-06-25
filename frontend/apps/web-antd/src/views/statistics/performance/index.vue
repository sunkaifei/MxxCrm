<script lang="ts" setup>
import { ref, computed, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Progress, Table, Tag } from 'ant-design-vue';
import { LucideTarget, LucideTrendingUp } from '@vben/icons';
import { getMonthlyPerformanceApi, getPerformanceRankingApi } from '#/api/core/statistics';

const selectedYear = ref(new Date().getFullYear());
const months = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
const monthlyData = ref<any[]>([]);
const rankingData = ref<any[]>([]);

const loadData = async () => {
  try {
    const [monthlyRes, rankingRes] = await Promise.all([
      getMonthlyPerformanceApi({ year: selectedYear.value }),
      getPerformanceRankingApi({ year: selectedYear.value, order_by: 'contract_amount' }),
    ]);
    
    if (monthlyRes.data && monthlyRes.data.data) {
      const stats = monthlyRes.data.data;
      monthlyData.value = stats.months || [];
    }
    
    if (rankingRes.data && rankingRes.data.data) {
      rankingData.value = rankingRes.data.data.map((item: any) => ({
        rank: item.rank,
        employeeName: item.employee_name,
        departmentName: item.department_name,
        contractAmount: item.contract_amount,
        contractTarget: item.contract_target,
        paymentAmount: item.payment_amount,
        paymentTarget: item.payment_target,
        completionRate: item.contract_completion_rate,
      }));
    }
  } catch (e) {
    console.error('加载业绩数据失败', e);
    monthlyData.value = [];
    rankingData.value = [];
  }
};

onMounted(() => {
  loadData();
});

const totalContractTarget = computed(() => {
  return monthlyData.value.reduce((sum, m) => sum + (m.contract_target || 0), 0);
});

const totalContractActual = computed(() => {
  return monthlyData.value.reduce((sum, m) => sum + (m.contract_actual || 0), 0);
});

const totalPaymentTarget = computed(() => {
  return monthlyData.value.reduce((sum, m) => sum + (m.payment_target || 0), 0);
});

const totalPaymentActual = computed(() => {
  return monthlyData.value.reduce((sum, m) => sum + (m.payment_actual || 0), 0);
});

function getRate(target: number, actual: number) {
  return target > 0 ? ((actual / target) * 100).toFixed(1) : '0';
}

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}万`;
}

const columns = [
  { title: '排名', dataIndex: 'rank', width: 60 },
  { title: '员工', dataIndex: 'employeeName' },
  { title: '部门', dataIndex: 'departmentName' },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '合同目标', dataIndex: 'contractTarget', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '合同完成率', dataIndex: 'completionRate', align: 'right', render: (val: number) => `${val}%` },
  { title: '回款金额', dataIndex: 'paymentAmount', align: 'right', render: (val: number) => formatCurrency(val) },
  { title: '回款目标', dataIndex: 'paymentTarget', align: 'right', render: (val: number) => formatCurrency(val) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">业绩目标统计</h2>
      
      <div class="flex items-center gap-3 mb-4">
        <select v-model="selectedYear" @change="loadData" class="px-3 py-1 border rounded">
          <option :value="new Date().getFullYear() - 2">{{ new Date().getFullYear() - 2 }}年</option>
          <option :value="new Date().getFullYear() - 1">{{ new Date().getFullYear() - 1 }}年</option>
          <option :value="new Date().getFullYear()">{{ new Date().getFullYear() }}年</option>
        </select>
      </div>
      
      <Row :gutter="16" class="mb-6">
        <Col :span="12">
          <Card title="合同目标完成" :extra="`${getRate(totalContractTarget, totalContractActual)}%`">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm text-gray-500">目标金额</div>
                <div class="text-xl font-bold">{{ formatCurrency(totalContractTarget) }}</div>
              </div>
              <div class="text-right">
                <div class="text-sm text-gray-500">实际完成</div>
                <div class="text-xl font-bold text-blue-600">{{ formatCurrency(totalContractActual) }}</div>
              </div>
            </div>
            <Progress :percent="Number(getRate(totalContractTarget, totalContractActual))" :stroke-width="12" class="mt-3" />
          </Card>
        </Col>
        <Col :span="12">
          <Card title="回款目标完成" :extra="`${getRate(totalPaymentTarget, totalPaymentActual)}%`">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm text-gray-500">目标金额</div>
                <div class="text-xl font-bold">{{ formatCurrency(totalPaymentTarget) }}</div>
              </div>
              <div class="text-right">
                <div class="text-sm text-gray-500">实际完成</div>
                <div class="text-xl font-bold text-green-600">{{ formatCurrency(totalPaymentActual) }}</div>
              </div>
            </div>
            <Progress :percent="Number(getRate(totalPaymentTarget, totalPaymentActual))" :stroke-width="12" class="mt-3" strokeColor="#52c41a" />
          </Card>
        </Col>
      </Row>
      
      <Card title="月度业绩" class="mb-6">
        <div class="grid grid-cols-12 gap-2">
          <div v-for="item in monthlyData" :key="item.month" class="text-center p-2 bg-gray-50 rounded">
            <div class="text-xs text-gray-500 mb-1">{{ item.month }}月</div>
            <div class="text-xs text-blue-600">{{ getRate(item.contract_target, item.contract_actual) }}%</div>
            <div class="h-8 mt-1">
              <div class="h-full bg-blue-200 rounded" :style="{ width: `${getRate(item.contract_target, item.contract_actual)}%` }"></div>
            </div>
          </div>
        </div>
      </Card>
      
      <Card title="员工业绩排行">
        <Table :columns="columns" :data-source="rankingData" :pagination="false" />
      </Card>
    </div>
  </Page>
</template>