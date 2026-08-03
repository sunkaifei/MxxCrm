<script lang="ts" setup>
import { computed, h, onMounted, reactive, ref } from 'vue';
import type { Key } from 'ant-design-vue/es/table/interface';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Col,
  Form,
  FormItem,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Row,
  Select,
  Table,
  Tabs,
  TabPane,
  Tag,
} from 'ant-design-vue';
import { UserPickerModal } from '#/components/UserPickerModal';

import {
  deleteTaxRateApi,
  getEmployeeTaxConfigListApi,
  getTaxRateListApi,
  upsertEmployeeTaxConfigApi,
  upsertTaxRateApi,
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
  return `¥${Number(val).toLocaleString()}`;
}

const taxTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: $t('page.finance.tax.taxType.cumulative'), color: 'blue' },
  2: { label: $t('page.finance.tax.taxType.annualBonus'), color: 'purple' },
};

const taxTypeOptions = [
  { value: 1, label: $t('page.finance.tax.taxType.cumulative') },
  { value: 2, label: $t('page.finance.tax.taxType.annualBonus') },
];

// ===== Tab1: 税率表管理 =====
const activeTab = ref('rate');
const rateLoading = ref(false);
const rateList = ref<any[]>([]);
const rateFilterType = ref<number | undefined>(undefined);

const rateColumns = computed(() => [
  { title: $t('page.finance.tax.column.level'), dataIndex: 'level', width: 80 },
  {
    title: $t('page.finance.tax.column.taxType'),
    dataIndex: 'taxType',
    width: 140,
    customRender: ({ text }: any) => {
      const m = taxTypeMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.tax.column.minAmount'),
    dataIndex: 'minAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.tax.column.maxAmount'),
    dataIndex: 'maxAmount',
    align: 'right' as const,
    customRender: ({ text }: any) =>
      text === null || text === undefined
        ? $t('page.finance.tax.column.above')
        : formatMoney(text),
  },
  {
    title: $t('page.finance.tax.column.rate'),
    dataIndex: 'rate',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) => `${Number(text || 0).toFixed(2)}%`,
  },
  {
    title: $t('page.finance.tax.column.quickDeduction'),
    dataIndex: 'quickDeduction',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 140,
    fixed: 'right' as const,
  },
]);

async function loadRateList() {
  rateLoading.value = true;
  try {
    const res: any = await getTaxRateListApi({
      taxType: rateFilterType.value,
    });
    const data = res?.data || res;
    rateList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.tax.message.loadFailed'));
    rateList.value = [];
  } finally {
    rateLoading.value = false;
  }
}

// 税率表表单
const rateFormVisible = ref(false);
const rateFormSubmitting = ref(false);
const rateForm = reactive({
  id: undefined as number | undefined,
  level: undefined as number | undefined,
  taxType: 1,
  minAmount: 0,
  maxAmount: undefined as number | undefined,
  rate: 0,
  quickDeduction: 0,
});

function openRateForm(record?: any) {
  if (record) {
    rateForm.id = record.id;
    rateForm.level = record.level;
    rateForm.taxType = record.taxType;
    rateForm.minAmount = Number(record.minAmount || 0);
    rateForm.maxAmount =
      record.maxAmount === null || record.maxAmount === undefined
        ? undefined
        : Number(record.maxAmount);
    rateForm.rate = Number(record.rate || 0);
    rateForm.quickDeduction = Number(record.quickDeduction || 0);
  } else {
    rateForm.id = undefined;
    rateForm.level = undefined;
    rateForm.taxType = 1;
    rateForm.minAmount = 0;
    rateForm.maxAmount = undefined;
    rateForm.rate = 0;
    rateForm.quickDeduction = 0;
  }
  rateFormVisible.value = true;
}

async function submitRateForm() {
  if (!rateForm.level) {
    message.warning($t('page.finance.tax.message.levelRequired'));
    return;
  }
  if (rateForm.rate < 0 || rateForm.rate > 100) {
    message.warning($t('page.finance.tax.message.rateRange'));
    return;
  }
  rateFormSubmitting.value = true;
  try {
    await upsertTaxRateApi({
      id: rateForm.id,
      level: rateForm.level,
      taxType: rateForm.taxType,
      minAmount: rateForm.minAmount,
      maxAmount: rateForm.maxAmount,
      rate: rateForm.rate,
      quickDeduction: rateForm.quickDeduction,
    });
    message.success($t('page.finance.tax.message.saveSuccess'));
    rateFormVisible.value = false;
    await loadRateList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    rateFormSubmitting.value = false;
  }
}

async function handleDeleteRate(id: number) {
  try {
    await deleteTaxRateApi(id);
    message.success($t('page.finance.tax.message.deleteSuccess'));
    await loadRateList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  }
}

// ===== Tab2: 员工个税配置 =====
const empConfigLoading = ref(false);
const empConfigList = ref<any[]>([]);
const empConfigFilterYear = ref<number | undefined>(undefined);

