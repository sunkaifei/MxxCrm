<script lang="ts" setup>
import { computed, onMounted, reactive, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Card,
  Col,
  Empty,
  Form,
  FormItem,
  message,
  Modal,
  Row,
  Select,
  Statistic,
  Table,
  Tabs,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import {
  calculateTeamCommissionApi,
  getAllocationLogApi,
  getPendingCommissionApi,
  getTeamCommissionListApi,
  getTeamCommissionSummaryApi,
} from '#/api/core/finance';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import AllocateModal from './AllocateModal.vue';
import CommissionPoolPanel from '../commission-pool/CommissionPoolPanel.vue';

const guideStepCount = 5;
const accessStore = useAccessStore();
const hasManagePermission = computed(() =>
  accessStore.hasAccessCode('finance:team-commission:manage'),
);

const formatMoney = (val: any) => Number(val || 0).toFixed(2);

const now = new Date();
// 默认展示上个月，工资核算在每月1号进行
const lastMonth = now.getMonth() === 0 ? 12 : now.getMonth();
const defaultYear =
  now.getMonth() === 0 ? now.getFullYear() - 1 : now.getFullYear();

const searchForm = reactive({
  year: defaultYear,
  month: lastMonth,
});

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: $t('page.finance.teamCommission.monthLabel', { month: i + 1 }),
}));

// ===== 列表数据 =====
const activeTab = ref('list');
const loading = ref(false);
const tableData = ref<any[]>([]);
const summaryData = ref<any[]>([]);

