<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Col,
  Descriptions,
  DescriptionsItem,
  Form,
  FormItem,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Row,
  Select,
  Statistic,
  Table,
  Upload,
} from 'ant-design-vue';
import { UserPickerModal } from '#/components/UserPickerModal';

import {
  batchImportAttendanceApi,
  calculateAttendanceDeductionApi,
  deleteAttendanceApi,
  getAttendanceListApi,
  upsertAttendanceApi,
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

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: $t('page.finance.attendance.message.monthOption', { n: i + 1 }),
}));

const now = new Date();

// ===== 搜索栏 =====
const searchForm = reactive({
  year: now.getFullYear() as number | undefined,
  month: now.getMonth() + 1 as number | undefined,
  employeeId: undefined as number | undefined,
});

// ===== 列表 =====
const loading = ref(false);
const tableData = ref<any[]>([]);

const columns = computed(() => [
  {
    title: $t('page.finance.attendance.column.employeeId'),
    dataIndex: 'employeeId',
    width: 90,
  },
  {
    title: $t('page.finance.attendance.column.employeeName'),
    dataIndex: 'employeeName',
    width: 120,
  },
  {
    title: $t('page.finance.attendance.column.yearMonth'),
    key: 'yearMonth',
    width: 110,
    customRender: ({ record }: any) => `${record.year}-${record.month}`,
  },
  {
    title: $t('page.finance.attendance.column.workDays'),
    dataIndex: 'workDays',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.actualWorkDays'),
    dataIndex: 'actualWorkDays',
    width: 100,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.lateCount'),
    dataIndex: 'lateCount',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.earlyLeaveCount'),
    dataIndex: 'earlyLeaveCount',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.absentCount'),
    dataIndex: 'absentCount',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.personalLeaveDays'),
    dataIndex: 'personalLeaveDays',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.sickLeaveDays'),
    dataIndex: 'sickLeaveDays',
    width: 90,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.overtimeWeekday'),
    dataIndex: 'overtimeHoursWeekday',
    width: 120,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.overtimeWeekend'),
    dataIndex: 'overtimeHoursWeekend',
    width: 120,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.attendance.column.overtimeHoliday'),
    dataIndex: 'overtimeHoursHoliday',
    width: 130,
    align: 'right' as const,
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 220,
    fixed: 'right' as const,
  },
]);

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getAttendanceListApi({
      year: searchForm.year,
      month: searchForm.month,
      employeeId: searchForm.employeeId,
    });
    const data = res?.data || res;
    tableData.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 新增/编辑弹窗 =====
const formVisible = ref(false);
const formSubmitting = ref(false);
const attendanceForm = reactive({
  id: undefined as number | undefined,
  employeeId: undefined as number | undefined,
  year: now.getFullYear(),
  month: now.getMonth() + 1,
  workDays: 22,
  actualWorkDays: 22,
  lateCount: 0,
  earlyLeaveCount: 0,
  absentCount: 0,
  personalLeaveDays: 0,
  sickLeaveDays: 0,
  overtimeHoursWeekday: 0,
  overtimeHoursWeekend: 0,
  overtimeHoursHoliday: 0,
});

