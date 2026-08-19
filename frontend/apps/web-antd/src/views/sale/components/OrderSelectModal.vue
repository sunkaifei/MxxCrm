<script lang="ts" setup>
import type { Key } from 'ant-design-vue/es/table/interface';

import { h, onMounted, ref, watch } from 'vue';

import { Button, Input, Modal, Table, Tag } from 'ant-design-vue';

import { getOrderListApi } from '#/api';

interface OrderOption {
  id: number;
  orderNo: string;
  title: string;
  customerName: string;
  totalAmount: number;
  currency: number;
  orderStatus: number;
  approvalStatus: number;
  receiverName?: string;
  receiverPhone?: string;
  shippingAddress?: string;
}

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', item: OrderOption): void;
}>();

const keyword = ref('');
const loading = ref(false);
const list = ref<OrderOption[]>([]);
const selectedRow = ref<null | OrderOption>(null);
const selectedKeys = ref<Key[]>([]);
const pagination = ref({ current: 1, pageSize: 10, total: 0 });

const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};

const orderStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待确认',
  3: '已确认',
  4: '备货中',
  5: '部分发货',
  6: '已发货',
  7: '已取消',
  8: '已交付',
  9: '已签收',
  10: '已完成',
};

const orderStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'blue',
  3: 'blue',
  4: 'orange',
  5: 'cyan',
  6: 'purple',
  7: 'red',
  8: 'cyan',
  9: 'green',
  10: 'blue',
};

const columns = [
  { title: '订单号', dataIndex: 'orderNo', width: 150 },
  { title: '订单标题', dataIndex: 'title', width: 200, ellipsis: true },
  { title: '客户名称', dataIndex: 'customerName', width: 140, ellipsis: true },
  {
    title: '订单金额',
    key: 'totalAmount',
    width: 120,
    customRender: ({ record }: any) =>
      `${currencySymbolMap[record.currency] || '¥'} ${Number(record.totalAmount || 0).toLocaleString()}`,
  },
  {
    title: '订单状态',
    dataIndex: 'orderStatus',
    width: 100,
    customRender: ({ record }: any) =>
      h(
        Tag,
        { color: orderStatusColorMap[record.orderStatus] || 'default' },
        () => orderStatusLabelMap[record.orderStatus] || '-',
      ),
  },
];

async function fetchData() {
  loading.value = true;
  try {
    const res: any = await getOrderListApi({
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: keyword.value || undefined,
    });
    const data = res?.data ?? res ?? {};
    const rawList = data.list || data.items || data.rows || [];
    // 仅显示「已确认(3) / 备货中(4) / 部分发货(5)」状态的订单（可发货订单）
    list.value = rawList.filter((o: any) =>
      [3, 4, 5].includes(Number(o.orderStatus)),
    );
    pagination.value.total = data.total || data.count || 0;
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  pagination.value.current = 1;
  fetchData();
}

function handleTableChange(p: any) {
  pagination.value.current = p.current;
  fetchData();
}

function handleRowClick(record: OrderOption) {
  selectedRow.value = record;
  selectedKeys.value = [record.id];
}

function handleConfirm() {
  if (!selectedRow.value) {
    return;
  }
  emit('select', selectedRow.value);
  handleClose();
}

function handleClose() {
  emit('update:visible', false);
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      keyword.value = '';
      selectedRow.value = null;
      selectedKeys.value = [];
      pagination.value.current = 1;
      fetchData();
    }
  },
  { immediate: true },
);

// 挂载时若已显示，也加载一次（兜底，防止 watch immediate 时机被错过）
onMounted(() => {
  if (props.visible) {
    fetchData();
  }
});

const rowSelection = {
  type: 'radio' as const,
  selectedRowKeys: selectedKeys.value,
  onChange: (keys: Key[], rows: OrderOption[]) => {
    selectedKeys.value = keys;
    selectedRow.value = rows[0] || null;
  },
  getCheckboxProps: (record: OrderOption) => ({
    onClick: () => handleRowClick(record),
  }),
};
</script>

<template>
  <Modal
    :open="props.visible"
    title="选择需要发货的订单"
    width="900px"
    :z-index="2100"
    :destroy-on-close="true"
    :mask-closable="false"
    ok-text="确认选择"
    cancel-text="取消"
    :ok-button-props="{ disabled: !selectedRow }"
    @ok="handleConfirm"
    @cancel="handleClose"
  >
    <div class="mb-3 flex items-center gap-2">
      <Input
        v-model:value="keyword"
        placeholder="搜索订单号 / 客户名称 / 订单标题"
        allow-clear
        style="width: 360px"
        @press-enter="handleSearch"
      />
      <Button type="primary" @click="handleSearch">搜索</Button>
      <span class="ml-2 text-xs text-gray-400">
        仅显示「已确认 / 备货中 / 部分发货」状态的可发货订单
      </span>
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
        showTotal: (total: number) => `共 ${total} 条`,
      }"
      :row-key="(record: any) => record.id"
      :row-selection="rowSelection"
      :custom-row="
        (record: any) => ({
          onClick: () => handleRowClick(record),
          style: { cursor: 'pointer' },
        })
      "
      size="small"
      :scroll="{ y: 400 }"
      @change="handleTableChange"
    />
  </Modal>
</template>
