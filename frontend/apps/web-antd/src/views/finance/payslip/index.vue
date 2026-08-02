<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Card,
  Checkbox,
  CheckboxGroup,
  Col,
  Drawer,
  Empty,
  Form,
  FormItem,
  InputNumber,
  message,
  Modal,
  Row,
  Select,
  Spin,
  Statistic,
  Table,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import {
  batchSendPayslipsApi,
  generatePayslipsApi,
  getPayslipListApi,
  getPayslipStatisticsApi,
  sendPayslipApi,
} from '#/api/core/finance';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';

const guideStepCount = 5;

const accessStore = useAccessStore();

const formatMoney = (val: any) => Number(val || 0).toFixed(2);

const now = new Date();
const searchForm = reactive({
  year: now.getFullYear(),
  month: now.getMonth() + 1,
});

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: `${i + 1}${$t('page.finance.common.month')}`,
}));

// ===== 发送状态映射 =====
const sendStatusMap: Record<number, { label: string; color: string }> = {
  0: { label: $t('page.finance.payslip.sendStatus.unsent'), color: 'default' },
  1: { label: $t('page.finance.payslip.sendStatus.sent'), color: 'processing' },
  2: { label: $t('page.finance.payslip.sendStatus.read'), color: 'success' },
};

// ===== 发送通道选项 =====
const channelOptions = [
  { label: $t('page.finance.payslip.channel.inbox'), value: 'site' },
  { label: $t('page.finance.payslip.channel.email'), value: 'email' },
  { label: $t('page.finance.payslip.channel.sms'), value: 'sms' },
  { label: $t('page.finance.payslip.channel.wechat'), value: 'wecom' },
  { label: $t('page.finance.payslip.channel.dingtalk'), value: 'dingtalk' },
  { label: $t('page.finance.payslip.channel.feishu'), value: 'feishu' },
];

const channelLabelMap: Record<string, string> = {
  site: $t('page.finance.payslip.channel.inbox'),
  email: $t('page.finance.payslip.channel.email'),
  sms: $t('page.finance.payslip.channel.sms'),
  wecom: $t('page.finance.payslip.channel.wechat'),
  dingtalk: $t('page.finance.payslip.channel.dingtalk'),
  feishu: $t('page.finance.payslip.channel.feishu'),
};

// ===== 列表数据 =====
const loading = ref(false);
const tableData = ref<any[]>([]);
const selectedRowKeys = ref<number[]>([]);

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getPayslipListApi(searchForm);
    const data = res?.data || res;
    tableData.value = Array.isArray(data) ? data : data?.items || data?.list || [];
    loadStatistics();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 统计数据 =====
const statistics = ref<any>({});
const statisticsLoading = ref(false);

async function loadStatistics() {
  statisticsLoading.value = true;
  try {
    const res: any = await getPayslipStatisticsApi({
      year: searchForm.year,
      month: searchForm.month,
    });
    statistics.value = res?.data || res || {};
  } catch {
    statistics.value = {};
  } finally {
    statisticsLoading.value = false;
  }
}

// ===== 生成工资条弹窗 =====
const generateVisible = ref(false);
const generateLoading = ref(false);
const generateForm = reactive({
  year: now.getFullYear(),
  month: now.getMonth() + 1,
});

async function handleGenerate() {
  generateLoading.value = true;
  try {
    await generatePayslipsApi({
      year: generateForm.year,
      month: generateForm.month,
    });
    message.success($t('page.finance.payslip.message.generateSuccess'));
    generateVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.payslip.message.generateFailed'));
  } finally {
    generateLoading.value = false;
  }
}

// ===== 发送通道弹窗 =====
const sendVisible = ref(false);
const sendLoading = ref(false);
const sendChannels = ref<string[]>([]);
const sendTargetId = ref<number | null>(null);
const sendTargetIds = ref<number[]>([]);
const sendIsBatch = ref(false);

function openSendModal(id: number) {
  sendIsBatch.value = false;
  sendTargetId.value = id;
  sendTargetIds.value = [];
  sendChannels.value = ['site'];
  sendVisible.value = true;
}