async function loadList() {
  loading.value = true;
  try {
    const [listRes, summaryRes] = await Promise.all([
      getTeamCommissionListApi(searchForm),
      getTeamCommissionSummaryApi(searchForm),
    ]);
    const list = listRes?.data || listRes || [];
    tableData.value = Array.isArray(list)
      ? list
      : list?.items || list?.list || [];
    const summary = summaryRes?.data || summaryRes || [];
    summaryData.value = Array.isArray(summary) ? summary : [];
  } catch (e: any) {
    message.error(
      e?.message || $t('page.finance.teamCommission.message.loadFailed'),
    );
    tableData.value = [];
    summaryData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 汇总统计 =====
const totalCommission = computed(() =>
  tableData.value.reduce(
    (sum, item) => sum + Number(item.teamCommissionAmount || 0),
    0,
  ),
);
const totalTeamPayment = computed(() =>
  summaryData.value.reduce(
    (sum, item) => sum + Number(item.teamTotalPayment || 0),
    0,
  ),
);
const managerCount = computed(() => summaryData.value.length);

// ===== 计算团队提成 =====
const calcVisible = ref(false);
const calcLoading = ref(false);

async function handleCalculate() {
  calcLoading.value = true;
  try {
    const res: any = await calculateTeamCommissionApi(searchForm);
    const count = res?.data ?? res;
    message.success(
      $t('page.finance.teamCommission.modal.calcSuccess', { count }),
    );
    calcVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(
      e?.message || $t('page.finance.teamCommission.message.calcFailed'),
    );
  } finally {
    calcLoading.value = false;
  }
}

// ===== 状态映射 =====
const statusMap: Record<number, { label: string; color: string }> = {
  0: {
    label: $t('page.finance.teamCommission.status.pending'),
    color: 'default',
  },
  1: {
    label: $t('page.finance.teamCommission.status.approved'),
    color: 'processing',
  },
  2: {
    label: $t('page.finance.teamCommission.status.paid'),
    color: 'success',
  },
};

// 状态颜色映射（用于 customRender 返回样式）
const statusColorMap: Record<string, string> = {
  default: '#8c8c8c',
  processing: '#1890ff',
  success: '#52c41a',
  error: '#ff4d4f',
};

// ===== 列表表格列 =====
const listColumns: ColumnsType = [
  {
    title: $t('page.finance.teamCommission.column.employee'),
    dataIndex: 'employeeName',
    width: 120,
  },
  {
    title: $t('page.finance.teamCommission.column.department'),
    dataIndex: 'departmentName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.teamCommission.column.yearMonth'),
    key: 'yearMonth',
    width: 100,
    customRender: ({ record }: any) => `${record.year}-${record.month}`,
  },
  {
    title: $t('page.finance.teamCommission.column.baseSalary'),
    dataIndex: 'baseSalary',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.teamCommission.column.teamCommissionAmount'),
    dataIndex: 'teamCommissionAmount',
    width: 130,
    align: 'right',
    customRender: ({ text }: any) => ({
      children: formatMoney(text),
      style: { color: '#1890ff', fontWeight: 500 },
    }),
  },
  {
    title: $t('page.finance.teamCommission.column.totalSalary'),
    dataIndex: 'totalSalary',
    width: 130,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.teamCommission.column.status'),
    dataIndex: 'status',
    width: 100,
    customRender: ({ text }: any) => {
      const s = statusMap[text as number];
      return s
        ? {
            children: s.label,
            style: { color: statusColorMap[s.color] },
          }
        : '-';
    },
  },
];

// ===== 汇总表格列 =====
const summaryColumns: ColumnsType = [
  {
    title: $t('page.finance.teamCommission.column.manager'),
    dataIndex: 'managerName',
    width: 120,
  },
  {
    title: $t('page.finance.teamCommission.column.teamTotalPayment'),
    dataIndex: 'teamTotalPayment',
    width: 150,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.teamCommission.column.commissionAmount'),
    dataIndex: 'commissionAmount',
    width: 130,
    align: 'right',
    customRender: ({ text }: any) => ({
      children: formatMoney(text),
      style: { color: '#1890ff', fontWeight: 500 },
    }),
  },
  {
    title: $t('page.finance.teamCommission.column.subordinatesCount'),
    dataIndex: 'subordinatesCount',
    width: 100,
    align: 'right',
  },
  {
    title: $t('page.finance.teamCommission.column.rate'),
    key: 'rate',
    width: 100,
    align: 'right',
    customRender: ({ record }: any) => {
      const payment = Number(record.teamTotalPayment || 0);
      const commission = Number(record.commissionAmount || 0);
      if (payment > 0) {
        return `${((commission / payment) * 100).toFixed(2)}%`;
      }
      return '-';
    },
  },
];

// ===== Tab 2: 待分配列表 =====
const pendingLoading = ref(false);
const pendingData = ref<any[]>([]);
const allocateVisible = ref(false);
const allocateRecord = ref<any>(null);

const pendingColumns: ColumnsType = [
  {
    title: $t('page.finance.teamCommission.column.beneficiary'),
    dataIndex: 'userName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.teamCommission.column.ruleName'),
    dataIndex: 'ruleName',
    width: 160,
    ellipsis: true,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.teamCommission.column.yearMonth'),
    key: 'yearMonth',
    width: 100,
    customRender: ({ record }: any) =>
      `${record.periodYear || record.year || '-'}-${record.periodMonth || record.month || '-'}`,
  },
  {
    title: $t('page.finance.teamCommission.column.pendingTotal'),
    dataIndex: 'commissionAmount',
    width: 140,
    align: 'right',
    customRender: ({ text }: any) => ({
      children: `¥${formatMoney(text)}`,
      style: { color: '#fa8c16', fontWeight: 600 },
    }),
  },
  {
    title: $t('page.finance.teamCommission.column.allocatedAmount'),
    dataIndex: 'allocatedAmount',
    width: 140,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.teamCommission.column.allocateStatus'),
    dataIndex: 'allocateStatus',
    width: 110,
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 100,
    fixed: 'right',
  },
];

async function loadPending() {
  pendingLoading.value = true;
  try {
    const res: any = await getPendingCommissionApi(searchForm);
    const list = res?.data || res || [];
    pendingData.value = Array.isArray(list)
      ? list
      : list?.items || list?.list || [];
  } catch (e: any) {
    message.error(
      e?.message || $t('page.finance.teamCommission.message.loadFailed'),
    );
    pendingData.value = [];
  } finally {
    pendingLoading.value = false;
  }
}

function openAllocate(record: any) {
  allocateRecord.value = record;
  allocateVisible.value = true;
}

function handleAllocateClose(success?: boolean) {
  allocateVisible.value = false;
  allocateRecord.value = null;
  if (success) {
    loadPending();
  }
}

// ===== Tab 4: 分配记录 =====
const logLoading = ref(false);
const logData = ref<any[]>([]);

const allocateMethodMap: Record<number, string> = {
  1: $t('page.finance.teamCommission.allocate.methodAverage'),
  2: $t('page.finance.teamCommission.allocate.methodByPerformance'),
  3: $t('page.finance.teamCommission.allocate.methodManual'),
};

