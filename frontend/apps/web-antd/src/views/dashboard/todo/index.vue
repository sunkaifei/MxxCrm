<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Badge,
  Button,
  Card,
  Empty,
  Input,
  Pagination,
  Select,
  Spin,
  Table,
  Tabs,
  Tag,
} from 'ant-design-vue';

import {
  getTodoApprovalListApi,
  getTodoContractListApi,
  getTodoFollowUpListApi,
  getTodoOpportunityListApi,
  getTodoPaymentListApi,
  getTodoSummaryApi,
} from '#/api';

const activeTab = ref('approval');
const loading = ref(false);
const summary = ref<any>({});

// 各 Tab 的数据
const approvalData = ref<any>({ items: [], total: 0 });
const followUpData = ref<any>({ items: [], total: 0 });
const paymentData = ref<any>({ items: [], total: 0 });
const contractData = ref<any>({ items: [], total: 0 });
const opportunityData = ref<any>({ items: [], total: 0 });

// 分页
const approvalPage = ref({ pageNum: 1, pageSize: 10 });
const followUpPage = ref({ pageNum: 1, pageSize: 10 });
const paymentPage = ref({ pageNum: 1, pageSize: 10 });
const contractPage = ref({ pageNum: 1, pageSize: 10 });
const opportunityPage = ref({ pageNum: 1, pageSize: 10 });

// 过滤
const approvalFilter = ref({ businessType: undefined, status: undefined, businessTitle: '' });
const followUpFilter = ref({ itemType: 'all', rangeType: 'all' });

// 审批状态映射
const approvalStatusMap: Record<number, { label: string; color: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
};

const businessTypeMap: Record<string, { label: string; color: string }> = {
  quotation: { label: '报价单', color: 'blue' },
  order: { label: '订单', color: 'cyan' },
  contract: { label: '合同', color: 'geekblue' },
  payment: { label: '回款', color: 'gold' },
  invoice: { label: '发票', color: 'purple' },
};

// 统计卡片配置
const summaryCards = computed(() => [
  {
    key: 'overdue',
    title: '逾期跟进',
    value: summary.value.overdueFollowUp || 0,
    color: '#ff4d4f',
    bg: '#fff2f0',
    icon: '🔔',
    tab: 'followUp',
  },
  {
    key: 'today',
    title: '今日待跟进',
    value: summary.value.todayFollowUp || 0,
    color: '#faad14',
    bg: '#fffbe6',
    icon: '📅',
    tab: 'followUp',
  },
  {
    key: 'approval',
    title: '待我审批',
    value: summary.value.pendingApproval || 0,
    color: '#1890ff',
    bg: '#e6f7ff',
    icon: '📋',
    tab: 'approval',
  },
  {
    key: 'payment',
    title: '待回款提醒',
    value: summary.value.pendingPayment || 0,
    color: '#722ed1',
    bg: '#f9f0ff',
    icon: '💰',
    tab: 'payment',
  },
  {
    key: 'contract',
    title: '即将到期合同',
    value: summary.value.expiringContract || 0,
    color: '#13c2c2',
    bg: '#e6fffb',
    icon: '📜',
    tab: 'contract',
  },
  {
    key: 'opportunity',
    title: '停滞商机',
    value: summary.value.stagnantOpportunity || 0,
    color: '#eb2f96',
    bg: '#fff0f6',
    icon: '⚠️',
    tab: 'opportunity',
  },
]);

// 审批待办列
const approvalColumns = [
  { title: '业务标题', dataIndex: 'businessTitle', ellipsis: true, width: 200 },
  { title: '类型', dataIndex: 'businessType', width: 100 },
  { title: '提交人', dataIndex: 'submitterName', width: 120 },
  { title: '提交时间', dataIndex: 'submittedAt', width: 170 },
  { title: '状态', dataIndex: 'status', width: 100 },
  { title: '候选审批人', dataIndex: 'candidateApprovers', width: 200 },
];

// 跟进待办列
const followUpColumns = [
  { title: '名称', dataIndex: 'name', ellipsis: true, width: 200 },
  { title: '类型', dataIndex: 'itemType', width: 100 },
  { title: '计划跟进时间', dataIndex: 'nextFollowAt', width: 170 },
  { title: '逾期', dataIndex: 'overdueDays', width: 100 },
];

