<script lang="ts" setup>
import { onMounted, ref } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Statistic } from 'ant-design-vue';
import { LucideTarget, LucideUsers, LucideFileText, LucideWallet, LucideTrendingUp, LucidePieChart } from '@vben/icons';
import { getMonthlyPerformanceApi, getPaymentCompletionApi, getCustomerTypeStatsApi } from '#/api/core/statistics';

const summary = ref({
  contractTarget: 0,
  contractActual: 0,
  contractRate: 0,
  paymentTarget: 0,
  paymentActual: 0,
  paymentRate: 0,
  totalCustomers: 0,
  contractCustomers: 0,
  customerRate: 0,
  totalContracts: 0,
  totalAmount: 0,
  overdueAmount: 0,
  employeeCount: 0,
});

const loadData = async () => {
  try {
    const [performanceRes, paymentRes, customerRes] = await Promise.all([
      getMonthlyPerformanceApi({ year: new Date().getFullYear() }),
      getPaymentCompletionApi(),
      getCustomerTypeStatsApi(),
    ]);
    
    if (performanceRes.data && performanceRes.data.data) {
      const perf = performanceRes.data.data;
      summary.value.contractTarget = perf.total_contract_target || 0;
      summary.value.contractActual = perf.total_contract_actual || 0;
      summary.value.contractRate = perf.contract_completion_rate || 0;
      summary.value.paymentTarget = perf.total_payment_target || 0;
      summary.value.paymentActual = perf.total_payment_actual || 0;
      summary.value.paymentRate = perf.payment_completion_rate || 0;
    }
    
    if (paymentRes.data && paymentRes.data.data) {
      const pay = paymentRes.data.data;
      summary.value.totalAmount = pay.total_contract_amount || 0;
      summary.value.overdueAmount = pay.overdue_amount || 0;
    }
    
    if (customerRes.data && customerRes.data.data) {
      const customers = customerRes.data.data;
      summary.value.totalCustomers = customers.reduce((sum: number, item: any) => sum + (item.total_count || 0), 0);
      summary.value.contractCustomers = customers.reduce((sum: number, item: any) => sum + (item.contract_count || 0), 0);
      summary.value.customerRate = summary.value.totalCustomers > 0 
        ? Number(((summary.value.contractCustomers / summary.value.totalCustomers) * 100).toFixed(1))
        : 0;
    }
  } catch (e) {
    console.error('加载概览数据失败', e);
    summary.value = {
      contractTarget: 0,
      contractActual: 0,
      contractRate: 0,
      paymentTarget: 0,
      paymentActual: 0,
      paymentRate: 0,
      totalCustomers: 0,
      contractCustomers: 0,
      customerRate: 0,
      totalContracts: 0,
      totalAmount: 0,
      overdueAmount: 0,
      employeeCount: 0,
    };
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}万`;
}
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">数据分析概览</h2>
      
      <Row :gutter="16" class="mb-6">
        <Col :span="8">
          <Card hoverable>
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-lg bg-blue-50 flex items-center justify-center">
                <LucideTarget class="w-6 h-6 text-blue-500" />
              </div>
              <div>
                <div class="text-xs text-gray-500">合同目标完成</div>
                <div class="text-2xl font-bold text-blue-600">{{ summary.contractRate }}%</div>
                <div class="text-xs text-gray-400">{{ formatCurrency(summary.contractActual) }}/{{ formatCurrency(summary.contractTarget) }}</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col :span="8">
          <Card hoverable>
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-lg bg-green-50 flex items-center justify-center">
                <LucideWallet class="w-6 h-6 text-green-500" />
              </div>
              <div>
                <div class="text-xs text-gray-500">回款目标完成</div>
                <div class="text-2xl font-bold text-green-600">{{ summary.paymentRate }}%</div>
                <div class="text-xs text-gray-400">{{ formatCurrency(summary.paymentActual) }}/{{ formatCurrency(summary.paymentTarget) }}</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col :span="8">
          <Card hoverable>
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-lg bg-purple-50 flex items-center justify-center">
                <LucideUsers class="w-6 h-6 text-purple-500" />
              </div>
              <div>
                <div class="text-xs text-gray-500">客户转化率</div>
                <div class="text-2xl font-bold text-purple-600">{{ summary.customerRate }}%</div>
                <div class="text-xs text-gray-400">{{ summary.contractCustomers }}/{{ summary.totalCustomers }} 客户</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>
      
      <Row :gutter="16" class="mb-6">
        <Col :span="6">
          <Card size="small">
            <Statistic title="合同总额" :value="summary.totalAmount" prefix="¥" :precision="0" />
          </Card>
        </Col>
        <Col :span="6">
          <Card size="small">
            <Statistic title="逾期金额" :value="summary.overdueAmount" prefix="¥" :precision="0" />
          </Card>
        </Col>
        <Col :span="6">
          <Card size="small">
            <Statistic title="客户总数" :value="summary.totalCustomers" suffix="个" />
          </Card>
        </Col>
        <Col :span="6">
          <Card size="small">
            <Statistic title="成交客户" :value="summary.contractCustomers" suffix="个" />
          </Card>
        </Col>
      </Row>
      
      <Row :gutter="16">
        <Col :span="12">
          <Card title="业绩趋势" extra="月度">
            <div class="h-64 flex items-center justify-center text-gray-400">
              <LucideTrendingUp class="w-12 h-12" />
              <span class="ml-2">业绩趋势图表</span>
            </div>
          </Card>
        </Col>
        <Col :span="12">
          <Card title="客户分布" extra="按类型">
            <div class="h-64 flex items-center justify-center text-gray-400">
              <LucidePieChart class="w-12 h-12" />
              <span class="ml-2">客户分布图表</span>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>