// 7项专项附加扣除
const specialDeductionFields = [
  { field: 'childrenEducation', label: $t('page.finance.tax.column.childrenEducation') },
  { field: 'continuingEducation', label: $t('page.finance.tax.column.continuingEducation') },
  { field: 'seriousIllness', label: $t('page.finance.tax.column.seriousIllness') },
  { field: 'housingLoan', label: $t('page.finance.tax.column.housingLoan') },
  { field: 'housingRent', label: $t('page.finance.tax.column.housingRent') },
  { field: 'supportingElderly', label: $t('page.finance.tax.column.supportingElderly') },
  { field: 'infantCare', label: $t('page.finance.tax.column.infantCare') },
];

const empConfigColumns = computed(() => {
  const cols: any[] = [
    { title: $t('page.finance.common.employeeId'), dataIndex: 'employeeId', width: 90 },
    { title: $t('page.finance.common.employeeName'), dataIndex: 'employeeName', width: 120 },
    {
      title: $t('page.finance.tax.column.year'),
      dataIndex: 'year',
      width: 80,
      customRender: ({ text }: any) => `${text}${$t('page.finance.common.year')}`,
    },
    {
      title: $t('page.finance.tax.column.taxThreshold'),
      dataIndex: 'taxThreshold',
      align: 'right' as const,
      customRender: ({ text }: any) => formatMoney(text),
    },
  ];
  for (const item of specialDeductionFields) {
    cols.push({
      title: item.label,
      dataIndex: item.field,
      width: 120,
      align: 'right' as const,
      customRender: ({ text }: any) => formatMoney(text),
    });
  }
  cols.push({
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 100,
    fixed: 'right' as const,
  });
  return cols;
});

async function loadEmpConfigList() {
  empConfigLoading.value = true;
  try {
    const res: any = await getEmployeeTaxConfigListApi({
      year: empConfigFilterYear.value,
    });
    const data = res?.data || res;
    empConfigList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.tax.message.loadFailed'));
    empConfigList.value = [];
  } finally {
    empConfigLoading.value = false;
  }
}

// 员工配置表单
const empConfigFormVisible = ref(false);
const empConfigFormSubmitting = ref(false);
const empConfigForm = reactive({
  id: undefined as number | undefined,
  employeeId: undefined as number | undefined,
  year: new Date().getFullYear(),
  taxThreshold: 5000,
  childrenEducation: 0,
  continuingEducation: 0,
  seriousIllness: 0,
  housingLoan: 0,
  housingRent: 0,
  supportElderly: 0,
  infantCare: 0,
});

function openEmpConfigForm(record?: any) {
  if (record) {
    empConfigForm.id = record.id;
    empConfigForm.employeeId = record.employeeId;
    empConfigForm.year = record.year;
    empConfigForm.taxThreshold = Number(record.taxThreshold || 5000);
    for (const item of specialDeductionFields) {
      (empConfigForm as any)[item.field] = Number(
        (record as any)[item.field] || 0,
      );
    }
  } else {
    empConfigForm.id = undefined;
    empConfigForm.employeeId = undefined;
    empConfigForm.year = new Date().getFullYear();
    empConfigForm.taxThreshold = 5000;
    for (const item of specialDeductionFields) {
      (empConfigForm as any)[item.field] = 0;
    }
  }
  empConfigFormVisible.value = true;
}

async function submitEmpConfigForm() {
  if (!empConfigForm.employeeId) {
    message.warning($t('page.finance.tax.message.employeeIdRequired'));
    return;
  }
  empConfigFormSubmitting.value = true;
  try {
    await upsertEmployeeTaxConfigApi({ ...empConfigForm });
    message.success($t('page.finance.tax.message.saveSuccess'));
    empConfigFormVisible.value = false;
    await loadEmpConfigList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    empConfigFormSubmitting.value = false;
  }
}

function onTabChange(key: Key) {
  if (key === 'rate' && rateList.value.length === 0) {
    loadRateList();
  } else if (key === 'empConfig' && empConfigList.value.length === 0) {
    loadEmpConfigList();
  }
}

