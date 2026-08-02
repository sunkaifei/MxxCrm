<script lang="ts" setup>
import { computed, h, onMounted, reactive, ref } from 'vue';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Col,
  DatePicker,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Row,
  Select,
  Table,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';
import dayjs from 'dayjs';
import { UserPickerModal } from '#/components/UserPickerModal';

import {
  approveSalaryAdjustmentApi,
  createSalaryAdjustmentApi,
  getSalaryAdjustmentHistoryApi,
  getSalaryAdjustmentListApi,
  rejectSalaryAdjustmentApi,
} from '#/api/core/finance';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';

const guideStepCount = 5;

// ===== 权限 =====
const { hasAccessByRoles } = useAccess();
const canManage = computed(() =>
  hasAccessByRoles(['super_admin', 'finance']),
);

// ===== 通用工具 =====
function formatMoney(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `¥${Number(val).toFixed(2)}`;
}

function formatDate(val: any) {
  if (!val) return '-';
  try {
    return dayjs(val).format('YYYY-MM-DD');
  } catch {
    return '-';
  }
}

const adjustmentTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: $t('page.finance.adjustment.adjustmentType.regularization'), color: 'blue' },
  2: { label: $t('page.finance.adjustment.adjustmentType.annual'), color: 'cyan' },
  3: { label: $t('page.finance.adjustment.adjustmentType.promotion'), color: 'gold' },
  4: { label: $t('page.finance.adjustment.adjustmentType.transfer'), color: 'orange' },
  5: { label: $t('page.finance.adjustment.adjustmentType.special'), color: 'purple' },
};

const adjustmentTypeOptions = [
  { value: 1, label: $t('page.finance.adjustment.adjustmentType.regularization') },
  { value: 2, label: $t('page.finance.adjustment.adjustmentType.annual') },
  { value: 3, label: $t('page.finance.adjustment.adjustmentType.promotion') },
  { value: 4, label: $t('page.finance.adjustment.adjustmentType.transfer') },
  { value: 5, label: $t('page.finance.adjustment.adjustmentType.special') },
];

const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: $t('page.finance.adjustment.status.pending'), color: 'blue' },
  1: { label: $t('page.finance.adjustment.status.approved'), color: 'green' },
  2: { label: $t('page.finance.adjustment.status.rejected'), color: 'red' },
};

// ===== 搜索栏 =====
const searchForm = reactive({
  employeeId: undefined as number | undefined,
});

// ===== 列表 =====
const loading = ref(false);
const tableData = ref<any[]>([]);

