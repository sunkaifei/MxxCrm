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
  Table,
  Textarea,
  message,
} from 'ant-design-vue';

import { saveCommissionRuleApi } from '#/api/core/finance';

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

const triggerTypeOptions = [
  { value: 1, label: '合同签订' },
  { value: 2, label: '回款到账' },
  { value: 3, label: '订单完成' },
  { value: 4, label: '发票开具' },
];

const formData = reactive<any>({
  ruleName: '',
  deptName: '',
  postName: '',
  triggerType: undefined,
  effectiveDate: undefined,
  expireDate: undefined,
  description: '',
});

const tiers = ref<any[]>([]);

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

function resetForm() {
  formData.ruleName = '';
  formData.deptName = '';
  formData.postName = '';
  formData.triggerType = undefined;
  formData.effectiveDate = undefined;
  formData.expireDate = undefined;
  formData.description = '';
  tiers.value = [];
  formRef.value?.resetFields();
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      resetForm();
      if (props.data) {
        const row = props.data;
        formData.ruleName = row.ruleName ?? '';
        formData.deptName = row.deptName ?? '';
        formData.postName = row.postName ?? '';
        formData.triggerType = row.triggerType;
        formData.effectiveDate = row.effectiveDate;
        formData.expireDate = row.expireDate;
        formData.description = row.description ?? '';
        tiers.value =
          row.tiers && row.tiers.length > 0
            ? row.tiers.map((t: any) => ({ ...t }))
            : [];
      }
      if (tiers.value.length === 0) {
        addTier();
      }
    }
  },
);

const tierColumns = computed(() => [
  {
    title: '最低金额',
    dataIndex: 'minAmount',
    width: 160,
    key: 'minAmount',
  },
  {
    title: '最高金额',
    dataIndex: 'maxAmount',
    width: 160,
    key: 'maxAmount',
  },
  {
    title: '提成比例',
    dataIndex: 'commissionRate',
    width: 160,
    key: 'commissionRate',
  },
  {
    title: '排序',
    dataIndex: 'sort',
    width: 120,
    key: 'sort',
  },
  {
    title: '操作',
    key: 'action',
    width: 80,
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
    :title="isEdit ? '编辑提成规则' : '新增提成规则'"
    :width="720"
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
      <FormItem
        name="ruleName"
        label="规则名称"
        :rules="[{ required: true, message: '请输入规则名称' }]"
      >
        <Input
          v-model:value="formData.ruleName"
          placeholder="请输入规则名称"
          allow-clear
        />
      </FormItem>

      <FormItem name="deptName" label="适用部门">
        <Input
          v-model:value="formData.deptName"
          placeholder="请输入适用部门"
          allow-clear
        />
      </FormItem>

      <FormItem name="postName" label="适用岗位">
        <Input
          v-model:value="formData.postName"
          placeholder="请输入适用岗位"
          allow-clear
        />
      </FormItem>

      <FormItem
        name="triggerType"
        label="触发条件"
        :rules="[{ required: true, message: '请选择触发条件' }]"
      >
        <Select
          v-model:value="formData.triggerType"
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

      <FormItem name="expireDate" label="失效日期">
        <DatePicker
          v-model:value="formData.expireDate"
          value-format="YYYY-MM-DD"
          style="width: 100%"
          placeholder="请选择失效日期"
        />
      </FormItem>

      <FormItem name="description" label="描述">
        <Textarea
          v-model:value="formData.description"
          :rows="3"
          placeholder="请输入描述"
          allow-clear
        />
      </FormItem>

      <FormItem label="阶梯配置" :wrapper-col="{ span: 18 }">
        <div class="mb-2">
          <Button type="dashed" size="small" @click="addTier">
            + 新增阶梯
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
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="handleClose">取消</Button>
        <Button type="primary" :loading="loading" @click="handleSubmit">
          提交
        </Button>
      </div>
    </template>
  </Drawer>
</template>
