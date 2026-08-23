<script lang="ts" setup>
import type { Key } from 'ant-design-vue/es/table/interface';

import { computed, h, onMounted, reactive, ref } from 'vue';

import { useAccess } from '@vben/access';
import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  Col,
  DatePicker,
  Divider,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Radio,
  Row,
  Select,
  Space,
  Table,
  TabPane,
  Tabs,
  Tag,
} from 'ant-design-vue';

import {
  deleteInsurancePolicyApi,
  getEmployeeInsuranceConfigListApi,
  getInsurancePolicyListApi,
  previewInsuranceCalcApi,
  upsertEmployeeInsuranceConfigApi,
  upsertInsurancePolicyApi,
} from '#/api/core/finance';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { UserPickerModal } from '#/components/UserPickerModal';
import { $t } from '#/locales';

const guideStepCount = 5;

// ===== 权限 =====
const { hasAccessByRoles } = useAccess();
const canManage = computed(() => hasAccessByRoles(['super_admin', 'finance']));

// ===== 通用工具 =====
function formatMoney(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `¥${Number(val).toLocaleString()}`;
}

// 存储小数比例(0.16) -> 录入百分数(16)
function rateToPct(val: any) {
  if (val === null || val === undefined || val === '') return 0;
  return Number((Number(val) * 100).toFixed(4));
}

// 录入百分数(16) -> 存储小数比例(0.16)
function pctToRate(val: any) {
  if (val === null || val === undefined || val === '') return 0;
  return Number((Number(val) / 100).toFixed(6));
}

function fmtDate(val: any) {
  if (!val) return '-';
  return String(val).slice(0, 10);
}

// 前端本地预览计算（比例为百分数，直接乘基数）
function localPreview(level: any) {
  const base = Number(level?.baseAmount || 0);
  const mk = (pctC: any, pctP: any) => {
    const company = Math.round(base * (Number(pctC) / 100) * 100) / 100;
    const personal = Math.round(base * (Number(pctP) / 100) * 100) / 100;
    return {
      company,
      personal,
      subtotal: Math.round((company + personal) * 100) / 100,
    };
  };
  const pension = mk(level.pensionCompanyRate, level.pensionPersonalRate);
  const medical = mk(level.medicalCompanyRate, level.medicalPersonalRate);
  const unemployment = mk(
    level.unemploymentCompanyRate,
    level.unemploymentPersonalRate,
  );
  const workinjury = mk(
    level.workinjuryCompanyRate,
    level.workinjuryPersonalRate,
  );
  const maternity = mk(
    level.maternityCompanyRate,
    level.maternityPersonalRate,
  );
  const housingFund = mk(
    level.housingFundCompanyRate,
    level.housingFundPersonalRate,
  );
  const ciCompany = Number(level.criticalIllnessCompanyAmount || 0);
  const ciPersonal = Number(level.criticalIllnessPersonalAmount || 0);
  const criticalIllness = {
    company: ciCompany,
    personal: ciPersonal,
    subtotal: ciCompany + ciPersonal,
  };
  const companyTotal =
    pension.company +
    medical.company +
    unemployment.company +
    workinjury.company +
    maternity.company +
    housingFund.company +
    criticalIllness.company;
  const personalTotal =
    pension.personal +
    medical.personal +
    unemployment.personal +
    workinjury.personal +
    maternity.personal +
    housingFund.personal +
    criticalIllness.personal;
  return {
    baseAmount: base,
    pension,
    medical,
    unemployment,
    workinjury,
    maternity,
    housingFund,
    criticalIllness,
    companyTotal: Math.round(companyTotal * 100) / 100,
    personalTotal: Math.round(personalTotal * 100) / 100,
    grandTotal: Math.round((companyTotal + personalTotal) * 100) / 100,
  };
}

// ===== Tab1: 城市政策库 =====
const activeTab = ref('policy');
const policyLoading = ref(false);
const policyList = ref<any[]>([]);
const policyFilterYear = ref<number | undefined>(undefined);

function minLevelBase(record: any) {
  const levels: any[] = record?.levels || [];
  if (levels.length === 0) return '-';
  return formatMoney(
    Math.min(...levels.map((l) => Number(l.baseAmount || 0))),
  );
}

const policyColumns = computed(() => [
  {
    title: $t('page.finance.insurance.column.cityCode'),
    key: 'cityCode',
    width: 100,
    customRender: ({ record }: any) => record.policy?.cityCode ?? '-',
  },
  {
    title: $t('page.finance.insurance.column.cityName'),
    key: 'cityName',
    width: 110,
    customRender: ({ record }: any) => record.policy?.cityName ?? '-',
  },
  {
    title: $t('page.finance.insurance.column.year'),
    key: 'year',
    width: 80,
    customRender: ({ record }: any) =>
      `${record.policy?.year ?? '-'}${$t('page.finance.common.year')}`,
  },
  {
    title: $t('page.finance.insurance.column.levels'),
    key: 'levelCount',
    width: 90,
    align: 'center' as const,
    customRender: ({ record }: any) => {
      const n = record?.levels?.length || 0;
      return h(
        Tag,
        { color: n > 0 ? 'green' : 'default' },
        () => `${n} ${$t('page.finance.insurance.column.levels')}`,
      );
    },
  },
  {
    title: $t('page.finance.insurance.column.levelBase'),
    key: 'levelBase',
    align: 'right' as const,
    width: 130,
    customRender: ({ record }: any) => minLevelBase(record),
  },
  {
    title: $t('page.finance.insurance.column.policyPeriod'),
    key: 'period',
    width: 210,
    customRender: ({ record }: any) =>
      `${fmtDate(record.policy?.effectiveDate)} ~ ${fmtDate(
        record.policy?.expiryDate,
      )}`,
  },
  {
    title: $t('page.finance.insurance.column.policyStatus'),
    key: 'status',
    width: 80,
    align: 'center' as const,
    customRender: ({ record }: any) => {
      const s = Number(record.policy?.status ?? 1);
      const m =
        s === 1
          ? { color: 'green', label: $t('page.finance.insurance.status.active') }
          : {
              color: 'default',
              label: $t('page.finance.insurance.status.inactive'),
            };
      return h(Tag, { color: m.color }, () => m.label);
    },
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 140,
    fixed: 'right' as const,
  },
]);

