<script lang="ts" setup>
import { ref, watch } from 'vue';
import { Modal, Input, Table, Tag, Button } from 'ant-design-vue';
import { getWarehouseListApi } from '#/api/core/product/warehouse';

interface WarehouseItem {
  id: number;
  warehouseName: string;
  code?: string;
  address?: string;
  managerName?: string;
  status?: number;
}

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', warehouse: WarehouseItem): void;
}>();

const keyword = ref('');
const loading = ref(false);
const list = ref<WarehouseItem[]>([]);

const columns = [
  { title: '仓库名称', dataIndex: 'warehouseName', ellipsis: true },
  { title: '仓库编码', dataIndex: 'code', width: 120, ellipsis: true },
  { title: '地址', dataIndex: 'address', ellipsis: true },
  { title: '负责人', dataIndex: 'managerName', width: 90 },
  { title: '状态', dataIndex: 'status', width: 80 },
  { title: '操作', dataIndex: 'action', width: 70, fixed: 'right' as const },
];

watch(
  () => props.visible,
  (val) => {
    if (val) {
      keyword.value = '';
      loadData();
    }
  },
);

async function loadData() {
  loading.value = true;
  try {
    const resp: any = await getWarehouseListApi({ page: 1, pageSize: 999 });
    const raw = resp?.data ?? resp;
    const items = raw?.items ?? raw?.list ?? (Array.isArray(raw) ? raw : []);
    list.value = items.map((w: any) => ({
      id: Number(w.id),
      warehouseName: w.warehouseName ?? w.name ?? '',
      code: w.warehouseCode ?? w.code ?? '',
      address: w.address ?? '',
      managerName: w.managerName ?? w.contactPerson ?? '',
      status: w.status ?? 1,
    }));

    if (keyword.value) {
      const kw = keyword.value.toLowerCase();
      list.value = list.value.filter(
        (w) =>
          w.warehouseName.toLowerCase().includes(kw) ||
          w.code.toLowerCase().includes(kw),
      );
    }
  } catch (e) {
    console.error('[仓库选择] 加载失败:', e);
  } finally {
    loading.value = false;
  }
}

function onSearch() {
  loadData();
}

// 点击"选择"按钮直接选中并关闭
function onSelectClick(record: WarehouseItem) {
  emit('select', record);
  emit('update:visible', false);
}
</script>

<template>
  <Modal
    :open="visible"
    title="选择仓库"
    width="760px"
    :footer="null"
    @cancel="emit('update:visible', false)"
  >
    <!-- 搜索框 -->
    <div style="margin-bottom: 12px">
      <Input
        v-model:value="keyword"
        placeholder="搜索仓库名称或编码"
        allow-clear
        @press-enter="onSearch"
      >
        <template #suffix>
          <span style="cursor: pointer; color: #999" @click="onSearch">搜索</span>
        </template>
      </Input>
    </div>

    <!-- 仓库列表 -->
    <Table
      :columns="columns"
      :data-source="list"
      :loading="loading"
      :row-key="(record) => record.id"
      :pagination="false"
      size="small"
      :scroll="{ y: 360 }"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.dataIndex === 'status'">
          <Tag :color="record.status === 1 ? 'success' : 'default'" :bordered="false">
            {{ record.status === 1 ? '启用' : '停用' }}
          </Tag>
        </template>
        <template v-else-if="column.dataIndex === 'action'">
          <Button
            type="link"
            size="small"
            @click="onSelectClick(record)"
          >
            选择
          </Button>
        </template>
      </template>
    </Table>
  </Modal>
</template>
