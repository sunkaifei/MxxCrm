<script lang="ts" setup>
import { h, ref, computed } from 'vue';
import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';
import { Button, Card, Checkbox, message, Modal, Popconfirm, Switch, Tag, Tooltip } from 'ant-design-vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';

// 段位类型映射
const SEGMENT_LABELS: Record<string, string> = {
  company: '公司简称',
  biz_type: '业务类型',
  year: '年份',
  dept: '部门',
  seq: '流水号',
  version: '版本号',
  fixed: '固定文本',
  date: '日期',
};

// 根据 segments 配置生成预览字符串
function buildPreviewText(row: any): string {
  const segments: Array<{ type: string; value?: string; format?: string }> = row.segments ?? [];
  if (!segments.length) return '-';
  const separator = row.separator || '-';
  return segments.map((seg) => {
    switch (seg.type) {
      case 'company': return seg.value || row.companyAbbr || '公司';
      case 'biz_type': return seg.value || row.bizTypeCode || 'XX';
      case 'year': return '2026';
      case 'dept': return seg.value || 'DEPT';
      case 'seq': {
        const len = row.seqLength || 4;
        return String(1).padStart(len, '0');
      }
      case 'version': return 'V1';
      case 'fixed': return seg.value || '';
      case 'date': return '202601';
      default: return '';
    }
  }).join(separator);
}
import {
  batchRegenerateApi,
  deleteCodeRuleApi,
  getCodeRuleListApi,
  toggleCodeRuleEnabledApi,
} from '#/api';
import { $t } from '#/locales';
import CodeRuleDrawer from './drawer.vue';

