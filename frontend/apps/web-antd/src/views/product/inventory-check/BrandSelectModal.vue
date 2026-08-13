<script lang="ts" setup>
import { ref, watch } from 'vue';
import { Modal, Input, Table, Tag, Button } from 'ant-design-vue';
import { getBrandListApi } from '#/api';

interface BrandItem {
  id: number;
  brandName: string;
  logo?: string;
  remark?: string;
  sort?: number;
}

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'select', brand: BrandItem): void;
}>();

const innerVisible = ref(false);
const keyword = ref('');
const loading = ref(false);
const list = ref<BrandItem[]>([]);

watch(
  () => props.visible,
  (val) => {
    innerVisible.value = val;
    if (val) {
      keyword.value = '';
      loadData();
    }
  },
);

function closeModal() {
  innerVisible.value = false;
  emit('update:visible', false);
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getBrandListApi({ pageSize: 999 });
    const raw = res?.list || res?.items || res || [];
    list.value = raw.map((b: any) => ({
      id: Number(b.id),
      brandName: b.brandName ?? b.name ?? '',
      logo: b.logo ?? '',
      remark: b.remark ?? '',
      sort: b.sort ?? 0,
    }));

    if (keyword.value) {
      const kw = keyword.value.toLowerCase();
      list.value = list.value.filter((b) =>
        b.brandName.toLowerCase().includes(kw),
      );
    }
  } catch (e) {
    console.error('[品牌选择] 加载失败:', e);
  } finally {
    loading.value = false;
  }
}

function onSearch() {
  loadData();
}

function onSelectClick(record: BrandItem) {
  closeModal();
  emit('select', record);
}

const columns = [
  { title: '品牌名称', dataIndex: 'brandName', ellipsis: true },
  { title: '备注', dataIndex: 'remark', ellipsis: true },
  { title: '操作', dataIndex: 'action', width: 70, fixed: 'right' as const },
];
</script>

<template>
  <Modal
    v-model:open="innerVisible"
    title="选择品牌"
    width="640px"
    :footer="null"
    @cancel="closeModal"
  >
    <div style="margin-bottom: 12px">
      <Input
        v-model:value="keyword"
        placeholder="搜索品牌名称"
        allow-clear
        @press-enter="onSearch"
      >
        <template #suffix>
          <span style="cursor: pointer; color: #999" @click="onSearch">搜索</span>
        </template>
      </Input>
    </div>

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
        <template v-if="column.dataIndex === 'action'">
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
