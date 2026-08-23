<script lang="ts" setup>
import type { ColumnsType } from 'ant-design-vue/es/table';

import { computed, onMounted, reactive, ref } from 'vue';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Card,
  Checkbox,
  CheckboxGroup,
  Col,
  Divider,
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

import {
  batchSendPayslipsApi,
  confirmPayslipApi,
  generatePayslipsApi,
  getPayslipDetailApi,
  getPayslipListApi,
  getPayslipStatisticsApi,
  sendPayslipApi,
} from '#/api/core/finance';
import { getVisibleDashboardCardsApi } from '#/api/core/system/dashboard-card';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { $t } from '#/locales';

const guideStepCount = 5;

const accessStore = useAccessStore();
const { hasAccessByRoles } = useAccess();

// 全量权限：超级管理员、财务、总经理、老板
const isFullScope = computed(() =>
  hasAccessByRoles([
    'super_admin',
    'admin',
    'finance',
    'general_manager',
    'boss',
    'cw',
  ]),
);

// 当前 Tab：全量权限默认 "all"，其他角色强制 "my"
const activeTab = ref<'all' | 'my'>(isFullScope.value ? 'all' : 'my');

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
const sendStatusMap: Record<number, { color: string; label: string }> = {
  0: { label: $t('page.finance.payslip.sendStatus.unsent'), color: 'default' },
  1: { label: $t('page.finance.payslip.sendStatus.sent'), color: 'processing' },
  2: { label: $t('page.finance.payslip.sendStatus.read'), color: 'success' },
  3: { label: $t('page.finance.payslip.sendStatus.confirmed'), color: 'green' },
  4: { label: $t('page.finance.payslip.sendStatus.withdrawn'), color: 'error' },
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

// ===== 列表数据 =====
const loading = ref(false);
const tableData = ref<any[]>([]);
const selectedRowKeys = ref<number[]>([]);

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getPayslipListApi({
      ...searchForm,
      listType: activeTab.value,
    });
    const data = res?.data || res;
    tableData.value = Array.isArray(data)
      ? data
      : data?.items || data?.list || [];
    loadStatistics();
  } catch (error: any) {
    message.error(error?.message || $t('page.finance.common.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 统计数据 =====
const statistics = ref<any>({});
const statisticsLoading = ref(false);
// 统计卡片是否对当前用户可见（由"工作台卡片配置"按角色控制）
const showStatsCard = ref(false);

async function loadVisibleCards() {
  try {
    const res: any = await getVisibleDashboardCardsApi();
    const list = Array.isArray(res) ? res : res?.data || res?.list || [];
    showStatsCard.value = list.some(
      (card: any) => card.cardCode === 'payslip_stat',
    );
  } catch {
    showStatsCard.value = false;
  }
}

async function loadStatistics() {
  if (!showStatsCard.value) {
    statistics.value = {};
    return;
  }
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
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payslip.message.generateFailed'),
    );
  } finally {
    generateLoading.value = false;
  }
}

// ===== 发送通道弹窗 =====
const sendVisible = ref(false);
const sendLoading = ref(false);
const sendChannels = ref<string[]>([]);
const sendTargetId = ref<null | number>(null);
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
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payslip.message.sendFailed'),
    );
  } finally {
    sendLoading.value = false;
  }
}

// ===== 详情抽屉 =====
const detailVisible = ref(false);
const detailLoading = ref(false);
const detailData = ref<any>(null);

