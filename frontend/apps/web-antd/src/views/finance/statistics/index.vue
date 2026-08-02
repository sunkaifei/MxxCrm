<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Col,
  DatePicker,
  Empty,
  Row,
  Select,
  Spin,
  Statistic,
  Table,
  message,
} from 'ant-design-vue';
import { RefreshCw } from 'lucide-vue-next';
import dayjs from 'dayjs';

import {
  generateDailyStatisticsApi,
  getFinanceStatisticsListApi,
  getFinanceStatisticsSummaryApi,
} from '#/api';
import { $t } from '#/locales';

const RangePicker = DatePicker.RangePicker;

// ===== 汇总数据 =====
const summaryLoading = ref(false);
const summary = ref<any>({});

async function loadSummary() {
  summaryLoading.value = true;
  try {
    const res: any = await getFinanceStatisticsSummaryApi();
    summary.value = res?.data ?? res ?? {};
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.statistics.message.loadFailed'));
  } finally {
    summaryLoading.value = false;
  }
}

// ===== 列表查询 =====
const listLoading = ref(false);
const listData = ref<any[]>([]);
const queryParams = reactive({
  statType: undefined as number | undefined,
  dateRange: undefined as [dayjs.Dayjs, dayjs.Dayjs] | undefined,
});

const statTypeOptions = [
  { label: $t('page.finance.statistics.statType.daily'), value: 1 },
  { label: $t('page.finance.statistics.statType.weekly'), value: 2 },
  { label: $t('page.finance.statistics.statType.monthly'), value: 3 },
];

async function loadList() {
  listLoading.value = true;
  try {
    const params: any = {};
    if (queryParams.statType !== undefined) {
      params.statType = queryParams.statType;
    }
    if (queryParams.dateRange && queryParams.dateRange.length === 2) {
      params.startDate = queryParams.dateRange[0].format('YYYY-MM-DD');
      params.endDate = queryParams.dateRange[1].format('YYYY-MM-DD');
    }
    const res: any = await getFinanceStatisticsListApi(params);
    listData.value = res?.data ?? res ?? [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.statistics.message.loadFailed'));
  } finally {
    listLoading.value = false;
  }
}

// ===== 生成昨日统计 =====
const generating = ref(false);
async function handleGenerate() {
  generating.value = true;
  try {
    await generateDailyStatisticsApi();
    message.success($t('page.finance.statistics.message.generateSuccess'));
    await Promise.all([loadSummary(), loadList()]);
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.statistics.message.generateFailed'));
  } finally {
    generating.value = false;
  }
}

// ===== 表格列 =====
const columns = [
  {
    title: $t('page.finance.statistics.column.statDate'),
    dataIndex: 'statDate',
    width: 120,
  },
  {
    title: $t('page.finance.statistics.column.statType'),
    dataIndex: 'statType',
    width: 100,
    customRender: ({ text }: any) => {
      const map: Record<number, string> = {
        1: $t('page.finance.statistics.statType.daily'),
        2: $t('page.finance.statistics.statType.weekly'),
        3: $t('page.finance.statistics.statType.monthly'),
      };
      return map[text] || text;
    },
  },
  {
    title: $t('page.finance.statistics.column.totalIncome'),
    dataIndex: 'totalIncome',
    width: 140,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.statistics.column.successAmount'),
    dataIndex: 'successAmount',
    width: 140,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.statistics.column.refundAmount'),
    dataIndex: 'refundAmount',
    width: 140,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.statistics.column.memberFeeAmount'),
    dataIndex: 'memberFeeAmount',
    width: 140,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥${Number(text || 0).toFixed(2)}`,
  },
  {
    title: $t('page.finance.statistics.column.orderCount'),
    dataIndex: 'orderCount',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.statistics.column.successCount'),
    dataIndex: 'successCount',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.statistics.column.refundCount'),
    dataIndex: 'refundCount',
    width: 100,
    align: 'right' as const,
  },
];

onMounted(() => {
  loadSummary();
  loadList();
});
</script>

<template>
  <Page :title="$t('page.finance.statistics.title')">
    <!-- 汇总卡片 -->
    <Spin :spinning="summaryLoading">
      <Row :gutter="16" class="mb-4">
        <Col :xs="24" :sm="12" :md="6">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.totalIncome')"
              :value="summary.totalIncome || 0"
              :precision="2"
              prefix="¥"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="6">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.successAmount')"
              :value="summary.successAmount || 0"
              :precision="2"
              prefix="¥"
              :value-style="{ color: '#3f8600' }"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="6">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.refundAmount')"
              :value="summary.refundAmount || 0"
              :precision="2"
              prefix="¥"
              :value-style="{ color: '#cf1322' }"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="12" :md="6">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.memberFeeAmount')"
              :value="summary.memberFeeAmount || 0"
              :precision="2"
              prefix="¥"
              :value-style="{ color: '#1890ff' }"
            />
          </Card>
        </Col>
      </Row>
      <Row :gutter="16" class="mb-4">
        <Col :xs="24" :sm="8">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.orderCount')"
              :value="summary.orderCount || 0"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="8">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.successCount')"
              :value="summary.successCount || 0"
            />
          </Card>
        </Col>
        <Col :xs="24" :sm="8">
          <Card>
            <Statistic
              :title="$t('page.finance.statistics.summary.refundCount')"
              :value="summary.refundCount || 0"
            />
          </Card>
        </Col>
      </Row>
    </Spin>

    <!-- 列表 -->
    <Card>
      <template #title>
        <div class="flex items-center justify-between">
          <span>{{ $t('page.finance.statistics.listTitle') }}</span>
          <div class="flex items-center gap-2">
            <Select
              v-model:value="queryParams.statType"
              :options="statTypeOptions"
              :placeholder="$t('page.finance.statistics.placeholder.statType')"
              allow-clear
              style="width: 140px"
              @change="loadList"
            />
            <RangePicker
              v-model:value="queryParams.dateRange"
              :placeholder="[
                $t('page.finance.statistics.placeholder.startDate'),
                $t('page.finance.statistics.placeholder.endDate'),
              ]"
              @change="loadList"
            />
            <Button :loading="generating" type="primary" @click="handleGenerate">
              <template #icon>
                <RefreshCw />
              </template>
              {{ $t('page.finance.statistics.button.generate') }}
            </Button>
          </div>
        </div>
      </template>
      <Table
        :columns="columns"
        :data-source="listData"
        :loading="listLoading"
        :pagination="false"
        row-key="id"
        size="middle"
        :scroll="{ x: 1200 }"
      >
        <template #emptyText>
          <Empty :description="$t('page.finance.statistics.message.empty')" />
        </template>
      </Table>
    </Card>
  </Page>
</template>