const logColumns: ColumnsType = [
  {
    title: $t('page.finance.teamCommission.column.allocator'),
    dataIndex: 'allocatorName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.teamCommission.column.employee'),
    dataIndex: 'employeeName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.teamCommission.column.yearMonth'),
    key: 'yearMonth',
    width: 100,
    customRender: ({ record }: any) =>
      `${record.year || '-'}-${record.month || '-'}`,
  },
  {
    title: $t('page.finance.teamCommission.allocate.amount'),
    dataIndex: 'amount',
    width: 130,
    align: 'right',
    customRender: ({ text }: any) => ({
      children: `¥${formatMoney(text)}`,
      style: { color: '#52c41a', fontWeight: 500 },
    }),
  },
  {
    title: $t('page.finance.teamCommission.column.allocateMethod'),
    dataIndex: 'allocateMethod',
    width: 120,
    customRender: ({ text }: any) =>
      allocateMethodMap[text as number] || '-',
  },
  {
    title: $t('page.finance.teamCommission.allocate.payment'),
    dataIndex: 'employeePayment',
    width: 130,
    align: 'right',
    customRender: ({ text }: any) => (text ? formatMoney(text) : '-'),
  },
  {
    title: $t('page.finance.teamCommission.column.allocateTime'),
    dataIndex: 'createTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
];

async function loadAllocationLog() {
  logLoading.value = true;
  try {
    const res: any = await getAllocationLogApi(searchForm);
    const list = res?.data || res || [];
    logData.value = Array.isArray(list)
      ? list
      : list?.items || list?.list || [];
  } catch (e: any) {
    message.error(
      e?.message || $t('page.finance.teamCommission.message.loadFailed'),
    );
    logData.value = [];
  } finally {
    logLoading.value = false;
  }
}

// ===== Tab 切换懒加载 =====
watch(
  activeTab,
  (val) => {
    if (val === 'pending' && pendingData.value.length === 0) {
      loadPending();
    } else if (val === 'allocationLog' && logData.value.length === 0) {
      loadAllocationLog();
    }
  },
  { immediate: false },
);

