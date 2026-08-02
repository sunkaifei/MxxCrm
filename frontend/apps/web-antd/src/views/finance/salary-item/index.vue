<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Alert,
  Button,
  Card,
  Empty,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Select,
  Switch,
  Table,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import {
  deleteSalaryItemApi,
  getSalaryItemListApi,
  upsertSalaryItemApi,
} from '#/api/core/finance';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';

const guideStepCount = 5;

const accessStore = useAccessStore();

const formatMoney = (val: any) => Number(val || 0).toFixed(2);

// ===== 项目类型映射：1=增项,2=减项 =====
const itemTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: $t('page.finance.salaryItem.itemType.addition'), color: 'green' },
  2: { label: $t('page.finance.salaryItem.itemType.deduction'), color: 'red' },
};

const itemTypeOptions = [
  { label: $t('page.finance.salaryItem.itemType.addition'), value: 1 },
  { label: $t('page.finance.salaryItem.itemType.deduction'), value: 2 },
];

// ===== 计算方式映射：1=固定值,2=公式,3=手动输入 =====
const calcModeMap: Record<number, string> = {
  1: $t('page.finance.salaryItem.calcMode.fixed'),
  2: $t('page.finance.salaryItem.calcMode.formula'),
  3: $t('page.finance.salaryItem.calcMode.manual'),
};

const calcModeOptions = [
  { label: $t('page.finance.salaryItem.calcMode.fixed'), value: 1 },
  { label: $t('page.finance.salaryItem.calcMode.formula'), value: 2 },
  { label: $t('page.finance.salaryItem.calcMode.manual'), value: 3 },
];

// ===== 列表数据 =====
const loading = ref(false);
const tableData = ref<any[]>([]);

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getSalaryItemListApi();
    const data = res?.data || res;
    tableData.value = Array.isArray(data) ? data : data?.items || data?.list || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 新增/编辑弹窗 =====
const formVisible = ref(false);
const formLoading = ref(false);
const isEdit = ref(false);

const formData = reactive({
  id: undefined as number | undefined,
  itemCode: '',
  itemName: '',
  itemType: 1,
  calcMode: 1,
  formula: '',
  defaultValue: 0,
  isTaxable: true,
  isPretax: true,
  sort: 0,
  enabled: true,
});

function resetForm() {
  formData.id = undefined;
  formData.itemCode = '';
  formData.itemName = '';
  formData.itemType = 1;
  formData.calcMode = 1;
  formData.formula = '';
  formData.defaultValue = 0;
  formData.isTaxable = true;
  formData.isPretax = true;
  formData.sort = 0;
  formData.enabled = true;
}

function openForm(record?: any) {
  if (record) {
    isEdit.value = true;
    formData.id = record.id;
    formData.itemCode = record.itemCode || '';
    formData.itemName = record.itemName || '';
    formData.itemType = record.itemType ?? 1;
    formData.calcMode = record.calcMode ?? 1;
    formData.formula = record.formula || '';
    formData.defaultValue = Number(record.defaultValue || 0);
    formData.isTaxable = record.isTaxable !== 0 && record.isTaxable !== false;
    formData.isPretax = record.isPretax !== 0 && record.isPretax !== false;
    formData.sort = record.sort ?? 0;
    formData.enabled = record.enabled !== 0 && record.enabled !== false;
  } else {
    isEdit.value = false;
    resetForm();
  }
  formVisible.value = true;
}