async function loadPolicyList() {
  policyLoading.value = true;
  try {
    const res: any = await getInsurancePolicyListApi({
      year: policyFilterYear.value,
    });
    const data = res?.data || res;
    policyList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.insurance.message.loadPolicyFailed'),
    );
    policyList.value = [];
  } finally {
    policyLoading.value = false;
  }
}

// ===== 政策表单 =====
function blankLevel(): any {
  return {
    id: undefined,
    levelType: 0,
    levelName: '',
    baseAmount: 0,
    baseLower: undefined,
    baseUpper: undefined,
    pensionCompanyRate: 0,
    pensionPersonalRate: 0,
    medicalCompanyRate: 0,
    medicalPersonalRate: 0,
    unemploymentCompanyRate: 0,
    unemploymentPersonalRate: 0,
    workinjuryCompanyRate: 0,
    workinjuryPersonalRate: 0,
    maternityCompanyRate: 0,
    maternityPersonalRate: 0,
    housingFundCompanyRate: 0,
    housingFundPersonalRate: 0,
    criticalIllnessCompanyAmount: 0,
    criticalIllnessPersonalAmount: 0,
  };
}

const policyFormVisible = ref(false);
const policyFormSubmitting = ref(false);
const policyForm = reactive({
  id: undefined as number | undefined,
  cityCode: '',
  cityName: '',
  year: new Date().getFullYear(),
  effectiveDate: undefined as string | undefined,
  expiryDate: undefined as string | undefined,
  status: 1,
  remark: '',
  levels: [] as any[],
});

const levelTypeOptions = [
  { value: 0, label: $t('page.finance.insurance.drawer.levelTypeLowest') },
  { value: 1, label: $t('page.finance.insurance.drawer.levelTypeHighest') },
  { value: 2, label: $t('page.finance.insurance.drawer.levelTypeCustom') },
];

function openPolicyForm(record?: any) {
  if (record) {
    const p = record.policy || {};
    policyForm.id = p.id;
    policyForm.cityCode = p.cityCode || '';
    policyForm.cityName = p.cityName || '';
    policyForm.year = p.year || new Date().getFullYear();
    policyForm.effectiveDate = fmtDate(p.effectiveDate);
    policyForm.expiryDate = fmtDate(p.expiryDate);
    policyForm.status = Number(p.status ?? 1);
    policyForm.remark = p.remark || '';
    policyForm.levels = (record.levels || []).map((lv: any) => ({
      id: lv.id,
      levelType: Number(lv.levelType ?? 0),
      levelName: lv.levelName || '',
      baseAmount: Number(lv.baseAmount || 0),
      baseLower: lv.baseLower === null || lv.baseLower === undefined ? undefined : Number(lv.baseLower),
      baseUpper: lv.baseUpper === null || lv.baseUpper === undefined ? undefined : Number(lv.baseUpper),
      pensionCompanyRate: rateToPct(lv.pensionCompanyRate),
      pensionPersonalRate: rateToPct(lv.pensionPersonalRate),
      medicalCompanyRate: rateToPct(lv.medicalCompanyRate),
      medicalPersonalRate: rateToPct(lv.medicalPersonalRate),
      unemploymentCompanyRate: rateToPct(lv.unemploymentCompanyRate),
      unemploymentPersonalRate: rateToPct(lv.unemploymentPersonalRate),
      workinjuryCompanyRate: rateToPct(lv.workinjuryCompanyRate),
      workinjuryPersonalRate: rateToPct(lv.workinjuryPersonalRate),
      maternityCompanyRate: rateToPct(lv.maternityCompanyRate),
      maternityPersonalRate: rateToPct(lv.maternityPersonalRate),
      housingFundCompanyRate: rateToPct(lv.housingFundCompanyRate),
      housingFundPersonalRate: rateToPct(lv.housingFundPersonalRate),
      criticalIllnessCompanyAmount: Number(
        lv.criticalIllnessCompanyAmount || 0,
      ),
      criticalIllnessPersonalAmount: Number(
        lv.criticalIllnessPersonalAmount || 0,
      ),
    }));
    if (policyForm.levels.length === 0) {
      policyForm.levels = [blankLevel()];
    }
  } else {
    policyForm.id = undefined;
    policyForm.cityCode = '';
    policyForm.cityName = '';
    policyForm.year = new Date().getFullYear();
    policyForm.effectiveDate = undefined;
    policyForm.expiryDate = undefined;
    policyForm.status = 1;
    policyForm.remark = '';
    policyForm.levels = [blankLevel()];
  }
  previewResult.value = undefined;
  policyFormVisible.value = true;
}

function addPolicyLevel() {
  policyForm.levels.push(blankLevel());
}

