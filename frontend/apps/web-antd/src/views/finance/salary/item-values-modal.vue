<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import {
  InputNumber,
  message,
  Modal,
  Table,
  Tag,
} from 'ant-design-vue';

import {
  getSalaryItemListApi,
  getSalaryItemValuesApi,
  saveSalaryItemValuesApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

const props = defineProps<{
  open: boolean;
  record: any;
}>();

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void;
  (e: 'success'): void;
}>();

const loading = ref(false);
const submitting = ref(false);
const rows = ref<any[]>([]);

const columns = computed(() => [
  { title: $t('page.finance.salary.itemValues.column.itemName'), dataIndex: 'itemName' },
  { title: $t('page.finance.salary.itemValues.column.itemType'), dataIndex: 'itemType', width: 90 },
  { title: $t('page.finance.salary.itemValues.column.amount'), dataIndex: 'amount', width: 180 },
]);

function handleItemType(itemType: number) {
  return itemType === 2
    ? { color: 'red', label: $t('page.finance.salary.itemValues.type.deduction') }
    : { color: 'green', label: $t('page.finance.salary.itemValues.type.addition') };
}

async function loadData() {
  if (!props.record?.id) return;
  loading.value = true;
  try {
    const [itemsRes, valuesRes] = await Promise.all([
      getSalaryItemListApi(),
      getSalaryItemValuesApi(props.record.id),
    ]);
    const items = (itemsRes?.data ?? itemsRes) || [];
    const values = (valuesRes?.data ?? valuesRes) || [];
    const valueMap = new Map<number, number>();
    for (const v of values) {
      valueMap.set(Number(v.itemId), Number(v.amount));
    }
    // 仅手动录入模式（calcMode=1）且启用中的项目可编辑
    rows.value = items
      .filter((item: any) => Number(item.calcMode) === 1 && Number(item.enabled) === 1)
      .map((item: any) => ({
        itemId: Number(item.id),
        itemName: item.itemName,
        itemType: Number(item.itemType ?? 1),
        amount: valueMap.get(Number(item.id)) ?? 0,
      }));
  } catch {
    rows.value = [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.open,
  (visible) => {
    if (visible) {
      loadData();
    }
  },
);

function handleAmountChange(itemId: number, value: string | number | null) {
  const row = rows.value.find((r) => r.itemId === itemId);
  if (row) {
    row.amount = value === null || value === '' ? 0 : Number(value);
  }
}

function handleCancel() {
  emit('update:open', false);
}

async function handleSubmit() {
  const negative = rows.value.some((r) => r.amount === null || r.amount < 0);
  if (negative) {
    message.warning($t('page.finance.salary.itemValues.message.amountInvalid'));
    return;
  }
  submitting.value = true;
  try {
    await saveSalaryItemValuesApi({
      salaryRecordId: props.record.id,
      values: rows.value.map((r) => ({ itemId: r.itemId, amount: r.amount ?? 0 })),
    });
    message.success($t('page.finance.salary.itemValues.message.saveSuccess'));
    emit('update:open', false);
    emit('success');
  } catch (error: any) {
    message.error(error?.message || $t('page.finance.salary.itemValues.message.saveFailed'));
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Modal
    :open="open"
    :title="$t('page.finance.salary.itemValues.title', {
      name: record?.employeeName || record?.employeeId || '',
      year: record?.year || '',
      month: record?.month || '',
    })"
    :confirm-loading="submitting"
    :mask-closable="false"
    :width="560"
    @cancel="handleCancel"
    @ok="handleSubmit"
  >
    <p class="text-gray-400 mb-2 text-xs">
      {{ $t('page.finance.salary.itemValues.tip') }}
    </p>
    <Table
      :columns="columns"
      :data-source="rows"
      :loading="loading"
      :pagination="false"
      row-key="itemId"
      size="small"
    >
      <template #bodyCell="{ column, record: row }">
        <template v-if="column.dataIndex === 'itemType'">
          <Tag :color="handleItemType(row.itemType).color">
            {{ handleItemType(row.itemType).label }}
          </Tag>
        </template>
        <template v-else-if="column.dataIndex === 'amount'">
          <InputNumber
            :min="0"
            :max="99999999"
            :precision="2"
            :value="row.amount"
            class="w-full"
            @update:value="
              (val: string | number | null) => handleAmountChange(row.itemId, val)
            "
          />
        </template>
      </template>
    </Table>
    <p v-if="!loading && rows.length === 0" class="text-gray-400 mt-2 text-center">
      {{ $t('page.finance.salary.itemValues.empty') }}
    </p>
  </Modal>
</template>
