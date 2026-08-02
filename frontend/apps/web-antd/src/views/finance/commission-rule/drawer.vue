<script lang="ts" setup>
import { computed, reactive, ref, watch } from 'vue';

import {
  Button,
  Collapse,
  CollapsePanel,
  DatePicker,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  Select,
  Switch,
  Table,
  TreeSelect,
  message,
} from 'ant-design-vue';

import { CommissionSchemePreview } from '#/components/CommissionSchemePreview';
import { saveCommissionRuleApi } from '#/api/core/finance';
import { getDeptTreeApi } from '#/api/core/system/dept';
import { $t } from '#/locales';

const props = defineProps<{
  visible: boolean;
  data?: any;
}>();

const emit = defineEmits<{
  (e: 'close', needRefresh?: boolean): void;
}>();

const formRef = ref();
const loading = ref(false);

const isEdit = computed(() => !!props.data?.id);

// ===== 选项定义 =====

// 提成性质（6种模式）
const categoryOptions = [
  { value: 1, label: $t('page.finance.commissionRule.category.personal') },
  { value: 2, label: $t('page.finance.commissionRule.category.management') },
  { value: 3, label: $t('page.finance.commissionRule.category.teamBonus') },
  { value: 4, label: $t('page.finance.commissionRule.category.poolFund') },
  { value: 5, label: $t('page.finance.commissionRule.category.reallocation') },
  { value: 6, label: $t('page.finance.commissionRule.category.profit') },
];

// 受益岗位全量选项
const allBeneficiaryOptions = [
  { value: 1, label: $t('page.finance.commissionRule.beneficiary.sales') },
  { value: 2, label: $t('page.finance.commissionRule.beneficiary.supervisor') },
  { value: 3, label: $t('page.finance.commissionRule.beneficiary.manager') },
  { value: 4, label: $t('page.finance.commissionRule.beneficiary.director') },
  { value: 5, label: $t('page.finance.commissionRule.beneficiary.gm') },
];

// 计算方式全量选项
const allCalcMethodOptions = [
  { value: 1, label: $t('page.finance.commissionRule.calcMethod.rate') },
  { value: 2, label: $t('page.finance.commissionRule.calcMethod.fixed') },
  { value: 3, label: $t('page.finance.commissionRule.calcMethod.tiered') },
  { value: 4, label: $t('page.finance.commissionRule.calcMethod.progressive') },
];

// 根据 category 动态过滤受益岗位可选项
const filteredBeneficiaryOptions = computed(() => {
  const cat = formData.commissionCategory;
  // category=1,6 仅销售本人
  if (cat === 1 || cat === 6) {
    return allBeneficiaryOptions.filter((o) => o.value === 1);
  }
  // category=2,3,4,5 管理岗位
  return allBeneficiaryOptions.filter((o) => o.value >= 2);
});

// 根据 category 动态过滤计算方式可选项
const filteredCalcMethodOptions = computed(() => {
  const cat = formData.commissionCategory;
  // category=3 仅固定金额
  if (cat === 3) {
    return allCalcMethodOptions.filter((o) => o.value === 2);
  }
  // category=2,4,5 仅按比例
  if (cat === 2 || cat === 4 || cat === 5) {
    return allCalcMethodOptions.filter((o) => o.value === 1);
  }
  // category=1,6 支持比例/阶梯/超额
  return allCalcMethodOptions.filter((o) => o.value !== 2);
});

// 是否显示阶梯配置
const showTierConfig = computed(() => {
  return formData.commissionCategory === 1 || formData.commissionCategory === 6;
});

// 是否显示达标门槛+固定奖金
const showBonusConfig = computed(() => formData.commissionCategory === 3);

// 是否显示资金池选择器
const showPoolConfig = computed(() => formData.commissionCategory === 4);

// 是否显示再分配说明
const showReallocationHint = computed(
  () => formData.commissionCategory === 5,
);