onMounted(() => {
  loadRateList();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.tax.guide.title')"
      :brief="$t('page.finance.tax.guide.brief')"
      :expand-text="$t('page.finance.tax.guide.expand')"
      :collapse-text="$t('page.finance.tax.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.tax.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.tax.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Card :bordered="false">
      <Tabs v-model:active-key="activeTab" @change="onTabChange">
        <TabPane key="rate" :tab="$t('page.finance.tax.tab.rateTable')">
          <div class="mb-4 flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span>{{ $t('page.finance.tax.column.taxType') }}：</span>
              <Select
                v-model:value="rateFilterType"
                :options="taxTypeOptions"
                allow-clear
                :placeholder="$t('page.finance.common.all')"
                style="width: 200px"
                @change="loadRateList"
              />
              <Button @click="loadRateList">{{ $t('page.finance.common.refresh') }}</Button>
            </div>
            <Button
              v-if="canManage"
              type="primary"
              @click="openRateForm()"
            >
              {{ $t('page.finance.tax.button.create') }}
            </Button>
          </div>
          <Table
            :columns="rateColumns"
            :data-source="rateList"
            :loading="rateLoading"
            row-key="id"
            :pagination="false"
            size="middle"
            :scroll="{ x: 900 }"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'action'">
                <Button
                  v-if="canManage"
                  type="link"
                  size="small"
                  @click="openRateForm(record)"
                >
                  {{ $t('page.finance.common.edit') }}
                </Button>
                <Popconfirm
                  v-if="canManage"
                  :title="$t('page.finance.tax.message.deleteConfirm')"
                  @confirm="handleDeleteRate(record.id)"
                >
                  <Button type="link" size="small" danger>{{ $t('page.finance.common.delete') }}</Button>
                </Popconfirm>
              </template>
            </template>
          </Table>
        </TabPane>

        <TabPane key="empConfig" :tab="$t('page.finance.tax.tab.empConfig')">
          <div class="mb-4 flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span>{{ $t('page.finance.tax.column.year') }}：</span>
              <InputNumber
                v-model:value="empConfigFilterYear"
                :min="2020"
                :max="2099"
                style="width: 140px"
                :placeholder="$t('page.finance.tax.drawer.yearPlaceholder')"
              />
              <Button @click="loadEmpConfigList">{{ $t('page.finance.common.refresh') }}</Button>
            </div>
          </div>
          <Table
            :columns="empConfigColumns"
            :data-source="empConfigList"
            :loading="empConfigLoading"
            row-key="id"
            :pagination="false"
            size="middle"
            :scroll="{ x: 1200 }"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'action'">
                <Button
                  v-if="canManage"
                  type="link"
                  size="small"
                  @click="openEmpConfigForm(record)"
                >
                  {{ $t('page.finance.common.edit') }}
                </Button>
              </template>
            </template>
          </Table>
        </TabPane>
      </Tabs>
    </Card>

    <!-- 税率表编辑弹窗 -->
    <Modal
      v-model:open="rateFormVisible"
      :title="rateForm.id ? $t('page.finance.tax.drawer.titleRateEdit') : $t('page.finance.tax.drawer.titleRateCreate')"
      :confirm-loading="rateFormSubmitting"
      width="560px"
      @ok="submitRateForm"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.drawer.level')" required>
              <InputNumber
                v-model:value="rateForm.level"
                :min="1"
                style="width: 100%"
                :placeholder="$t('page.finance.tax.drawer.levelPlaceholder')"
              />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.drawer.taxType')" required>
              <Select
                v-model:value="rateForm.taxType"
                :options="taxTypeOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.drawer.minAmount')" required>
              <InputNumber
                v-model:value="rateForm.minAmount"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.drawer.maxAmountHintLabel')">
              <InputNumber
                v-model:value="rateForm.maxAmount"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
                :placeholder="$t('page.finance.tax.drawer.maxAmountPlaceholder')"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.column.rate')" required>
              <InputNumber
                v-model:value="rateForm.rate"
                :min="0"
                :max="100"
                :step="1"
                :precision="2"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.drawer.quickDeduction')">
              <InputNumber
                v-model:value="rateForm.quickDeduction"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
        </Row>
      </Form>
    </Modal>

    <!-- 员工个税配置编辑弹窗 -->
    <Modal
      v-model:open="empConfigFormVisible"
      :title="empConfigForm.id ? $t('page.finance.tax.drawer.titleEmpConfigEdit') : $t('page.finance.tax.drawer.titleEmpConfigCreate')"
      :confirm-loading="empConfigFormSubmitting"
      width="640px"
      @ok="submitEmpConfigForm"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.common.employeeId')" required>
              <UserPickerModal v-model:value="empConfigForm.employeeId" :disabled="!!empConfigForm.id" />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.tax.drawer.year')" required>
              <InputNumber
                v-model:value="empConfigForm.year"
                :min="2020"
                :max="2099"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <FormItem :label="$t('page.finance.tax.drawer.taxThresholdUnitLabel')" required>
          <InputNumber
            v-model:value="empConfigForm.taxThreshold"
            :min="0"
            :precision="2"
            style="width: 100%"
            prefix="¥"
          />
        </FormItem>
        <div class="mb-2 font-semibold">{{ $t('page.finance.tax.drawer.specialDeductionUnitLabel') }}</div>
        <Row :gutter="16">
          <Col v-for="item in specialDeductionFields" :key="item.field" :span="12">
            <FormItem :label="item.label">
              <InputNumber
                v-model:value="(empConfigForm as any)[item.field]"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
        </Row>
      </Form>
    </Modal>
  </Page>
</template>