onMounted(() => {
  loadList();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.teamCommission.guide.title')"
      :brief="$t('page.finance.teamCommission.guide.brief')"
      :expand-text="$t('page.finance.teamCommission.guide.expand')"
      :collapse-text="$t('page.finance.teamCommission.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.teamCommission.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.teamCommission.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>

    <!-- 搜索栏 -->
    <Card class="mb-4" :bordered="false">
      <Form layout="inline">
        <FormItem :label="$t('page.finance.teamCommission.column.yearMonth')">
          <Select
            v-model:value="searchForm.year"
            :options="
              Array.from({ length: 10 }, (_, i) => ({
                value: defaultYear - 5 + i,
                label: $t('page.finance.teamCommission.yearLabel', {
                  year: defaultYear - 5 + i,
                }),
              }))
            "
            style="width: 120px"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.common.month')">
          <Select
            v-model:value="searchForm.month"
            :options="monthOptions"
            style="width: 120px"
          />
        </FormItem>
        <FormItem>
          <Button type="primary" @click="loadList">
            {{ $t('page.finance.teamCommission.button.query') }}
          </Button>
        </FormItem>
      </Form>
    </Card>

    <!-- 汇总统计 -->
    <Row :gutter="16" class="mb-4">
      <Col :span="8">
        <Card :bordered="false">
          <Statistic
            :title="$t('page.finance.teamCommission.statistic.managerCount')"
            :value="managerCount"
            :suffix="
              $t('page.finance.teamCommission.statistic.managerCountSuffix')
            "
          />
        </Card>
      </Col>
      <Col :span="8">
        <Card :bordered="false">
          <Statistic
            :title="
              $t('page.finance.teamCommission.statistic.totalTeamPayment')
            "
            :value="formatMoney(totalTeamPayment)"
            prefix="¥"
            :value-style="{ color: '#1890ff' }"
          />
        </Card>
      </Col>
      <Col :span="8">
        <Card :bordered="false">
          <Statistic
            :title="
              $t('page.finance.teamCommission.statistic.totalCommission')
            "
            :value="formatMoney(totalCommission)"
            prefix="¥"
            :value-style="{ color: '#52c41a' }"
          />
        </Card>
      </Col>
    </Row>

    <Card :bordered="false" :title="$t('page.finance.teamCommission.title')">
      <template #extra>
        <Button
          v-if="hasManagePermission"
          type="primary"
          @click="calcVisible = true"
        >
          {{ $t('page.finance.teamCommission.button.calculate') }}
        </Button>
      </template>

      <Tabs v-model:activeKey="activeTab">
        <!-- Tab 1: 提成明细 -->
        <Tabs.TabPane
          key="list"
          :tab="$t('page.finance.teamCommission.tab.list')"
        >
          <Table
            :columns="listColumns"
            :data-source="tableData"
            :loading="loading"
            row-key="id"
            size="small"
            :pagination="{
              pageSize: 20,
              showSizeChanger: true,
              showTotal: (t: number) =>
                $t('page.finance.common.total', { count: t }),
            }"
          >
            <template #emptyText>
              <Empty
                :description="$t('page.finance.teamCommission.empty.list')"
              />
            </template>
          </Table>
        </Tabs.TabPane>

        <!-- Tab 2: 按管理者汇总 -->
        <Tabs.TabPane
          key="summary"
          :tab="$t('page.finance.teamCommission.tab.summary')"
        >
          <Table
            :columns="summaryColumns"
            :data-source="summaryData"
            :loading="loading"
            row-key="managerId"
            size="small"
            :pagination="false"
          >
            <template #emptyText>
              <Empty
                :description="$t('page.finance.teamCommission.empty.summary')"
              />
            </template>
          </Table>
        </Tabs.TabPane>

        <!-- Tab 3: 待分配 -->
        <Tabs.TabPane
          key="pending"
          :tab="$t('page.finance.teamCommission.tab.pending')"
        >
          <div class="mb-3" style="text-align: right">
            <Button type="primary" size="small" @click="loadPending">
              {{ $t('page.finance.common.refresh') }}
            </Button>
          </div>
          <Table
            :columns="pendingColumns"
            :data-source="pendingData"
            :loading="pendingLoading"
            row-key="id"
            size="small"
            :pagination="{
              pageSize: 20,
              showSizeChanger: true,
              showTotal: (t: number) =>
                $t('page.finance.common.total', { count: t }),
            }"
          >
            <template #emptyText>
              <Empty
                :description="$t('page.finance.teamCommission.empty.pending')"
              />
            </template>
            <template #bodyCell="{ column, record }">
              <template v-if="column.dataIndex === 'allocateStatus'">
                <Tag
                  :color="
                    record.allocateStatus === 2 ? 'success' : 'processing'
                  "
                >
                  {{
                    record.allocateStatus === 2
                      ? $t('page.finance.teamCommission.allocateStatus.allocated')
                      : $t('page.finance.teamCommission.allocateStatus.pending')
                  }}
                </Tag>
              </template>
              <template v-else-if="column.key === 'action'">
                <Button
                  v-if="record.allocateStatus !== 2"
                  type="link"
                  size="small"
                  @click="openAllocate(record)"
                >
                  {{ $t('page.finance.teamCommission.allocate.btn') }}
                </Button>
                <span v-else style="color: #8c8c8c">-</span>
              </template>
            </template>
          </Table>
        </Tabs.TabPane>

        <!-- Tab 4: 团建资金池 -->
        <Tabs.TabPane
          key="pool"
          :tab="$t('page.finance.teamCommission.tab.pool')"
        >
          <CommissionPoolPanel embedded />
        </Tabs.TabPane>

        <!-- Tab 5: 分配记录 -->
        <Tabs.TabPane
          key="allocationLog"
          :tab="$t('page.finance.teamCommission.tab.allocationLog')"
        >
          <div class="mb-3" style="text-align: right">
            <Button type="primary" size="small" @click="loadAllocationLog">
              {{ $t('page.finance.common.refresh') }}
            </Button>
          </div>
          <Table
            :columns="logColumns"
            :data-source="logData"
            :loading="logLoading"
            row-key="id"
            size="small"
            :pagination="{
              pageSize: 20,
              showSizeChanger: true,
              showTotal: (t: number) =>
                $t('page.finance.common.total', { count: t }),
            }"
          >
            <template #emptyText>
              <Empty
                :description="
                  $t('page.finance.teamCommission.empty.allocationLog')
                "
              />
            </template>
          </Table>
        </Tabs.TabPane>
      </Tabs>
    </Card>

    <!-- 计算确认弹窗 -->
    <Modal
      v-model:open="calcVisible"
      :title="$t('page.finance.teamCommission.modal.calcTitle')"
      :confirm-loading="calcLoading"
      @ok="handleCalculate"
    >
      <p class="py-4">
        {{
          $t('page.finance.teamCommission.modal.calcContent', {
            year: searchForm.year,
            month: searchForm.month,
          })
        }}
      </p>
      <p class="text-gray-500">
        {{ $t('page.finance.teamCommission.modal.calcLogic') }}
      </p>
    </Modal>

    <!-- 分配弹窗 -->
    <AllocateModal
      :visible="allocateVisible"
      :record="allocateRecord"
      @close="handleAllocateClose"
    />
  </Page>
</template>