// 提成基数字段选项
const calcBaseFieldOptions = [
  { value: 'payment_amount', label: $t('page.finance.commissionRule.calcBaseField.paymentAmount') },
  { value: 'contract_amount', label: $t('page.finance.commissionRule.calcBaseField.contractAmount') },
  { value: 'profit', label: $t('page.finance.commissionRule.calcBaseField.profit') },
  { value: 'net_profit', label: $t('page.finance.commissionRule.calcBaseField.netProfit') },
];

// 客户分类选项
const customerCategoryOptions = [
  { value: '', label: $t('page.finance.commissionRule.field.customerCategoryAll') },
  { value: 'new', label: $t('page.finance.commissionRule.field.customerCategoryNew') },
  { value: 'old', label: $t('page.finance.commissionRule.field.customerCategoryOld') },
];

const applyScopeOptions = [
  { value: 1, label: $t('page.finance.commissionRule.applyScope.designatedDept') },
  { value: 2, label: $t('page.finance.commissionRule.applyScope.company') },
  { value: 3, label: $t('page.finance.commissionRule.applyScope.designatedPost') },
  { value: 4, label: $t('page.finance.commissionRule.applyScope.designatedMember') },
];

const triggerTypeOptions = [
  { value: 1, label: $t('page.finance.commissionRule.triggerCondition.contractSign') },
  { value: 2, label: $t('page.finance.commissionRule.triggerCondition.paymentReceived') },
  { value: 3, label: $t('page.finance.commissionRule.triggerCondition.orderComplete') },
  { value: 4, label: $t('page.finance.commissionRule.triggerCondition.invoiceIssued') },
];

const calcBaseTypeOptions = [
  { value: 1, label: $t('page.finance.commissionRule.calcBaseType.personalMonthly') },
  { value: 2, label: $t('page.finance.commissionRule.calcBaseType.teamMonthly') },
  { value: 3, label: $t('page.finance.commissionRule.calcBaseType.singleContract') },
  { value: 4, label: $t('page.finance.commissionRule.calcBaseType.singlePayment') },
];

// 产品线/区域/客户类型 维度选项
const productLineOptions = [
  { value: 'standard', label: $t('page.finance.commissionRule.dimension.productLine.standard') },
  { value: 'premium', label: $t('page.finance.commissionRule.dimension.productLine.premium') },
  { value: 'enterprise', label: $t('page.finance.commissionRule.dimension.productLine.enterprise') },
  { value: 'custom', label: $t('page.finance.commissionRule.dimension.productLine.custom') },
];

const regionCodeOptions = [
  { value: 'north', label: $t('page.finance.commissionRule.dimension.region.north') },
  { value: 'east', label: $t('page.finance.commissionRule.dimension.region.east') },
  { value: 'south', label: $t('page.finance.commissionRule.dimension.region.south') },
  { value: 'west', label: $t('page.finance.commissionRule.dimension.region.west') },
  { value: 'central', label: $t('page.finance.commissionRule.dimension.region.central') },
];

const customerTypeOptions = [
  { value: 'vip', label: $t('page.finance.commissionRule.dimension.customerType.vip') },
  { value: 'strategy', label: $t('page.finance.commissionRule.dimension.customerType.strategy') },
  { value: 'normal', label: $t('page.finance.commissionRule.dimension.customerType.normal') },
  { value: 'new', label: $t('page.finance.commissionRule.dimension.customerType.new') },
];

