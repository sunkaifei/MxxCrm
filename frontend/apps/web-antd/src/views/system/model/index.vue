<script lang="ts" setup>
/**
 * 系统模型（管理入口重组）
 * 列出全部可建模业务模块，行操作：
 *  - 模板设置：表单/详情布局设计器（抽屉内嵌，支持选项卡/排序/半行整行/角色作用域）
 *  - 字段设置：跳转字段管理页并按该模块过滤
 */
import { computed, h, onMounted, reactive, ref } from 'vue';
import { useMediaQuery } from '@vueuse/core';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Button,
  Card,
  Drawer,
  message,
  Table,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useAccessStore } from '@vben/stores';

import { getFormLayoutApi } from '#/api/core/system/form-layout';
import { getFieldListApi } from '#/api/core/system/field';
import ModelDesigner from './designer.vue';
import FieldListPanel from './field-list.vue';

interface ModelRow {
  module: string;
  name: string;
  desc: string;
  icon: string;
  systemFields: number | '-';
  customFields: number | '-';
  formLayout: null | { version: number; roleKey?: string };
  detailLayout: null | { version: number; roleKey?: string };
  loading: boolean;
}


const MODULE_NAMES: Record<string, string> = {
  crm_customer: '客户模型',
  crm_lead: '线索模型',
  crm_contact: '联系人模型',
  crm_opportunity: '商机模型',
  sale_quotation: '报价模型',
  sale_order: '订单模型',
  crm_contract: '合同模型',
};
// 无对应权限码时跳过统计请求，避免 403 弹窗刷屏（按钮操作由后端权限码兜底）
const accessStore = useAccessStore();
// 移动端（<768px）：列表转卡片堆叠，抽屉默认全宽
const isMobile = useMediaQuery('(max-width: 767px)');
const canLayout = computed(() => accessStore.hasAccessCode('system:form-layout:list'));
const canField = computed(() => accessStore.hasAccessCode('system:field:list'));

const designerModuleName = computed(
  () => MODULE_NAMES[designerModule.value] ?? designerModule.value,
);
const fieldDrawerModuleName = computed(
  () => MODULE_NAMES[fieldDrawerModule.value] ?? fieldDrawerModule.value,
);

const MODULES: Omit<ModelRow, 'systemFields' | 'customFields' | 'formLayout' | 'detailLayout' | 'loading'>[] = [
  { module: 'crm_customer', name: '客户模型', desc: '客户档案与归属', icon: 'lucide:building-2' },
  { module: 'crm_lead', name: '线索模型', desc: '线索接入与转化', icon: 'lucide:lightbulb' },
  { module: 'crm_contact', name: '联系人模型', desc: '客户联系人', icon: 'lucide:users' },
  { module: 'crm_opportunity', name: '商机模型', desc: '销售机会与阶段推进', icon: 'lucide:target' },
  { module: 'sale_quotation', name: '报价模型', desc: '报价单与审批', icon: 'lucide:file-text' },
  { module: 'sale_order', name: '订单模型', desc: '销售订单与交付', icon: 'lucide:package' },
  { module: 'crm_contract', name: '合同模型', desc: '合同签订与履约', icon: 'lucide:stamp' },
];

const rows = reactive<ModelRow[]>(
  MODULES.map((m) => ({
    ...m,
    systemFields: '-',
    customFields: '-',
    formLayout: null,
    detailLayout: null,
    loading: true,
  })),
);

async function loadRow(row: ModelRow) {
  row.loading = true;
  try {
    const [allRes, customRes, formRes, detailRes] = await Promise.all([
      canField.value
        ? getFieldListApi({ module: row.module, page: 1, pageSize: 1 })
        : Promise.resolve(null),
      canField.value
        ? getFieldListApi({ module: row.module, page: 1, pageSize: 1, isSystem: 0 })
        : Promise.resolve(null),
      canLayout.value ? getFormLayoutApi(row.module, 1) : Promise.resolve(null),
      canLayout.value ? getFormLayoutApi(row.module, 2) : Promise.resolve(null),
    ]);
    const total = Number((allRes as any)?.total ?? 0) || 0;
    const custom = Number((customRes as any)?.total ?? 0) || 0;
    row.systemFields = canField.value ? total - custom : '-';
    row.customFields = canField.value ? custom : '-';
    row.formLayout = formRes
      ? { version: formRes.version, roleKey: formRes.roleKey }
      : null;
    row.detailLayout = detailRes
      ? { version: detailRes.version, roleKey: detailRes.roleKey }
      : null;
  } catch {
    /* 统计失败不阻塞操作 */
  } finally {
    row.loading = false;
  }
}