function removePolicyLevel(index: number) {
  if (policyForm.levels.length <= 1) {
    message.warning($t('page.finance.insurance.message.levelRequired'));
    return;
  }
  policyForm.levels.splice(index, 1);
  if (previewLevelIndex.value >= policyForm.levels.length) {
    previewLevelIndex.value = policyForm.levels.length - 1;
  }
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
  if (!policyForm.levels || policyForm.levels.length === 0) {
    message.warning($t('page.finance.insurance.message.levelRequired'));
    return;
  }
  for (const lv of policyForm.levels) {
    if (lv.levelType === undefined || lv.levelType === null) {
      message.warning($t('page.finance.insurance.message.levelTypeRequired'));
      return;
    }
    if (!lv.baseAmount || Number(lv.baseAmount) <= 0) {
      message.warning($t('page.finance.insurance.message.levelBaseRequired'));
      return;
    }
  }

  // 表头比例回填最低档（保证旧结构/降级计算可用）
  const first = policyForm.levels[0];
  const headerRates = {
    pensionCompanyRate: pctToRate(first.pensionCompanyRate),
    pensionPersonalRate: pctToRate(first.pensionPersonalRate),
    medicalCompanyRate: pctToRate(first.medicalCompanyRate),
    medicalPersonalRate: pctToRate(first.medicalPersonalRate),
    unemploymentCompanyRate: pctToRate(first.unemploymentCompanyRate),
    unemploymentPersonalRate: pctToRate(first.unemploymentPersonalRate),
    workinjuryCompanyRate: pctToRate(first.workinjuryCompanyRate),
    maternityCompanyRate: pctToRate(first.maternityCompanyRate),
    housingFundCompanyRate: pctToRate(first.housingFundCompanyRate),
    housingFundPersonalRate: pctToRate(first.housingFundPersonalRate),
  };

  const payload = {
    id: policyForm.id,
    cityCode: policyForm.cityCode,
    cityName: policyForm.cityName,
    year: policyForm.year,
    baseLower: first.baseLower ?? first.baseAmount,
    baseUpper: first.baseUpper ?? first.baseAmount,
    effectiveDate: policyForm.effectiveDate || undefined,
    expiryDate: policyForm.expiryDate || undefined,
    status: policyForm.status,
    remark: policyForm.remark,
    ...headerRates,
    levels: policyForm.levels.map((lv: any) => ({
      id: lv.id,
      levelType: lv.levelType,
      levelName: lv.levelName,
      baseAmount: lv.baseAmount,
      baseLower: lv.baseLower ?? lv.baseAmount,
      baseUpper: lv.baseUpper ?? lv.baseAmount,
      pensionCompanyRate: pctToRate(lv.pensionCompanyRate),
      pensionPersonalRate: pctToRate(lv.pensionPersonalRate),
      medicalCompanyRate: pctToRate(lv.medicalCompanyRate),
      medicalPersonalRate: pctToRate(lv.medicalPersonalRate),
      unemploymentCompanyRate: pctToRate(lv.unemploymentCompanyRate),
      unemploymentPersonalRate: pctToRate(lv.unemploymentPersonalRate),
      workinjuryCompanyRate: pctToRate(lv.workinjuryCompanyRate),
      workinjuryPersonalRate: pctToRate(lv.workinjuryPersonalRate),
      maternityCompanyRate: pctToRate(lv.maternityCompanyRate),
      maternityPersonalRate: pctToRate(lv.maternityPersonalRate),
      housingFundCompanyRate: pctToRate(lv.housingFundCompanyRate),
      housingFundPersonalRate: pctToRate(lv.housingFundPersonalRate),
      criticalIllnessCompanyAmount: lv.criticalIllnessCompanyAmount,
      criticalIllnessPersonalAmount: lv.criticalIllnessPersonalAmount,
    })),
  };

  policyFormSubmitting.value = true;
  try {
    await upsertInsurancePolicyApi(payload);
    message.success($t('page.finance.common.saveSuccess'));
    policyFormVisible.value = false;
    await loadPolicyList();
  } catch (error: any) {
    message.error(error?.message || $t('page.finance.common.saveFailed'));
  } finally {
    policyFormSubmitting.value = false;
  }
}

async function handleDeletePolicy(id: number) {
  try {
    await deleteInsurancePolicyApi(id);
    message.success($t('page.finance.common.deleteSuccess'));
    await loadPolicyList();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.insurance.message.deleteFailed'),
    );
  }
}

// ===== 政策实时预览 =====
const previewResult = ref<any>(undefined);
const previewLoading = ref(false);
// 用索引定位档次（新增未保存的档次 id 为 undefined，无法按 id 匹配）
const previewLevelIndex = ref<number>(0);

const previewColumns = [
  { title: $t('page.finance.insurance.drawer.insuranceSummary'), dataIndex: 'name', width: 130 },
  { title: $t('page.finance.insurance.drawer.company'), dataIndex: 'company', align: 'right' as const },
  { title: $t('page.finance.insurance.drawer.personal'), dataIndex: 'personal', align: 'right' as const },
  { title: $t('page.finance.insurance.drawer.subtotal'), dataIndex: 'subtotal', align: 'right' as const },
];

const previewRows = computed(() => {
  if (!previewResult.value) return [];
  const r = previewResult.value;
  const mk = (name: string, item: any) => ({
    name,
    company: formatMoney(item?.company),
    personal: formatMoney(item?.personal),
    subtotal: formatMoney(item?.subtotal),
  });
  const rows = [
    mk($t('page.finance.insurance.drawer.pensionInsurance'), r.pension),
    mk($t('page.finance.insurance.drawer.medicalInsurance'), r.medical),
    mk($t('page.finance.insurance.drawer.unemploymentInsurance'), r.unemployment),
    mk($t('page.finance.insurance.drawer.workinjuryInsurance'), r.workinjury),
    mk($t('page.finance.insurance.drawer.maternityInsurance'), r.maternity),
    mk($t('page.finance.insurance.drawer.housingFundLabel'), r.housingFund),
    mk($t('page.finance.insurance.column.criticalIllness'), r.criticalIllness),
  ];
  rows.push({
    name: $t('page.finance.insurance.drawer.grandTotal'),
    company: formatMoney(r.companyTotal),
    personal: formatMoney(r.personalTotal),
    subtotal: formatMoney(r.grandTotal),
  });
  return rows;
});

async function runPolicyPreview() {
  const level = policyForm.levels[previewLevelIndex.value];
  if (!level) {
    message.warning($t('page.finance.insurance.message.levelRequiredSelect'));
    return;
  }
  // 本地计算，支持未保存的新增档次即时预览
  previewResult.value = localPreview(level);
}