// ===== 表单数据 =====
const formData = reactive<any>({
  ruleName: '',
  // v2 新增：提成性质（主分类字段）
  commissionCategory: 1,
  beneficiaryRole: 1,
  calcMethod: 1,
  // v2 新增：模式相关字段
  bonusTarget: undefined,
  bonusFixedAmount: undefined,
  commissionCap: undefined,
  commissionFloor: undefined,
  customerCategory: '',
  deferMonths: 0,
  poolId: undefined,
  calcBaseField: 'payment_amount',
  tierMode: undefined,
  // 保留字段（向后兼容）
  ruleType: 1,
  applyScope: 2,
  departmentId: undefined,
  postId: undefined,
  priority: 0,
  isDefault: false,
  enabled: true,
  calcBaseType: 1,
  triggerCondition: undefined,
  // 维度字段
  productLine: undefined,
  regionCode: undefined,
  customerType: undefined,
  effectiveDate: undefined,
  expiryDate: undefined,
  description: '',
});

const tiers = ref<any[]>([]);
const members = ref<any[]>([]);
const deptTreeData = ref<any[]>([]);
const poolOptions = ref<any[]>([]);

// ===== 资金池选项加载 =====
async function loadPoolOptions() {
  try {
    const { getCommissionPoolListApi } = await import('#/api/core/finance');
    const res = await getCommissionPoolListApi({ status: 1 });
    const list = res?.data || res || [];
    const items = Array.isArray(list) ? list : list?.items || list?.list || [];
    poolOptions.value = items.map((p: any) => ({
      value: p.id,
      label: p.poolName || p.pool_name,
    }));
  } catch {
    poolOptions.value = [];
  }
}

async function loadDeptTree() {
  try {
    const res = await getDeptTreeApi();
    const data = res?.data || [];
    deptTreeData.value = _convertDeptTreeValues(data);
  } catch (e) {
    console.error('Failed to load dept tree:', e);
  }
}

function _convertDeptTreeValues(nodes: any[]): any[] {
  return nodes.map((node) => ({
    ...node,
    value: node.value ? Number(node.value) : node.value,
    children: node.children ? _convertDeptTreeValues(node.children) : undefined,
  }));
}

// ===== category 切换时自动调整关联字段 =====
watch(
  () => formData.commissionCategory,
  (cat) => {
    // 自动调整 beneficiaryRole
    if (cat === 1 || cat === 6) {
      formData.beneficiaryRole = 1; // 销售本人
    } else if (formData.beneficiaryRole === 1) {
      formData.beneficiaryRole = 2; // 默认直属主管
    }
    // 自动调整 calcMethod
    if (cat === 3) {
      formData.calcMethod = 2; // 固定金额
    } else if (cat === 2 || cat === 4 || cat === 5) {
      formData.calcMethod = 1; // 按比例
    } else if (formData.calcMethod === 2) {
      formData.calcMethod = 1; // 个人/利润提成默认按比例
    }
  },
);

function addTier() {
  tiers.value.push({
    minAmount: 0,
    maxAmount: 0,
    commissionRate: 0,
    sort: tiers.value.length + 1,
  });
}

function removeTier(index: number) {
  tiers.value.splice(index, 1);
}

function addMember() {
  members.value.push({
    memberType: 1,
    memberName: '',
    distributionType: 1,
    fixedRate: 0,
    roleName: '',
    defaultRatio: 0,
    required: 0,
    sort: members.value.length + 1,
  });
}

function removeMember(index: number) {
  members.value.splice(index, 1);
}