async function loadAll() {
  await Promise.all(rows.map((r) => loadRow(r)));
}
onMounted(loadAll);

// ===== 页面设置抽屉（表单设置 / 详情设置） =====
const designerVisible = ref(false);
const designerModule = ref('crm_opportunity');
const designerTab = ref<number>(1);
// 75% 默认宽，最大化切换 100%
const drawerWidth = ref('75%');
const maximized = ref(false);
function toggleMaximize() {
  maximized.value = !maximized.value;
  drawerWidth.value = maximized.value || isMobile.value ? '100%' : '75%';
}
function openDesigner(row: ModelRow, layoutType: number) {
  designerModule.value = row.module;
  designerTab.value = layoutType;
  maximized.value = false;
  drawerWidth.value = isMobile.value ? '100%' : '75%';
  designerVisible.value = true;
}
function onDesignerClosed() {
  loadAll();
}

// ===== 字段设置 =====
// ===== 字段设置抽屉 =====
const fieldDrawerVisible = ref(false);
const fieldDrawerModule = ref('crm_opportunity');
const fieldDrawerWidth = ref('75%');
const fieldMaximized = ref(false);
function toggleFieldMaximize() {
  fieldMaximized.value = !fieldMaximized.value;
  fieldDrawerWidth.value = fieldMaximized.value ? '100%' : '75%';
}
function openFields(row: ModelRow) {
  fieldDrawerModule.value = row.module;
  fieldMaximized.value = false;
  fieldDrawerWidth.value = isMobile.value ? '100%' : '75%';
  fieldDrawerVisible.value = true;
}

const columns = [
  {
    title: '系统模型',
    dataIndex: 'name',
    width: 220,
  },
  {
    title: '模块标识',
    dataIndex: 'module',
    width: 180,
  },
  {
    title: '字段（系统/自定义）',
    dataIndex: 'fields',
    width: 170,
  },
  {
    title: '表单布局',
    dataIndex: 'formLayout',
    width: 130,
  },
  {
    title: '详情布局',
    dataIndex: 'detailLayout',
    width: 130,
  },
  {
    title: '操作',
    dataIndex: 'action',
    width: 200,
  },
];
</script>