function openBatchSendModal() {
  if (selectedRowKeys.value.length === 0) {
    message.warning($t('page.finance.payslip.message.selectFirst'));
    return;
  }
  sendIsBatch.value = true;
  sendTargetId.value = null;
  sendTargetIds.value = [...selectedRowKeys.value];
  sendChannels.value = ['site'];
  sendVisible.value = true;
}

async function handleSend() {
  if (sendChannels.value.length === 0) {
    message.warning($t('page.finance.payslip.message.selectChannelRequired'));
    return;
  }
  const channels = [...sendChannels.value];
  sendLoading.value = true;
  try {
    if (sendIsBatch.value) {
      await batchSendPayslipsApi({ ids: sendTargetIds.value, channels });
      message.success($t('page.finance.payslip.message.batchSendSuccess'));
    } else if (sendTargetId.value !== null) {
      await sendPayslipApi({ id: sendTargetId.value, channels });
      message.success($t('page.finance.payslip.modal.sendSuccess'));
    }
    sendVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.payslip.message.sendFailed'));
  } finally {
    sendLoading.value = false;
  }
}

// ===== 详情抽屉 =====
const detailVisible = ref(false);
const detailRecord = ref<any>(null);

const detailItems = computed(() => {
  if (!detailRecord.value) return [];
  const raw = detailRecord.value.detailJson || detailRecord.value.detail_json;
  if (!raw) return [];
  try {
    const parsed = typeof raw === 'string' ? JSON.parse(raw) : raw;
    // 后端 detail_json 存的是对象（{baseSalary, commissionAmount, ...}），转为数组展示
    if (Array.isArray(parsed)) return parsed;
    if (parsed && typeof parsed === 'object') {
      const labelMap: Record<string, string> = {
        baseSalary: $t('page.finance.payslip.detail.baseSalary'),
        positionAllowance: $t('page.finance.payslip.detail.positionAllowance'),
        commissionAmount: $t('page.finance.payslip.detail.commission'),
        performanceBonus: $t('page.finance.payslip.detail.performanceBonus'),
        teamCommissionAmount: $t('page.finance.payslip.detail.teamCommission'),
        bonusAmount: $t('page.finance.payslip.detail.bonusAmount'),
        allocatedCommission: $t('page.finance.payslip.detail.allocatedCommission'),
        deferredCommission: $t('page.finance.payslip.detail.deferredCommission'),
        deductionAmount: $t('page.finance.payslip.detail.deduction'),
        socialInsurancePersonal: $t('page.finance.payslip.detail.socialInsurancePersonal'),
        housingFundPersonal: $t('page.finance.payslip.detail.housingFundPersonal'),
        taxAmount: $t('page.finance.payslip.detail.tax'),
        totalSalary: $t('page.finance.payslip.detail.totalSalary'),
        netSalary: $t('page.finance.payslip.detail.netSalary'),
      };
      return Object.entries(parsed)
        .filter(([, v]) => v !== null && v !== undefined && v !== '')
        .map(([k, v]) => ({ label: labelMap[k] || k, value: v, key: k }));
    }
    return [];
  } catch {
    return [];
  }
});

function openDetail(record: any) {
  detailRecord.value = record;
  detailVisible.value = true;
}

const columns: ColumnsType = [
  { title: $t('page.finance.payslip.column.employeeId'), dataIndex: 'employeeId', width: 90 },
  {
    title: $t('page.finance.payslip.column.yearMonth'),
    key: 'yearMonth',
    width: 110,
    customRender: ({ record }: any) => `${record.year}-${record.month}`,
  },
  {
    title: $t('page.finance.payslip.column.totalSalary'),
    dataIndex: 'totalSalary',
    width: 110,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.payslip.column.socialInsurancePersonal'),
    dataIndex: 'socialInsurancePersonal',
    width: 110,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.payslip.column.taxAmount'),
    dataIndex: 'taxAmount',
    width: 100,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.payslip.column.netSalary'),
    dataIndex: 'netSalary',
    width: 120,
    align: 'right',
  },
  {
    title: $t('page.finance.payslip.column.sendStatus'),
    dataIndex: 'sendStatus',
    width: 100,
  },
  {
    title: $t('page.finance.payslip.column.sendChannels'),
    dataIndex: 'sendChannels',
    width: 160,
  },
  {
    title: $t('page.finance.payslip.column.sendTime'),
    dataIndex: 'sendTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.payslip.column.readTime'),
    dataIndex: 'readTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 160,
    fixed: 'right',
  },
];