function resetForm() {
  formData.ruleName = '';
  formData.commissionCategory = 1;
  formData.beneficiaryRole = 1;
  formData.calcMethod = 1;
  formData.bonusTarget = undefined;
  formData.bonusFixedAmount = undefined;
  formData.commissionCap = undefined;
  formData.commissionFloor = undefined;
  formData.customerCategory = '';
  formData.deferMonths = 0;
  formData.poolId = undefined;
  formData.calcBaseField = 'payment_amount';
  formData.tierMode = undefined;
  formData.ruleType = 1;
  formData.applyScope = 2;
  formData.departmentId = undefined;
  formData.postId = undefined;
  formData.priority = 0;
  formData.isDefault = false;
  formData.enabled = true;
  formData.calcBaseType = 1;
  formData.triggerCondition = undefined;
  formData.productLine = undefined;
  formData.regionCode = undefined;
  formData.customerType = undefined;
  formData.effectiveDate = undefined;
  formData.expiryDate = undefined;
  formData.description = '';
  tiers.value = [];
  members.value = [];
  formRef.value?.resetFields();
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      resetForm();
      loadDeptTree();
      loadPoolOptions();
      if (props.data) {
        const row = props.data;
        formData.ruleName = row.ruleName ?? '';
        formData.commissionCategory = row.commissionCategory ?? 1;
        formData.beneficiaryRole = row.beneficiaryRole ?? 1;
        formData.calcMethod = row.calcMethod ?? 1;
        formData.bonusTarget = row.bonusTarget;
        formData.bonusFixedAmount = row.bonusFixedAmount;
        formData.commissionCap = row.commissionCap;
        formData.commissionFloor = row.commissionFloor;
        formData.customerCategory = row.customerCategory ?? '';
        formData.deferMonths = row.deferMonths ?? 0;
        formData.poolId = row.poolId;
        formData.calcBaseField = row.calcBaseField ?? 'payment_amount';
        formData.tierMode = row.tierMode;
        formData.ruleType = row.ruleType ?? 1;
        formData.applyScope = row.applyScope ?? 2;
        formData.departmentId = row.departmentId;
        formData.postId = row.postId;
        formData.priority = row.priority ?? 0;
        formData.isDefault = row.isDefault ?? false;
        formData.enabled = row.enabled ?? true;
        formData.calcBaseType = row.calcBaseType ?? 1;
        formData.triggerCondition = row.triggerCondition;
        formData.productLine = row.productLine;
        formData.regionCode = row.regionCode;
        formData.customerType = row.customerType;
        formData.effectiveDate = row.effectiveDate;
        formData.expiryDate = row.expiryDate;
        formData.description = row.description ?? '';
        tiers.value =
          row.tiers && row.tiers.length > 0
            ? row.tiers.map((t: any) => ({ ...t }))
            : [];
        members.value =
          row.members && row.members.length > 0
            ? row.members.map((m: any) => ({ ...m }))
            : [];
      }
      if (tiers.value.length === 0 && showTierConfig.value) {
        addTier();
      }
    }
  },
);

const tierColumns = computed(() => [
  { title: $t('page.finance.commissionRule.drawer.tierMinAmount'), dataIndex: 'minAmount', width: 140, key: 'minAmount' },
  { title: $t('page.finance.commissionRule.drawer.tierMaxAmount'), dataIndex: 'maxAmount', width: 140, key: 'maxAmount' },
  { title: $t('page.finance.commissionRule.drawer.tierRate'), dataIndex: 'commissionRate', width: 140, key: 'commissionRate' },
  { title: $t('page.finance.commissionRule.drawer.tierSort'), dataIndex: 'sort', width: 100, key: 'sort' },
  { title: $t('page.finance.common.action'), key: 'action', width: 80 },
]);

const memberColumns = computed(() => [
  { title: $t('page.finance.commissionRule.drawer.memberType'), dataIndex: 'memberType', width: 110, key: 'memberType' },
  { title: $t('page.finance.commissionRule.drawer.memberName'), dataIndex: 'memberName', width: 120, key: 'memberName' },
  { title: $t('page.finance.commissionRule.drawer.distributionType'), dataIndex: 'distributionType', width: 100, key: 'distributionType' },
  { title: $t('page.finance.commissionRule.drawer.fixedRate'), dataIndex: 'fixedRate', width: 100, key: 'fixedRate' },
  { title: $t('page.finance.commissionRule.drawer.roleName'), dataIndex: 'roleName', width: 120, key: 'roleName' },
  { title: $t('page.finance.commissionRule.drawer.defaultRatio'), dataIndex: 'defaultRatio', width: 100, key: 'defaultRatio' },
  { title: $t('page.finance.commissionRule.drawer.required'), dataIndex: 'required', width: 70, key: 'required' },
  { title: $t('page.finance.commissionRule.drawer.tierSort'), dataIndex: 'sort', width: 70, key: 'sort' },
  { title: $t('page.finance.common.action'), key: 'action', width: 70 },
]);