function openForm(record?: any) {
  if (record) {
    attendanceForm.id = record.id;
    attendanceForm.employeeId = record.employeeId;
    attendanceForm.year = record.year;
    attendanceForm.month = record.month;
    attendanceForm.workDays = Number(record.workDays ?? 0);
    attendanceForm.actualWorkDays = Number(record.actualWorkDays ?? 0);
    attendanceForm.lateCount = Number(record.lateCount ?? 0);
    attendanceForm.earlyLeaveCount = Number(record.earlyLeaveCount ?? 0);
    attendanceForm.absentCount = Number(record.absentCount ?? 0);
    attendanceForm.personalLeaveDays = Number(record.personalLeaveDays ?? 0);
    attendanceForm.sickLeaveDays = Number(record.sickLeaveDays ?? 0);
    attendanceForm.overtimeHoursWeekday = Number(
      record.overtimeHoursWeekday ?? 0,
    );
    attendanceForm.overtimeHoursWeekend = Number(
      record.overtimeHoursWeekend ?? 0,
    );
    attendanceForm.overtimeHoursHoliday = Number(
      record.overtimeHoursHoliday ?? 0,
    );
  } else {
    attendanceForm.id = undefined;
    attendanceForm.employeeId = undefined;
    attendanceForm.year = now.getFullYear();
    attendanceForm.month = now.getMonth() + 1;
    attendanceForm.workDays = 22;
    attendanceForm.actualWorkDays = 22;
    attendanceForm.lateCount = 0;
    attendanceForm.earlyLeaveCount = 0;
    attendanceForm.absentCount = 0;
    attendanceForm.personalLeaveDays = 0;
    attendanceForm.sickLeaveDays = 0;
    attendanceForm.overtimeHoursWeekday = 0;
    attendanceForm.overtimeHoursWeekend = 0;
    attendanceForm.overtimeHoursHoliday = 0;
  }
  formVisible.value = true;
}

async function submitForm() {
  if (!attendanceForm.employeeId) {
    message.warning($t('page.finance.attendance.drawer.employeeIdPlaceholder'));
    return;
  }
  formSubmitting.value = true;
  try {
    await upsertAttendanceApi({ ...attendanceForm });
    message.success($t('page.finance.common.saveSuccess'));
    formVisible.value = false;
    await loadData();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    formSubmitting.value = false;
  }
}

async function handleDelete(id: number) {
  try {
    await deleteAttendanceApi(id);
    message.success($t('page.finance.common.deleteSuccess'));
    await loadData();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.deleteFailed'));
  }
}

// ===== 计算扣款弹窗 =====
const calcVisible = ref(false);
const calcLoading = ref(false);
const calcResult = ref<any>(null);
const calcTarget = ref<any>(null);

async function openCalcModal(record: any) {
  calcTarget.value = record;
  calcResult.value = null;
  calcVisible.value = true;
  calcLoading.value = true;
  try {
    const res: any = await calculateAttendanceDeductionApi({
      employeeId: record.employeeId,
      year: record.year,
      month: record.month,
    });
    calcResult.value = res?.data || res || {};
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.attendance.message.calcFailed'));
    calcResult.value = {};
  } finally {
    calcLoading.value = false;
  }
}

// ===== 批量导入 =====
const importVisible = ref(false);
const importLoading = ref(false);
const importData = ref<any[]>([]);

function openImportModal() {
  importData.value = [];
  importVisible.value = true;
}

async function handleImportSubmit() {
  if (importData.value.length === 0) {
    message.warning($t('page.finance.attendance.import.uploadFirst'));
    return;
  }
  importLoading.value = true;
  try {
    await batchImportAttendanceApi(importData.value);
    message.success($t('page.finance.attendance.message.importSuccess'));
    importVisible.value = false;
    await loadData();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.attendance.message.importFailed'));
  } finally {
    importLoading.value = false;
  }
}

// 简化处理：上传文件后提示用户（实际解析需后端或专门库）
function beforeUpload(file: File) {
  message.info(
    $t('page.finance.attendance.import.selectedFile', { name: file.name }),
  );
  importData.value = [{ fileName: file.name, file }];
  return false; // 阻止自动上传，仅记录
}

