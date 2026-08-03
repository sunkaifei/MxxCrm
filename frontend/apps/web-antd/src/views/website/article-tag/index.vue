<script lang="ts" setup>
import { ref, h, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import {
  LucidePlus,
  LucideSearch,
  LucideTrash2,
  LucideEdit,
} from '@vben/icons';
import {
  Button,
  Input,
  Table,
  Modal,
  message,
  Tag,
  Tooltip,
  Popconfirm,
  Form,
  Input as AInput,
  InputNumber,
  Switch,
} from 'ant-design-vue';
import { articleTagApi } from '#/api';
import type { ArticleTagVO } from '#/api/core/website/article-tag';

const accessStore = useAccessStore();

// --- State ---
const dataSource = ref<ArticleTagVO[]>([]);
const loading = ref(false);
const pagination = ref({ current: 1, pageSize: 20, total: 0 });
const keyword = ref('');

// Drawer state
const drawerVisible = ref(false);
const editingRecord = ref<ArticleTagVO | null>(null);
const saving = ref(false);

// Form
const formState = ref<{
  name?: string;
  slug?: string;
  color?: string;
  sort?: number;
  status?: number;
}>({
  name: '',
  slug: '',
  color: '#1677ff',
  sort: 0,
  status: 1,
});

// --- Load ---
async function loadData() {
  loading.value = true;
  try {
    const res: any = await articleTagApi.list({
      page: pagination.value.current,
      pageSize: pagination.value.pageSize,
      keywords: keyword.value || undefined,
    });
    dataSource.value = res?.items || [];
    pagination.value.total = res?.total || 0;
  } catch {
    dataSource.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => loadData());

// --- Search ---
function handleSearch() {
  pagination.value.current = 1;
  loadData();
}

// --- Create / Edit ---
function openCreate() {
  editingRecord.value = null;
  formState.value = { name: '', slug: '', color: '#1677ff', sort: 0, status: 1 };
  drawerVisible.value = true;
}

function openEdit(record: ArticleTagVO) {
  editingRecord.value = record;
  formState.value = {
    name: record.name || '',
    slug: record.slug || '',
    color: record.color || '#1677ff',
    sort: record.sort || 0,
    status: record.status ?? 1,
  };
  drawerVisible.value = true;
}

async function handleSave() {
  saving.value = true;
  try {
    if (editingRecord.value) {
      await articleTagApi.update(editingRecord.value.id, formState.value);
      message.success('修改成功');
    } else {
      await articleTagApi.add(formState.value);
      message.success('添加成功');
    }
    drawerVisible.value = false;
    loadData();
  } catch (err: any) {
    message.error(err?.message || '操作失败');
  } finally {
    saving.value = false;
  }
}

// --- Delete ---
async function handleDelete(record: ArticleTagVO) {
  try {
    await articleTagApi.delete([record.id]);
    message.success('删除成功');
    loadData();
  } catch {
    message.error('删除失败');
  }
}

// --- Columns ---
const columns = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  { title: '标签名称', dataIndex: 'name', width: 200 },
  { title: '别名', dataIndex: 'slug', width: 160 },
  {
    title: '颜色',
    dataIndex: 'color',
    width: 100,
    customRender: ({ text }: { text: string }) =>
      text ? h(Tag, { color: text }, () => text) : '-',
  },
  { title: '排序', dataIndex: 'sort', width: 80 },
  {
    title: '文章数',
    dataIndex: 'articleCount',
    width: 80,
  },
  {
    title: '状态',
    dataIndex: 'status',
    width: 80,
    customRender: ({ text }: { text: number }) =>
      text === 1
        ? h(Tag, { color: 'success' }, () => '启用')
        : h(Tag, { color: 'default' }, () => '禁用'),
  },
  { title: '创建时间', dataIndex: 'createTime', width: 180 },
  {
    title: '操作',
    key: 'action',
    width: 140,
    fixed: 'right' as const,
    customRender: ({ record }: { record: ArticleTagVO }) =>
      h('div', { style: 'white-space:nowrap' }, [
        h(Tooltip, { title: '编辑' }, () =>
          h(Button, {
            type: 'link',
            size: 'small',
            onClick: () => openEdit(record),
          }, () => h(LucideEdit, { style: 'font-size:14px' })),
        ),
        h(Popconfirm, { title: '确定删除此标签？', onConfirm: () => handleDelete(record) }, () =>
          h(Tooltip, { title: '删除' }, () =>
            h(Button, { type: 'link', size: 'small', danger: true }, () => h(LucideTrash2, { style: 'font-size:14px' })),
          ),
        ),
      ]),
  },
];
</script>

<template>
  <Page auto-content-height>
    <div class="article-tag-page">
      <!-- Toolbar -->
      <div class="toolbar">
        <div class="toolbar-left">
          <Input
            v-model:value="keyword"
            placeholder="搜索标签名称…"
            allow-clear
            style="width: 260px"
            @pressEnter="handleSearch"
          >
            <template #prefix>
              <component :is="LucideSearch" style="color: #bfbfbf; font-size: 14px" />
            </template>
          </Input>
          <Button type="primary" @click="handleSearch">搜索</Button>
        </div>
        <div class="toolbar-right">
          <Button
            v-if="accessStore.hasAccessCode('website:article:tag:add')"
            type="primary"
            @click="openCreate"
          >
            <template #icon><component :is="LucidePlus" /></template>
            新增标签
          </Button>
        </div>
      </div>

      <!-- Table -->
      <div class="table-wrap">
        <Table
          :dataSource="dataSource"
          :columns="columns"
          :loading="loading"
          :pagination="{
            current: pagination.current,
            pageSize: pagination.pageSize,
            total: pagination.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total: number) => `共 ${total} 项`,
            onChange: (page: number, pageSize: number) => {
              pagination.current = page;
              pagination.pageSize = pageSize;
              loadData();
            },
          }"
          rowKey="id"
          size="middle"
          bordered
        />
      </div>
    </div>

    <!-- Create/Edit Drawer -->
    <Modal
      v-model:open="drawerVisible"
      :title="editingRecord ? '编辑标签' : '新增标签'"
      :confirm-loading="saving"
      ok-text="保存"
      cancel-text="取消"
      width="480px"
      @ok="handleSave"
      @cancel="drawerVisible = false"
    >
      <Form
        :model="formState"
        layout="vertical"
      >
        <Form.Item label="标签名称" required>
          <AInput
            v-model:value="formState.name"
            placeholder="请输入标签名称"
          />
        </Form.Item>
        <Form.Item label="别名（slug）">
          <AInput
            v-model:value="formState.slug"
            placeholder="用于URL中的友好名称"
          />
        </Form.Item>
        <Form.Item label="标签颜色">
          <div style="display: flex; align-items: center; gap: 8px">
            <AInput
              v-model:value="formState.color"
              placeholder="#1677ff"
              style="width: 120px"
            />
            <div
              :style="{
                width: 24,
                height: 24,
                borderRadius: 4,
                backgroundColor: formState.color,
                border: '1px solid #d9d9d9',
              }"
            />
          </div>
        </Form.Item>
        <Form.Item label="排序">
          <InputNumber
            v-model:value="formState.sort"
            :min="0"
            :max="9999"
          />
        </Form.Item>
        <Form.Item label="状态">
          <Switch
            :checked="formState.status === 1"
            @change="(val: any) => { formState.status = val ? 1 : 0 }"
          />
        </Form.Item>
      </Form>
    </Modal>
  </Page>
</template>

<style scoped>
.article-tag-page {
  padding: 0;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  gap: 12px;
  flex-wrap: wrap;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.table-wrap {
  background: #fff;
  border-radius: 8px;
  padding: 16px;
}
</style>