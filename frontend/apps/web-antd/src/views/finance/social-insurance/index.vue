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
  Input,
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
  deleteInsurancePolicyApi,
  getEmployeeInsuranceConfigListApi,
  getInsurancePolicyListApi,
  upsertEmployeeInsuranceConfigApi,
  upsertInsurancePolicyApi,
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

function formatRate(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `${Number(val).toFixed(4)}%`;
}

// ===== Tab1: 城市政策库 =====
const activeTab = ref('policy');
const policyLoading = ref(false);
const policyList = ref<any[]>([]);
const policyFilterYear = ref<number | undefined>(undefined);

const policyColumns = computed(() => [
  { title: $t('page.finance.insurance.column.cityCode'), dataIndex: 'cityCode', width: 100 },
  { title: $t('page.finance.insurance.column.cityName'), dataIndex: 'cityName', width: 120 },
  {
    title: $t('page.finance.insurance.column.year'),
    dataIndex: 'year',
    width: 80,
    customRender: ({ text }: any) => `${text}${$t('page.finance.common.year')}`,
  },
  {
    title: $t('page.finance.insurance.column.baseLower'),
    dataIndex: 'baseLower',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.insurance.column.baseUpper'),
    dataIndex: 'baseUpper',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.insurance.column.pensionCompany'),
    dataIndex: 'pensionEnterprise',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.pensionPersonal'),
    dataIndex: 'pensionPersonal',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.medicalCompany'),
    dataIndex: 'medicalEnterprise',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.medicalPersonal'),
    dataIndex: 'medicalPersonal',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.unemploymentCompany'),
    dataIndex: 'unemploymentEnterprise',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.unemploymentPersonal'),
    dataIndex: 'unemploymentPersonal',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.workinjuryCompany'),
    dataIndex: 'injuryEnterprise',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.maternityCompany'),
    dataIndex: 'maternityEnterprise',
    width: 110,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.housingFundCompany'),
    dataIndex: 'housingFundEnterprise',
    width: 120,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  {
    title: $t('page.finance.insurance.column.housingFundPersonal'),
    dataIndex: 'housingFundPersonal',
    width: 120,
    align: 'right' as const,
    customRender: ({ text }: any) => formatRate(text),
  },
  { title: $t('page.finance.common.action'), key: 'action', width: 140, fixed: 'right' as const },
]);

async function loadPolicyList() {
  policyLoading.value = true;
  try {
    const res: any = await getInsurancePolicyListApi({
      year: policyFilterYear.value,
    });
    const data = res?.data || res;
    policyList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.insurance.message.loadPolicyFailed'));
    policyList.value = [];
  } finally {
    policyLoading.value = false;
  }
}

// 政策表单
const policyFormVisible = ref(false);
const policyFormSubmitting = ref(false);
const policyForm = reactive({
  id: undefined as number | undefined,
  cityCode: '',
  cityName: '',
  year: new Date().getFullYear(),
  baseLower: 0,
  baseUpper: 0,
  pensionEnterprise: 0,
  pensionPersonal: 0,
  medicalEnterprise: 0,
  medicalPersonal: 0,
  unemploymentEnterprise: 0,
  unemploymentPersonal: 0,
  injuryEnterprise: 0,
  maternityEnterprise: 0,
  housingFundEnterprise: 0,
  housingFundPersonal: 0,
});

function openPolicyForm(record?: any) {
  if (record) {
    policyForm.id = record.id;
    policyForm.cityCode = record.cityCode || '';
    policyForm.cityName = record.cityName || '';
    policyForm.year = record.year;
    policyForm.baseLower = Number(record.baseLower || 0);
    policyForm.baseUpper = Number(record.baseUpper || 0);
    policyForm.pensionEnterprise = Number(record.pensionEnterprise || 0);
    policyForm.pensionPersonal = Number(record.pensionPersonal || 0);
    policyForm.medicalEnterprise = Number(record.medicalEnterprise || 0);
    policyForm.medicalPersonal = Number(record.medicalPersonal || 0);
    policyForm.unemploymentEnterprise = Number(
      record.unemploymentEnterprise || 0,
    );
    policyForm.unemploymentPersonal = Number(
      record.unemploymentPersonal || 0,
    );
    policyForm.injuryEnterprise = Number(record.injuryEnterprise || 0);
    policyForm.maternityEnterprise = Number(record.maternityEnterprise || 0);
    policyForm.housingFundEnterprise = Number(
      record.housingFundEnterprise || 0,
    );
    policyForm.housingFundPersonal = Number(
      record.housingFundPersonal || 0,
    );
  } else {
    policyForm.id = undefined;
    policyForm.cityCode = '';
    policyForm.cityName = '';
    policyForm.year = new Date().getFullYear();
    policyForm.baseLower = 0;
    policyForm.baseUpper = 0;
    policyForm.pensionEnterprise = 0;
    policyForm.pensionPersonal = 0;
    policyForm.medicalEnterprise = 0;
    policyForm.medicalPersonal = 0;
    policyForm.unemploymentEnterprise = 0;
    policyForm.unemploymentPersonal = 0;
    policyForm.injuryEnterprise = 0;
    policyForm.maternityEnterprise = 0;
    policyForm.housingFundEnterprise = 0;
    policyForm.housingFundPersonal = 0;
  }
  policyFormVisible.value = true;
}

async function submitPolicyForm() {
  if (!policyForm.cityCode) {
    message.warning($t('page.finance.insurance.message.cityCodeRequired'));
    return;
  }
  if (!policyForm.cityName) {
    message.warning($t('page.finance.insurance.message.cityNameRequired'));
    return;
  }
  policyFormSubmitting.value = true;
  try {
    await upsertInsurancePolicyApi({ ...policyForm });
    message.success($t('page.finance.common.saveSuccess'));
    policyFormVisible.value = false;
    await loadPolicyList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    policyFormSubmitting.value = false;
  }
}

async function handleDeletePolicy(id: number) {
  try {
    await deleteInsurancePolicyApi(id);
    message.success($t('page.finance.common.deleteSuccess'));
    await loadPolicyList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.insurance.message.deleteFailed'));
  }
}

// ===== Tab2: 员工社保配置 =====
const empInsLoading = ref(false);
const empInsList = ref<any[]>([]);

const participateMap: Record<number, { label: string; color: string }> = {
  0: { label: $t('page.finance.insurance.status.notParticipated'), color: 'default' },
  1: { label: $t('page.finance.insurance.status.participated'), color: 'green' },
};

const empInsColumns = computed(() => [
  { title: $t('page.finance.common.employeeId'), dataIndex: 'employeeId', width: 90 },
  { title: $t('page.finance.common.employeeName'), dataIndex: 'employeeName', width: 120 },
  { title: $t('page.finance.insurance.column.cityCode'), dataIndex: 'cityCode', width: 100 },
  { title: $t('page.finance.insurance.column.cityName'), dataIndex: 'cityName', width: 120 },
  {
    title: $t('page.finance.insurance.column.baseAmount'),
    dataIndex: 'baseAmount',
    align: 'right' as const,
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.insurance.column.pension'),
    dataIndex: 'pensionParticipate',
    width: 90,
    customRender: ({ text }: any) => {
      const m = participateMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.insurance.column.medical'),
    dataIndex: 'medicalParticipate',
    width: 90,
    customRender: ({ text }: any) => {
      const m = participateMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.insurance.column.unemployment'),
    dataIndex: 'unemploymentParticipate',
    width: 90,
    customRender: ({ text }: any) => {
      const m = participateMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.insurance.column.workinjury'),
    dataIndex: 'injuryParticipate',
    width: 90,
    customRender: ({ text }: any) => {
      const m = participateMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.insurance.column.maternity'),
    dataIndex: 'maternityParticipate',
    width: 90,
    customRender: ({ text }: any) => {
      const m = participateMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  {
    title: $t('page.finance.insurance.column.housingFund'),
    dataIndex: 'housingFundParticipate',
    width: 90,
    customRender: ({ text }: any) => {
      const m = participateMap[text as number];
      return m ? h(Tag, { color: m.color }, () => m.label) : text;
    },
  },
  { title: $t('page.finance.common.action'), key: 'action', width: 100, fixed: 'right' as const },
]);

async function loadEmpInsList() {
  empInsLoading.value = true;
  try {
    const res: any = await getEmployeeInsuranceConfigListApi();
    const data = res?.data || res;
    empInsList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.insurance.message.loadEmpConfigFailed'));
    empInsList.value = [];
  } finally {
    empInsLoading.value = false;
  }
}

// 员工社保配置表单
const empInsFormVisible = ref(false);
const empInsFormSubmitting = ref(false);
const empInsForm = reactive({
  id: undefined as number | undefined,
  employeeId: undefined as number | undefined,
  cityCode: '',
  baseAmount: 0,
  pensionParticipate: 1,
  medicalParticipate: 1,
  unemploymentParticipate: 1,
  injuryParticipate: 1,
  maternityParticipate: 1,
  housingFundParticipate: 1,
});

const participateOptions = [
  { value: 0, label: $t('page.finance.insurance.status.notParticipated') },
  { value: 1, label: $t('page.finance.insurance.status.participated') },
];

function openEmpInsForm(record?: any) {
  if (record) {
    empInsForm.id = record.id;
    empInsForm.employeeId = record.employeeId;
    empInsForm.cityCode = record.cityCode || '';
    empInsForm.baseAmount = Number(record.baseAmount || 0);
    empInsForm.pensionParticipate = Number(record.pensionParticipate ?? 1);
    empInsForm.medicalParticipate = Number(record.medicalParticipate ?? 1);
    empInsForm.unemploymentParticipate = Number(
      record.unemploymentParticipate ?? 1,
    );
    empInsForm.injuryParticipate = Number(record.injuryParticipate ?? 1);
    empInsForm.maternityParticipate = Number(record.maternityParticipate ?? 1);
    empInsForm.housingFundParticipate = Number(
      record.housingFundParticipate ?? 1,
    );
  } else {
    empInsForm.id = undefined;
    empInsForm.employeeId = undefined;
    empInsForm.cityCode = '';
    empInsForm.baseAmount = 0;
    empInsForm.pensionParticipate = 1;
    empInsForm.medicalParticipate = 1;
    empInsForm.unemploymentParticipate = 1;
    empInsForm.injuryParticipate = 1;
    empInsForm.maternityParticipate = 1;
    empInsForm.housingFundParticipate = 1;
  }
  empInsFormVisible.value = true;
}

async function submitEmpInsForm() {
  if (!empInsForm.employeeId) {
    message.warning($t('page.finance.insurance.message.employeeIdRequired'));
    return;
  }
  if (!empInsForm.cityCode) {
    message.warning($t('page.finance.insurance.message.cityCodeRequired'));
    return;
  }
  empInsFormSubmitting.value = true;
  try {
    await upsertEmployeeInsuranceConfigApi({ ...empInsForm });
    message.success($t('page.finance.common.saveSuccess'));
    empInsFormVisible.value = false;
    await loadEmpInsList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    empInsFormSubmitting.value = false;
  }
}

function onTabChange(key: Key) {
  if (key === 'policy' && policyList.value.length === 0) {
    loadPolicyList();
  } else if (key === 'empIns' && empInsList.value.length === 0) {
    loadEmpInsList();
  }
}

onMounted(() => {
  loadPolicyList();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.insurance.guide.title')"
      :brief="$t('page.finance.insurance.guide.brief')"
      :expand-text="$t('page.finance.insurance.guide.expand')"
      :collapse-text="$t('page.finance.insurance.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.insurance.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.insurance.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Card :bordered="false">
      <Tabs v-model:active-key="activeTab" @change="onTabChange">
        <TabPane key="policy" :tab="$t('page.finance.insurance.tab.policy')">
          <div class="mb-4 flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span>{{ $t('page.finance.insurance.column.year') }}：</span>
              <InputNumber
                v-model:value="policyFilterYear"
                :min="2020"
                :max="2099"
                style="width: 140px"
                :placeholder="$t('page.finance.insurance.placeholder.allYears')"
              />
              <Button @click="loadPolicyList">{{ $t('page.finance.common.refresh') }}</Button>
            </div>
            <Button
              v-if="canManage"
              type="primary"
              @click="openPolicyForm()"
            >
              {{ $t('page.finance.insurance.button.createPolicyShort') }}
            </Button>
          </div>
          <Table
            :columns="policyColumns"
            :data-source="policyList"
            :loading="policyLoading"
            row-key="id"
            :pagination="false"
            size="middle"
            :scroll="{ x: 1700 }"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'action'">
                <Button
                  v-if="canManage"
                  type="link"
                  size="small"
                  @click="openPolicyForm(record)"
                >
                  {{ $t('page.finance.common.edit') }}
                </Button>
                <Popconfirm
                  v-if="canManage"
                  :title="$t('page.finance.insurance.message.deletePolicyConfirm')"
                  @confirm="handleDeletePolicy(record.id)"
                >
                  <Button type="link" size="small" danger>{{ $t('page.finance.common.delete') }}</Button>
                </Popconfirm>
              </template>
            </template>
          </Table>
        </TabPane>

        <TabPane key="empIns" :tab="$t('page.finance.insurance.tab.empConfig')">
          <div class="mb-4 flex items-center justify-between">
            <Button @click="loadEmpInsList">{{ $t('page.finance.common.refresh') }}</Button>
          </div>
          <Table
            :columns="empInsColumns"
            :data-source="empInsList"
            :loading="empInsLoading"
            row-key="id"
            :pagination="false"
            size="middle"
            :scroll="{ x: 1300 }"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'action'">
                <Button
                  v-if="canManage"
                  type="link"
                  size="small"
                  @click="openEmpInsForm(record)"
                >
                  {{ $t('page.finance.common.edit') }}
                </Button>
              </template>
            </template>
          </Table>
        </TabPane>
      </Tabs>
    </Card>

    <!-- 城市政策编辑弹窗 -->
    <Modal
      v-model:open="policyFormVisible"
      :title="policyForm.id ? $t('page.finance.insurance.drawer.titlePolicyEdit') : $t('page.finance.insurance.drawer.titlePolicyCreate')"
      :confirm-loading="policyFormSubmitting"
      width="760px"
      @ok="submitPolicyForm"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.cityCode')" required>
              <Input
                v-model:value="policyForm.cityCode"
                :placeholder="$t('page.finance.insurance.drawer.cityCodePlaceholder')"
                :disabled="!!policyForm.id"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.cityName')" required>
              <Input v-model:value="policyForm.cityName" :placeholder="$t('page.finance.insurance.drawer.cityNamePlaceholder')" />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.year')" required>
              <InputNumber
                v-model:value="policyForm.year"
                :min="2020"
                :max="2099"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.insurance.drawer.baseLower')" required>
              <InputNumber
                v-model:value="policyForm.baseLower"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.insurance.drawer.baseUpper')" required>
              <InputNumber
                v-model:value="policyForm.baseUpper"
                :min="0"
                :precision="2"
                style="width: 100%"
                prefix="¥"
              />
            </FormItem>
          </Col>
        </Row>
        <div class="mb-2 font-semibold">{{ $t('page.finance.insurance.drawer.rateTip') }}</div>
        <Row :gutter="16">
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.pensionCompany')">
              <InputNumber
                v-model:value="policyForm.pensionEnterprise"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.pensionPersonal')">
              <InputNumber
                v-model:value="policyForm.pensionPersonal"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.medicalCompany')">
              <InputNumber
                v-model:value="policyForm.medicalEnterprise"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.medicalPersonal')">
              <InputNumber
                v-model:value="policyForm.medicalPersonal"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.unemploymentCompany')">
              <InputNumber
                v-model:value="policyForm.unemploymentEnterprise"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.unemploymentPersonal')">
              <InputNumber
                v-model:value="policyForm.unemploymentPersonal"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.workinjuryCompany')">
              <InputNumber
                v-model:value="policyForm.injuryEnterprise"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="6">
            <FormItem :label="$t('page.finance.insurance.column.maternityCompany')">
              <InputNumber
                v-model:value="policyForm.maternityEnterprise"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.insurance.column.housingFundCompany')">
              <InputNumber
                v-model:value="policyForm.housingFundEnterprise"
                :min="0"
                :max="100"
                :step="1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.insurance.column.housingFundPersonal')">
              <InputNumber
                v-model:value="policyForm.housingFundPersonal"
                :min="0"
                :max="100"
                :step="1"
                :precision="4"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
      </Form>
    </Modal>

    <!-- 员工社保配置编辑弹窗 -->
    <Modal
      v-model:open="empInsFormVisible"
      :title="empInsForm.id ? $t('page.finance.insurance.drawer.titleEmpConfigEdit') : $t('page.finance.insurance.drawer.titleEmpConfigCreate')"
      :confirm-loading="empInsFormSubmitting"
      width="640px"
      @ok="submitEmpInsForm"
    >
      <Form layout="vertical" class="py-4" autocomplete="off">
        <Row :gutter="16">
          <Col :span="12">
            <FormItem :label="$t('page.finance.common.employeeId')" required>
              <UserPickerModal v-model:value="empInsForm.employeeId" :disabled="!!empInsForm.id" />
            </FormItem>
          </Col>
          <Col :span="12">
            <FormItem :label="$t('page.finance.insurance.drawer.cityCode')" required>
              <Input
                v-model:value="empInsForm.cityCode"
                :placeholder="$t('page.finance.insurance.drawer.cityCodePlaceholder')"
              />
            </FormItem>
          </Col>
        </Row>
        <FormItem :label="$t('page.finance.insurance.drawer.baseAmount')" required>
          <InputNumber
            v-model:value="empInsForm.baseAmount"
            :min="0"
            :precision="2"
            style="width: 100%"
            prefix="¥"
          />
        </FormItem>
        <div class="mb-2 font-semibold">{{ $t('page.finance.insurance.drawer.participateTitle') }}</div>
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.pensionInsurance')">
              <Select
                v-model:value="empInsForm.pensionParticipate"
                :options="participateOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.medicalInsurance')">
              <Select
                v-model:value="empInsForm.medicalParticipate"
                :options="participateOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.unemploymentInsurance')">
              <Select
                v-model:value="empInsForm.unemploymentParticipate"
                :options="participateOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.workinjuryInsurance')">
              <Select
                v-model:value="empInsForm.injuryParticipate"
                :options="participateOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.maternityInsurance')">
              <Select
                v-model:value="empInsForm.maternityParticipate"
                :options="participateOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.insurance.drawer.housingFundLabel')">
              <Select
                v-model:value="empInsForm.housingFundParticipate"
                :options="participateOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
      </Form>
    </Modal>
  </Page>
</template>