const columns = computed(() => [
  { title: $t('page.finance.adjustment.column.employeeId'), dataIndex: 'employeeId', width: 90 },
  { title: $t('page.finance.adjustment.column.employeeName'), dataIndex: 'employeeName', width: 120 },
  {
    title: $t('page.finance.adjustment.column.adjustmentDate'),
    dataIndex: 'adjustmentDate',
    width: 120,
    customRender: ({ text }: any) => formatDate(text),
  },
  {
    title: $t('page.finance.adjustment.column.adjustmentType'),
    dataIndex: 'adjustmentType',
    width: 110,
    customRender: ({ text }: any) => {
      const m = adjustmentTypeMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.adjustment.column.oldBaseSalary'),
    dataIndex: 'oldBaseSalary',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.adjustment.column.newBaseSalary'),
    dataIndex: 'newBaseSalary',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.adjustment.column.oldPositionAllowance'),
    dataIndex: 'oldPositionAllowance',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.adjustment.column.newPositionAllowance'),
    dataIndex: 'newPositionAllowance',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.common.status'),
    dataIndex: 'status',
    width: 100,
    customRender: ({ text }: any) => {
      const m = statusMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  { title: $t('page.finance.common.action'), key: 'action', width: 240, fixed: 'right' as const },
]);

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getSalaryAdjustmentListApi({
      employeeId: searchForm.employeeId,
    });
    const data = res?.data || res;
    tableData.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.adjustment.message.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 新增弹窗 =====
const formVisible = ref(false);
const formSubmitting = ref(false);
const adjustmentForm = reactive({
  employeeId: undefined as number | undefined,
  adjustmentDate: dayjs() as any,
  adjustmentType: 2,
  oldBaseSalary: 0,
  newBaseSalary: 0,
  oldPositionAllowance: 0,
  newPositionAllowance: 0,
  newPerformanceBase: 0,
  reason: '',
});

function openForm() {
  adjustmentForm.employeeId = undefined;
  adjustmentForm.adjustmentDate = dayjs();
  adjustmentForm.adjustmentType = 2;
  adjustmentForm.oldBaseSalary = 0;
  adjustmentForm.newBaseSalary = 0;
  adjustmentForm.oldPositionAllowance = 0;
  adjustmentForm.newPositionAllowance = 0;
  adjustmentForm.newPerformanceBase = 0;
  adjustmentForm.reason = '';
  formVisible.value = true;
}

async function submitForm() {
  if (!adjustmentForm.employeeId) {
    message.warning($t('page.finance.adjustment.drawer.employeeIdPlaceholder'));
    return;
  }
  if (!adjustmentForm.adjustmentDate) {
    message.warning($t('page.finance.adjustment.message.dateRequired'));
    return;
  }
  if (!adjustmentForm.reason.trim()) {
    message.warning($t('page.finance.adjustment.message.reasonRequired'));
    return;
  }
  formSubmitting.value = true;
  try {
    await createSalaryAdjustmentApi({
      employeeId: adjustmentForm.employeeId,
      adjustmentDate: dayjs(adjustmentForm.adjustmentDate).format(
        'YYYY-MM-DD',
      ),
      adjustmentType: adjustmentForm.adjustmentType,
      oldBaseSalary: adjustmentForm.oldBaseSalary,
      newBaseSalary: adjustmentForm.newBaseSalary,
      oldPositionAllowance: adjustmentForm.oldPositionAllowance,
      newPositionAllowance: adjustmentForm.newPositionAllowance,
      newPerformanceBase: adjustmentForm.newPerformanceBase,
      reason: adjustmentForm.reason,
    });
    message.success($t('page.finance.adjustment.message.createSuccess'));
    formVisible.value = false;
    await loadData();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.adjustment.message.createFailed'));
  } finally {
    formSubmitting.value = false;
  }
}

// ===== 审批通过 =====
async function handleApprove(id: number) {
  try {
    await approveSalaryAdjustmentApi(id);
    message.success($t('page.finance.adjustment.message.approveSuccess'));
    await loadData();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.adjustment.message.approveFailed'));
  }
}

// ===== 驳回弹窗 =====
const rejectVisible = ref(false);
const rejectLoading = ref(false);
const rejectTarget = ref<any>(null);
const rejectForm = reactive({ reason: '' });

function openRejectModal(record: any) {
  rejectTarget.value = record;
  rejectForm.reason = '';
  rejectVisible.value = true;
}

async function submitReject() {
  if (!rejectTarget.value) return;
  if (!rejectForm.reason.trim()) {
    message.warning($t('page.finance.adjustment.message.rejectReasonRequired'));
    return;
  }
  rejectLoading.value = true;
  try {
    await rejectSalaryAdjustmentApi({
      id: rejectTarget.value.id,
      reason: rejectForm.reason,
    });
    message.success($t('page.finance.adjustment.message.rejectSuccess'));
    rejectVisible.value = false;
    await loadData();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.adjustment.message.rejectFailed'));
  } finally {
    rejectLoading.value = false;
  }
}

// ===== 详情弹窗 =====
const detailVisible = ref(false);
const detailRecord = ref<any>(null);

function openDetailModal(record: any) {
  detailRecord.value = record;
  detailVisible.value = true;
}

// ===== 历史时间轴抽屉 =====
const historyVisible = ref(false);
const historyLoading = ref(false);
const historyList = ref<any[]>([]);
const historyEmployee = ref<any>(null);

async function openHistoryDrawer(record: any) {
  historyEmployee.value = record;
  historyVisible.value = true;
  historyLoading.value = true;
  try {
    const res: any = await getSalaryAdjustmentHistoryApi(record.employeeId);
    const data = res?.data || res;
    historyList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.adjustment.message.loadHistoryFailed'));
    historyList.value = [];
  } finally {
    historyLoading.value = false;
  }
}

onMounted(() => {
  loadData();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.adjustment.guide.title')"
      :brief="$t('page.finance.adjustment.guide.brief')"
      :expand-text="$t('page.finance.adjustment.guide.expand')"
      :collapse-text="$t('page.finance.adjustment.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.adjustment.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.adjustment.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Card :bordered="false">
      <!-- 搜索栏 -->
      <div class="mb-4 flex flex-wrap items-center gap-3">
        <span>{{ $t('page.finance.adjustment.column.employeeId') }}：</span>
        <UserPickerModal v-model:value="searchForm.employeeId" style="width: 160px" />
        <Button type="primary" @click="loadData">{{ $t('page.finance.common.query') }}</Button>
        <Button @click="loadData">{{ $t('page.finance.common.refresh') }}</Button>
        <div class="flex-1" />
        <Button v-if="canManage" type="primary" @click="openForm">
          {{ $t('page.finance.adjustment.button.create') }}
        </Button>
      </div>

      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        row-key="id"
        :pagination="{ pageSize: 20, showSizeChanger: true }"
        size="middle"
        :scroll="{ x: 1400 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'action'">
            <Button
              type="link"
              size="small"
              @click="openDetailModal(record)"
            >
              {{ $t('page.finance.adjustment.button.detail') }}
            </Button>
            <Button
              type="link"
              size="small"
              @click="openHistoryDrawer(record)"
            >
              {{ $t('page.finance.adjustment.button.history') }}
            </Button>
            <Popconfirm
              v-if="canManage && record.status === 0"
              :title="$t('page.finance.adjustment.modal.approveConfirm')"
              @confirm="handleApprove(record.id)"
            >
              <Button type="link" size="small">{{ $t('page.finance.adjustment.button.approveAction') }}</Button>
            </Popconfirm>
            <Button
              v-if="canManage && record.status === 0"
              type="link"
              size="small"
              danger
              @click="openRejectModal(record)"
            >
              {{ $t('page.finance.adjustment.button.reject') }}
            </Button>
          </template>
        </template>
      </Table>
    </Card>

    <!-- 新增调薪弹窗 -->
    <Modal
      v-model:open="formVisible"
      :title="$t('page.finance.adjustment.drawer.titleCreate')"
      :confirm-loading="formSubmitting"
      width="640px"
      @ok="submitForm"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.adjustment.drawer.employeeId')" required>
              <UserPickerModal v-model:value="adjustmentForm.employeeId" />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.adjustment.drawer.adjustmentDate')" required>
              <DatePicker
                v-model:value="adjustmentForm.adjustmentDate"
                style="width: 100%"
                :placeholder="$t('page.finance.adjustment.drawer.adjustmentDatePlaceholder')"
              />
            </FormItem>
          </Col>
        </Row>
        <FormItem :label="$t('page.finance.adjustment.drawer.adjustmentType')" required>
          <Select
            v-model:value="adjustmentForm.adjustmentType"
            :options="adjustmentTypeOptions"
            style="width: 100%"
          />
        </FormItem>
        <Row :gutter="16">
          <Col :span="6">
            <FormItem :label="$t('page.finance.adjustment.drawer.oldBaseSalary')">
              <InputNumber
                v-model:value="adjustmentForm.oldBaseSalary"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.adjustment.drawer.newBaseSalary')" required>
              <InputNumber
                v-model:value="adjustmentForm.newBaseSalary"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.adjustment.drawer.oldPositionAllowance')">
              <InputNumber
                v-model:value="adjustmentForm.oldPositionAllowance"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.adjustment.drawer.newPositionAllowance')">
              <InputNumber
                v-model:value="adjustmentForm.newPositionAllowance"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
        </Row>
        <FormItem :label="$t('page.finance.adjustment.drawer.newPerformanceBase')">
          <InputNumber
            v-model:value="adjustmentForm.newPerformanceBase"
            :min="0"
            :precision="2"
            style="width: 100%"
            prefix="¥"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.adjustment.drawer.reason')" required>
          <Input.TextArea
            v-model:value="adjustmentForm.reason"
            :rows="4"
            :placeholder="$t('page.finance.adjustment.drawer.reasonPlaceholder')"
            :maxlength="500"
            show-count
          />
        </FormItem>
      </Form>
    </Modal>

    <!-- 驳回弹窗 -->
    <Modal
      v-model:open="rejectVisible"
      :title="$t('page.finance.adjustment.modal.titleReject')"
      :confirm-loading="rejectLoading"
      width="480px"
      @ok="submitReject"
    >
      <div class="py-4">
        <p class="mb-2">
          {{ $t('page.finance.common.employee') }}：{{ rejectTarget?.employeeName || rejectTarget?.employeeId }}
        </p>
        <p class="mb-4">
          {{ $t('page.finance.adjustment.drawer.adjustmentDate') }}：{{ formatDate(rejectTarget?.adjustmentDate) }}
        </p>
        <FormItem :label="$t('page.finance.adjustment.drawer.rejectReason')" required>
          <Input.TextArea
            v-model:value="rejectForm.reason"
            :rows="4"
            :placeholder="$t('page.finance.adjustment.drawer.rejectReasonPlaceholder')"
            :maxlength="500"
            show-count
          />
        </FormItem>
      </div>
    </Modal>

    <!-- 详情弹窗 -->
    <Modal
      v-model:open="detailVisible"
      :title="$t('page.finance.adjustment.modal.titleDetail')"
      :footer="null"
      width="640px"
    >
      <div v-if="detailRecord" class="py-4">
        <Descriptions bordered :column="2" size="small">
          <DescriptionsItem :label="$t('page.finance.adjustment.column.employeeId')">
            {{ detailRecord.employeeId }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.employeeName')">
            {{ detailRecord.employeeName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.adjustmentDate')">
            {{ formatDate(detailRecord.adjustmentDate) }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.adjustmentType')">
            {{ adjustmentTypeMap[detailRecord.adjustmentType]?.label || '-' }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.oldBaseSalary')">
            {{ formatMoney(detailRecord.oldBaseSalary) }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.newBaseSalary')">
            {{ formatMoney(detailRecord.newBaseSalary) }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.oldPositionAllowance')">
            {{ formatMoney(detailRecord.oldPositionAllowance) }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.column.newPositionAllowance')">
            {{ formatMoney(detailRecord.newPositionAllowance) }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.drawer.newPerformanceBase')">
            {{ formatMoney(detailRecord.newPerformanceBase) }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.common.status')">
            <Tag :color="statusMap[detailRecord.status]?.color">
              {{ statusMap[detailRecord.status]?.label }}
            </Tag>
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.adjustment.drawer.reason')" :span="2">
            {{ detailRecord.reason || '-' }}
          </DescriptionsItem>
          <DescriptionsItem v-if="detailRecord.rejectReason" :label="$t('page.finance.adjustment.drawer.rejectReason')" :span="2">
            {{ detailRecord.rejectReason }}
          </DescriptionsItem>
        </Descriptions>
      </div>
    </Modal>

    <!-- 历史时间轴抽屉 -->
    <Drawer
      v-model:open="historyVisible"
      :title="$t('page.finance.adjustment.drawer.historyTitle', { employeeId: historyEmployee?.employeeId || '' })"
      width="640px"
      :body-style="{ padding: '24px' }"
    >
      <div v-if="historyLoading" class="py-8 text-center text-gray-500">
        {{ $t('page.finance.adjustment.history.loading') }}
      </div>
      <Timeline v-else-if="historyList.length > 0">
        <TimelineItem
          v-for="(item, idx) in historyList"
          :key="idx"
          :color="statusMap[item.status]?.color || 'blue'"
        >
          <div class="mb-1 flex items-center gap-2">
            <span class="font-semibold">
              {{ formatDate(item.adjustmentDate) }}
            </span>
            <Tag :color="adjustmentTypeMap[item.adjustmentType]?.color">
              {{ adjustmentTypeMap[item.adjustmentType]?.label }}
            </Tag>
            <Tag :color="statusMap[item.status]?.color">
              {{ statusMap[item.status]?.label }}
            </Tag>
          </div>
          <div class="text-sm text-gray-600">
            <div>
              {{ $t('page.finance.adjustment.history.baseSalary') }}：{{ formatMoney(item.oldBaseSalary) }} →
              <span class="font-semibold text-primary">
                {{ formatMoney(item.newBaseSalary) }}
              </span>
            </div>
            <div>
              {{ $t('page.finance.adjustment.history.positionAllowance') }}：{{ formatMoney(item.oldPositionAllowance) }} →
              {{ formatMoney(item.newPositionAllowance) }}
            </div>
            <div v-if="item.reason">{{ $t('page.finance.adjustment.history.reason') }}：{{ item.reason }}</div>
            <div v-if="item.rejectReason" class="text-red-500">
              {{ $t('page.finance.adjustment.history.rejectLabel') }}：{{ item.rejectReason }}
            </div>
          </div>
        </TimelineItem>
      </Timeline>
      <div v-else class="py-8 text-center text-gray-500">
        {{ $t('page.finance.adjustment.history.empty') }}
      </div>
    </Drawer>
  </Page>
</template>