async function handleSubmit() {
  if (!formData.itemCode.trim()) {
    message.warning($t('page.finance.salaryItem.message.itemCodeRequired'));
    return;
  }
  if (!formData.itemName.trim()) {
    message.warning($t('page.finance.salaryItem.message.itemNameRequired'));
    return;
  }
  if (formData.calcMode === 2 && !formData.formula.trim()) {
    message.warning($t('page.finance.salaryItem.message.formulaRequired'));
    return;
  }
  formLoading.value = true;
  try {
    await upsertSalaryItemApi({
      id: formData.id,
      itemCode: formData.itemCode,
      itemName: formData.itemName,
      itemType: formData.itemType,
      calcMode: formData.calcMode,
      formula: formData.calcMode === 2 ? formData.formula : '',
      defaultValue: formData.defaultValue,
      isTaxable: formData.isTaxable ? 1 : 0,
      isPretax: formData.isPretax ? 1 : 0,
      sort: formData.sort,
      enabled: formData.enabled ? 1 : 0,
    });
    message.success(
      isEdit.value
        ? $t('page.finance.salaryItem.message.updateSuccess')
        : $t('page.finance.salaryItem.message.createSuccess'),
    );
    formVisible.value = false;
    loadList();
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.common.saveFailed'));
  } finally {
    formLoading.value = false;
  }
}

async function handleDelete(record: any) {
  Modal.confirm({
    title: $t('page.finance.salaryItem.message.confirmDeleteTitle'),
    content: $t('page.finance.salaryItem.drawer.deleteConfirm', {
      name: record.itemName,
    }),
    okText: $t('page.finance.common.delete'),
    okType: 'danger',
    cancelText: $t('page.finance.common.cancel'),
    async onOk() {
      try {
        await deleteSalaryItemApi(record.id);
        message.success($t('page.finance.common.deleteSuccess'));
        loadList();
      } catch (e: any) {
        message.error(e?.message || $t('page.finance.common.deleteFailed'));
      }
    },
  });
}

const columns: ColumnsType = [
  { title: $t('page.finance.salaryItem.column.itemCode'), dataIndex: 'itemCode', width: 140 },
  { title: $t('page.finance.salaryItem.column.itemName'), dataIndex: 'itemName', width: 160 },
  {
    title: $t('page.finance.salaryItem.column.itemType'),
    dataIndex: 'itemType',
    width: 90,
  },
  {
    title: $t('page.finance.salaryItem.column.calcMode'),
    dataIndex: 'calcMode',
    width: 100,
    customRender: ({ text }: any) => calcModeMap[text as number] || '-',
  },
  {
    title: $t('page.finance.salaryItem.column.formula'),
    dataIndex: 'formula',
    ellipsis: true,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.salaryItem.column.defaultValue'),
    dataIndex: 'defaultValue',
    width: 110,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salaryItem.column.isTaxable'),
    dataIndex: 'isTaxable',
    width: 80,
  },
  {
    title: $t('page.finance.salaryItem.column.sort'),
    dataIndex: 'sort',
    width: 80,
    align: 'right',
  },
  {
    title: $t('page.finance.salaryItem.column.enabled'),
    dataIndex: 'enabled',
    width: 80,
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 140,
    fixed: 'right',
  },
];