const rowSelection: any = {
  onChange: (keys: number[]) => {
    selectedRowKeys.value = keys;
  },
};

onMounted(() => {
  loadList();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.payslip.guide.title')"
      :brief="$t('page.finance.payslip.guide.brief')"
      :expand-text="$t('page.finance.payslip.guide.expand')"
      :collapse-text="$t('page.finance.payslip.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.payslip.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.payslip.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <!-- 统计卡片 -->
    <Card class="mb-4" :bordered="false">
      <Spin :spinning="statisticsLoading">
        <Row :gutter="16">
          <Col :span="6">
            <Statistic
              :title="$t('page.finance.payslip.statistic.total')"
              :value="statistics.total || 0"
              :value-style="{ color: '#1890ff' }"
            />
          </Col>
          <Col :span="6">
            <Statistic
              :title="$t('page.finance.payslip.statistic.sent')"
              :value="statistics.sent || 0"
              :value-style="{ color: '#fa8c16' }"
            />
          </Col>
          <Col :span="6">
            <Statistic
              :title="$t('page.finance.payslip.statistic.read')"
              :value="statistics.read || 0"
              :value-style="{ color: '#52c41a' }"
            />
          </Col>
          <Col :span="6">
            <Statistic
              :title="$t('page.finance.payslip.statistic.unread')"
              :value="statistics.unread || 0"
              :value-style="{ color: '#ff4d4f' }"
            />
          </Col>
        </Row>
      </Spin>
    </Card>

    <!-- 搜索栏 -->
    <Card class="mb-4" :bordered="false">
      <Form layout="inline">
        <FormItem :label="$t('page.finance.common.year')">
          <InputNumber
            v-model:value="searchForm.year"
            :min="2020"
            :max="2099"
            style="width: 120px"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.common.month')">
          <Select
            v-model:value="searchForm.month"
            :options="monthOptions"
            allow-clear
            :placeholder="$t('page.finance.common.all')"
            style="width: 120px"
          />
        </FormItem>
        <FormItem>
          <Button type="primary" @click="loadList">{{ $t('page.finance.common.query') }}</Button>
        </FormItem>
      </Form>
    </Card>

    <Card :bordered="false" :title="$t('page.finance.payslip.title')">
      <template #extra>
        <Button
          v-if="accessStore.hasAccessCode('finance:payslip:manage')"
          type="primary"
          class="mr-2"
          @click="generateVisible = true"
        >
          {{ $t('page.finance.payslip.button.generate') }}
        </Button>
        <Button
          v-if="accessStore.hasAccessCode('finance:payslip:manage')"
          @click="openBatchSendModal"
        >
          {{ $t('page.finance.payslip.button.batchSend') }}
        </Button>
      </template>

      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        row-key="id"
        :row-selection="rowSelection"
        :pagination="{ pageSize: 20 }"
        size="middle"
        :scroll="{ x: 1300 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'netSalary'">
            <span class="font-semibold text-primary">
              {{ formatMoney(record.netSalary) }}
            </span>
          </template>
          <template v-else-if="column.dataIndex === 'sendStatus'">
            <Tag :color="sendStatusMap[record.sendStatus]?.color ?? 'default'">
              {{ sendStatusMap[record.sendStatus]?.label ?? '-' }}
            </Tag>
          </template>
          <template v-else-if="column.dataIndex === 'sendChannels'">
            <span v-if="!record.sendChannels">-</span>
            <Tag
              v-for="ch in String(record.sendChannels).split(',')"
              v-else
              :key="ch"
              class="mb-1"
            >
              {{ channelLabelMap[ch] || ch }}
            </Tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <Button
              v-if="
                accessStore.hasAccessCode('finance:payslip:manage') &&
                record.sendStatus !== 2
              "
              type="link"
              size="small"
              @click="openSendModal(record.id)"
            >
              {{ $t('page.finance.payslip.button.sendAction') }}
            </Button>
            <Button type="link" size="small" @click="openDetail(record)">
              {{ $t('page.finance.common.viewDetail') }}
            </Button>
          </template>
        </template>
        <template #emptyText>
          <Empty :description="$t('page.finance.payslip.message.noData')" />
        </template>
      </Table>
    </Card>

    <!-- 生成工资条弹窗 -->
    <Modal
      v-model:open="generateVisible"
      :title="$t('page.finance.payslip.modal.generateTitle')"
      :confirm-loading="generateLoading"
      @ok="handleGenerate"
    >
      <div class="py-4 flex items-center gap-3">
        <span>{{ $t('page.finance.common.year') }}</span>
        <InputNumber
          v-model:value="generateForm.year"
          :min="2020"
          :max="2099"
          style="width: 120px"
        />
        <span>{{ $t('page.finance.common.month') }}</span>
        <Select
          v-model:value="generateForm.month"
          :options="monthOptions"
          style="width: 120px"
        />
      </div>
    </Modal>

    <!-- 发送通道弹窗 -->
    <Modal
      v-model:open="sendVisible"
      :title="sendIsBatch ? $t('page.finance.payslip.modal.batchSendTitle') : $t('page.finance.payslip.modal.sendTitle')"
      :confirm-loading="sendLoading"
      @ok="handleSend"
    >
      <div class="py-4">
        <p class="mb-3">
          {{ $t('page.finance.payslip.modal.selectChannelTip') }}
        </p>
        <CheckboxGroup v-model:value="sendChannels">
          <Checkbox
            v-for="ch in channelOptions"
            :key="ch.value"
            :value="ch.value"
            class="mb-2"
          >
            {{ ch.label }}
          </Checkbox>
        </CheckboxGroup>
      </div>
    </Modal>

    <!-- 详情抽屉 -->
    <Drawer
      v-model:open="detailVisible"
      :title="$t('page.finance.payslip.detail.title')"
      width="520px"
      :body-style="{ padding: '16px' }"
    >
      <div v-if="detailRecord">
        <Row :gutter="16" class="mb-4">
          <Col :span="12">
            <Statistic
              :title="$t('page.finance.payslip.detail.totalSalary')"
              :value="formatMoney(detailRecord.totalSalary)"
            />
          </Col>
          <Col :span="12">
            <Statistic
              :title="$t('page.finance.payslip.detail.netSalary')"
              :value="formatMoney(detailRecord.netSalary)"
              :value-style="{ color: '#1890ff', fontWeight: 'bold' }"
            />
          </Col>
        </Row>
        <Row :gutter="16" class="mb-4">
          <Col :span="12">
            <Statistic
              :title="$t('page.finance.payslip.detail.socialInsurancePersonal')"
              :value="formatMoney(detailRecord.socialInsurancePersonal)"
            />
          </Col>
          <Col :span="12">
            <Statistic
              :title="$t('page.finance.payslip.detail.tax')"
              :value="formatMoney(detailRecord.taxAmount)"
            />
          </Col>
        </Row>

        <div v-if="detailItems.length > 0" class="mt-4">
          <div class="mb-2 font-semibold">{{ $t('page.finance.payslip.detail.itemsTitle') }}</div>
          <Table
            :data-source="detailItems"
            :pagination="false"
            size="small"
            row-key="itemCode"
            :columns="[
              { title: $t('page.finance.payslip.detail.itemColumn'), dataIndex: 'itemName' },
              { title: $t('page.finance.payslip.detail.amountColumn'), dataIndex: 'amount', align: 'right' },
            ]"
          />
        </div>
        <Empty v-else :description="$t('page.finance.payslip.detail.noDetailData')" />
      </div>
    </Drawer>
  </Page>
</template>