const detailItems = computed(() => {
  if (!detailData.value) return [];
  const raw =
    detailData.value.detail ||
    detailData.value.detailJson ||
    detailData.value.detail_json;
  if (!raw) return [];
  try {
    const parsed = typeof raw === 'string' ? JSON.parse(raw) : raw;
    if (Array.isArray(parsed)) return parsed;
    if (parsed && typeof parsed === 'object') {
      const labelMap: Record<string, string> = {
        baseSalary: $t('page.finance.payslip.detail.baseSalary'),
        commissionAmount: $t('page.finance.payslip.detail.commission'),
        performanceBonus: $t('page.finance.payslip.detail.performanceBonus'),
        teamCommissionAmount: $t('page.finance.payslip.detail.teamCommission'),
        bonusAmount: $t('page.finance.payslip.detail.bonusAmount'),
        allocatedCommission: $t(
          'page.finance.payslip.detail.allocatedCommission',
        ),
        deferredCommission: $t(
          'page.finance.payslip.detail.deferredCommission',
        ),
        deductionAmount: $t('page.finance.payslip.detail.deduction'),
        socialInsurancePersonal: $t(
          'page.finance.payslip.detail.socialInsurancePersonal',
        ),
        socialInsuranceCompany: $t(
          'page.finance.payslip.detail.socialInsuranceCompany',
        ),
        housingFundPersonal: $t(
          'page.finance.payslip.detail.housingFundPersonal',
        ),
        housingFundCompany: $t(
          'page.finance.payslip.detail.housingFundCompany',
        ),
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

const commissionList = computed(() => detailData.value?.commissions || []);

const commissionColumns = computed(() => [
  {
    title: '#',
    key: 'seq',
    width: 45,
    customRender: ({ index }: any) => index + 1,
  },
  {
    title: $t('page.finance.payslip.detail.contractName'),
    dataIndex: 'contractName',
    ellipsis: true,
    customRender: ({ value }: any) => value || '-',
  },
  {
    title: $t('page.finance.payslip.detail.contractAmount'),
    dataIndex: 'contractAmount',
    width: 120,
    align: 'right' as const,
    customRender: ({ value }: any) =>
      value ? `¥${Number(value).toFixed(2)}` : '-',
  },
  {
    title: $t('page.finance.payslip.detail.paymentAmount'),
    dataIndex: 'paymentAmount',
    width: 120,
    align: 'right' as const,
    customRender: ({ value }: any) =>
      value ? `¥${Number(value).toFixed(2)}` : '-',
  },
  {
    title: $t('page.finance.payslip.detail.commissionRate'),
    dataIndex: 'commissionRate',
    width: 90,
    align: 'right' as const,
    customRender: ({ value }: any) =>
      value ? `${(Number(value) * 100).toFixed(1)}%` : '-',
  },
  {
    title: $t('page.finance.payslip.detail.commissionAmount'),
    dataIndex: 'commissionAmount',
    width: 110,
    align: 'right' as const,
    customRender: ({ value }: any) =>
      value ? `¥${Number(value).toFixed(2)}` : '-',
  },
  {
    title: $t('page.finance.payslip.detail.ruleName'),
    dataIndex: 'ruleName',
    width: 140,
    ellipsis: true,
    customRender: ({ value }: any) => value || '-',
  },
]);

async function openDetail(record: any) {
  detailVisible.value = true;
  detailData.value = null;
  detailLoading.value = true;
  try {
    const res: any = await getPayslipDetailApi(record.id);
    detailData.value = res?.data || res;
  } catch {
    detailData.value = record;
  } finally {
    detailLoading.value = false;
  }
}

// ===== 确认工资条 =====
const confirming = ref(false);

async function handleConfirm() {
  if (!detailData.value?.id) return;
  confirming.value = true;
  try {
    await confirmPayslipApi(detailData.value.id);
    message.success($t('page.finance.payslip.detail.confirmSuccess'));
    // 更新本地状态
    if (detailData.value) {
      detailData.value.sendStatus = 3;
    }
    loadList();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payslip.detail.confirmFailed'),
    );
  } finally {
    confirming.value = false;
  }
}

// 快捷确认（列表操作列）
async function handleQuickConfirm(record: any) {
  try {
    await confirmPayslipApi(record.id);
    message.success($t('page.finance.payslip.detail.confirmSuccess'));
    loadList();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.payslip.detail.confirmFailed'),
    );
  }
}

// ===== 表格列 =====
const columns = computed<ColumnsType>(() => {
  const cols: any[] = [
    {
      title: $t('page.finance.payslip.column.employeeId'),
      dataIndex: 'employeeId',
      width: 90,
    },
  ];

  // "全部" Tab 才显示员工姓名列（"我的"不需要）
  if (activeTab.value === 'all') {
    cols.push({
      title: $t('page.finance.payslip.column.employeeName'),
      dataIndex: 'employeeName',
      width: 100,
    });
  }

  cols.push(
    {
      title: $t('page.finance.payslip.column.yearMonth'),
      key: 'yearMonth',
      width: 100,
      customRender: ({ record }: any) => `${record.year}-${record.month}`,
    },
    {
      title: $t('page.finance.payslip.column.baseSalary'),
      dataIndex: 'baseSalary',
      width: 110,
      align: 'right',
      customRender: ({ record }: any) => {
        const detail = record.detailJson || record.detail_json;
        if (detail) {
          try {
            const parsed =
              typeof detail === 'string' ? JSON.parse(detail) : detail;
            return formatMoney(parsed.baseSalary);
          } catch {
            /* ignore */
          }
        }
        return '-';
      },
    },
    {
      title: $t('page.finance.payslip.column.totalSalary'),
      dataIndex: 'totalSalary',
      width: 110,
      align: 'center',
      customRender: ({ text }: any) => formatMoney(text),
    },
    {
      title: $t('page.finance.payslip.column.socialInsurancePersonal'),
      dataIndex: 'socialInsurancePersonal',
      width: 110,
      align: 'center',
      customRender: ({ text }: any) => formatMoney(text),
    },
    {
      title: $t('page.finance.payslip.column.taxAmount'),
      dataIndex: 'taxAmount',
      width: 100,
      align: 'center',
      customRender: ({ text }: any) => formatMoney(text),
    },
    {
      title: $t('page.finance.payslip.column.netSalary'),
      dataIndex: 'netSalary',
      width: 120,
      align: 'center',
    },
    {
      title: $t('page.finance.payslip.column.sendStatus'),
      dataIndex: 'sendStatus',
      width: 100,
    },
    {
      title: $t('page.finance.payslip.column.confirmTime'),
      dataIndex: 'confirmTime',
      width: 170,
      customRender: ({ text }: any) => text || '-',
    },
    {
      title: $t('page.finance.common.action'),
      key: 'action',
      width: 200,
      align: 'center',
      fixed: 'right',
    },
  );

  return cols;
});

const rowSelection: any = {
  onChange: (keys: number[]) => {
    selectedRowKeys.value = keys;
  },
};

function onTabChange(key: string) {
  activeTab.value = key as 'all' | 'my';
  selectedRowKeys.value = [];
  loadList();
}

onMounted(async () => {
  // 先确认统计卡片可见性，再加载列表（列表加载完成后会拉取统计数据）
  await loadVisibleCards();
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
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
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

    <!-- 统计卡片（按"工作台卡片配置"角色可见性动态渲染） -->
    <Card
      v-if="showStatsCard"
      class="mb-4"
      style="margin-bottom: 16px"
    >
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
    <Card class="mb-4" style="margin-bottom: 16px">
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
          <Button type="primary" @click="loadList">
            {{ $t('page.finance.common.query') }}
          </Button>
        </FormItem>
      </Form>
    </Card>

    <Card :title="$t('page.finance.payslip.title')">
      <template #extra>
        <!-- Tab 切换按钮 -->
        <div v-if="isFullScope" class="mr-4 inline-flex gap-2">
          <Button
            :type="activeTab === 'all' ? 'primary' : 'default'"
            size="small"
            @click="onTabChange('all')"
          >
            {{ $t('page.finance.payslip.tab.all') }}
          </Button>
          <Button
            :type="activeTab === 'my' ? 'primary' : 'default'"
            size="small"
            @click="onTabChange('my')"
          >
            {{ $t('page.finance.payslip.tab.my') }}
          </Button>
        </div>
        <Button
          v-if="
            accessStore.hasAccessCode('finance:payslip:manage') &&
            activeTab === 'all'
          "
          type="primary"
          class="mr-2"
          @click="generateVisible = true"
        >
          {{ $t('page.finance.payslip.button.generate') }}
        </Button>
        <Button
          v-if="
            accessStore.hasAccessCode('finance:payslip:manage') &&
            activeTab === 'all'
          "
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
        :row-selection="activeTab === 'all' ? rowSelection : undefined"
        :pagination="{ pageSize: 20 }"
        size="middle"
        :scroll="{ x: 1400 }"
        :header-cell-style="{ textAlign: 'center' }"
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
          <template v-else-if="column.key === 'action'">
            <!-- 员工：确认 / 查看详情 -->
            <template v-if="activeTab === 'my'">
              <Button
                v-if="record.sendStatus === 1 || record.sendStatus === 2"
                type="link"
                size="small"
                @click="handleQuickConfirm(record)"
              >
                {{ $t('page.finance.payslip.detail.confirmButton') }}
              </Button>
              <Button type="link" size="small" @click="openDetail(record)">
                {{ $t('page.finance.common.viewDetail') }}
              </Button>
            </template>
            <!-- 财务：发送 / 查看详情 -->
            <template v-else>
              <Button
                v-if="
                  accessStore.hasAccessCode('finance:payslip:manage') &&
                  record.sendStatus !== 2 &&
                  record.sendStatus !== 3 &&
                  record.sendStatus !== 4
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
      :title="
        sendIsBatch
          ? $t('page.finance.payslip.modal.batchSendTitle')
          : $t('page.finance.payslip.modal.sendTitle')
      "
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
      width="640px"
      :body-style="{ padding: '16px' }"
    >
      <Spin :spinning="detailLoading">
        <div v-if="detailData">
          <!-- 汇总 -->
          <Row :gutter="16" class="mb-4">
            <Col :span="8">
              <Statistic
                :title="$t('page.finance.payslip.detail.totalSalary')"
                :value="formatMoney(detailData.totalSalary)"
              />
            </Col>
            <Col :span="8">
              <Statistic
                :title="$t('page.finance.payslip.detail.netSalary')"
                :value="formatMoney(detailData.netSalary)"
                :value-style="{ color: '#1890ff', fontWeight: 'bold' }"
              />
            </Col>
            <Col :span="8">
              <Statistic
                :title="$t('page.finance.payslip.column.taxAmount')"
                :value="formatMoney(detailData.taxAmount)"
                :value-style="{ color: '#ff4d4f' }"
              />
            </Col>
          </Row>

          <!-- 工资构成明细 -->
          <div v-if="detailItems.length > 0" class="mb-4">
            <Divider class="mt-0 mb-4" dashed>
              {{ $t('page.finance.payslip.detail.itemsTitle') }}
            </Divider>
            <Table
              :data-source="detailItems"
              :pagination="false"
              size="small"
              row-key="key"
              :columns="[
                {
                  title: $t('page.finance.payslip.detail.itemColumn'),
                  dataIndex: 'label',
                },
                {
                  title: $t('page.finance.payslip.detail.amountColumn'),
                  dataIndex: 'value',
                  align: 'right',
                },
              ]"
            >
              <template #bodyCell="{ column, record: row }">
                <template v-if="column.dataIndex === 'value'">
                  <span
                    :class="{
                      'font-semibold text-primary': row.key === 'netSalary',
                    }"
                  >
                    ¥{{ Number(row.value).toFixed(2) }}
                  </span>
                </template>
              </template>
            </Table>
          </div>

          <!-- 提成明细（可追溯） -->
          <Divider class="my-4" dashed>
            {{ $t('page.finance.payslip.detail.commissionTitle') }}
          </Divider>
          <div v-if="commissionList.length > 0">
            <Table
              :columns="commissionColumns"
              :data-source="commissionList"
              :pagination="false"
              size="small"
              row-key="contractId"
              :scroll="{ x: 700 }"
            />
          </div>
          <Empty
            v-else
            class="mt-4"
            :description="$t('page.finance.payslip.detail.commissionEmpty')"
          />

          <!-- 确认按钮 -->
          <div
            v-if="detailData.sendStatus === 1 || detailData.sendStatus === 2"
            class="mt-6 text-center"
          >
            <Button
              type="primary"
              size="large"
              :loading="confirming"
              @click="handleConfirm"
            >
              {{ $t('page.finance.payslip.detail.confirmButton') }}
            </Button>
          </div>
          <div v-else-if="detailData.sendStatus === 3" class="mt-6 text-center">
            <Tag color="green" style="padding: 4px 16px; font-size: 14px">
              {{ $t('page.finance.payslip.sendStatus.confirmed') }}
            </Tag>
          </div>
        </div>
        <Empty
          v-else
          :description="$t('page.finance.payslip.detail.noDetailData')"
        />
      </Spin>
    </Drawer>
  </Page>
</template>
