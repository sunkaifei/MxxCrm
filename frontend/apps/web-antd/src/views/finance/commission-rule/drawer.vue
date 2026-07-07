<script lang="ts" setup>
import { computed, reactive, ref, watch } from 'vue';

import {
  Button,
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

import { saveCommissionRuleApi } from '#/api/core/finance';
import { getDeptTreeApi } from '#/api/core/system/dept';

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

const ruleTypeOptions = [
  { value: 1, label: '个人业绩' },
  { value: 2, label: '团队分成' },
  { value: 3, label: '部门经理' },
  { value: 4, label: '总监' },
  { value: 5, label: '团队长' },
];

const applyScopeOptions = [
  { value: 1, label: '指定部门' },
  { value: 2, label: '全公司' },
  { value: 3, label: '指定岗位' },
  { value: 4, label: '指定人员' },
];

const triggerTypeOptions = [
  { value: 1, label: '合同签订' },
  { value: 2, label: '回款到账' },
  { value: 3, label: '订单完成' },
  { value: 4, label: '发票开具' },
];

const calcBaseTypeOptions = [
  { value: 1, label: '个人月累计' },
  { value: 2, label: '团队月累计' },
  { value: 3, label: '单笔合同' },
  { value: 4, label: '单笔回款' },
];

const roleTypeOptions = [
  { value: 1, label: '主签人' },
  { value: 2, label: '参与人' },
  { value: 3, label: '技术支持' },
  { value: 4, label: '部门经理' },
  { value: 5, label: '其他' },
];

const formData = reactive<any>({
  ruleName: '',
  ruleType: 1,
  applyScope: 2,
  departmentId: undefined,
  postId: undefined,
  priority: 0,
  isDefault: false,
  enabled: true,
  calcBaseType: 1,
  triggerCondition: undefined,
  effectiveDate: undefined,
  expiryDate: undefined,
  description: '',
});

const tiers = ref<any[]>([]);
const members = ref<any[]>([]);
const deptTreeData = ref<any[]>([]);

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
  formData.ruleType = 1;
  formData.applyScope = 2;
  formData.departmentId = undefined;
  formData.postId = undefined;
  formData.priority = 0;
  formData.isDefault = false;
  formData.enabled = true;
  formData.calcBaseType = 1;
  formData.triggerCondition = undefined;
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
      if (props.data) {
        const row = props.data;
        formData.ruleName = row.ruleName ?? '';
        formData.ruleType = row.ruleType ?? 1;
        formData.applyScope = row.applyScope ?? 2;
        formData.departmentId = row.departmentId;
        formData.postId = row.postId;
        formData.priority = row.priority ?? 0;
        formData.isDefault = row.isDefault ?? false;
        formData.enabled = row.enabled ?? true;
        formData.calcBaseType = row.calcBaseType ?? 1;
        formData.triggerCondition = row.triggerCondition;
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
      if (tiers.value.length === 0) {
        addTier();
      }
      if (members.value.length === 0 && formData.ruleType === 2) {
        addMember();
      }
    }
  },
);

const tierColumns = computed(() => [
  {
    title: '最低金额',
    dataIndex: 'minAmount',
    width: 140,
    key: 'minAmount',
  },
  {
    title: '最高金额',
    dataIndex: 'maxAmount',
    width: 140,
    key: 'maxAmount',
  },
  {
    title: '提成比例',
    dataIndex: 'commissionRate',
    width: 140,
    key: 'commissionRate',
  },
  {
    title: '排序',
    dataIndex: 'sort',
    width: 100,
    key: 'sort',
  },
  {
    title: '操作',
    key: 'action',
    width: 80,
  },
]);

const memberTypeOptions = [
  { value: 1, label: '业务员' },
  { value: 2, label: '直属经理' },
  { value: 3, label: '部门总监' },
  { value: 4, label: '其他' },
];

const distributionTypeOptions = [
  { value: 1, label: '固定比例' },
];

