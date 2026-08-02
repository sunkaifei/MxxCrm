<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import {
  Button,
  Card,
  Col,
  DatePicker,
  Empty,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Row,
  Statistic,
  Table,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import {
  expenseCommissionPoolApi,
  getCommissionPoolDetailApi,
  getCommissionPoolListApi,
  getCommissionPoolLogApi,
  saveCommissionPoolApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

const props = withDefaults(
  defineProps<{
    // 是否在 Tab 内嵌使用时隐藏外层 Card 标题（由父组件提供标题）
    embedded?: boolean;
  }>(),
  {
    embedded: false,
  },
);

const formatMoney = (val: any) => Number(val || 0).toFixed(2);

// ===== 列表数据 =====
const loading = ref(false);
const tableData = ref<any[]>([]);

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getCommissionPoolListApi();
    const list = res?.data || res || [];
    tableData.value = Array.isArray(list)
      ? list
      : list?.items || list?.list || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 顶部统计 =====
const totalCount = computed(() => tableData.value.length);
const totalBalance = computed(() =>
  tableData.value.reduce((sum, i) => sum + Number(i.balance || 0), 0),
);
const totalDeposit = computed(() =>
  tableData.value.reduce((sum, i) => sum + Number(i.totalAmount || 0), 0),
);
const totalUsed = computed(() =>
  tableData.value.reduce((sum, i) => sum + Number(i.usedAmount || 0), 0),
);

// ===== 状态映射：1=活跃 2=冻结 3=已关闭 =====
const statusMap: Record<number, { label: string; color: string }> = {
  1: { label: $t('page.finance.pool.statusActive'), color: 'green' },
  2: { label: $t('page.finance.pool.statusFrozen'), color: 'orange' },
  3: { label: $t('page.finance.pool.statusClosed'), color: 'default' },
};

// ===== 列表表格列 =====
const columns: ColumnsType = [
  {
    title: $t('page.finance.pool.column.name'),
    dataIndex: 'poolName',
    width: 180,
    ellipsis: true,
  },
  {
    title: $t('page.finance.pool.column.balance'),
    dataIndex: 'balance',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.pool.column.total'),
    dataIndex: 'totalAmount',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.pool.column.used'),
    dataIndex: 'usedAmount',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.pool.column.manager'),
    dataIndex: 'managerName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.pool.column.status'),
    dataIndex: 'status',
    width: 100,
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 260,
    fixed: 'right',
  },
];

// ===== 新建/编辑资金池 =====
const editVisible = ref(false);
const editMode = ref<'create' | 'edit'>('create');
const editLoading = ref(false);
const editFormRef = ref();
const editForm = reactive({
  id: 0,
  poolName: '',
  managerId: undefined as number | undefined,
  departmentId: undefined as number | undefined,
  description: '',
  status: 1,
});
const editRules: Record<string, any[]> = {
  poolName: [
    {
      required: true,
      message: $t('page.finance.pool.namePlaceholder'),
      trigger: 'blur',
    },
  ],
};

function resetEditForm() {
  editForm.id = 0;
  editForm.poolName = '';
  editForm.managerId = undefined;
  editForm.departmentId = undefined;
  editForm.description = '';
  editForm.status = 1;
}

function openCreate() {
  resetEditForm();
  editMode.value = 'create';
  editVisible.value = true;
}

async function openEdit(row: any) {
  resetEditForm();
  editMode.value = 'edit';
  editForm.id = row.id;
  editForm.poolName = row.poolName || '';
  editForm.managerId = row.managerId;
  editForm.departmentId = row.departmentId;
  editForm.description = row.description || '';
  editForm.status = row.status ?? 1;
  editVisible.value = true;
  try {
    const res: any = await getCommissionPoolDetailApi(row.id);
    const d = (res?.data || res) as any;
    if (d && typeof d === 'object') {
      editForm.poolName = d.poolName ?? editForm.poolName;
      editForm.managerId = d.managerId ?? editForm.managerId;
      editForm.departmentId = d.departmentId ?? editForm.departmentId;
      editForm.description = d.description ?? editForm.description;
      editForm.status = d.status ?? editForm.status;
    }
  } catch {
    // 使用列表行数据兜底
  }
}

async function handleSave() {
  try {
    await editFormRef.value?.validate();
  } catch {
    return;
  }
  editLoading.value = true;
  try {
    const payload: any = {
      poolName: editForm.poolName,
      managerId: editForm.managerId || undefined,
      departmentId: editForm.departmentId || undefined,
      description: editForm.description || undefined,
      status: editForm.status,
    };
    if (editMode.value === 'edit') payload.id = editForm.id;
    await saveCommissionPoolApi(payload);
    message.success($t('page.finance.pool.saveSuccess'));
    editVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    editLoading.value = false;
  }
}

// ===== 支出登记 =====
const expenseVisible = ref(false);
const expenseLoading = ref(false);
const expenseFormRef = ref();
const expenseForm = reactive({
  poolId: 0,
  poolName: '',
  balance: 0,
  amount: undefined as number | undefined,
  usageDate: '',
  usageDescription: '',
});
const expenseRules: Record<string, any[]> = {
  amount: [
    {
      required: true,
      type: 'number',
      message: $t('page.finance.pool.expenseAmount'),
      trigger: 'change',
    },
  ],
  usageDescription: [
    {
      required: true,
      message: $t('page.finance.pool.expenseReasonPlaceholder'),
      trigger: 'blur',
    },
  ],
};

function openExpense(row: any) {
  expenseForm.poolId = row.id;
  expenseForm.poolName = row.poolName || '';
  expenseForm.balance = row.balance ?? 0;
  expenseForm.amount = undefined;
  expenseForm.usageDate = '';
  expenseForm.usageDescription = '';
  expenseVisible.value = true;
}

async function handleExpense() {
  try {
    await expenseFormRef.value?.validate();
  } catch {
    return;
  }
  if (!expenseForm.amount || expenseForm.amount <= 0) {
    message.warning($t('page.finance.pool.expenseAmount'));
    return;
  }
  expenseLoading.value = true;
  try {
    await expenseCommissionPoolApi({
      poolId: expenseForm.poolId,
      amount: expenseForm.amount,
      usageDate: expenseForm.usageDate || undefined,
      usageDescription: expenseForm.usageDescription || undefined,
    });
    message.success($t('page.finance.pool.expenseSuccess'));
    expenseVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.failed'));
  } finally {
    expenseLoading.value = false;
  }
}

// ===== 流水明细 =====
const logVisible = ref(false);
const logLoading = ref(false);
const logData = ref<any[]>([]);
const logPoolName = ref('');

async function openLog(row: any) {
  logPoolName.value = row.poolName || '';
  logVisible.value = true;
  logLoading.value = true;
  logData.value = [];
  try {
    const res: any = await getCommissionPoolLogApi(row.id);
    const list = res?.data || res || [];
    logData.value = Array.isArray(list)
      ? list
      : list?.items || list?.list || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.loadFailed'));
    logData.value = [];
  } finally {
    logLoading.value = false;
  }
}

const logColumns: ColumnsType = [
  {
    title: $t('page.finance.pool.logDate'),
    dataIndex: 'createTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.pool.logType'),
    dataIndex: 'logType',
    width: 100,
  },
  {
    title: $t('page.finance.pool.logAmount'),
    dataIndex: 'amount',
    width: 130,
    align: 'right',
    customRender: ({ text, record }: any) => {
      const isDeposit = record.logType === 1;
      const sign = isDeposit ? '+' : '-';
      return {
        children: `${sign}${formatMoney(text)}`,
        style: {
          color: isDeposit ? '#52c41a' : '#ff4d4f',
          fontWeight: 500,
        },
      };
    },
  },
  {
    title: $t('page.finance.pool.logSource'),
    key: 'source',
    minWidth: 180,
    customRender: ({ record }: any) =>
      record.source || record.usageDescription || '-',
  },
];

