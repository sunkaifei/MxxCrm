<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';
import type { ContentModelFieldVO } from '#/api/core/website/content-model';

import { computed, h, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucidePlus, LucideTrash2 } from '@vben/icons';

import { Button, message, Modal, Select, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getContentModelFieldListApi, getContentModelListApi } from '#/api';
import { contentDataApi } from '#/api/core/website/content-data';

import ContentDataDrawer from './drawer.vue';

const route = useRoute();
const router = useRouter();

/** 模型编码（菜单路由 /website/content-data/{code}，或「管理内容」按钮跳入） */
const modelCode = ref<string>('');
const model = ref<any>({});
const fields = ref<ContentModelFieldVO[]>([]);

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '标题', allowClear: true },
    },
  ],
};

/** 依据字段定义构造列表列 */
function buildColumns(): any[] {
  const cols: any[] = [{ title: '序号', type: 'seq', width: 70 }];
  cols.push({ title: '标题', field: 'title', minWidth: 180 });
  for (const f of fields.value) {
    if (f.isListShow === 1) {
      cols.push({
        title: f.fieldLabel || f.fieldName,
        field: f.fieldName,
        minWidth: 120,
      });
    }
  }
  cols.push({ title: '排序', field: 'sort', width: 80 });
  cols.push({ title: '状态', field: 'status', width: 90, slots: { default: 'status' } });
  cols.push({ title: '创建时间', field: 'createTime', width: 170 });
  cols.push({
    title: '操作',
    field: 'action',
    fixed: 'right',
    width: 150,
    slots: { default: 'action' },
  });
  return cols;
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  pagerConfig: { pageSize: 20 },
  checkboxConfig: { checkMethod: () => true },
  stripe: true,
  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }, formValues) =>
        await contentDataApi.list(modelCode.value, {
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: formValues.keywords,
        }),
    },
  },
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContentDataDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

/** 全部模型清单（工作台顶部的模型切换下拉用） */
const modelList = ref<any[]>([]);

function loadModelAndFields() {
  return getContentModelListApi({ page: 1, pageSize: 999 }).then(async (res: any) => {
    const list = res?.items || res?.rows || [];
    modelList.value = list;
    model.value = list.find((m: any) => m.modelCode === modelCode.value) || {};
    if (model.value.id) {
      const fres: any = await getContentModelFieldListApi({
        modelId: model.value.id,
        page: 1,
        pageSize: 999,
      });
      fields.value = fres?.items || fres?.rows || [];
    } else {
      fields.value = [];
    }
  });
}

/** 有专属管理模块的模型编码（文章/产品走各自模块，不共用动态表） */
const DEDICATED_MODULE_CODES = new Set(['article', 'product']);

/** 工作台顶部下拉的可选模型：无专属模块的模型（含下载等内置）都可管理 */
const modelOptions = computed(() =>
  modelList.value.map((m: any) => ({
    label: Number(m.isSystem) === 1 ? `${m.modelName}（内置）` : m.modelName,
    value: m.modelCode,
    disabled: DEDICATED_MODULE_CODES.has(String(m.modelCode || '')),
  })),
);

/** 切换模型：刷新字段/列头/数据，并同步 URL 方便分享 */
async function handleSwitchModel(code: string) {
  modelCode.value = code;
  router.replace({ path: `/website/content-data/${code}` });
  await loadModelAndFields();
  (gridApi as any)?.setGridOptions?.({ columns: buildColumns() });
  gridApi.query();
}

/** 去模型管理：新建模型 / 给当前模型配字段（fields 深链直达字段弹窗） */
function goModelManage() {
  router.push('/website/content-model');
}
function goFieldManage() {
  router.push({
    path: '/website/content-model',
    query: { fields: modelCode.value },
  });
}

onMounted(async () => {
  // 兼容三种入口：菜单字面路径 /content-data/{code}（无 :param，需从 path 解析）、query 旧链接、裸路径
  const fromPath = route.path.match(/\/content-data\/([A-Za-z][\w-]*)/)?.[1];
  modelCode.value =
    (route.params.code as string) ||
    (route.query.code as string) ||
    fromPath ||
    '';
  if (!modelCode.value) {
    // 从菜单直接进入（无 ?code=）：默认定位到第一个自定义模型，没有则回退第一个模型
    await loadModelAndFields();
    const first =
      modelList.value.find((m: any) => Number(m.isSystem) === 0) ||
      modelList.value[0];
    if (!first?.modelCode) {
      message.warning('暂无内容模型，请点击右上角「模型管理」新建模型');
      return;
    }
    modelCode.value = first.modelCode;
    model.value = first;
  }
  try {
    await loadModelAndFields();
  } finally {
    (gridApi as any)?.setGridOptions?.({ columns: buildColumns() });
    gridApi.query();
  }
});

function handleAdd() {
  drawerApi.setData({
    create: true,
    code: modelCode.value,
    model: model.value,
    fields: fields.value,
  });
  drawerApi.open();
}

function handleEdit(row: any) {
  drawerApi.setData({
    create: false,
    code: modelCode.value,
    model: model.value,
    fields: fields.value,
    row,
  });
  drawerApi.open();
}

function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除「${row.title || row.id}」吗？`,
    okType: 'danger',
    onOk: async () => {
      await contentDataApi.delete(modelCode.value, [row.id]);
      message.success('删除成功');
      gridApi.query();
    },
  });
}

function handleBatchDelete() {
  const records = (gridApi as any)?.getCheckboxRecords?.() || [];
  if (records.length === 0) {
    message.warning('请先勾选要删除的记录');
    return;
  }
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除选中的 ${records.length} 条内容吗？`,
    okType: 'danger',
    onOk: async () => {
      await contentDataApi.delete(
        modelCode.value,
        records.map((r: any) => r.id),
      );
      message.success('删除成功');
      gridApi.query();
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="`内容管理 - ${model.modelName || modelCode}`">
      <template #toolbar-tools>
        <Select
          :value="modelCode"
          :options="modelOptions"
          show-search
          option-filter-prop="label"
          style="width: 180px"
          placeholder="切换模型"
          @change="(v: any) => handleSwitchModel(String(v))"
        />
        <Button @click="goFieldManage">字段管理</Button>
        <Button @click="goModelManage">新建模型</Button>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增内容
        </Button>
        <Button danger :icon="h(LucideTrash2)" @click="handleBatchDelete">
          批量删除
        </Button>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">启用</Tag>
        <Tag v-else color="error">禁用</Tag>
      </template>

      <template #action="{ row }">
        <Button type="primary" link @click="() => handleEdit(row)">编辑</Button>
        <Button type="primary" link danger @click="() => handleDelete(row)">
          删除
        </Button>
      </template>
    </Grid>

    <Drawer />
  </Page>
</template>