// ===== Tab2: 员工社保配置 =====
const empInsLoading = ref(false);
const empInsList = ref<any[]>([]);

function joinedTypes(record: any) {
  const types: string[] = [];
  if (record.participatePension === 1) types.push('养老');
  if (record.participateMedical === 1) types.push('医疗');
  if (record.participateUnemployment === 1) types.push('失业');
  if (record.participateWorkinjury === 1) types.push('工伤');
  if (record.participateMaternity === 1) types.push('生育');
  if (record.participateHousingFund === 1) types.push('公积金');
  if (record.participateCriticalIllness === 1) types.push('大病');
  return types;
}

const empInsColumns = computed(() => [
  {
    title: $t('page.finance.common.employeeId'),
    dataIndex: 'employeeId',
    width: 90,
  },
  {
    title: $t('page.finance.common.employeeName'),
    dataIndex: 'employeeName',
    width: 110,
  },
  {
    title: $t('page.finance.insurance.column.cityName'),
    dataIndex: 'cityName',
    width: 110,
  },
  {
    title: $t('page.finance.insurance.column.year'),
    key: 'policyYear',
    width: 80,
    customRender: ({ record }: any) =>
      record.policyYear ? `${record.policyYear}` : '-',
  },
  {
    title: $t('page.finance.insurance.column.levelName'),
    key: 'levelName',
    width: 110,
    customRender: ({ record }: any) =>
      record.levelName || $t('page.finance.insurance.drawer.levelTypeCustom'),
  },
  {
    title: $t('page.finance.insurance.drawer.baseUsed'),
    key: 'baseAmount',
    align: 'right' as const,
    width: 120,
    customRender: ({ record }: any) => formatMoney(record.baseAmount),
  },
  {
    title: $t('page.finance.insurance.column.status'),
    key: 'types',
    customRender: ({ record }: any) => {
      const types = joinedTypes(record);
      if (types.length === 0) {
        return h(
          Tag,
          { color: 'default' },
          () => $t('page.finance.insurance.status.notParticipated'),
        );
      }
      return h(
        Space,
        { size: 4, wrap: true },
        () =>
          types.map((t) =>
            h(Tag, { color: 'green', key: t }, () => t),
          ) as any,
      );
    },
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 100,
    fixed: 'right' as const,
  },
]);

async function loadEmpInsList() {
  empInsLoading.value = true;
  try {
    const res: any = await getEmployeeInsuranceConfigListApi();
    const data = res?.data || res;
    empInsList.value = Array.isArray(data) ? data : data?.items || [];
  } catch (error: any) {
    message.error(
      error?.message ||
        $t('page.finance.insurance.message.loadEmpConfigFailed'),
    );
    empInsList.value = [];
  } finally {
    empInsLoading.value = false;
  }
}

// ===== 员工社保配置表单 =====
const empInsFormVisible = ref(false);
const empInsFormSubmitting = ref(false);
const empInsForm = reactive({
  id: undefined as number | undefined,
  employeeId: undefined as number | undefined,
  cityCode: undefined as string | undefined,
  policyId: undefined as number | undefined,
  policyLevelId: undefined as number | undefined,
  usePolicyBase: true,
  baseAmount: 0,
  housingFundBase: undefined as number | undefined,
  housingFundCompanyRate: undefined as number | undefined,
  housingFundPersonalRate: undefined as number | undefined,
  participatePension: 1,
  participateMedical: 1,
  participateUnemployment: 1,
  participateWorkinjury: 1,
  participateMaternity: 1,
  participateHousingFund: 1,
  participateCriticalIllness: 1,
  workinjuryCompanyRate: undefined as number | undefined,
  workinjuryPersonalRate: undefined as number | undefined,
});

const participateOptions = [
  { value: 0, label: $t('page.finance.insurance.status.notParticipated') },
  { value: 1, label: $t('page.finance.insurance.status.participated') },
];

// 城市/政策/档次级联数据（来自政策库）
const cityOptions = computed(() => {
  const map = new Map<string, string>();
  for (const item of policyList.value) {
    const p = item.policy;
    if (p && !map.has(p.cityCode)) {
      map.set(p.cityCode, p.cityName || p.cityCode);
    }
  }
  return Array.from(map.entries()).map(([value, label]) => ({ value, label }));
});

const cityPolicies = computed(() =>
  policyList.value.filter((item) => item.policy?.cityCode === empInsForm.cityCode),
);

const policyOptions = computed(() =>
  cityPolicies.value.map((item) => ({
    value: item.policy.id,
    label: `${item.policy.year}${$t('page.finance.common.year')}(${fmtDate(
      item.policy.effectiveDate,
    )}~${fmtDate(item.policy.expiryDate)})`,
  })),
);

const selectedPolicyLevels = computed(() => {
  const item = cityPolicies.value.find(
    (i) => i.policy.id === empInsForm.policyId,
  );
  return item?.levels || [];
});

const levelOptions = computed(() =>
  selectedPolicyLevels.value.map((lv: any) => ({
    value: lv.id,
    label: `${lv.levelName || ''}(${formatMoney(lv.baseAmount)})`,
  })),
);

function onCityChange() {
  empInsForm.policyId = undefined;
  empInsForm.policyLevelId = undefined;
  empInsForm.usePolicyBase = true;
  empInsForm.baseAmount = 0;
}

function onPolicyChange() {
  empInsForm.policyLevelId = undefined;
  empInsForm.baseAmount = 0;
}