onMounted(() => {
  loadList();
});

defineExpose({ loadList });
</script>

<template>
  <div>
    <!-- 顶部统计卡片 -->
    <Row :gutter="16" class="mb-4">
      <Col :span="6">
        <Card :bordered="false">
          <Statistic
            :title="$t('page.finance.pool.title')"
            :value="totalCount"
          />
        </Card>
      </Col>
      <Col :span="6">
        <Card :bordered="false">
          <Statistic
            :title="$t('page.finance.pool.balance')"
            :value="totalBalance"
            :precision="2"
            prefix="¥"
            :value-style="{ color: '#1890ff' }"
          />
        </Card>
      </Col>
      <Col :span="6">
        <Card :bordered="false">
          <Statistic
            :title="$t('page.finance.pool.total')"
            :value="totalDeposit"
            :precision="2"
            prefix="¥"
            :value-style="{ color: '#52c41a' }"
          />
        </Card>
      </Col>
      <Col :span="6">
        <Card :bordered="false">
          <Statistic
            :title="$t('page.finance.pool.used')"
            :value="totalUsed"
            :precision="2"
            prefix="¥"
            :value-style="{ color: '#ff4d4f' }"
          />
        </Card>
      </Col>
    </Row>

    <!-- 资金池列表 -->
    <Card :bordered="false" :title="embedded ? undefined : $t('page.finance.pool.title')">
      <template v-if="!embedded" #extra>
        <Button type="primary" @click="openCreate">
          {{ $t('page.finance.pool.create') }}
        </Button>
      </template>
      <div v-if="embedded" class="mb-3" style="text-align: right">
        <Button type="primary" @click="openCreate">
          {{ $t('page.finance.pool.create') }}
        </Button>
      </div>
      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        row-key="id"
        size="small"
        :pagination="{
          pageSize: 20,
          showSizeChanger: true,
          showTotal: (t: number) =>
            $t('page.finance.common.total', { count: t }),
        }"
      >
        <template #emptyText>
          <Empty :description="$t('page.finance.pool.empty')" />
        </template>
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'status'">
            <Tag :color="statusMap[record.status]?.color || 'default'">
              {{ statusMap[record.status]?.label || record.status }}
            </Tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <Button type="link" size="small" @click="openExpense(record)">
              {{ $t('page.finance.pool.expense') }}
            </Button>
            <Button type="link" size="small" @click="openLog(record)">
              {{ $t('page.finance.pool.log') }}
            </Button>
            <Button type="link" size="small" @click="openEdit(record)">
              {{ $t('page.finance.common.edit') }}
            </Button>
          </template>
        </template>
      </Table>
    </Card>

    <!-- 新建/编辑资金池 -->
    <Modal
      v-model:open="editVisible"
      :title="
        editMode === 'create'
          ? $t('page.finance.pool.create')
          : $t('page.finance.pool.edit')
      "
      :confirm-loading="editLoading"
      :ok-text="$t('page.finance.common.save')"
      :cancel-text="$t('page.finance.common.cancel')"
      @ok="handleSave"
    >
      <Form
        ref="editFormRef"
        :model="editForm"
        :rules="editRules"
        layout="vertical"
        class="pt-2"
      >
        <FormItem :label="$t('page.finance.pool.name')" name="poolName">
          <Input
            v-model:value="editForm.poolName"
            :placeholder="$t('page.finance.pool.namePlaceholder')"
            allow-clear
          />
        </FormItem>
        <FormItem :label="$t('page.finance.pool.manager')" name="managerId">
          <InputNumber
            v-model:value="editForm.managerId"
            :min="1"
            :placeholder="$t('page.finance.pool.manager')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.common.department')"
          name="departmentId"
        >
          <InputNumber
            v-model:value="editForm.departmentId"
            :min="1"
            :placeholder="$t('page.finance.common.department')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.pool.description')"
          name="description"
        >
          <Input.TextArea
            v-model:value="editForm.description"
            :rows="3"
            :placeholder="$t('page.finance.pool.descriptionPlaceholder')"
          />
        </FormItem>
      </Form>
    </Modal>

    <!-- 支出登记 -->
    <Modal
      v-model:open="expenseVisible"
      :title="$t('page.finance.pool.expense')"
      :confirm-loading="expenseLoading"
      :ok-text="$t('page.finance.common.save')"
      :cancel-text="$t('page.finance.common.cancel')"
      @ok="handleExpense"
    >
      <div class="mb-3 flex items-center justify-between">
        <span>
          {{ $t('page.finance.pool.name') }}：
          <strong>{{ expenseForm.poolName || '-' }}</strong>
        </span>
        <span>
          {{ $t('page.finance.pool.balance') }}：
          <strong class="text-blue-500">¥{{ formatMoney(expenseForm.balance) }}</strong>
        </span>
      </div>
      <Form
        ref="expenseFormRef"
        :model="expenseForm"
        :rules="expenseRules"
        layout="vertical"
      >
        <FormItem :label="$t('page.finance.pool.expenseAmount')" name="amount">
          <InputNumber
            v-model:value="expenseForm.amount"
            :min="0"
            :precision="2"
            :placeholder="$t('page.finance.pool.expenseAmount')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.pool.expenseDate')" name="usageDate">
          <DatePicker
            v-model:value="expenseForm.usageDate"
            value-format="YYYY-MM-DD"
            :placeholder="$t('page.finance.pool.expenseDate')"
            style="width: 100%"
          />
        </FormItem>
        <FormItem
          :label="$t('page.finance.pool.expenseReason')"
          name="usageDescription"
        >
          <Input.TextArea
            v-model:value="expenseForm.usageDescription"
            :rows="3"
            :placeholder="$t('page.finance.pool.expenseReasonPlaceholder')"
          />
        </FormItem>
      </Form>
    </Modal>

    <!-- 流水明细 -->
    <Modal
      v-model:open="logVisible"
      :title="
        $t('page.finance.pool.log') +
        (logPoolName ? ' - ' + logPoolName : '')
      "
      width="760"
      :footer="null"
    >
      <Table
        :columns="logColumns"
        :data-source="logData"
        :loading="logLoading"
        row-key="id"
        size="small"
        :pagination="false"
      >
        <template #emptyText>
          <Empty :description="$t('page.finance.common.noData')" />
        </template>
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'logType'">
            <Tag :color="record.logType === 1 ? 'green' : 'red'">
              {{
                record.logType === 1
                  ? $t('page.finance.pool.logDeposit')
                  : $t('page.finance.pool.logExpense')
              }}
            </Tag>
          </template>
        </template>
      </Table>
    </Modal>
  </div>
</template>
