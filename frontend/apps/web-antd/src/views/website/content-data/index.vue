<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';
import type { ContentModelFieldVO } from '#/api/core/website/content-model';

import { h, onMounted, reactive, ref } from 'vue';
import { useRoute } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucidePlus, LucideTrash2 } from '@vben/icons';

import { Button, message, Modal, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getContentModelFieldListApi, getContentModelListApi } from '#/api';
import { contentDataApi } from '#/api/core/website/content-data';
import { getUserListApi } from '#/api/core/system/user';

import ContentDataDrawer from './drawer.vue';

const route = useRoute();

/** 按钮权限码（与自动菜单下的 BUTTON 节点对应，按模型粒度授权） */
const perms = {
  add: ['content:data:add'],
  update: ['content:data:update'],
  remove: ['content:data:delete'],
};

/** 用户名映射（字段类型 11=用户：列表渲染 id → 姓名） */
const userNameMap = ref<Record<number, string>>({});

async function loadUserNameMap() {
  try {
    const res: any = await getUserListApi({ page: 1, pageSize: 999 });
    const list = res?.items || res?.rows || [];
    const map: Record<number, string> = {};
    for (const u of list) {
      const id = Number(u.id ?? u.userId);
      if (id) map[id] = u.realName || u.nickname || u.username || String(id);
    }
    userNameMap.value = map;
    rebuildSearchSchema();
  } catch {
    /* 权限不足等情况静默 */
  }
}

function formatUserCell(value: any): string {
  if (value === undefined || value === null || value === '') return '';
  return userNameMap.value[Number(value)] || String(value);
}

/** 模型编码（菜单路由 /website/content-data/{code}，或「管理内容」按钮跳入） */
const modelCode = ref<string>('');
const model = ref<any>({});
const fields = ref<ContentModelFieldVO[]>([]);

/** 搜索卡片：关键词 + 创建时间范围 + 「可搜索」字段（isSearchable）动态生成 */
const formOptions = reactive({
  collapsed: true,
  showCollapseButton: true,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '标题', allowClear: true },
    },
    {
      component: 'RangePicker',
      fieldName: 'createTimeRange',
      label: '创建时间',
      componentProps: {
        style: 'width:100%',
        valueFormat: 'YYYY-MM-DD',
      },
    },
  ],
} as any);

// 用户类型字段的下拉选项（由 userNameMap 生成）
function userSelectOptions() {
  return Object.entries(userNameMap.value).map(([id, name]) => ({
    label: name,
    value: Number(id),
  }));
}

/** 依据「可搜索」字段定义重建搜索表单（fields 加载后调用） */
function rebuildSearchSchema() {
  const schema: any[] = [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '标题', allowClear: true },
    },
    {
      component: 'RangePicker',
      fieldName: 'createTimeRange',
      label: '创建时间',
      componentProps: { style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
    },
  ];
  for (const f of fields.value) {
    if (f.isSearchable !== 1) continue;
    const name = f.fieldName;
    const label = f.fieldLabel || f.fieldName;
    switch (Number(f.fieldType)) {
      case 4: {
        schema.push({
          component: 'InputNumber',
          fieldName: `sf_${name}`,
          label,
          componentProps: { class: 'w-full' },
        });
        break;
      }
      case 5: {
        schema.push({
          component: 'RangePicker',
          fieldName: `df_${name}`,
          label,
          componentProps: { style: 'width:100%', valueFormat: 'YYYY-MM-DD' },
        });
        break;
      }
      case 6:
      case 7: {
        schema.push({
          component: 'Select',
          fieldName: `sf_${name}`,
          label,
          componentProps: {
            options: parseFieldOptions(f.fieldOptions),
            allowClear: true,
          },
        });
        break;
      }
      case 11: {
        schema.push({
          component: 'Select',
          fieldName: `sf_${name}`,
          label,
          componentProps: {
            options: userSelectOptions(),
            showSearch: true,
            optionFilterProp: 'label',
            allowClear: true,
          },
        });
        break;
      }
      default: {
        schema.push({
          component: 'Input',
          fieldName: `sf_${name}`,
          label,
          componentProps: { placeholder: `输入${label}关键词`, allowClear: true },
        });
      }
    }
  }
  // vxe grid 的 formOptions watch 合并方向会让旧 schema 胜出，
  // 这里用 formApi.setState 直接替换 schema（grid 挂载后可用）
  try {
    (gridApi as any)?.formApi?.setState?.({ schema });
  } catch {
    /* grid 未挂载时忽略 */
  }
}