onMounted(() => {
  loadData();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.attendance.guide.title')"
      :brief="$t('page.finance.attendance.guide.brief')"
      :expand-text="$t('page.finance.attendance.guide.expand')"
      :collapse-text="$t('page.finance.attendance.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.attendance.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.attendance.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Card :bordered="false">
      <!-- 搜索栏 -->
      <div class="mb-4 flex flex-wrap items-center gap-3">
        <span>{{ $t('page.finance.attendance.search.yearLabel') }}：</span>
        <InputNumber
          v-model:value="searchForm.year"
          :min="2020"
          :max="2099"
          style="width: 120px"
          :placeholder="$t('page.finance.attendance.search.yearPlaceholder')"
        />
        <span>{{ $t('page.finance.attendance.search.monthLabel') }}：</span>
        <Select
          v-model:value="searchForm.month"
          :options="monthOptions"
          allow-clear
          :placeholder="$t('page.finance.attendance.search.allMonths')"
          style="width: 120px"
        />
        <span>{{ $t('page.finance.common.employeeId') }}：</span>
        <UserPickerModal v-model:value="searchForm.employeeId" style="width: 160px" />
        <Button type="primary" @click="loadData">
          {{ $t('page.finance.common.query') }}
        </Button>
        <Button @click="loadData">{{ $t('page.finance.common.refresh') }}</Button>
        <div class="flex-1" />
        <Button v-if="canManage" type="primary" @click="openForm()">
          {{ $t('page.finance.attendance.button.create') }}
        </Button>
        <Button v-if="canManage" @click="openImportModal">
          {{ $t('page.finance.attendance.button.import') }}
        </Button>
      </div>

      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        row-key="id"
        :pagination="{ pageSize: 20, showSizeChanger: true }"
        size="middle"
        :scroll="{ x: 1500 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'action'">
            <Button
              type="link"
              size="small"
              @click="openCalcModal(record)"
            >
              {{ $t('page.finance.attendance.button.calc') }}
            </Button>
            <Button
              v-if="canManage"
              type="link"
              size="small"
              @click="openForm(record)"
            >
              {{ $t('page.finance.common.edit') }}
            </Button>
            <Popconfirm
              v-if="canManage"
              :title="$t('page.finance.common.deleteConfirm')"
              @confirm="handleDelete(record.id)"
            >
              <Button type="link" size="small" danger>
                {{ $t('page.finance.common.delete') }}
              </Button>
            </Popconfirm>
          </template>
        </template>
      </Table>
    </Card>

    <!-- 新增/编辑弹窗 -->
    <Modal
      v-model:open="formVisible"
      :title="
        attendanceForm.id
          ? $t('page.finance.attendance.drawer.titleEdit')
          : $t('page.finance.attendance.drawer.titleCreate')
      "
      :confirm-loading="formSubmitting"
      width="720px"
      @ok="submitForm"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.employeeId')" required>
              <UserPickerModal v-model:value="attendanceForm.employeeId" :disabled="!!attendanceForm.id" />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.search.yearLabel')" required>
              <InputNumber
                v-model:value="attendanceForm.year"
                :min="2020"
                :max="2099"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.search.monthLabel')" required>
              <Select
                v-model:value="attendanceForm.month"
                :options="monthOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="6">
            <FormItem :label="$t('page.finance.attendance.drawer.workDays')">
              <InputNumber
                v-model:value="attendanceForm.workDays"
                :min="0"
                :max="31"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.attendance.drawer.actualWorkDays')">
              <InputNumber
                v-model:value="attendanceForm.actualWorkDays"
                :min="0"
                :max="31"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.attendance.drawer.lateCount')">
              <InputNumber
                v-model:value="attendanceForm.lateCount"
                :min="0"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.attendance.drawer.earlyLeaveCount')">
              <InputNumber
                v-model:value="attendanceForm.earlyLeaveCount"
                :min="0"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.absentCount')">
              <InputNumber
                v-model:value="attendanceForm.absentCount"
                :min="0"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.personalLeaveDays')">
              <InputNumber
                v-model:value="attendanceForm.personalLeaveDays"
                :min="0"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.sickLeaveDays')">
              <InputNumber
                v-model:value="attendanceForm.sickLeaveDays"
                :min="0"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <div class="mb-2 font-semibold">
          {{ $t('page.finance.attendance.drawer.overtimeSection') }}
        </div>
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.overtimeWeekday')">
              <InputNumber
                v-model:value="attendanceForm.overtimeHoursWeekday"
                :min="0"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.overtimeWeekend')">
              <InputNumber
                v-model:value="attendanceForm.overtimeHoursWeekend"
                :min="0"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.attendance.drawer.overtimeHoliday')">
              <InputNumber
                v-model:value="attendanceForm.overtimeHoursHoliday"
                :min="0"
                :step="0.5"
                :precision="1"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
      </Form>
    </Modal>

    <!-- 计算扣款弹窗 -->
    <Modal
      v-model:open="calcVisible"
      :title="
        $t('page.finance.attendance.calc.title', {
          employeeId: calcTarget?.employeeId || '',
        })
      "
      :footer="null"
      width="640px"
    >
      <div v-if="calcLoading" class="py-8 text-center text-gray-500">
        {{ $t('page.finance.attendance.calc.calculating') }}
      </div>
      <div v-else-if="calcResult">
        <Descriptions
          bordered
          :column="2"
          size="small"
          class="mb-4"
        >
          <DescriptionsItem :label="$t('page.finance.common.yearMonth')">
            {{ calcTarget?.year }}{{ $t('page.finance.common.year') }}{{ calcTarget?.month }}{{ $t('page.finance.common.month') }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.finance.common.employeeId')">
            {{ calcTarget?.employeeId }}
          </DescriptionsItem>
        </Descriptions>
        <Row :gutter="16">
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.attendance.calc.lateDeduction')"
              :value="calcResult.lateDeduction || 0"
              :value-style="{ color: '#ff4d4f' }"
              :formatter="(v: any) => formatMoney(v.value)"
            />
          </Col>
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.attendance.calc.earlyLeaveDeduction')"
              :value="calcResult.earlyLeaveDeduction || 0"
              :value-style="{ color: '#ff4d4f' }"
              :formatter="(v: any) => formatMoney(v.value)"
            />
          </Col>
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.attendance.calc.absentDeduction')"
              :value="calcResult.absentDeduction || 0"
              :value-style="{ color: '#ff4d4f' }"
              :formatter="(v: any) => formatMoney(v.value)"
            />
          </Col>
        </Row>
        <Row :gutter="16" class="mt-4">
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.attendance.calc.leaveDeduction')"
              :value="calcResult.leaveDeduction || 0"
              :value-style="{ color: '#fa8c16' }"
              :formatter="(v: any) => formatMoney(v.value)"
            />
          </Col>
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.attendance.calc.overtimePay')"
              :value="calcResult.overtimePay || 0"
              :value-style="{ color: '#52c41a' }"
              :formatter="(v: any) => formatMoney(v.value)"
            />
          </Col>
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.attendance.calc.fullAttendanceBonus')"
              :value="calcResult.fullAttendanceBonus || 0"
              :value-style="{ color: '#722ed1' }"
              :formatter="(v: any) => formatMoney(v.value)"
            />
          </Col>
        </Row>
        <div class="mt-4 border-t border-gray-200 pt-4">
          <Statistic
            :title="$t('page.finance.attendance.calc.netAdjustment')"
            :value="calcResult.netAdjustment || 0"
            :value-style="{
              color: (calcResult.netAdjustment || 0) >= 0 ? '#52c41a' : '#ff4d4f',
              fontWeight: 'bold',
              fontSize: '20px',
            }"
            :formatter="(v: any) => formatMoney(v.value)"
          />
        </div>
      </div>
      <div v-else class="py-8 text-center text-gray-500">
        {{ $t('page.finance.common.noData') }}
      </div>
    </Modal>

    <!-- 批量导入弹窗 -->
    <Modal
      v-model:open="importVisible"
      :title="$t('page.finance.attendance.import.title')"
      :confirm-loading="importLoading"
      width="520px"
      @ok="handleImportSubmit"
    >
      <div class="py-4">
        <p class="mb-3 text-sm text-gray-500">
          {{ $t('page.finance.attendance.import.tip') }}
        </p>
        <Upload
          :before-upload="beforeUpload"
          :max-count="1"
          accept=".xlsx,.xls"
        >
          <Button type="primary">
            {{ $t('page.finance.attendance.import.selectFile') }}
          </Button>
        </Upload>
        <div v-if="importData.length > 0" class="mt-3 text-sm text-green-600">
          {{
            $t('page.finance.attendance.import.selected', {
              name: importData[0]?.fileName,
            })
          }}
        </div>
      </div>
    </Modal>
  </Page>
</template>