const memberTypeOptions = [
  { value: 1, label: $t('page.finance.commissionRule.memberType.salesman') },
  { value: 2, label: $t('page.finance.commissionRule.memberType.directManager') },
  { value: 3, label: $t('page.finance.commissionRule.memberType.deptDirector') },
  { value: 4, label: $t('page.finance.commissionRule.memberType.other') },
];

const distributionTypeOptions = [
  { value: 1, label: $t('page.finance.commissionRule.distributionType.fixedRate') },
];

// 折叠面板默认不展开
const activeCollapse = ref<string[]>([]);

async function handleSubmit() {
  try {
    await formRef.value.validate();
  } catch {
    return;
  }

  if (showTierConfig.value && tiers.value.length === 0) {
    message.warning($t('page.finance.commissionRule.drawer.tierEmpty'));
    return;
  }

  if (showPoolConfig.value && !formData.poolId) {
    message.warning($t('page.finance.commissionRule.drawer.poolRequired'));
    return;
  }

  loading.value = true;
  try {
    const payload = {
      ...formData,
      // 将 boolean 转为数据库期望的 0/1
      isDefault: formData.isDefault ? 1 : 0,
      enabled: formData.enabled ? 1 : 0,
      tiers: showTierConfig.value ? tiers.value : [],
      members: formData.ruleType === 2 ? members.value : [],
      ...(props.data?.id ? { id: props.data.id } : {}),
    };
    await saveCommissionRuleApi(payload);
    message.success(isEdit.value ? $t('page.finance.commissionRule.drawer.updateSuccess') : $t('page.finance.commissionRule.drawer.createSuccess'));
    emit('close', true);
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    loading.value = false;
  }
}

function handleClose() {
  emit('close');
}
</script>