function parseFieldOptions(raw?: string): { label: string; value: any }[] {
  if (!raw) return [];
  try {
    const arr = JSON.parse(raw);
    if (Array.isArray(arr)) {
      return arr.map((o: any) =>
        typeof o === 'object'
          ? { label: o.label ?? o.value, value: o.value }
          : { label: String(o), value: o },
      );
    }
  } catch {
    /* ignore */
  }
  return [];
}

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
        // 用户类型字段：id → 姓名显示
        formatter:
          Number(f.fieldType) === 11
            ? ({ cellValue }: any) => formatUserCell(cellValue)
            : undefined,
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
      query: async ({ page }, formValues) => {
        // 搜索卡片 → 后端参数：
        // - keywords 直接透传
        // - createTimeRange [起, 止] → createTimeFrom/createTimeTo
        // - sf_{field} 字段值筛选（文本模糊/数字用户等值）
        // - df_{field} [起, 止] → df_{field}_from/df_{field}_to 时间段
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
        };
        const kv: Record<string, any> = formValues || {};
        for (const [key, value] of Object.entries(kv)) {
          if (value === undefined || value === null) continue;
          if (key === 'keywords') {
            params.keywords = value;
          } else if (key === 'createTimeRange') {
            if (Array.isArray(value) && value.length === 2) {
              params.createTimeFrom = value[0];
              params.createTimeTo = value[1];
            }
          } else if (key.startsWith('df_')) {
            if (Array.isArray(value) && value.length === 2) {
              params[`${key}_from`] = value[0];
              params[`${key}_to`] = value[1];
            }
          } else if (key.startsWith('sf_')) {
            params[key] = value;
          }
        }
        return await contentDataApi.list(modelCode.value, params);
      },
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

function loadModelAndFields() {
  return getContentModelListApi({ page: 1, pageSize: 999 }).then(async (res: any) => {
    const list = res?.items || res?.rows || [];
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
    // 按「可搜索」字段重建搜索卡片
    rebuildSearchSchema();
  });
}

onMounted(async () => {
  // 用户名映射（字段类型 11=用户 的列渲染用），失败不影响页面
  loadUserNameMap();
  // 兼容三种入口：菜单字面路径 /content-data/{code}（无 :param，需从 path 解析）、query 旧链接、裸路径
  const fromPath = route.path.match(/\/content-data\/([A-Za-z][\w-]*)/)?.[1];
  modelCode.value =
    (route.params.code as string) ||
    (route.query.code as string) ||
    fromPath ||
    '';
  if (!modelCode.value) {
    // 从菜单直接进入（无 ?code=）：默认定位到第一个自定义模型，没有则回退第一个模型
    const res: any = await getContentModelListApi({ page: 1, pageSize: 999 });
    const all = res?.items || res?.rows || [];
    const first =
      all.find((m: any) => Number(m.isSystem) === 0) || all[0];
    if (!first?.modelCode) {
      message.warning('暂无内容模型，请进入「模型管理」新建模型');
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
    <Grid :table-title="model.modelName || modelCode">
      <template #toolbar-tools>
        <Button
          v-access:code="perms.add"
          type="primary"
          :icon="h(LucidePlus)"
          @click="handleAdd"
        >
          新增内容
        </Button>
        <Button
          v-access:code="perms.remove"
          danger
          :icon="h(LucideTrash2)"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 1" color="success">启用</Tag>
        <Tag v-else color="error">禁用</Tag>
      </template>

      <template #action="{ row }">
        <a
          v-access:code="perms.update"
          class="text-action-link"
          @click="() => handleEdit(row)"
        >
          编辑
        </a>
        <a
          v-access:code="perms.remove"
          class="text-action-link text-action-danger"
          @click="() => handleDelete(row)"
        >
          删除
        </a>
      </template>
    </Grid>

    <Drawer />
  </Page>
</template>

<style scoped>
/* 操作列纯文字链接（无图标、非表单按钮） */
.text-action-link {
  color: #2185eb;
  cursor: pointer;
  font-size: 13px;
  margin-right: 10px;
}

.text-action-link:hover {
  text-decoration: underline;
}

.text-action-danger {
  color: #e54545;
}
</style>