function onLevelChange() {
  const lv = selectedPolicyLevels.value.find(
    (l: any) => l.id === empInsForm.policyLevelId,
  );
  if (!lv) return;
  empInsForm.usePolicyBase = true;
  empInsForm.baseAmount = Number(lv.baseAmount || 0);
  empInsForm.housingFundBase = Number(lv.baseAmount || 0);
  empInsForm.housingFundCompanyRate = rateToPct(lv.housingFundCompanyRate);
  empInsForm.housingFundPersonalRate = rateToPct(lv.housingFundPersonalRate);
  empInsForm.workinjuryCompanyRate = rateToPct(lv.workinjuryCompanyRate);
  empInsForm.workinjuryPersonalRate = rateToPct(lv.workinjuryPersonalRate);
  empInsForm.participatePension = 1;
  empInsForm.participateMedical = 1;
  empInsForm.participateUnemployment = 1;
  empInsForm.participateWorkinjury = 1;
  empInsForm.participateMaternity = 1;
  empInsForm.participateHousingFund = 1;
  empInsForm.participateCriticalIllness = 1;
}

function openEmpInsForm(record?: any) {
  if (record) {
    empInsForm.id = record.id;
    empInsForm.employeeId = record.employeeId;
    empInsForm.cityCode = record.cityCode || undefined;
    empInsForm.policyId = record.policyId ?? undefined;
    empInsForm.policyLevelId = record.policyLevelId ?? undefined;
    empInsForm.usePolicyBase = record.usePolicyBase ?? true;
    empInsForm.baseAmount = Number(record.baseAmount || 0);
    empInsForm.housingFundBase =
      record.housingFundBase === null || record.housingFundBase === undefined
        ? undefined
        : Number(record.housingFundBase);
    empInsForm.housingFundCompanyRate =
      record.housingFundCompanyRate === null ||
      record.housingFundCompanyRate === undefined
        ? undefined
        : rateToPct(record.housingFundCompanyRate);
    empInsForm.housingFundPersonalRate =
      record.housingFundPersonalRate === null ||
      record.housingFundPersonalRate === undefined
        ? undefined
        : rateToPct(record.housingFundPersonalRate);
    empInsForm.participatePension = Number(record.participatePension ?? 1);
    empInsForm.participateMedical = Number(record.participateMedical ?? 1);
    empInsForm.participateUnemployment = Number(
      record.participateUnemployment ?? 1,
    );
    empInsForm.participateWorkinjury = Number(
      record.participateWorkinjury ?? 1,
    );
    empInsForm.participateMaternity = Number(
      record.participateMaternity ?? 1,
    );
    empInsForm.participateHousingFund = Number(
      record.participateHousingFund ?? 1,
    );
    empInsForm.participateCriticalIllness = Number(
      record.participateCriticalIllness ?? 1,
    );
    empInsForm.workinjuryCompanyRate =
      record.workinjuryCompanyRate === null ||
      record.workinjuryCompanyRate === undefined
        ? undefined
        : rateToPct(record.workinjuryCompanyRate);
    empInsForm.workinjuryPersonalRate =
      record.workinjuryPersonalRate === null ||
      record.workinjuryPersonalRate === undefined
        ? undefined
        : rateToPct(record.workinjuryPersonalRate);
  } else {
    empInsForm.id = undefined;
    empInsForm.employeeId = undefined;
    empInsForm.cityCode = undefined;
    empInsForm.policyId = undefined;
    empInsForm.policyLevelId = undefined;
    empInsForm.usePolicyBase = true;
    empInsForm.baseAmount = 0;
    empInsForm.housingFundBase = undefined;
    empInsForm.housingFundCompanyRate = undefined;
    empInsForm.housingFundPersonalRate = undefined;
    empInsForm.participatePension = 1;
    empInsForm.participateMedical = 1;
    empInsForm.participateUnemployment = 1;
    empInsForm.participateWorkinjury = 1;
    empInsForm.participateMaternity = 1;
    empInsForm.participateHousingFund = 1;
    empInsForm.participateCriticalIllness = 1;
    empInsForm.workinjuryCompanyRate = undefined;
    empInsForm.workinjuryPersonalRate = undefined;
  }
  empInsPreviewResult.value = undefined;
  empInsFormVisible.value = true;
}

async function submitEmpInsForm() {
  if (!empInsForm.employeeId) {
    message.warning($t('page.finance.insurance.message.employeeIdRequired'));
    return;
  }
  if (!empInsForm.cityCode) {
    message.warning($t('page.finance.insurance.message.cityRequired'));
    return;
  }
  if (!empInsForm.policyLevelId) {
    message.warning($t('page.finance.insurance.message.levelRequiredSelect'));
    return;
  }
  const payload = {
    id: empInsForm.id,
    employeeId: empInsForm.employeeId,
    cityCode: empInsForm.cityCode,
    policyId: empInsForm.policyId,
    policyLevelId: empInsForm.policyLevelId,
    usePolicyBase: empInsForm.usePolicyBase,
    baseAmount: empInsForm.baseAmount,
    housingFundBase: empInsForm.housingFundBase,
    housingFundCompanyRate:
      empInsForm.housingFundCompanyRate === undefined
        ? undefined
        : pctToRate(empInsForm.housingFundCompanyRate),
    housingFundPersonalRate:
      empInsForm.housingFundPersonalRate === undefined
        ? undefined
        : pctToRate(empInsForm.housingFundPersonalRate),
    participatePension: empInsForm.participatePension,
    participateMedical: empInsForm.participateMedical,
    participateUnemployment: empInsForm.participateUnemployment,
    participateWorkinjury: empInsForm.participateWorkinjury,
    participateMaternity: empInsForm.participateMaternity,
    participateHousingFund: empInsForm.participateHousingFund,
    participateCriticalIllness: empInsForm.participateCriticalIllness,
    workinjuryCompanyRate:
      empInsForm.workinjuryCompanyRate === undefined
        ? undefined
        : pctToRate(empInsForm.workinjuryCompanyRate),
    workinjuryPersonalRate:
      empInsForm.workinjuryPersonalRate === undefined
        ? undefined
        : pctToRate(empInsForm.workinjuryPersonalRate),
  };
  empInsFormSubmitting.value = true;
  try {
    await upsertEmployeeInsuranceConfigApi(payload);
    message.success($t('page.finance.common.saveSuccess'));
    empInsFormVisible.value = false;
    await loadEmpInsList();
  } catch (error: any) {
    message.error(error?.message || $t('page.finance.common.saveFailed'));
  } finally {
    empInsFormSubmitting.value = false;
  }
}