<template>
  <Page title="系统模型">
    <Card>
      <div class="mb-3 flex items-center gap-2 text-sm text-muted-foreground">
        <IconifyIcon icon="lucide:info" />
        每个模型的显示名、字段、表单与详情页布局均可在此自定义；系统内置字段仅可改名，不可删除。
      </div>
      <!-- 移动端：卡片堆叠 -->
      <div class="flex flex-col gap-4 md:hidden">
        <div
          v-for="row in rows"
          :key="row.module"
          class="rounded-xl border border-border/70 bg-card p-3 shadow-sm transition-all duration-200 hover:border-primary/50 hover:shadow-md"
        >
          <div class="flex items-center gap-2.5">
            <span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <IconifyIcon :icon="row.icon" class="text-lg" />
            </span>
            <span class="font-medium">{{ row.name }}</span>
            <Tag color="geekblue" class="ml-auto">{{ row.module }}</Tag>
          </div>
          <div class="mt-1 pl-[42px] text-xs text-muted-foreground">{{ row.desc }}</div>
          <div class="mt-2 flex flex-wrap items-center gap-2 pl-[42px] text-xs text-muted-foreground">
            <span>字段：{{ row.systemFields }} 系统 / {{ row.customFields }} 自定义</span>
            <Tag v-if="row.formLayout" color="green">表单 v{{ row.formLayout.version }}</Tag>
            <Tag v-else>表单默认</Tag>
            <Tag v-if="row.detailLayout" color="green">详情 v{{ row.detailLayout.version }}</Tag>
            <Tag v-else>详情默认</Tag>
          </div>
          <div class="mt-3 flex gap-2 border-t border-border/60 pt-3">
            <Button size="small" type="primary" ghost class="flex-1" @click="openDesigner(row, 1)">表单设置</Button>
            <Button size="small" type="primary" ghost class="flex-1" @click="openDesigner(row, 2)">详情设置</Button>
            <Button size="small" class="flex-1" @click="openFields(row)">字段设置</Button>
          </div>
        </div>
      </div>

      <!-- 桌面端：表格 -->
      <div class="hidden md:block">
      <Table
        :columns="columns"
        :data-source="rows"
        :loading="rows.some((r) => r.loading)"
        :pagination="false"
        row-key="module"
        size="middle"
        :scroll="{ x: 860 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'name'">
            <div class="flex items-center gap-2">
              <IconifyIcon :icon="record.icon" class="text-primary" />
              <span class="font-medium">{{ record.name }}</span>
            </div>
            <div class="pl-6 text-xs text-muted-foreground">{{ record.desc }}</div>
          </template>
          <template v-else-if="column.dataIndex === 'module'">
            <Tag color="geekblue">{{ record.module }}</Tag>
          </template>
          <template v-else-if="column.dataIndex === 'fields'">
            <span>{{ record.systemFields }} / {{ record.customFields }}</span>
          </template>
          <template v-else-if="column.dataIndex === 'formLayout'">
            <Tooltip title="未配置时按系统默认布局渲染">
              <Tag v-if="record.formLayout" color="green">
                已配置 v{{ record.formLayout.version }}
              </Tag>
              <Tag v-else>默认</Tag>
            </Tooltip>
          </template>
          <template v-else-if="column.dataIndex === 'detailLayout'">
            <Tooltip title="未配置时按系统默认布局渲染">
              <Tag v-if="record.detailLayout" color="green">
                已配置 v{{ record.detailLayout.version }}
              </Tag>
              <Tag v-else>默认</Tag>
            </Tooltip>
          </template>
          <template v-else-if="column.dataIndex === 'action'">
            <Button size="small" type="link" @click="openDesigner(record, 1)">
              <template #icon><IconifyIcon icon="lucide:layout-template" /></template>
              表单设置
            </Button>
            <Button size="small" type="link" @click="openDesigner(record, 2)">
              <template #icon><IconifyIcon icon="lucide:file-search" /></template>
              详情设置
            </Button>
            <Button size="small" type="link" @click="openFields(record)">
              <template #icon><IconifyIcon icon="lucide:list-plus" /></template>
              字段设置
            </Button>
          </template>
        </template>
      </Table>
      </div>
    </Card>

    <Drawer
      v-model:open="fieldDrawerVisible"
      :extra="
        h(Button, {
          type: 'text',
          size: 'small',
          onClick: toggleFieldMaximize,
          title: fieldMaximized ? '还原' : '最大化',
        }, () => h(IconifyIcon, { icon: fieldMaximized ? 'lucide:minimize-2' : 'lucide:maximize-2' }))
      "
      :width="fieldDrawerWidth"
      destroy-on-close
      placement="right"
      :title="`${fieldDrawerModuleName} · 字段设置`"
    >
      <FieldListPanel v-if="fieldDrawerVisible" :module="fieldDrawerModule" />
    </Drawer>

    <Drawer
      v-model:open="designerVisible"
      :extra="
        h(Button, {
          type: 'text',
          size: 'small',
          onClick: toggleMaximize,
          title: maximized ? '还原' : '最大化',
        }, () => h(IconifyIcon, { icon: maximized ? 'lucide:minimize-2' : 'lucide:maximize-2' }))
      "
      :width="drawerWidth"
      destroy-on-close
      placement="right"
      :title="`${designerModuleName} · ${designerTab === 2 ? '详情页设置' : '新建/修改页设置'}`"
      @closed="onDesignerClosed"
    >
      <ModelDesigner
        v-if="designerVisible"
        :key="designerModule + designerTab"
        :initial-layout-type="designerTab"
        :module="designerModule"
      />
    </Drawer>
  </Page>
</template>