// 待回款列
const paymentColumns = [
  { title: '阶段', dataIndex: 'stageName', width: 150 },
  { title: '计划金额', dataIndex: 'planAmount', width: 120 },
  { title: '已收金额', dataIndex: 'receivedAmount', width: 120 },
  { title: '计划日期', dataIndex: 'planDate', width: 120 },
  { title: '剩余天数', dataIndex: 'remainingDays', width: 100 },
];

// 合同到期列
const contractColumns = [
  { title: '合同编号', dataIndex: 'contractNo', width: 150 },
  { title: '标题', dataIndex: 'title', ellipsis: true, width: 200 },
  { title: '到期日期', dataIndex: 'endDate', width: 120 },
  { title: '剩余天数', dataIndex: 'remainingDays', width: 100 },
];

// 停滞商机列
const opportunityColumns = [
  { title: '商机名称', dataIndex: 'title', ellipsis: true, width: 200 },
  { title: '阶段', dataIndex: 'stageName', width: 120 },
  { title: '最后更新', dataIndex: 'updateTime', width: 170 },
  { title: '停滞天数', dataIndex: 'stagnantDays', width: 100 },
];

function formatOverdue(days: number) {
  if (days > 0) return { text: `逾期${days}天`, color: 'red' };
  if (days === 0) return { text: '今日', color: 'orange' };
  return { text: `${-days}天后`, color: 'blue' };
}

function formatRemaining(days: number) {
  if (days < 0) return { text: `逾期${-days}天`, color: 'red' };
  if (days === 0) return { text: '今日到期', color: 'orange' };
  return { text: `${days}天后`, color: 'blue' };
}

async function loadSummary() {
  try {
    summary.value = await getTodoSummaryApi();
  } catch {
    // 忽略
  }
}

async function loadApproval() {
  loading.value = true;
  try {
    approvalData.value = await getTodoApprovalListApi({
      pageNum: approvalPage.value.pageNum,
      pageSize: approvalPage.value.pageSize,
      businessType: approvalFilter.value.businessType,
      status: approvalFilter.value.status,
      businessTitle: approvalFilter.value.businessTitle || undefined,
    });
  } finally {
    loading.value = false;
  }
}

async function loadFollowUp() {
  loading.value = true;
  try {
    followUpData.value = await getTodoFollowUpListApi({
      pageNum: followUpPage.value.pageNum,
      pageSize: followUpPage.value.pageSize,
      itemType: followUpFilter.value.itemType,
      rangeType: followUpFilter.value.rangeType,
    });
  } finally {
    loading.value = false;
  }
}

async function loadPayment() {
  loading.value = true;
  try {
    paymentData.value = await getTodoPaymentListApi({
      pageNum: paymentPage.value.pageNum,
      pageSize: paymentPage.value.pageSize,
      days: 7,
    });
  } finally {
    loading.value = false;
  }
}

async function loadContract() {
  loading.value = true;
  try {
    contractData.value = await getTodoContractListApi({
      pageNum: contractPage.value.pageNum,
      pageSize: contractPage.value.pageSize,
      days: 30,
    });
  } finally {
    loading.value = false;
  }
}

async function loadOpportunity() {
  loading.value = true;
  try {
    opportunityData.value = await getTodoOpportunityListApi({
      pageNum: opportunityPage.value.pageNum,
      pageSize: opportunityPage.value.pageSize,
      days: 30,
    });
  } finally {
    loading.value = false;
  }
}

function handleTabChange(key: string) {
  activeTab.value = key;
  if (key === 'approval' && approvalData.value.items.length === 0) loadApproval();
  if (key === 'followUp' && followUpData.value.items.length === 0) loadFollowUp();
  if (key === 'payment' && paymentData.value.items.length === 0) loadPayment();
  if (key === 'contract' && contractData.value.items.length === 0) loadContract();
  if (key === 'opportunity' && opportunityData.value.items.length === 0) loadOpportunity();
}

function jumpToTab(tab: string) {
  activeTab.value = tab;
  handleTabChange(tab);
}

onMounted(() => {
  loadSummary();
  loadApproval();
});
</script>