// ===== 员工配置实时预览 =====
const empInsPreviewResult = ref<any>(undefined);
const empInsPreviewLoading = ref(false);

async function runEmpInsPreview() {
  if (!empInsForm.policyLevelId) {
    message.warning($t('page.finance.insurance.message.levelRequiredSelect'));
    return;
  }
  empInsPreviewLoading.value = true;
  try {
    const res: any = await previewInsuranceCalcApi({
      policyId: empInsForm.policyId,
      levelId: empInsForm.policyLevelId,
      baseAmount: empInsForm.baseAmount,
      usePolicyBase: empInsForm.usePolicyBase,
      housingFundBase: empInsForm.housingFundBase,
      housingFundCompanyRate:
        empInsForm.housingFundCompanyRate === undefined
          ? undefined
          : pctToRate(empInsForm.housingFundCompanyRate),
      housingFundPersonalRate:
        empInsForm.housingFundPersonalRate === undefined
          ? undefined
          : pctToRate(empInsForm.housingFundPersonalRate),
      participatePension: empInsForm.participatePension,
      participateMedical: empInsForm.participateMedical,
      participateUnemployment: empInsForm.participateUnemployment,
      participateWorkinjury: empInsForm.participateWorkinjury,
      participateMaternity: empInsForm.participateMaternity,
      participateHousingFund: empInsForm.participateHousingFund,
      participateCriticalIllness: empInsForm.participateCriticalIllness,
      workinjuryCompanyRate:
        empInsForm.workinjuryCompanyRate === undefined
          ? undefined
          : pctToRate(empInsForm.workinjuryCompanyRate),
      workinjuryPersonalRate:
        empInsForm.workinjuryPersonalRate === undefined
          ? undefined
          : pctToRate(empInsForm.workinjuryPersonalRate),
    });
    empInsPreviewResult.value = res?.data || res;
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.insurance.message.previewFailed'),
    );
    empInsPreviewResult.value = undefined;
  } finally {
    empInsPreviewLoading.value = false;
  }
}