<template>
  <Drawer
    :open="visible"
    :title="isEdit ? $t('page.finance.commissionRule.drawer.titleEdit') : $t('page.finance.commissionRule.drawer.titleCreate')"
    :width="960"
    :mask-closable="false"
    :destroy-on-close="true"
    @close="handleClose"
  >
    <Form
      ref="formRef"
      :model="formData"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 18 }"
    >
      <!-- ===== 第一组：基本信息 ===== -->
      <div class="section-title">{{ $t('page.finance.commissionRule.drawer.baseInfo') }}</div>

      <FormItem
        name="ruleName"
        :label="$t('page.finance.commissionRule.drawer.ruleName')"
        :rules="[{ required: true, message: $t('page.finance.commissionRule.drawer.ruleNameRequired') }]"
      >
        <Input
          v-model:value="formData.ruleName"
          :placeholder="$t('page.finance.commissionRule.drawer.ruleNamePlaceholder')"
          allow-clear
        />
      </FormItem>

      <FormItem
        name="commissionCategory"
        :label="$t('page.finance.commissionRule.drawer.commissionCategory')"
        :rules="[{ required: true }]"
      >
        <Select
          v-model:value="formData.commissionCategory"
          :options="categoryOptions"
        />
      </FormItem>

      <FormItem
        name="beneficiaryRole"
        :label="$t('page.finance.commissionRule.drawer.beneficiaryRole')"
      >
        <Select
          v-model:value="formData.beneficiaryRole"
          :options="filteredBeneficiaryOptions"
        />
      </FormItem>

      <FormItem
        name="calcMethod"
        :label="$t('page.finance.commissionRule.drawer.calcMethod')"
      >
        <Select
          v-model:value="formData.calcMethod"
          :options="filteredCalcMethodOptions"
        />
      </FormItem>

      <FormItem name="applyScope" :label="$t('page.finance.commissionRule.drawer.applyScope')">
        <Select
          v-model:value="formData.applyScope"
          :options="applyScopeOptions"
        />
      </FormItem>

      <FormItem v-if="formData.applyScope === 1" name="departmentId" :label="$t('page.finance.commissionRule.drawer.department')">
        <TreeSelect
          v-model:value="formData.departmentId"
          :tree-data="deptTreeData"
          tree-node-filter-prop="label"
          style="width: 100%"
          allow-clear
          show-search
          tree-default-expand-all
        />
      </FormItem>

      <FormItem v-if="formData.applyScope === 3" name="postId" :label="$t('page.finance.commissionRule.drawer.post')">
        <InputNumber
          v-model:value="formData.postId"
          style="width: 100%"
          :min="0"
        />
      </FormItem>

      <!-- ===== 第二组：计算方式（动态显示）===== -->
      <template v-if="showBonusConfig">
        <div class="section-title">{{ $t('page.finance.commissionRule.category.teamBonus') }}</div>
        <FormItem :label="$t('page.finance.commissionRule.field.bonusTarget')">
          <InputNumber
            v-model:value="formData.bonusTarget"
            :min="0"
            :precision="2"
            style="width: 100%"
            :placeholder="$t('page.finance.commissionRule.field.bonusTarget')"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.commissionRule.field.bonusFixedAmount')">
          <InputNumber
            v-model:value="formData.bonusFixedAmount"
            :min="0"
            :precision="2"
            style="width: 100%"
            :placeholder="$t('page.finance.commissionRule.field.bonusFixedAmount')"
          />
        </FormItem>
      </template>

      <template v-if="showPoolConfig">
        <div class="section-title">{{ $t('page.finance.commissionRule.category.poolFund') }}</div>
        <FormItem :label="$t('page.finance.commissionRule.field.pool')">
          <Select
            v-model:value="formData.poolId"
            :options="poolOptions"
            :placeholder="$t('page.finance.commissionRule.field.pool')"
            allow-clear
          />
        </FormItem>
      </template>

      <template v-if="showReallocationHint">
        <div class="section-title">{{ $t('page.finance.commissionRule.category.reallocation') }}</div>
        <FormItem :wrapper-col="{ span: 24 }">
          <div class="reallocation-hint">
            {{ $t('page.finance.commissionRule.preview.reallocationDesc') }}
          </div>
        </FormItem>
      </template>

      <!-- ===== 第三组：方案预览 ===== -->
      <div class="section-title">{{ $t('page.finance.commissionRule.preview.title') }}</div>
      <FormItem :wrapper-col="{ span: 24 }">
        <CommissionSchemePreview :form="formData" :tiers="tiers" />
      </FormItem>

      <!-- ===== 第四组：筛选条件（折叠）===== -->
      <Collapse v-model:active-key="activeCollapse" :bordered="false" ghost>
        <CollapsePanel key="filter" :header="$t('page.finance.commissionRule.drawer.filterConditions')">
          <FormItem name="calcBaseField" :label="$t('page.finance.commissionRule.field.calcBaseField')">
            <Select
              v-model:value="formData.calcBaseField"
              :options="calcBaseFieldOptions"
            />
          </FormItem>
          <FormItem name="productLine" :label="$t('page.finance.commissionRule.drawer.productLine')">
            <Select
              v-model:value="formData.productLine"
              :options="productLineOptions"
              allow-clear
            />
          </FormItem>
          <FormItem name="regionCode" :label="$t('page.finance.commissionRule.drawer.region')">
            <Select
              v-model:value="formData.regionCode"
              :options="regionCodeOptions"
              allow-clear
            />
          </FormItem>
          <FormItem name="customerType" :label="$t('page.finance.commissionRule.drawer.customerType')">
            <Select
              v-model:value="formData.customerType"
              :options="customerTypeOptions"
              allow-clear
            />
          </FormItem>
          <FormItem name="customerCategory" :label="$t('page.finance.commissionRule.field.customerCategory')">
            <Select
              v-model:value="formData.customerCategory"
              :options="customerCategoryOptions"
            />
          </FormItem>
        </CollapsePanel>

        <!-- ===== 第五组：限制规则（折叠）===== -->
        <CollapsePanel key="limit" :header="$t('page.finance.commissionRule.limit.title')">
          <FormItem :label="$t('page.finance.commissionRule.field.commissionCap')">
            <InputNumber
              v-model:value="formData.commissionCap"
              :min="0"
              :precision="2"
              style="width: 100%"
              :placeholder="$t('page.finance.commissionRule.limit.capPlaceholder')"
            />
          </FormItem>
          <FormItem :label="$t('page.finance.commissionRule.field.commissionFloor')">
            <InputNumber
              v-model:value="formData.commissionFloor"
              :min="0"
              :precision="2"
              style="width: 100%"
              :placeholder="$t('page.finance.commissionRule.limit.floorPlaceholder')"
            />
          </FormItem>
          <FormItem :label="$t('page.finance.commissionRule.field.deferMonths')">
            <InputNumber
              v-model:value="formData.deferMonths"
              :min="0"
              :max="36"
              style="width: 100%"
              :placeholder="$t('page.finance.commissionRule.limit.deferPlaceholder')"
            />
          </FormItem>
        </CollapsePanel>
      </Collapse>

      <!-- ===== 第六组：生效设置 ===== -->
      <div class="section-title">{{ $t('page.finance.commissionRule.drawer.effectiveSettings') }}</div>

      <FormItem
        name="calcBaseType"
        :label="$t('page.finance.commissionRule.drawer.calcBaseType')"
        :rules="[{ required: true, message: $t('page.finance.commissionRule.drawer.calcBaseTypeRequired') }]"
      >
        <Select
          v-model:value="formData.calcBaseType"
          :options="calcBaseTypeOptions"
        />
      </FormItem>

      <FormItem
        name="triggerCondition"
        :label="$t('page.finance.commissionRule.drawer.triggerCondition')"
        :rules="[{ required: true, message: $t('page.finance.commissionRule.drawer.triggerConditionRequired') }]"
      >
        <Select
          v-model:value="formData.triggerCondition"
          :options="triggerTypeOptions"
          allow-clear
        />
      </FormItem>

      <FormItem name="priority" :label="$t('page.finance.commissionRule.drawer.priority')">
        <InputNumber
          v-model:value="formData.priority"
          :min="0"
          style="width: 100%"
        />
      </FormItem>

      <FormItem name="effectiveDate" :label="$t('page.finance.commissionRule.drawer.effectiveDate')">
        <DatePicker
          v-model:value="formData.effectiveDate"
          value-format="YYYY-MM-DD"
          style="width: 100%"
        />
      </FormItem>

      <FormItem name="expiryDate" :label="$t('page.finance.commissionRule.drawer.expiryDate')">
        <DatePicker
          v-model:value="formData.expiryDate"
          value-format="YYYY-MM-DD"
          style="width: 100%"
        />
      </FormItem>

      <FormItem name="isDefault" :label="$t('page.finance.commissionRule.drawer.isDefault')">
        <Switch v-model:checked="formData.isDefault" />
      </FormItem>

      <FormItem name="enabled" :label="$t('page.finance.common.status')">
        <Switch v-model:checked="formData.enabled" :checked-children="$t('page.finance.common.enabled')" :un-checked-children="$t('page.finance.common.disabled')" />
      </FormItem>

      <FormItem name="description" :label="$t('page.finance.commissionRule.drawer.description')">
        <Input.TextArea
          v-model:value="formData.description"
          :rows="3"
          allow-clear
        />
      </FormItem>

      <!-- ===== 阶梯配置（仅 category=1,6）===== -->
      <template v-if="showTierConfig">
        <div class="section-title">{{ $t('page.finance.commissionRule.drawer.tierConfig') }}</div>
        <FormItem :wrapper-col="{ span: 24 }">
          <div class="mb-2">
            <Button type="dashed" size="small" @click="addTier">
              + {{ $t('page.finance.commissionRule.drawer.addTier') }}
            </Button>
          </div>
          <Table
            :data-source="tiers"
            :columns="tierColumns"
            :pagination="false"
            row-key="sort"
            size="small"
            bordered
          >
            <template #bodyCell="{ column, index }">
              <template v-if="column.key === 'minAmount'">
                <InputNumber v-model:value="tiers[index].minAmount" :min="0" :precision="2" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'maxAmount'">
                <InputNumber v-model:value="tiers[index].maxAmount" :min="0" :precision="2" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'commissionRate'">
                <InputNumber v-model:value="tiers[index].commissionRate" :min="0" :max="1" :step="0.0001" :precision="4" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'sort'">
                <InputNumber v-model:value="tiers[index].sort" :min="0" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'action'">
                <Button type="link" danger size="small" @click="removeTier(index)">
                  {{ $t('page.finance.common.delete') }}
                </Button>
              </template>
            </template>
          </Table>
        </FormItem>
      </template>

      <!-- ===== 成员配置（保留，仅 ruleType=2）===== -->
      <template v-if="formData.ruleType === 2">
        <div class="section-title">{{ $t('page.finance.commissionRule.drawer.memberConfig') }}</div>
        <FormItem :wrapper-col="{ span: 24 }">
          <div class="mb-2">
            <Button type="dashed" size="small" @click="addMember">
              + {{ $t('page.finance.commissionRule.drawer.addMember') }}
            </Button>
          </div>
          <Table
            :data-source="members"
            :columns="memberColumns"
            :pagination="false"
            row-key="sort"
            size="small"
            bordered
          >
            <template #bodyCell="{ column, index }">
              <template v-if="column.key === 'memberType'">
                <Select v-model:value="members[index].memberType" :options="memberTypeOptions" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'memberName'">
                <Input v-model:value="members[index].memberName" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'distributionType'">
                <Select v-model:value="members[index].distributionType" :options="distributionTypeOptions" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'fixedRate'">
                <InputNumber v-model:value="members[index].fixedRate" :min="0" :max="1" :step="0.0001" :precision="4" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'roleName'">
                <Input v-model:value="members[index].roleName" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'defaultRatio'">
                <InputNumber v-model:value="members[index].defaultRatio" :min="0" :max="1" :step="0.0001" :precision="4" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'required'">
                <Select v-model:value="members[index].required" :options="[{ value: 0, label: $t('page.finance.common.no') }, { value: 1, label: $t('page.finance.common.yes') }]" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'sort'">
                <InputNumber v-model:value="members[index].sort" :min="0" style="width: 100%" />
              </template>
              <template v-else-if="column.key === 'action'">
                <Button type="link" danger size="small" @click="removeMember(index)">
                  {{ $t('page.finance.common.delete') }}
                </Button>
              </template>
            </template>
          </Table>
        </FormItem>
      </template>
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="handleClose">{{ $t('page.finance.common.cancel') }}</Button>
        <Button type="primary" :loading="loading" @click="handleSubmit">
          {{ $t('page.finance.common.save') }}
        </Button>
      </div>
    </template>
  </Drawer>
</template>

<style scoped>
.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
  margin: 16px 0 12px;
  padding-left: 8px;
  border-left: 3px solid #1677ff;
}

.reallocation-hint {
  padding: 12px 16px;
  background: #f3e8ff;
  border: 1px solid #d8b4fe;
  border-radius: 6px;
  color: #6b21a8;
  font-size: 13px;
  line-height: 1.6;
}
</style>
