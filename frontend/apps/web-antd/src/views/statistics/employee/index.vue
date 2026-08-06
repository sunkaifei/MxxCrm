<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { Card, Row, Col, Table } from 'ant-design-vue';
import { $t } from '#/locales';
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
        employeeName: item.employeeName,
        departmentName: item.departmentName,
        totalCustomers: item.totalCustomers,
        newCustomersThisMonth: item.newCustomersThisMonth,
        contractCustomers: item.contractCustomers,
        customerConversionRate: item.customerConversionRate,
      }));
    }
    
    if (followRes.data && followRes.data.data) {
      followUpData.value = followRes.data.data.map((item: any) => ({
        employeeName: item.employeeName,
        departmentName: item.departmentName,
        totalFollowUp: item.totalFollowUp,
        customerFollowUp: item.customerFollowUp,
        opportunityFollowUp: item.opportunityFollowUp,
        avgFollowInterval: item.avgFollowInterval,
        customersWithoutFollow30Days: item.customersWithoutFollow30Days,
      }));
    }
    
    if (conversionRes.data && conversionRes.data.data) {
      conversionData.value = conversionRes.data.data.map((item: any) => ({
        employeeName: item.employeeName,
        departmentName: item.departmentName,
        totalOpportunities: item.totalOpportunities,
        wonOpportunities: item.wonOpportunities,
        lostOpportunities: item.lostOpportunities,
        opportunityWinRate: item.opportunityWinRate,
        totalContracts: item.totalContracts,
        contractAmount: item.contractAmount,
        avgContractAmount: item.avgContractAmount,
        avgSalesCycleDays: item.avgSalesCycleDays,
      }));
    }
  } catch (e) {
    console.error($t('page.statistics.loadEmployeeFailed'), e);
    customerCountData.value = [];
    followUpData.value = [];
    conversionData.value = [];
  }
};

onMounted(() => {
  loadData();
});

function formatCurrency(val: number) {
  return `¥${(val / 10000).toFixed(1)}${$t('page.statistics.currencyFormat')}`;
}

const customerColumns = [
  { title: $t('page.statistics.employee'), dataIndex: 'employeeName' },
  { title: $t('page.statistics.department'), dataIndex: 'departmentName' },
  { title: $t('page.statistics.totalCustomers'), dataIndex: 'totalCustomers', align: 'right' as const },
  { title: $t('page.statistics.newCustomersThisMonth'), dataIndex: 'newCustomersThisMonth', align: 'right' as const },
  { title: $t('page.statistics.contractCustomers'), dataIndex: 'contractCustomers', align: 'right' as const },
  { title: $t('page.statistics.conversionRate'), dataIndex: 'customerConversionRate', align: 'right' as const, render: (val: number) => `${val}%` },
];

const followUpColumns = [
  { title: $t('page.statistics.employee'), dataIndex: 'employeeName' },
  { title: $t('page.statistics.department'), dataIndex: 'departmentName' },
  { title: $t('page.statistics.totalFollowUp'), dataIndex: 'totalFollowUp', align: 'right' as const },
  { title: $t('page.statistics.customerFollowUp'), dataIndex: 'customerFollowUp', align: 'right' as const },
  { title: $t('page.statistics.opportunityFollowUp'), dataIndex: 'opportunityFollowUp', align: 'right' as const },
  { title: $t('page.statistics.avgFollowInterval'), dataIndex: 'avgFollowInterval', align: 'right' as const },
];

const conversionColumns = [
  { title: $t('page.statistics.employee'), dataIndex: 'employeeName' },
  { title: $t('page.statistics.department'), dataIndex: 'departmentName' },
  { title: $t('page.statistics.totalOpportunities'), dataIndex: 'totalOpportunities', align: 'right' as const },
  { title: $t('page.statistics.wonOpportunities'), dataIndex: 'wonOpportunities', align: 'right' as const },
  { title: $t('page.statistics.lostOpportunities'), dataIndex: 'lostOpportunities', align: 'right' as const },
  { title: $t('page.statistics.opportunityWinRate'), dataIndex: 'opportunityWinRate', align: 'right' as const, render: (val: number) => `${val}%` },
  { title: $t('page.statistics.contractAmount'), dataIndex: 'contractAmount', align: 'right' as const, render: (val: number) => formatCurrency(val) },
];
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">{{ $t('page.statistics.employeeStats') }}</h2>
      
      <Card :title="$t('page.statistics.employeeCustomerCount')" class="mb-6">
        <Table :columns="customerColumns" :data-source="customerCountData" :pagination="false" />
      </Card>
      
      <Row :gutter="16">
        <Col :span="12">
          <Card :title="$t('page.statistics.followUpAnalysis')">
            <Table :columns="followUpColumns" :data-source="followUpData" :pagination="false" size="small" />
          </Card>
        </Col>
        <Col :span="12">
          <Card :title="$t('page.statistics.conversionAnalysis')">
            <Table :columns="conversionColumns" :data-source="conversionData" :pagination="false" size="small" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>