const accessStore = useAccessStore();

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'moduleCode',
      label: $t('page.company.codeRule.moduleCode'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'enabled',
      label: $t('ui.table.status'),
      componentProps: {
        options: [
          { label: '启用', value: 1 },
          { label: '停用', value: 0 },
        ],
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  minHeight: 150,
  pagerConfig: {},
  cellConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { trigger: 'row' },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getCodeRuleListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          moduleCode: formValues.moduleCode,
          enabled: formValues.enabled,
        });
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: $t('page.company.codeRule.moduleCode'), field: 'moduleCode', width: 130 },
    { title: $t('page.company.codeRule.moduleName'), field: 'moduleName', width: 130 },
    { title: $t('page.company.codeRule.ruleName'), field: 'ruleName', width: 110  },
    { title: $t('page.company.codeRule.bizTypeCode'), field: 'bizTypeCode', width: 110 },
    { title: $t('page.company.codeRule.separator'), field: 'separator', width: 80 },
    {
      title: $t('page.company.codeRule.preview'),
      field: 'preview',
      slots: { default: 'preview' },
      width: 220,
    },
    {
      title: $t('ui.table.status'),
      field: 'enabled',
      slots: { default: 'enabled' },
      width: 100,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
      width: 160,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 160,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleEnabledChanged(row: any, checked: boolean) {
  row.pending = true;
  const newEnabled = checked ? 1 : 0;
  try {
    await toggleCodeRuleEnabledApi(Number(row.id), newEnabled);
    row.enabled = newEnabled;
    message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: CodeRuleDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteCodeRuleApi(Number(row.id));
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

const batchRegenerating = ref(false);
const yearModalVisible = ref(false);
const selectedYears = ref<number[]>([]);
const pendingModuleCodes = ref<string[]>([]);

// 可选年份（2020 ~ 当前年份）
const availableYears = computed(() => {
  const current = new Date().getFullYear();
  const years: number[] = [];
  for (let y = current; y >= 2020; y--) {
    years.push(y);
  }
  return years;
});

async function handleBatchRegenerate() {
  const rows = gridApi.grid?.getCheckboxRecords() ?? [];
  if (!rows.length) {
    message.warning('请先勾选要更新编号的模块');
    return;
  }

  const moduleCodes = rows.map((r: any) => r.moduleCode).filter(Boolean) as string[];
  const hasCustomer = moduleCodes.includes('customer');

  if (hasCustomer) {
    // 客户模块：弹出年份选择对话框
    selectedYears.value = [new Date().getFullYear()]; // 默认勾选当前年份
    pendingModuleCodes.value = moduleCodes;
    yearModalVisible.value = true;
    return;
  }

  // 非客户模块：直接确认
  try {
    await new Promise<void>((resolve, reject) => {
      Modal.confirm({
        title: '一键更新已有编号',
        content: `此操作将重新生成勾选的 ${rows.length} 个模块下所有记录的编号，会覆盖已有编号且不可撤销。是否继续？`,
        okText: '继续',
        cancelText: '取消',
        okType: 'danger',
        onOk: () => resolve(),
        onCancel: () => reject(new Error('cancel')),
      });
    });
  } catch {
    return;
  }

  await doBatchRegenerate(moduleCodes, undefined);
}

async function doBatchRegenerate(moduleCodes: string[], years: number[] | undefined) {
  batchRegenerating.value = true;
  try {
    await batchRegenerateApi({ moduleCodes, years });
    message.success('一键更新任务已启动，请稍后查看进度');
    gridApi.grid?.clearCheckboxRow();
  } finally {
    batchRegenerating.value = false;
  }
}

async function confirmYearRegenerate() {
  if (!selectedYears.value.length) {
    message.warning('请至少选择一个年份');
    return;
  }
  yearModalVisible.value = false;
  await doBatchRegenerate(pendingModuleCodes.value, selectedYears.value);
}
</script>

<template>
  <Page auto-content-height>
    <div class="code-rule-wrapper">
      <Card :bordered="false" class="code-rule-help-card card-top-round">
        <div class="text-sm leading-6">
          <div class="font-medium text-base mb-2">编码规则配置说明</div>
          <ul class="list-disc pl-5 space-y-1 text-gray-600">
            <li><strong>段位配置</strong>：编码由多个段位组合而成，支持公司简称、业务类型、年份、部门、流水号、版本号、固定文本、日期等段位。</li>
            <li><strong>公司简称</strong>：在规则编辑中自定义，无需从企业信息表读取。</li>
            <li><strong>流水号</strong>：根据年份自动递增，每年从 0001 开始。新增时自动生成编号。</li>
            <li><strong>一键更新</strong>：勾选模块后点击"一键更新"，可按年份重新编号（<span class="text-red-500">覆盖已有编号且不可撤销</span>）。</li>
            <li>配置完成后即生效，新增记录自动按规则生成编号。</li>
          </ul>
        </div>
      </Card>
      <Grid>
        <template #toolbar-tools>
          <Button
            v-if="accessStore.hasAccessCode('company:code:add')"
            type="primary"
            class="mr-2"
            @click="handleCreate"
          >
            {{ $t('page.company.codeRule.button.add') }}
          </Button>
        <Button
          v-if="accessStore.hasAccessCode('company:code:regenerate')"
          type="default"
          class="mr-2"
          :loading="batchRegenerating"
          @click="handleBatchRegenerate"
        >
          <template #icon>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 16h5v5"/></svg>
          </template>
          {{ $t('page.company.codeRule.button.regenerate') }}
        </Button>
      </template>

      <template #preview="{ row }">
        <Tooltip :title="row.segments?.map((s: any) => SEGMENT_LABELS[s.type] || s.type).join(' + ') || ''">
          <Tag color="blue">{{ buildPreviewText(row) }}</Tag>
        </Tooltip>
      </template>

      <template #enabled="{ row }">
        <Switch
          :disabled="!accessStore.hasAccessCode('company:code:update')"
          v-model:checked="row.enabled"
          :checked-value="1"
          :un-checked-value="0"
          :loading="row.pending"
          @change="(checked: boolean) => handleEnabledChanged(row, checked)"
        />
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('company:code:update')"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="handleEdit(row)"
        />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.company.codeRule.module') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('company:code:delete')"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    </div>
  <Drawer />

    <!-- 年份选择模态框 -->
    <Modal
      v-model:open="yearModalVisible"
      title="一键更新客户编号"
      ok-text="确认更新"
      cancel-text="取消"
      :ok-button-props="{ danger: true }"
      @ok="confirmYearRegenerate"
    >
      <div class="space-y-4">
        <div class="bg-red-50 border border-red-200 rounded p-3 text-sm text-red-700">
          <strong>风险提示：</strong>更新后客户编号将重新从 0001 开始按年份顺序编号（按创建时间排序），
          旧编号将作废，可能导致之前打印或导出的资料与系统不一致。此操作不可撤销，请谨慎操作。
        </div>
        <div>
          <div class="font-medium mb-2">选择要更新的年份：</div>
          <Checkbox.Group v-model:value="selectedYears">
            <Checkbox v-for="y in availableYears" :key="y" :value="y" class="!ml-0 !mr-4 !mb-2">
              {{ y }} 年
            </Checkbox>
          </Checkbox.Group>
        </div>
      </div>
    </Modal>
  </Page>
</template>

<!--
  用非 scoped 的 <style> 块解决两个问题：
  1. Vue scoped 样式无法穿透到 Vxe Grid 内部 DOM（缺少 data-v-xxx 属性）
  2. Grid 组件根 div 自带 Tailwind rounded-md（6px 圆角），scoped 样式无法覆盖
  所有选择器以 .code-rule-wrapper 前缀限定，不会影响其他页面。
-->
<style>
/* ===== 顶部说明卡片：左/右上角 2px，底边 0，与下方间距 15px ===== */
.code-rule-wrapper > .ant-card {
  margin-bottom: 15px !important;
  border-top-left-radius: 2px !important;
  border-top-right-radius: 2px !important;
  border-bottom-left-radius: 0 !important;
  border-bottom-right-radius: 0 !important;
}
.code-rule-wrapper > .ant-card > .ant-card-body {
  border-top-left-radius: 2px !important;
  border-top-right-radius: 2px !important;
  border-bottom-left-radius: 0 !important;
  border-bottom-right-radius: 0 !important;
}

/* ===== 下方 Grid 根 div：覆盖 Tailwind rounded-md（6px→0）===== */
.code-rule-wrapper > div:not(.ant-card) {
  border-radius: 0 !important;
}

/* ===== Grid 内部所有容器圆角清零 ===== */
.code-rule-wrapper .vxe-grid,
.code-rule-wrapper .vxe-toolbar,
.code-rule-wrapper .vxe-grid--form-wrapper,
.code-rule-wrapper .vxe-grid--table-wrapper,
.code-rule-wrapper .vxe-grid .ant-card,
.code-rule-wrapper .vxe-grid .ant-card-head,
.code-rule-wrapper .vxe-grid .ant-card-body,
.code-rule-wrapper .vxe-grid .ant-form {
  border-radius: 0 !important;
}
</style>