onMounted(() => {
  loadList();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.salaryItem.guide.title')"
      :brief="$t('page.finance.salaryItem.guide.brief')"
      :expand-text="$t('page.finance.salaryItem.guide.expand')"
      :collapse-text="$t('page.finance.salaryItem.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.salaryItem.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.salaryItem.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <Alert
      type="info"
      show-icon
      :message="
        $t('page.finance.salaryItem.alert', {
          example: '{baseSalary}+{commission}',
        })
      "
      class="mb-4"
    />

    <Card :bordered="false" :title="$t('page.finance.salaryItem.manageTitle')">
      <template #extra>
        <Button
          v-if="accessStore.hasAccessCode('finance:salary-item:manage')"
          type="primary"
          @click="openForm()"
        >
          {{ $t('page.finance.salaryItem.button.create') }}
        </Button>
      </template>

      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        row-key="id"
        :pagination="false"
        size="middle"
        :scroll="{ x: 1100 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'itemType'">
            <Tag :color="itemTypeMap[record.itemType]?.color ?? 'default'">
              {{ itemTypeMap[record.itemType]?.label ?? '-' }}
            </Tag>
          </template>
          <template v-else-if="column.dataIndex === 'isTaxable'">
            <Tag
              v-if="record.isTaxable === 1 || record.isTaxable === true"
              color="orange"
            >
              {{ $t('page.finance.common.yes') }}
            </Tag>
            <span v-else class="text-gray-400">
              {{ $t('page.finance.common.no') }}
            </span>
          </template>
          <template v-else-if="column.dataIndex === 'enabled'">
            <Tag
              v-if="record.enabled === 1 || record.enabled === true"
              color="green"
            >
              {{ $t('page.finance.common.enabled') }}
            </Tag>
            <Tag v-else>{{ $t('page.finance.common.disabled') }}</Tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <Button
              v-if="accessStore.hasAccessCode('finance:salary-item:manage')"
              type="link"
              size="small"
              @click="openForm(record)"
            >
              {{ $t('page.finance.common.edit') }}
            </Button>
            <Button
              v-if="accessStore.hasAccessCode('finance:salary-item:manage')"
              type="link"
              size="small"
              danger
              @click="handleDelete(record)"
            >
              {{ $t('page.finance.common.delete') }}
            </Button>
          </template>
        </template>
        <template #emptyText>
          <Empty :description="$t('page.finance.common.noData')" />
        </template>
      </Table>
    </Card>

    <!-- 新增/编辑弹窗 -->
    <Modal
      v-model:open="formVisible"
      :title="
        isEdit
          ? $t('page.finance.salaryItem.drawer.titleEdit')
          : $t('page.finance.salaryItem.drawer.titleCreate')
      "
      :confirm-loading="formLoading"
      width="560px"
      @ok="handleSubmit"
    >
      <Form layout="vertical" class="py-4">
        <FormItem :label="$t('page.finance.salaryItem.drawer.itemCode')" required>
          <Input
            v-model:value="formData.itemCode"
            :placeholder="$t('page.finance.salaryItem.drawer.itemCodePlaceholder')"
            :disabled="isEdit"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salaryItem.drawer.itemName')" required>
          <Input
            v-model:value="formData.itemName"
            :placeholder="$t('page.finance.salaryItem.drawer.itemNamePlaceholder')"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salaryItem.drawer.itemType')" required>
          <Select
            v-model:value="formData.itemType"
            :options="itemTypeOptions"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salaryItem.drawer.calcMode')" required>
          <Select
            v-model:value="formData.calcMode"
            :options="calcModeOptions"
          />
        </FormItem>
        <FormItem
          v-if="formData.calcMode === 2"
          :label="$t('page.finance.salaryItem.drawer.formula')"
          required
          :extra="
            $t('page.finance.salaryItem.drawer.formulaExtra', {
              example: '{baseSalary}、{commission}、{performanceBonus}',
            })
          "
        >
          <Input.TextArea
            v-model:value="formData.formula"
            :rows="3"
            :placeholder="
              $t('page.finance.salaryItem.drawer.formulaPlaceholder', {
                example: '{baseSalary}+{commission}*0.1',
              })
            "
          />
        </FormItem>
        <FormItem
          v-if="formData.calcMode === 1"
          :label="$t('page.finance.salaryItem.drawer.defaultValue')"
          :extra="$t('page.finance.salaryItem.drawer.defaultValueExtra')"
        >
          <InputNumber
            v-model:value="formData.defaultValue"
            :min="0"
            :precision="2"
            style="width: 100%"
            prefix="¥"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salaryItem.drawer.isTaxable')">
          <Switch v-model:checked="formData.isTaxable" />
        </FormItem>
        <FormItem
          :label="$t('page.finance.salaryItem.drawer.isPretax')"
          :extra="$t('page.finance.salaryItem.drawer.isPretaxExtra')"
        >
          <Switch v-model:checked="formData.isPretax" />
        </FormItem>
        <FormItem :label="$t('page.finance.salaryItem.drawer.sort')">
          <InputNumber
            v-model:value="formData.sort"
            :min="0"
            style="width: 100%"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salaryItem.drawer.enabled')">
          <Switch v-model:checked="formData.enabled" />
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>
