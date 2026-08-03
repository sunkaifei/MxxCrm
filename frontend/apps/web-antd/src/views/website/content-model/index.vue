<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import {
  LucideFilePenLine,
  LucideList,
  LucidePlus,
  LucideTrash2,
} from '@vben/icons';
import { Button, Modal, Popconfirm, Table, Tag, message } from 'ant-design-vue';
import type { TableColumnsType } from 'ant-design-vue';
import ContentModelDrawer from './drawer.vue';
import ContentModelFieldDrawer from './field-drawer.vue';
import {
  deleteContentModelApi,
  deleteContentModelFieldApi,
  getContentModelFieldListApi,
  getContentModelListApi,
} from '#/api';

const fieldTypeMap: Record<number, string> = {
  1: '单行文本',
  2: '多行文本',
  3: '富文本',
  4: '数字',
  5: '日期',
  6: '下拉选择',
  7: '单选',
  8: '多选',
  9: '图片',
  10: '文件',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: {
        placeholder: '模型名称',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '启用', value: 1 },
          { label: '禁用', value: 0 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getContentModelListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
          status: formValues.status || undefined,
        });
      },
    },
  },

  columns: [
    {
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: '模型编码',
      field: 'modelCode',
      width: 140,
    },
    {
      title: '模型名称',
      field: 'modelName',
      width: 140,
    },
    {
      title: '图标',
      field: 'modelIcon',
      width: 120,
    },
    {
      title: '描述',
      field: 'description',
      minWidth: 200,
    },
    {
      title: '排序',
      field: 'sort',
      width: 80,
    },
    {
      title: '系统内置',
      field: 'isSystem',
      slots: { default: 'isSystem' },
      width: 100,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 80,
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 170,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 240,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContentModelDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function handleAdd() {
  drawerApi.setData({ create: true });
  drawerApi.open();
}

function handleEdit(row: any) {
  drawerApi.setData({ create: false, row });
  drawerApi.open();
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除模型"${row.modelName}"吗？`,
    okType: 'danger',
    onOk: async () => {
      await deleteContentModelApi([row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}

// ===== 字段管理 =====
const fieldModalVisible = ref(false);
const currentModel = ref<any>(null);
const fieldList = ref<any[]>([]);
const fieldLoading = ref(false);

async function loadFieldList(modelId: number) {
  fieldLoading.value = true;
  try {
    const res: any = await getContentModelFieldListApi({
      modelId,
      page: 1,
      pageSize: 9999,
    });
    fieldList.value = res?.rows || res?.list || res?.data?.rows || [];
  } catch {
    fieldList.value = [];
  } finally {
    fieldLoading.value = false;
  }
}

function handleFieldManage(row: any) {
  currentModel.value = row;
  fieldModalVisible.value = true;
  loadFieldList(row.id);
}

const [FieldDrawer, fieldDrawerApi] = useVbenDrawer({
  connectedComponent: ContentModelFieldDrawer,
  onClosed() {
    const data = fieldDrawerApi.getData();
    if (data && data.needRefresh && currentModel.value) {
      loadFieldList(currentModel.value.id);
    }
  },
});

function handleFieldAdd() {
  fieldDrawerApi.setData({ create: true, modelId: currentModel.value.id });
  fieldDrawerApi.open();
}

function handleFieldEdit(row: any) {
  fieldDrawerApi.setData({
    create: false,
    modelId: currentModel.value.id,
    row,
  });
  fieldDrawerApi.open();
}

async function handleFieldDelete(row: any) {
  await deleteContentModelFieldApi([row.id]);
  message.success('删除成功');
  if (currentModel.value) {
    loadFieldList(currentModel.value.id);
  }
}

const fieldColumns: TableColumnsType = [
  {
    title: '字段名称',
    dataIndex: 'fieldName',
    width: 120,
  },
  {
    title: '字段标签',
    dataIndex: 'fieldLabel',
    width: 120,
  },
  {
    title: '类型',
    dataIndex: 'fieldType',
    width: 100,
    key: 'fieldType',
  },
  {
    title: '必填',
    dataIndex: 'isRequired',
    width: 70,
    key: 'isRequired',
  },
  {
    title: '排序',
    dataIndex: 'sort',
    width: 70,
  },
  {
    title: '状态',
    dataIndex: 'status',
    width: 80,
    key: 'status',
  },
  {
    title: '操作',
    key: 'action',
    width: 150,
    fixed: 'right',
  },
];
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="内容模型管理">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增模型
        </Button>
      </template>

      <template #isSystem="{ row }">
        <Tag v-if="row.isSystem === 1" color="purple">系统内置</Tag>
        <Tag v-else color="default">自定义</Tag>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">启用</Tag>
        <Tag v-else color="error">禁用</Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        >
          编辑
        </Button>
        <Button
          type="primary"
          link
          :icon="h(LucideList)"
          @click="() => handleFieldManage(row)"
        >
          字段管理
        </Button>
        <Popconfirm
          v-if="row.isSystem !== 1"
          title="确定要删除该模型吗？"
          @confirm="handleDelete(row)"
        >
          <Button type="primary" link danger :icon="h(LucideTrash2)">
            删除
          </Button>
        </Popconfirm>
        <Button v-else type="primary" link disabled>删除</Button>
      </template>
    </Grid>

    <Drawer />

    <!-- 字段管理弹窗 -->
    <Modal
      v-model:open="fieldModalVisible"
      :title="`字段管理 - ${currentModel?.modelName || ''}`"
      width="900px"
      :footer="null"
    >
      <div class="mb-4">
        <Button type="primary" :icon="h(LucidePlus)" @click="handleFieldAdd">
          新增字段
        </Button>
      </div>
      <Table
        :columns="fieldColumns"
        :data-source="fieldList"
        :loading="fieldLoading"
        row-key="id"
        :pagination="false"
        size="small"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'fieldType'">
            <Tag>{{ fieldTypeMap[record.fieldType] || '未知' }}</Tag>
          </template>
          <template v-else-if="column.key === 'isRequired'">
            <Tag :color="record.isRequired === 1 ? 'red' : 'default'">
              {{ record.isRequired === 1 ? '是' : '否' }}
            </Tag>
          </template>
          <template v-else-if="column.key === 'status'">
            <Tag :color="record.status === 1 ? 'success' : 'error'">
              {{ record.status === 1 ? '启用' : '禁用' }}
            </Tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <Button type="primary" link @click="handleFieldEdit(record)">
              编辑
            </Button>
            <Popconfirm
              title="确定要删除该字段吗？"
              @confirm="handleFieldDelete(record)"
            >
              <Button type="primary" link danger>删除</Button>
            </Popconfirm>
          </template>
        </template>
      </Table>
    </Modal>

    <FieldDrawer />
  </Page>
</template>
