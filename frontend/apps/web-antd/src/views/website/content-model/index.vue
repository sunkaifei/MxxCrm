<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue';

import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucidePlus } from '@vben/icons';

import { Button, Drawer as AntDrawer, message, Modal, Popconfirm, Table, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  copyContentModelApi,
  deleteContentModelApi,
  deleteContentModelFieldApi,
  getContentModelFieldListApi,
  getContentModelListApi,
} from '#/api';

import ContentModelDrawer from './drawer.vue';
import ContentModelFieldDrawer from './field-drawer.vue';
import LayoutDesigner from './layout-designer.vue';

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
  11: '用户',
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
      width: 340,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const router = useRouter();
const route = useRoute();

/** 有专属管理模块的模型编码（文章/产品走各自模块的数据表，不共用动态表） */
const DEDICATED_MODULE_CODES = new Set(['article', 'product']);
function hasDedicatedModule(row: any) {
  return DEDICATED_MODULE_CODES.has(String(row?.modelCode || ''));
}

/** 深链支持：/website/content-model?fields={code} 直达该模型的字段管理（内容管理页「字段管理」按钮跳入） */
function openFieldManageByCode(code: string) {
  getContentModelListApi({ page: 1, pageSize: 999 }).then((res: any) => {
    const list = res?.items || res?.rows || [];
    const row = list.find((m: any) => m.modelCode === code);
    if (row) {
      handleFieldManage(row);
    }
  });
}
onMounted(() => {
  const fieldsCode = route.query.fields as string;
  if (fieldsCode) {
    openFieldManageByCode(fieldsCode);
  }
});

/** T-P1.2：跳转通用内容页管理该模型的动态内容（路径参数形式，query 会破坏路由匹配） */
function handleManageContent(row: any) {
  router.push({ path: `/website/content-data/${row.modelCode}` });
}

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

/** 12-E：复制模型（含字段定义），新编码 = 原编码_copy */
async function handleCopy(row: any) {
  try {
    await copyContentModelApi(row.id);
    message.success(`已复制为「${row.modelName}副本」`);
    gridApi.query();
  } catch {
    // 错误提示由请求拦截器统一处理
  }
}

// ===== 字段管理 =====
const fieldModalVisible = ref(false);
const currentModel = ref<any>(null);
const fieldList = ref<any[]>([]);
const fieldLoading = ref(false);
// 表单布局设计器
const layoutDesignerOpen = ref(false);

function handleOpenLayoutDesigner() {
  layoutDesignerOpen.value = true;
}

async function handleLayoutSaved() {
  // 布局保存在模型行上：拉最新行覆盖 currentModel，并刷新列表
  const res: any = await getContentModelListApi({ page: 1, pageSize: 999 });
  const list = res?.items || res?.rows || [];
  const fresh = list.find((m: any) => m.id === currentModel.value?.id);
  if (fresh) currentModel.value = fresh;
  message.success('布局已更新');
  gridApi.query();
}

async function loadFieldList(modelId: number) {
  fieldLoading.value = true;
  try {
    const res: any = await getContentModelFieldListApi({
      modelId,
      page: 1,
      pageSize: 9999,
    });
    fieldList.value = res?.items || res?.rows || [];
  } catch {
    fieldList.value = [];
  } finally {
    fieldLoading.value = false;
  }
}

function handleFieldManage(row: any) {
  currentModel.value = row;
  fieldFullscreen.value = false;
  fieldModalVisible.value = true;
  loadFieldList(row.id);
}

// 字段管理抽屉最大化/还原
const fieldFullscreen = ref(false);
function toggleFieldFullscreen() {
  fieldFullscreen.value = !fieldFullscreen.value;
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
  fieldDrawerApi.setData({
    create: true,
    modelId: currentModel.value.id,
    // T-P1.4：前端唯一性校验依据（同模型内 field_name 不可重复）
    existingFields: fieldList.value.map((f: any) => f.fieldName),
  });
  fieldDrawerApi.open();
}

function handleFieldEdit(row: any) {
  fieldDrawerApi.setData({
    create: false,
    modelId: currentModel.value.id,
    row,
    existingFields: fieldList.value.map((f: any) => f.fieldName),
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
    <Grid table-title="模型管理">
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
        <Button type="link" size="small" @click="() => handleEdit(row)">
          编辑
        </Button>
        <Button type="link" size="small" @click="() => handleFieldManage(row)">
          字段管理
        </Button>
        <Button
          type="link"
          size="small"
          :disabled="hasDedicatedModule(row) || row.status !== 1"
          :title="
            hasDedicatedModule(row)
              ? '该模型有专属管理模块（文章/产品），不走通用内容管理'
              : ''
          "
          @click="() => handleManageContent(row)"
        >
          管理内容
        </Button>
        <Button
          type="link"
          size="small"
          :disabled="row.isSystem === 1"
          title="复制模型定义与全部字段（不复制内容数据）"
          @click="() => handleCopy(row)"
        >
          复制
        </Button>
        <Popconfirm
          v-if="row.isSystem !== 1"
          title="确定要删除该模型吗？"
          @confirm="handleDelete(row)"
        >
          <Button type="link" danger size="small">删除</Button>
        </Popconfirm>
        <Button v-else type="link" size="small" disabled>删除</Button>
      </template>
    </Grid>

    <Drawer />

    <!-- 字段管理抽屉 -->
    <AntDrawer
      v-model:open="fieldModalVisible"
      :title="`字段管理 - ${currentModel?.modelName || ''}`"
      :width="fieldFullscreen ? '100%' : '75%'"
      :footer="null"
    >
      <template #extra>
        <Button
          type="link"
          size="small"
          class="mr-2"
          @click="handleOpenLayoutDesigner"
        >
          布局设计
        </Button>
        <Button type="link" size="small" @click="toggleFieldFullscreen">
          {{ fieldFullscreen ? '还原' : '最大化' }}
        </Button>
      </template>
      <div class="mb-4">
        <Button type="primary" :icon="h(LucidePlus)" @click="handleFieldAdd">
          新增字段
        </Button>
        <span class="ml-3 text-xs" style="color: hsl(var(--foreground) / 50%)">
          提示：id、状态、创建时间等系统字段在创建模型时已自动生成，无需手动添加
        </span>
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
            <Button type="link" size="small" @click="handleFieldEdit(record)">
              编辑
            </Button>
            <Popconfirm
              title="确定要删除该字段吗？"
              @confirm="handleFieldDelete(record)"
            >
              <Button type="link" danger size="small">删除</Button>
            </Popconfirm>
          </template>
        </template>
      </Table>
    </AntDrawer>

    <!-- 表单布局设计器（选项卡/列数/字段摆放） -->
    <AntDrawer
      v-model:open="layoutDesignerOpen"
      :title="`布局设计 - ${currentModel?.modelName || ''}`"
      width="60%"
      :footer="null"
      :z-index="2600"
    >
      <LayoutDesigner
        v-model:open="layoutDesignerOpen"
        :model="currentModel"
        :fields="fieldList"
        @saved="handleLayoutSaved"
      />
    </AntDrawer>

    <FieldDrawer />
  </Page>
</template>
