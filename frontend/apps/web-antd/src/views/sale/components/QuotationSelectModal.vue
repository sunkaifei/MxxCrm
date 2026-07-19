<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';

import { Button, Input, Modal, Table, Tag } from 'ant-design-vue';

import { getQuotationListApi } from '#/api';

interface QuotationOption {
  id: number;
  quotationNo: string;
  title: string;
  customerName: string;
  grandTotal: number;
  currency: number;
  status: number;
  approvalStatus: number;
}

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', item: QuotationOption): void;
}>();

const keyword = ref('');
const loading = ref(false);
const list = ref<QuotationOption[]>([]);
const selectedRow = ref<QuotationOption | null>(null);
const pagination = ref({ current: 1, pageSize: 10, total: 0 });

const currencySymbolMap: Record<number, string> = {
  1: '¥', 2: '$', 3: '€', 4: '£', 5: '¥', 6: 'HK$',
};

const approvalStatusLabelMap: Record<number, string> = {
  0: '草稿', 1: '草稿', 2: '审批中', 3: '已通过', 4: '已驳回',
};

const statusColorMap: Record<number, string> = {
  0: 'default', 1: 'default', 2: 'processing', 3: 'success', 4: 'error',
};

const columns = [
  { title: '报价编号', dataIndex: 'quotationNo', width: 150 },
  { title: '标题', dataIndex: 'title', width: 200 },
  { title: '客户名称', dataIndex: 'customerName', width: 140 },
  {
    title: '金额',
    key: 'grandTotal',
    width: 120,
    customRender: ({ record }: any) =>
      `${currencySymbolMap[record.currency] || '¥'} ${Number(record.grandTotal || 0).toLocaleString()}`,
  },
  {
    title: '审批状态',
    dataIndex: 'approvalStatus',
    width: 100,
    customRender: ({ record }: any) =>
      h(Tag, { color: statusColorMap[record.approvalStatus] || 'default' }, () =>
        approvalStatusLabelMap[record.approvalStatus] || '草稿',
      ),
  },
];

async function fetchData() {
  loading.value = true;
  try {
    const res = await getQuotationListApi({
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: keyword.value || undefined,
    });
    const data = res?.data ?? res ?? {};
    list.value = data.list || data.items || data.rows || [];
    pagination.value.total = data.total || data.count || 0;
  } finally {
    loading.value = false;
  }
}

function handlePageChange(page: number) {
  pagination.value.current = page;
  fetchData();
}

function handleSearch() {
  pagination.value.current = 1;
  fetchData();
}

function handleSelect() {
  if (!selectedRow.value) return;
  emit('select', selectedRow.value);
  emit('update:visible', false);
  selectedRow.value = null;
}

function handleCancel() {
  emit('update:visible', false);
  selectedRow.value = null;
}

onMounted(() => {
  if (props.visible) fetchData();
});
</script>

<template>
  <Modal
    :open="visible"
    title="选择报价单"
    width="800px"
    :destroy-on-close="true"
    @ok="handleSelect"
    @cancel="handleCancel"
    :ok-button-props="{ disabled: !selectedRow }"
  >
    <div class="mb-3 flex gap-2">
      <Input
        v-model:value="keyword"
        placeholder="搜索报价单号/标题"
        allow-clear
        style="width: 240px"
        @press-enter="handleSearch"
      />
      <Button type="primary" @click="handleSearch">搜索</Button>
    </div>
    <Table
      :columns="columns"
      :data-source="list"
      :loading="loading"
      :pagination="{
        current: pagination.current,
        pageSize: pagination.pageSize,
        total: pagination.total,
        showSizeChanger: false,
        onChange: handlePageChange,
      }"
      size="small"
      bordered
      row-key="id"
      :row-selection="{
        type: 'radio',
        selectedRowKeys: selectedRow ? [selectedRow.id] : [],
        onSelect: (record: any) => { selectedRow = record; },
      }"
      :scroll="{ y: 400 }"
    />
  </Modal>
</template>
