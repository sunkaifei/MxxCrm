<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { getEmployeeCustomerCountApi, getEmployeeFollowUpApi, getEmployeeConversionApi } from '#/api/core/statistics';

const customerCountData = ref<any[]>([]);
const followUpData = ref<any[]>([]);
const conversionData = ref<any[]>([]);

const loadData = async () => {
  try {
    const [customerRes, followRes, conversionRes] = await Promise.all([
      getEmployeeCustomerCountApi(),
      getEmployeeFollowUpApi(),
      getEmployeeConversionApi(),
    ]);
    
    if (customerRes.data && customerRes.data.data) {
      customerCountData.value = customerRes.data.data.map((item: any) => ({
        employeeName: item.employee_name,
        departmentName: item.department_name,
        totalCustomers: item.total_customers,
        newCustomersThisMonth: item.new_customers_this_month,
        contractCustomers: item.contract_customers,
        customerConversionRate: item.customer_conversion_rate,
      }));
    }
    
    if (followRes.data && followRes.data.data) {
      followUpData.value = followRes.data.data.map((item: any) => ({
        employeeName: item.employee_name,
        departmentName: item.department_name,
        totalFollowUp: item.total_follow_up,
        customerFollowUp: item.customer_follow_up,
        opportunityFollowUp: item.opportunity_follow_up,
        avgFollowInterval: item.avg_follow_interval,
        customersWithoutFollow30Days: item.customers_without_follow_30_days,
      }));
    }
    
    if (conversionRes.data && conversionRes.data.data) {
      conversionData.value = conversionRes.data.data.map((item: any) => ({
        employeeName: item.employee_name,
        departmentName: item.department_name,
        totalOpportunities: item.total_opportunities,
        wonOpportunities: item.won_opportunities,
        lostOpportunities: item.lost_opportunities,
        opportunityWinRate: item.opportunity_win_rate,
        totalContracts: item.total_contracts,
        contractAmount: item.contract_amount,
        avgContractAmount: item.avg_contract_amount,
        avgSalesCycleDays: item.avg_sales_cycle_days,
      }));
    }
  } catch (e) {
    console.error('加载员工统计数据失败', e);
    customerCountData.value = [];
    followUpData.value = [];
    conversionData.value = [];
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}万`;
}

const customerColumns = [
  { title: '员工', dataIndex: 'employeeName' },
  { title: '部门', dataIndex: 'departmentName' },
  { title: '客户总数', dataIndex: 'totalCustomers', align: 'right' },
  { title: '本月新增', dataIndex: 'newCustomersThisMonth', align: 'right' },
  { title: '成交客户', dataIndex: 'contractCustomers', align: 'right' },
  { title: '转化率', dataIndex: 'customerConversionRate', align: 'right', render: (val: number) => `${val}%` },
];

const followUpColumns = [
  { title: '员工', dataIndex: 'employeeName' },
  { title: '部门', dataIndex: 'departmentName' },
  { title: '总跟进次数', dataIndex: 'totalFollowUp', align: 'right' },
  { title: '客户跟进', dataIndex: 'customerFollowUp', align: 'right' },
  { title: '商机跟进', dataIndex: 'opportunityFollowUp', align: 'right' },
  { title: '平均跟进间隔(天)', dataIndex: 'avgFollowInterval', align: 'right' },
];

const conversionColumns = [
  { title: '员工', dataIndex: 'employeeName' },
  { title: '部门', dataIndex: 'departmentName' },
  { title: '商机总数', dataIndex: 'totalOpportunities', align: 'right' },
  { title: '赢单', dataIndex: 'wonOpportunities', align: 'right' },
  { title: '输单', dataIndex: 'lostOpportunities', align: 'right' },
  { title: '赢单率', dataIndex: 'opportunityWinRate', align: 'right', render: (val: number) => `${val}%` },
  { title: '合同金额', dataIndex: 'contractAmount', align: 'right', render: (val: number) => formatCurrency(val) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">员工统计分析</h2>
      
      <Card title="员工客户量统计" class="mb-6">
        <Table :columns="customerColumns" :data-source="customerCountData" :pagination="false" />
      </Card>
      
      <Row :gutter="16">
        <Col :span="12">
          <Card title="跟进频次分析">
            <Table :columns="followUpColumns" :data-source="followUpData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="12">
          <Card title="成交率分析">
            <Table :columns="conversionColumns" :data-source="conversionData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>