const empInsPreviewRows = computed(() => {
  if (!empInsPreviewResult.value) return [];
  const r = empInsPreviewResult.value;
  const mk = (name: string, item: any) => ({
    name,
    company: formatMoney(item?.company),
    personal: formatMoney(item?.personal),
    subtotal: formatMoney(item?.subtotal),
  });
  const rows = [
    mk($t('page.finance.insurance.drawer.pensionInsurance'), r.pension),
    mk($t('page.finance.insurance.drawer.medicalInsurance'), r.medical),
    mk($t('page.finance.insurance.drawer.unemploymentInsurance'), r.unemployment),
    mk($t('page.finance.insurance.drawer.workinjuryInsurance'), r.workinjury),
    mk($t('page.finance.insurance.drawer.maternityInsurance'), r.maternity),
    mk($t('page.finance.insurance.drawer.housingFundLabel'), r.housingFund),
    mk($t('page.finance.insurance.column.criticalIllness'), r.criticalIllness),
  ];
  rows.push({
    name: $t('page.finance.insurance.drawer.grandTotal'),
    company: formatMoney(r.companyTotal),
    personal: formatMoney(r.personalTotal),
    subtotal: formatMoney(r.grandTotal),
  });
  return rows;
});

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
              <Button @click="loadPolicyList">
                {{ $t('page.finance.common.refresh') }}
              </Button>
            </div>
            <Button v-if="canManage" type="primary" @click="openPolicyForm()">
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
            :scroll="{ x: 1100 }"
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
                  :title="
                    $t('page.finance.insurance.message.deletePolicyConfirm')
                  "
                  @confirm="handleDeletePolicy(record.policy.id)"
                >
                  <Button type="link" size="small" danger>
                    {{ $t('page.finance.common.delete') }}
                  </Button>
                </Popconfirm>
              </template>
            </template>
          </Table>
        </TabPane>

        <TabPane key="empIns" :tab="$t('page.finance.insurance.tab.empConfig')">
          <div class="mb-4 flex items-center justify-between">
            <Button @click="loadEmpInsList">
              {{ $t('page.finance.common.refresh') }}
            </Button>
            <Button v-if="canManage" type="primary" @click="openEmpInsForm()">
              {{ $t('page.finance.insurance.button.createEmpConfig') }}
            </Button>
          </div>
          <Table
            :columns="empInsColumns"
            :data-source="empInsList"
            :loading="empInsLoading"
            row-key="id"
            :pagination="false"
            size="middle"
            :scroll="{ x: 1050 }"
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

    <!-- 城市政策编辑弹窗（多档次） -->
    <Modal
      v-model:open="policyFormVisible"
      :title="
        policyForm.id
          ? $t('page.finance.insurance.drawer.titlePolicyEdit')
          : $t('page.finance.insurance.drawer.titlePolicyCreate')
      "
      :confirm-loading="policyFormSubmitting"
      width="1000px"
      @ok="submitPolicyForm"
    >
      <div class="max-h-[62vh] overflow-y-auto py-2 pr-2">
        <!-- 基本信息 -->
        <div class="mb-2 font-semibold">
          {{ $t('page.finance.insurance.drawer.cityCode') }}
        </div>
        <Form layout="vertical" autocomplete="off">
          <Row :gutter="16">
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.cityCode')"
                required
              >
                <Input
                  v-model:value="policyForm.cityCode"
                  :placeholder="
                    $t('page.finance.insurance.drawer.cityCodePlaceholder')
                  "
                  :disabled="!!policyForm.id"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.cityName')"
                required
              >
                <Input
                  v-model:value="policyForm.cityName"
                  :placeholder="
                    $t('page.finance.insurance.drawer.cityNamePlaceholder')
                  "
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.drawer.year')"
                required
              >
                <InputNumber
                  v-model:value="policyForm.year"
                  :min="2020"
                  :max="2099"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="8">
              <FormItem
                :label="$t('page.finance.insurance.column.policyPeriod')"
              >
                <Space.Compact style="width: 100%">
                  <DatePicker
                    v-model:value="policyForm.effectiveDate"
                    value-format="YYYY-MM-DD"
                    style="width: 50%"
                    :placeholder="
                      $t('page.finance.insurance.drawer.effectiveDatePlaceholder')
                    "
                  />
                  <DatePicker
                    v-model:value="policyForm.expiryDate"
                    value-format="YYYY-MM-DD"
                    style="width: 50%"
                    :placeholder="
                      $t('page.finance.insurance.drawer.expiryDatePlaceholder')
                    "
                  />
                </Space.Compact>
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="16">
            <Col :span="8">
              <FormItem :label="$t('page.finance.insurance.drawer.policyStatus')">
                <Select
                  v-model:value="policyForm.status"
                  :options="[
                    {
                      value: 1,
                      label: $t('page.finance.insurance.status.active'),
                    },
                    {
                      value: 0,
                      label: $t('page.finance.insurance.status.inactive'),
                    },
                  ]"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="16">
              <FormItem :label="$t('page.finance.insurance.drawer.remark')">
                <Input
                  v-model:value="policyForm.remark"
                  :placeholder="
                    $t('page.finance.insurance.drawer.remarkPlaceholder')
                  "
                />
              </FormItem>
            </Col>
          </Row>
        </Form>

        <!-- 档次列表 -->
        <Divider orientation="left">
          {{ $t('page.finance.insurance.drawer.levelList') }}
          <Button
            type="dashed"
            size="small"
            style="margin-left: 8px"
            @click="addPolicyLevel"
          >
            + {{ $t('page.finance.insurance.drawer.addLevel') }}
          </Button>
        </Divider>

        <div
          v-for="(lv, idx) in policyForm.levels"
          :key="idx"
          class="mb-4 rounded-md border border-solid border-gray-200 p-3"
        >
          <Row :gutter="12">
            <Col :span="5">
              <FormItem :label="$t('page.finance.insurance.drawer.levelType')">
                <Select
                  v-model:value="lv.levelType"
                  :options="levelTypeOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem :label="$t('page.finance.insurance.column.levelName')">
                <Input
                  v-model:value="lv.levelName"
                  :placeholder="
                    $t('page.finance.insurance.drawer.levelTypeLowest')
                  "
                />
              </FormItem>
            </Col>
            <Col :span="5">
              <FormItem
                :label="$t('page.finance.insurance.column.levelBase')"
                required
              >
                <InputNumber
                  v-model:value="lv.baseAmount"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                />
              </FormItem>
            </Col>
            <Col :span="5">
              <FormItem :label="$t('page.finance.insurance.column.baseLower')">
                <InputNumber
                  v-model:value="lv.baseLower"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                />
              </FormItem>
            </Col>
            <Col :span="3">
              <FormItem :label="$t('page.finance.insurance.column.baseUpper')">
                <InputNumber
                  v-model:value="lv.baseUpper"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="12">
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.pensionCompany')"
              >
                <InputNumber
                  v-model:value="lv.pensionCompanyRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.pensionPersonal')"
              >
                <InputNumber
                  v-model:value="lv.pensionPersonalRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.medicalCompany')"
              >
                <InputNumber
                  v-model:value="lv.medicalCompanyRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.medicalPersonal')"
              >
                <InputNumber
                  v-model:value="lv.medicalPersonalRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.unemploymentCompany')"
              >
                <InputNumber
                  v-model:value="lv.unemploymentCompanyRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.unemploymentPersonal')"
              >
                <InputNumber
                  v-model:value="lv.unemploymentPersonalRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="12">
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.workinjuryCompany')"
              >
                <InputNumber
                  v-model:value="lv.workinjuryCompanyRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.drawer.workinjuryPersonalRate')"
              >
                <InputNumber
                  v-model:value="lv.workinjuryPersonalRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.maternityCompany')"
              >
                <InputNumber
                  v-model:value="lv.maternityCompanyRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.drawer.maternityPersonalRate')"
              >
                <InputNumber
                  v-model:value="lv.maternityPersonalRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.housingFundCompany')"
              >
                <InputNumber
                  v-model:value="lv.housingFundCompanyRate"
                  :min="0"
                  :max="100"
                  :step="1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.column.housingFundPersonal')"
              >
                <InputNumber
                  v-model:value="lv.housingFundPersonalRate"
                  :min="0"
                  :max="100"
                  :step="1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="12">
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.drawer.criticalIllnessCompany')"
              >
                <InputNumber
                  v-model:value="lv.criticalIllnessCompanyAmount"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                />
              </FormItem>
            </Col>
            <Col :span="4">
              <FormItem
                :label="$t('page.finance.insurance.drawer.criticalIllnessPersonal')"
              >
                <InputNumber
                  v-model:value="lv.criticalIllnessPersonalAmount"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                />
              </FormItem>
            </Col>
            <Col :span="4" :offset="8">
              <FormItem label="&nbsp;">
                <Popconfirm
                  :title="$t('page.finance.insurance.message.deleteLevelConfirm')"
                  @confirm="removePolicyLevel(idx)"
                >
                  <Button type="link" size="small" danger block>
                    {{ $t('page.finance.insurance.drawer.deleteLevel') }}
                  </Button>
                </Popconfirm>
              </FormItem>
            </Col>
          </Row>
        </div>

        <!-- 实时预览 -->
        <Divider orientation="left">
          {{ $t('page.finance.insurance.drawer.preview') }}
        </Divider>
        <div class="mb-3 flex items-center gap-3">
          <span style="white-space: nowrap">
            {{ $t('page.finance.insurance.drawer.previewLevel') }}：
          </span>
          <Select
            v-model:value="previewLevelIndex"
            :placeholder="$t('page.finance.insurance.drawer.previewPlaceholder')"
            style="width: 260px"
            :options="policyForm.levels.map((lv: any, i: number) => ({
              value: i,
              label: `${lv.levelName || '档次' + (i + 1)}(${formatMoney(lv.baseAmount)})`,
            }))"
          />
          <Button type="primary" :loading="previewLoading" @click="runPolicyPreview">
            {{ $t('page.finance.insurance.drawer.preview') }}
          </Button>
        </div>
        <Table
          v-if="previewResult"
          :columns="previewColumns"
          :data-source="previewRows"
          :pagination="false"
          size="small"
          :scroll="{ x: 520 }"
        />
      </div>
    </Modal>

    <!-- 员工社保配置编辑弹窗 -->
    <Modal
      v-model:open="empInsFormVisible"
      :title="
        empInsForm.id
          ? $t('page.finance.insurance.drawer.titleEmpConfigEdit')
          : $t('page.finance.insurance.drawer.titleEmpConfigCreate')
      "
      :confirm-loading="empInsFormSubmitting"
      width="860px"
      @ok="submitEmpInsForm"
    >
      <div class="max-h-[62vh] overflow-y-auto py-2 pr-2">
        <Form layout="vertical" autocomplete="off">
          <Row :gutter="16">
            <Col :span="12">
              <FormItem :label="$t('page.finance.common.employeeId')" required>
                <UserPickerModal
                  v-model:value="empInsForm.employeeId"
                  :disabled="!!empInsForm.id"
                />
              </FormItem>
            </Col>
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.drawer.selectCity')"
                required
              >
                <Select
                  v-model:value="empInsForm.cityCode"
                  :options="cityOptions"
                  :placeholder="$t('page.finance.insurance.drawer.selectCity')"
                  style="width: 100%"
                  @change="onCityChange"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="16">
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.drawer.selectPolicy')"
              >
                <Select
                  v-model:value="empInsForm.policyId"
                  :options="policyOptions"
                  :placeholder="$t('page.finance.insurance.drawer.selectPolicy')"
                  style="width: 100%"
                  :disabled="!empInsForm.cityCode"
                  @change="onPolicyChange"
                />
              </FormItem>
            </Col>
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.drawer.selectLevel')"
                required
              >
                <Select
                  v-model:value="empInsForm.policyLevelId"
                  :options="levelOptions"
                  :placeholder="$t('page.finance.insurance.drawer.selectLevel')"
                  style="width: 100%"
                  :disabled="!empInsForm.policyId"
                  @change="onLevelChange"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="16">
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.drawer.baseUsed')"
                required
              >
                <Radio.Group
                  v-model:value="empInsForm.usePolicyBase"
                  class="mb-2"
                >
                  <Radio :value="true">
                    {{ $t('page.finance.insurance.drawer.usePolicyBase') }}
                  </Radio>
                  <Radio :value="false">
                    {{ $t('page.finance.insurance.drawer.customBase') }}
                  </Radio>
                </Radio.Group>
                <InputNumber
                  v-model:value="empInsForm.baseAmount"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                  :disabled="empInsForm.usePolicyBase"
                />
              </FormItem>
            </Col>
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.column.housingFundBase')"
              >
                <InputNumber
                  v-model:value="empInsForm.housingFundBase"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  prefix="¥"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="16">
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.column.housingFundCompany')"
              >
                <InputNumber
                  v-model:value="empInsForm.housingFundCompanyRate"
                  :min="0"
                  :max="100"
                  :step="1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
            <Col :span="12">
              <FormItem
                :label="$t('page.finance.insurance.column.housingFundPersonal')"
              >
                <InputNumber
                  v-model:value="empInsForm.housingFundPersonalRate"
                  :min="0"
                  :max="100"
                  :step="1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
          </Row>

          <Divider orientation="left">
            {{ $t('page.finance.insurance.drawer.participateTitle') }}
          </Divider>
          <Row :gutter="16">
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.pensionInsurance')"
              >
                <Select
                  v-model:value="empInsForm.participatePension"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.medicalInsurance')"
              >
                <Select
                  v-model:value="empInsForm.participateMedical"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.unemploymentInsurance')"
              >
                <Select
                  v-model:value="empInsForm.participateUnemployment"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.workinjuryInsurance')"
              >
                <Select
                  v-model:value="empInsForm.participateWorkinjury"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
          </Row>
          <Row :gutter="16">
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.maternityInsurance')"
              >
                <Select
                  v-model:value="empInsForm.participateMaternity"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.housingFundLabel')"
              >
                <Select
                  v-model:value="empInsForm.participateHousingFund"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.drawer.participateCriticalIllness')"
              >
                <Select
                  v-model:value="empInsForm.participateCriticalIllness"
                  :options="participateOptions"
                  style="width: 100%"
                />
              </FormItem>
            </Col>
            <Col :span="6">
              <FormItem
                :label="$t('page.finance.insurance.column.workinjuryCompany')"
              >
                <InputNumber
                  v-model:value="empInsForm.workinjuryCompanyRate"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :precision="4"
                  style="width: 100%"
                  addon-after="%"
                />
              </FormItem>
            </Col>
          </Row>

          <Divider orientation="left">
            {{ $t('page.finance.insurance.drawer.preview') }}
          </Divider>
          <div class="mb-3">
            <Button
              type="primary"
              :loading="empInsPreviewLoading"
              @click="runEmpInsPreview"
            >
              {{ $t('page.finance.insurance.drawer.preview') }}
            </Button>
          </div>
          <Table
            v-if="empInsPreviewResult"
            :columns="previewColumns"
            :data-source="empInsPreviewRows"
            :pagination="false"
            size="small"
            :scroll="{ x: 520 }"
          />
        </Form>
      </div>
    </Modal>
  </Page>
</template>