const memberColumns = computed(() => [
  {
    title: '成员类型',
    dataIndex: 'memberType',
    width: 110,
    key: 'memberType',
  },
  {
    title: '成员名称',
    dataIndex: 'memberName',
    width: 120,
    key: 'memberName',
  },
  {
    title: '分配类型',
    dataIndex: 'distributionType',
    width: 100,
    key: 'distributionType',
  },
  {
    title: '固定比例',
    dataIndex: 'fixedRate',
    width: 100,
    key: 'fixedRate',
  },
  {
    title: '角色名称',
    dataIndex: 'roleName',
    width: 120,
    key: 'roleName',
  },
  {
    title: '默认比例',
    dataIndex: 'defaultRatio',
    width: 100,
    key: 'defaultRatio',
  },
  {
    title: '必选',
    dataIndex: 'required',
    width: 70,
    key: 'required',
  },
  {
    title: '排序',
    dataIndex: 'sort',
    width: 70,
    key: 'sort',
  },
  {
    title: '操作',
    key: 'action',
    width: 70,
  },
]);

async function handleSubmit() {
  try {
    await formRef.value.validate();
  } catch {
    return;
  }

  if (tiers.value.length === 0) {
    message.warning('请至少添加一条阶梯配置');
    return;
  }

  loading.value = true;
  try {
    const payload = {
      ...formData,
      tiers: tiers.value,
      members: formData.ruleType === 2 ? members.value : [],
      ...(props.data?.id ? { id: props.data.id } : {}),
    };
    await saveCommissionRuleApi(payload);
    message.success(isEdit.value ? '更新成功' : '创建成功');
    emit('close', true);
  } catch (e: any) {
    message.error(e?.message || '操作失败');
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
    :title="isEdit ? '编辑提成方案' : '新增提成方案'"
    :width="800"
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
      <div class="text-base font-semibold mb-3 mt-2">基本信息</div>

      <FormItem
        name="ruleName"
        label="方案名称"
        :rules="[{ required: true, message: '请输入方案名称' }]"
      >
        <Input
          v-model:value="formData.ruleName"
          placeholder="请输入方案名称"
          allow-clear
        />
      </FormItem>

      <FormItem
        name="ruleType"
        label="方案类型"
        :rules="[{ required: true, message: '请选择方案类型' }]"
      >
        <Select
          v-model:value="formData.ruleType"
          placeholder="请选择方案类型"
          :options="ruleTypeOptions"
        />
      </FormItem>

      <FormItem name="applyScope" label="适用范围">
        <Select
          v-model:value="formData.applyScope"
          placeholder="请选择适用范围"
          :options="applyScopeOptions"
        />
      </FormItem>

      <FormItem v-if="formData.applyScope === 1" name="departmentId" label="适用部门">
        <TreeSelect
          v-model:value="formData.departmentId"
          :tree-data="deptTreeData"
          tree-node-filter-prop="label"
          placeholder="请选择适用部门"
          style="width: 100%"
          allow-clear
          show-search
          tree-default-expand-all
        />
      </FormItem>

      <FormItem v-if="formData.applyScope === 3" name="postId" label="适用岗位">
        <InputNumber
          v-model:value="formData.postId"
          placeholder="请输入岗位ID"
          style="width: 100%"
          :min="0"
        />
      </FormItem>

      <FormItem name="priority" label="优先级">
        <InputNumber
          v-model:value="formData.priority"
          placeholder="请输入优先级"
          style="width: 100%"
          :min="0"
        />
      </FormItem>

      <FormItem
        name="calcBaseType"
        label="计算基准"
        :rules="[{ required: true, message: '请选择计算基准' }]"
      >
        <Select
          v-model:value="formData.calcBaseType"
          placeholder="请选择计算基准"
          :options="calcBaseTypeOptions"
        />
      </FormItem>

      <FormItem
        name="triggerCondition"
        label="触发条件"
        :rules="[{ required: true, message: '请选择触发条件' }]"
      >
        <Select
          v-model:value="formData.triggerCondition"
          placeholder="请选择触发条件"
          :options="triggerTypeOptions"
          allow-clear
        />
      </FormItem>

      <FormItem name="effectiveDate" label="生效日期">
        <DatePicker
          v-model:value="formData.effectiveDate"
          value-format="YYYY-MM-DD"
          style="width: 100%"
          placeholder="请选择生效日期"
        />
      </FormItem>

      <FormItem name="expiryDate" label="失效日期">
        <DatePicker
          v-model:value="formData.expiryDate"
          value-format="YYYY-MM-DD"
          style="width: 100%"
          placeholder="请选择失效日期"
        />
      </FormItem>

      <FormItem name="isDefault" label="是否默认">
        <Switch v-model:checked="formData.isDefault" />
      </FormItem>

      <FormItem name="enabled" label="状态">
        <Switch v-model:checked="formData.enabled" checked-children="启用" un-checked-children="禁用" />
      </FormItem>

      <FormItem name="description" label="描述">
        <Input.TextArea
          v-model:value="formData.description"
          :rows="3"
          placeholder="请输入描述"
          allow-clear
        />
      </FormItem>

      <div class="text-base font-semibold mb-3 mt-4">金额阶梯配置</div>

      <FormItem :wrapper-col="{ span: 24 }">
        <div class="mb-2">
          <Button type="dashed" size="small" @click="addTier">
            + 添加阶梯
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
              <InputNumber
                v-model:value="tiers[index].minAmount"
                :min="0"
                :precision="2"
                style="width: 100%"
                placeholder="最低金额"
              />
            </template>
            <template v-else-if="column.key === 'maxAmount'">
              <InputNumber
                v-model:value="tiers[index].maxAmount"
                :min="0"
                :precision="2"
                style="width: 100%"
                placeholder="最高金额"
              />
            </template>
            <template v-else-if="column.key === 'commissionRate'">
              <InputNumber
                v-model:value="tiers[index].commissionRate"
                :min="0"
                :max="1"
                :step="0.0001"
                :precision="4"
                style="width: 100%"
                placeholder="提成比例"
              />
            </template>
            <template v-else-if="column.key === 'sort'">
              <InputNumber
                v-model:value="tiers[index].sort"
                :min="0"
                style="width: 100%"
                placeholder="排序"
              />
            </template>
            <template v-else-if="column.key === 'action'">
              <Button
                type="link"
                danger
                size="small"
                @click="removeTier(index)"
              >
                删除
              </Button>
            </template>
          </template>
        </Table>
      </FormItem>

      <div v-if="formData.ruleType === 2" class="text-base font-semibold mb-3 mt-4">默认成员配置</div>

      <FormItem v-if="formData.ruleType === 2" :wrapper-col="{ span: 24 }">
        <div class="mb-2">
          <Button type="dashed" size="small" @click="addMember">
            + 添加成员
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
              <Select
                v-model:value="members[index].memberType"
                :options="memberTypeOptions"
                style="width: 100%"
              />
            </template>
            <template v-else-if="column.key === 'memberName'">
              <Input
                v-model:value="members[index].memberName"
                placeholder="成员名称"
                style="width: 100%"
              />
            </template>
            <template v-else-if="column.key === 'distributionType'">
              <Select
                v-model:value="members[index].distributionType"
                :options="distributionTypeOptions"
                style="width: 100%"
              />
            </template>
            <template v-else-if="column.key === 'fixedRate'">
              <InputNumber
                v-model:value="members[index].fixedRate"
                :min="0"
                :max="1"
                :step="0.0001"
                :precision="4"
                style="width: 100%"
                placeholder="固定比例"
              />
            </template>
            <template v-else-if="column.key === 'roleName'">
              <Input
                v-model:value="members[index].roleName"
                placeholder="角色名称"
                style="width: 100%"
              />
            </template>
            <template v-else-if="column.key === 'defaultRatio'">
              <InputNumber
                v-model:value="members[index].defaultRatio"
                :min="0"
                :max="1"
                :step="0.0001"
                :precision="4"
                style="width: 100%"
                placeholder="默认比例"
              />
            </template>
            <template v-else-if="column.key === 'required'">
              <Select
                v-model:value="members[index].required"
                :options="[{ value: 0, label: '否' }, { value: 1, label: '是' }]"
                style="width: 100%"
              />
            </template>
            <template v-else-if="column.key === 'sort'">
              <InputNumber
                v-model:value="members[index].sort"
                :min="0"
                style="width: 100%"
                placeholder="排序"
              />
            </template>
            <template v-else-if="column.key === 'action'">
              <Button
                type="link"
                danger
                size="small"
                @click="removeMember(index)"
              >
                删除
              </Button>
            </template>
          </template>
        </Table>
      </FormItem>
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="handleClose">取消</Button>
        <Button type="primary" :loading="loading" @click="handleSubmit">
          保存
        </Button>
      </div>
    </template>
  </Drawer>
</template>