<template>
  <div class="todo-center p-4">
    <!-- 顶部统计卡片区 -->
    <div class="summary-cards grid grid-cols-2 gap-3 mb-4 md:grid-cols-3 lg:grid-cols-6">
      <Card
        v-for="card in summaryCards"
        :key="card.key"
        hover-class="summary-card-hover"
        class="summary-card cursor-pointer border-0 transition-all hover:shadow-lg"
        :style="{ background: card.bg }"
        @click="jumpToTab(card.tab)"
      >
        <div class="flex items-center justify-between p-2">
          <div>
            <div class="text-xs text-gray-500">{{ card.title }}</div>
            <div class="text-2xl font-bold mt-1" :style="{ color: card.color }">
              {{ card.value }}
            </div>
          </div>
          <div class="text-2xl">{{ card.icon }}</div>
        </div>
      </Card>
    </div>

    <!-- 主体内容区 -->
    <Card class="todo-main-card">
      <Tabs v-model:active-key="activeTab" @change="handleTabChange">
        <!-- 审批待办 Tab -->
        <Tabs.TabPane key="approval" tab="审批待办">
          <div class="filter-bar mb-3 flex items-center gap-3 flex-wrap">
            <Input
              v-model:value="approvalFilter.businessTitle"
              placeholder="搜索业务标题"
              allow-clear
              style="width: 200px"
              @press-enter="loadApproval"
            />
            <Select
              v-model:value="approvalFilter.businessType"
              placeholder="业务类型"
              allow-clear
              style="width: 140px"
              :options="[
                { value: 'quotation', label: '报价单' },
                { value: 'order', label: '订单' },
                { value: 'contract', label: '合同' },
                { value: 'payment', label: '回款' },
                { value: 'invoice', label: '发票' },
              ]"
              @change="loadApproval"
            />
            <Select
              v-model:value="approvalFilter.status"
              placeholder="状态"
              allow-clear
              style="width: 120px"
              :options="[
                { value: 1, label: '待审批' },
                { value: 2, label: '审批中' },
                { value: 3, label: '已通过' },
                { value: 4, label: '已驳回' },
              ]"
              @change="loadApproval"
            />
            <Button type="primary" @click="loadApproval">查询</Button>
          </div>
          <Spin :spinning="loading">
            <Table
              :columns="approvalColumns"
              :data-source="approvalData.items"
              :pagination="false"
              row-key="id"
              size="small"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.dataIndex === 'businessType'">
                  <Tag :color="businessTypeMap[record.businessType]?.color || 'default'">
                    {{ businessTypeMap[record.businessType]?.label || record.businessType }}
                  </Tag>
                </template>
                <template v-else-if="column.dataIndex === 'submittedAt'">
                  {{ formatDateTime(record.submittedAt) }}
                </template>
                <template v-else-if="column.dataIndex === 'status'">
                  <Tag :color="approvalStatusMap[record.status]?.color || 'default'">
                    {{ approvalStatusMap[record.status]?.label || '未知' }}
                  </Tag>
                </template>
                <template v-else-if="column.dataIndex === 'candidateApprovers'">
                  <div v-if="record.candidateApproverNames?.length > 0" class="flex flex-wrap gap-1">
                    <span
                      v-for="(name, idx) in record.candidateApproverNames"
                      :key="idx"
                      class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs"
                      :class="record.processedApprovers?.includes(record.candidateApprovers?.[idx]) ? 'bg-green-100 text-green-700' : 'bg-blue-50 text-blue-600'"
                    >
                      {{ name }}
                    </span>
                  </div>
                  <span v-else class="text-gray-400">-</span>
                </template>
              </template>
            </Table>
            <Pagination
              v-if="approvalData.total > 0"
              class="mt-3 text-right"
              :current="approvalPage.pageNum"
              :page-size="approvalPage.pageSize"
              :total="approvalData.total"
              show-size-changer
              show-quick-jumper
              @change="(p, ps) => { approvalPage.pageNum = p; approvalPage.pageSize = ps; loadApproval(); }"
            />
          </Spin>
        </Tabs.TabPane>

        <!-- 跟进待办 Tab -->
        <Tabs.TabPane key="followUp" tab="跟进待办">
          <div class="filter-bar mb-3 flex items-center gap-3">
            <Select
              v-model:value="followUpFilter.itemType"
              style="width: 120px"
              :options="[
                { value: 'all', label: '全部' },
                { value: 'customer', label: '客户' },
                { value: 'lead', label: '线索' },
              ]"
              @change="loadFollowUp"
            />
            <Select
              v-model:value="followUpFilter.rangeType"
              style="width: 120px"
              :options="[
                { value: 'all', label: '全部' },
                { value: 'overdue', label: '逾期' },
                { value: 'today', label: '今日' },
              ]"
              @change="loadFollowUp"
            />
          </div>
          <Spin :spinning="loading">
            <Table
              :columns="followUpColumns"
              :data-source="followUpData.items"
              :pagination="false"
              row-key="id"
              size="small"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.dataIndex === 'itemType'">
                  <Tag :color="record.itemType === 'customer' ? 'blue' : 'green'">
                    {{ record.itemType === 'customer' ? '客户' : '线索' }}
                  </Tag>
                </template>
                <template v-else-if="column.dataIndex === 'nextFollowAt'">
                  {{ formatDateTime(record.nextFollowAt) }}
                </template>
                <template v-else-if="column.dataIndex === 'overdueDays'">
                  <Tag :color="formatOverdue(record.overdueDays).color">
                    {{ formatOverdue(record.overdueDays).text }}
                  </Tag>
                </template>
              </template>
            </Table>
            <Pagination
              v-if="followUpData.total > 0"
              class="mt-3 text-right"
              :current="followUpPage.pageNum"
              :page-size="followUpPage.pageSize"
              :total="followUpData.total"
              show-size-changer
              show-quick-jumper
              @change="(p, ps) => { followUpPage.pageNum = p; followUpPage.pageSize = ps; loadFollowUp(); }"
            />
          </Spin>
        </Tabs.TabPane>

        <!-- 待回款提醒 Tab -->
        <Tabs.TabPane key="payment" tab="待回款提醒">
          <Spin :spinning="loading">
            <Table
              :columns="paymentColumns"
              :data-source="paymentData.items"
              :pagination="false"
              row-key="id"
              size="small"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.dataIndex === 'remainingDays'">
                  <Tag :color="formatRemaining(record.remainingDays).color">
                    {{ formatRemaining(record.remainingDays).text }}
                  </Tag>
                </template>
              </template>
            </Table>
            <Pagination
              v-if="paymentData.total > 0"
              class="mt-3 text-right"
              :current="paymentPage.pageNum"
              :page-size="paymentPage.pageSize"
              :total="paymentData.total"
              show-size-changer
              show-quick-jumper
              @change="(p, ps) => { paymentPage.pageNum = p; paymentPage.pageSize = ps; loadPayment(); }"
            />
          </Spin>
        </Tabs.TabPane>

        <!-- 合同到期 Tab -->
        <Tabs.TabPane key="contract" tab="合同到期">
          <Spin :spinning="loading">
            <Table
              :columns="contractColumns"
              :data-source="contractData.items"
              :pagination="false"
              row-key="id"
              size="small"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.dataIndex === 'remainingDays'">
                  <Tag :color="formatRemaining(record.remainingDays).color">
                    {{ formatRemaining(record.remainingDays).text }}
                  </Tag>
                </template>
              </template>
            </Table>
            <Pagination
              v-if="contractData.total > 0"
              class="mt-3 text-right"
              :current="contractPage.pageNum"
              :page-size="contractPage.pageSize"
              :total="contractData.total"
              show-size-changer
              show-quick-jumper
              @change="(p, ps) => { contractPage.pageNum = p; contractPage.pageSize = ps; loadContract(); }"
            />
          </Spin>
        </Tabs.TabPane>

        <!-- 停滞商机 Tab -->
        <Tabs.TabPane key="opportunity" tab="停滞商机">
          <Spin :spinning="loading">
            <Table
              :columns="opportunityColumns"
              :data-source="opportunityData.items"
              :pagination="false"
              row-key="id"
              size="small"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.dataIndex === 'updateTime'">
                  {{ formatDateTime(record.updateTime) }}
                </template>
                <template v-else-if="column.dataIndex === 'stagnantDays'">
                  <Tag color="red">停滞{{ record.stagnantDays }}天</Tag>
                </template>
              </template>
            </Table>
            <Pagination
              v-if="opportunityData.total > 0"
              class="mt-3 text-right"
              :current="opportunityPage.pageNum"
              :page-size="opportunityPage.pageSize"
              :total="opportunityData.total"
              show-size-changer
              show-quick-jumper
              @change="(p, ps) => { opportunityPage.pageNum = p; opportunityPage.pageSize = ps; loadOpportunity(); }"
            />
          </Spin>
        </Tabs.TabPane>
      </Tabs>
    </Card>
  </div>
</template>

<style scoped>
.todo-center {
  min-height: calc(100vh - 64px);
}

.summary-card {
  border-radius: 8px;
  transition: all 0.3s ease;
}

.summary-card:hover {
  transform: translateY(-2px);
}

.todo-main-card {
  border-radius: 8px;
}

.todo-main-card :deep(.ant-card-body) {
  padding: 16px 24px;
}

.filter-bar {
  padding: 4px 0;
}
</style>
