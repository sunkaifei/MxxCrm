<script lang="ts" setup>
/**
 * 工作台设计器（方案 5.3-M2：全屏画布 + 卡片库 + 工作台切换 + 保存/预览/重置）
 * 权限：路由菜单 system:dashboard:design（d30 已注册），保存接口后端二次校验
 * 画布节点采用 gridstack JS 模式动态增删（与 index.vue 的声明式模式隔离，5.4-1）
 */
import type { GridStack, GridStackElement } from 'gridstack';

import { computed, nextTick, onBeforeUnmount, reactive, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';

import {
  Button,
  Modal,
  Popover,
  Select,
  Switch,
  Tag,
  Tooltip,
  message,
} from 'ant-design-vue';

import {
  getCardLayoutApi,
  saveCardLayoutApi,
  updateDashboardCardApi,
} from '#/api/core/system/dashboard-card';
import { getWorkspaceListApi } from '#/api/core/system/workspace';
import { $t } from '#/locales';

import { WORKSPACE_CANVAS_CARDS } from '../../dashboard/workspace/canvas';
import { workspaceDisplayName } from '../../dashboard/workspace/config';

// ===== 单卡配置（调研文档 8.2 第二档：显示形态 / 统计时间范围） =====
interface CardConfigEdit {
  displayForm: 'bar' | 'line' | 'pie' | 'value';
  timeRange: 'month' | 'quarter' | 'year';
}
const cardConfigDrafts = reactive<Record<string, CardConfigEdit>>({});
const configSavingCode = ref('');
/** 每张卡配置弹层的独立开合状态 */
const configPopVisible = reactive<Record<string, boolean>>({});

function configDraftOf(card: CardLayoutVO): CardConfigEdit {
  const code = String(card.cardCode || '');
  if (!cardConfigDrafts[code]) {
    let parsed: any = {};
    try {
      parsed = card.cardConfig ? JSON.parse(card.cardConfig) : {};
    } catch {
      parsed = {};
    }
    cardConfigDrafts[code] = {
      displayForm: ['bar', 'line', 'pie', 'value'].includes(parsed.displayForm)
        ? parsed.displayForm
        : 'value',
      timeRange: ['month', 'quarter', 'year'].includes(parsed.timeRange)
        ? parsed.timeRange
        : 'month',
    };
  }
  return cardConfigDrafts[code]!;
}

async function saveCardConfig(card: CardLayoutVO) {
  const code = String(card.cardCode || '');
  const draft = cardConfigDrafts[code];
  if (!draft) return;
  configSavingCode.value = code;
  try {
    // update 接口为分支式部分更新：仅传 id + card_config 不会覆盖其他字段
    await updateDashboardCardApi({ id: card.id, cardConfig: JSON.stringify(draft) });
    card.cardConfig = JSON.stringify(draft);
    message.success(`「${card.cardName || code}」卡片配置已保存`);
  } catch {
    message.error('卡片配置保存失败');
  } finally {
    configSavingCode.value = '';
  }
}

interface CardLayoutVO {
  cardCode?: null | string;
  cardName?: null | string;
  cardConfig?: null | string;
  h?: number;
  id?: number;
  pageKey?: null | string;
  status?: null | number;
  w?: number;
  x?: number;
  y?: number;
}

const router = useRouter();

const pageKey = ref('default');
const workspaceOptions = ref<Array<{ label: string; value: string }>>([]);
const paletteCards = ref<CardLayoutVO[]>([]);
const inCanvasCodes = ref<Set<string>>(new Set());
const editMode = ref(true);
const dirty = ref(false);
const saving = ref(false);
const canvasEl = ref<HTMLElement | null>(null);

/** 栏数 → 12 列栅格宽度映射（1 栏 = 全宽 12 列，2 栏 = 6 列，3 栏 = 4 列） */
const COL_WIDTH_MAP = { one: 12, three: 4, two: 6 } as const;
const defaultColWidth = ref<number>(COL_WIDTH_MAP.one);
const colWidthOptions = computed(() => [
  {
    label: $t('page.system.dashboardDesigner.cols.one'),
    value: COL_WIDTH_MAP.one,
  },
  {
    label: $t('page.system.dashboardDesigner.cols.two'),
    value: COL_WIDTH_MAP.two,
  },
  {
    label: $t('page.system.dashboardDesigner.cols.three'),
    value: COL_WIDTH_MAP.three,
  },
]);

let gridStack: GridStack | null = null;

function cardConstraint(code: string) {
  return WORKSPACE_CANVAS_CARDS.find((c) => c.code === code);
}

/** 按画布约束（canvas.ts min/max）钳制后的实际尺寸，用于卡片库/画布的尺寸徽标 */
function clampSize(
  w: number,
  h: number,
  code: string,
): { h: number; w: number } {
  const c = cardConstraint(code);
  return {
    h: Math.min(Math.max(h, c?.minH ?? 1), c?.maxH ?? 24),
    w: Math.min(Math.max(w, c?.minW ?? 1), c?.maxW ?? 12),
  };
}

/** 卡片库落位尺寸：按画布约束钳制，保证卡片库徽标/拖入尺寸与画布实际尺寸一致 */
function paletteSize(card: CardLayoutVO): { h: number; w: number } {
  return clampSize(
    defaultColWidth.value,
    Math.max(1, Number(card.h) || 6),
    String(card.cardCode || ''),
  );
}

function refreshInCanvas() {
  const codes = new Set<string>();
  for (const node of gridStack?.engine.nodes || []) {
    if (node.id) codes.add(String(node.id));
  }
  inCanvasCodes.value = codes;
}

/** 设计器卡片预览主题色（按注册卡编码，未命中回退主色） */
const CARD_PREVIEW_COLORS: Record<string, string> = {
  workspace_onboarding: '#3b82f6',
  workspace_todo_overview: '#14b8a6',
  workspace_smart_todo: '#0ea5e9',
  workspace_week_load: '#8b5cf6',
  workspace_announcement: '#f59e0b',
  workspace_stock_alert: '#ef4444',
  workspace_stock_doc_todo: '#3b82f6',
  workspace_purchase_approval: '#f97316',
  workspace_payment_reminder: '#06b6d4',
  workspace_payslip_stat: '#22c55e',
  workspace_hr_todo: '#a855f7',
  workspace_sales_performance: '#6366f1',
};

function cardAccentColor(code: string): string {
  return CARD_PREVIEW_COLORS[code] || 'hsl(var(--primary))';
}

/** 卡片骨架预览分型：stats 数字+趋势图 / steps 进度+步骤点 / list 行列表 */
const CARD_PREVIEW_VARIANTS: Record<string, 'list' | 'stats' | 'steps'> = {
  workspace_onboarding: 'steps',
  workspace_payslip_stat: 'stats',
  workspace_sales_performance: 'stats',
  workspace_todo_overview: 'stats',
  workspace_week_load: 'stats',
};

const DESIGNER_SK_CHART_BARS = [42, 68, 55, 82, 61, 90, 74, 58];

function buildSkeletonHtml(code: string): string {
  const variant = CARD_PREVIEW_VARIANTS[code] || 'list';
  if (variant === 'stats') {
    const stats = [0, 1, 2]
      .map(
        (i) =>
          `<div class="designer-sk-stat"><span class="designer-sk-badge${
            i === 1 ? ' sm' : ''
          }"></span><span class="designer-sk-big"></span></div>`,
      )
      .join('');
    const bars = DESIGNER_SK_CHART_BARS.map(
      (v) => `<span class="designer-sk-chart-bar" style="--v: ${v}%"></span>`,
    ).join('');
    return `<div class="designer-sk-stats">${stats}</div><div class="designer-sk-chart">${bars}</div>`;
  }
  if (variant === 'steps') {
    return [
      '<div class="designer-sk-line"><span class="designer-sk-bar" style="width: 46%"></span><span class="designer-sk-num"></span></div>',
      '<div class="designer-sk-progress"><span style="width: 62%"></span></div>',
      '<div class="designer-sk-steps"><span class="done"></span><span class="done"></span><span></span><span></span></div>',
      '<div class="designer-sk-line"><span class="designer-sk-avatar"></span><span class="designer-sk-bar" style="width: 34%"></span><span class="designer-sk-badge"></span></div>',
    ].join('');
  }
  return [
    '<div class="designer-sk-line"><span class="designer-sk-avatar"></span><span class="designer-sk-bar" style="width: 58%"></span><span class="designer-sk-badge"></span></div>',
    '<div class="designer-sk-line"><span class="designer-sk-avatar"></span><span class="designer-sk-bar" style="width: 42%"></span><span class="designer-sk-badge sm"></span></div>',
    '<div class="designer-sk-line"><span class="designer-sk-avatar"></span><span class="designer-sk-bar" style="width: 68%"></span><span class="designer-sk-num"></span></div>',
    '<div class="designer-sk-line"><span class="designer-sk-avatar"></span><span class="designer-sk-bar" style="width: 36%"></span><span class="designer-sk-badge"></span></div>',
    '<div class="designer-sk-line"><span class="designer-sk-avatar"></span><span class="designer-sk-bar" style="width: 52%"></span><span class="designer-sk-badge sm"></span></div>',
  ].join('');
}

function buildItemEl(card: CardLayoutVO): HTMLElement {
  // gridstack 规范结构：item > content，间距经 --gs-item-margin-* 内缩 content 实现，
  // 卡片视觉必须挂在 content 层上，卡片间 16px 间隔才会生效
  const el = document.createElement('div');
  const content = document.createElement('div');
  content.className = 'designer-item grid-stack-item-content';
  el.appendChild(content);
  const disabled = Number(card.status) === 0;
  const code = String(card.cardCode || '');
  content.style.setProperty('--designer-accent', cardAccentColor(code));

  const head = document.createElement('div');
  head.className = 'designer-item-head';
  const dot = document.createElement('span');
  dot.className = 'designer-item-dot';
  head.appendChild(dot);
  const name = document.createElement('span');
  name.className = 'designer-item-name';
  name.textContent = card.cardName || card.cardCode || '';
  head.appendChild(name);
  if (disabled) {
    const tag = document.createElement('span');
    tag.className = 'designer-item-tag';
    tag.textContent = $t('page.system.dashboardDesigner.disabledTag');
    head.appendChild(tag);
  }
  // 尺寸标签（编辑时随拖拽缩放由 refreshSizeTags 同步）
  const size = document.createElement('span');
  size.className = 'designer-item-size';
  const init = clampSize(
    Math.max(1, Number(card.w) || 12),
    Math.max(1, Number(card.h) || 6),
    code,
  );
  size.textContent = `${init.w}×${init.h}`;
  head.appendChild(size);

  // 1/2/3 栏宽快捷按钮（12 列栅格：12/6/4 列）
  const colBtns = document.createElement('span');
  colBtns.className = 'designer-item-cols';
  for (const { n, w } of [
    { n: 1, w: COL_WIDTH_MAP.one },
    { n: 2, w: COL_WIDTH_MAP.two },
    { n: 3, w: COL_WIDTH_MAP.three },
  ]) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'designer-item-col-btn';
    btn.dataset.w = String(w);
    btn.title = $t('page.system.dashboardDesigner.colTip', { n });
    btn.textContent = String(n);
    btn.addEventListener('mousedown', (e) => {
      // 卡片头是拖拽手柄（draggable.handle），不拦截 mousedown 会顺手拖走卡片
      e.stopPropagation();
    });
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      e.preventDefault();
      gridStack?.update(el as GridStackElement, { w });
    });
    colBtns.appendChild(btn);
  }
  head.appendChild(colBtns);

  const removeBtn = document.createElement('button');
  removeBtn.type = 'button';
  removeBtn.className = 'designer-item-remove';
  removeBtn.title = $t('page.system.dashboardDesigner.removeCard');
  removeBtn.textContent = $t('page.system.dashboardDesigner.removeText');
  removeBtn.addEventListener('mousedown', (e) => {
    e.stopPropagation();
  });
  removeBtn.addEventListener('click', () => removeCard(code));
  head.appendChild(removeBtn);

  const codeEl = document.createElement('div');
  codeEl.className = 'designer-item-code';
  codeEl.textContent = code;

  // 内容骨架占位：按卡片分型模拟真实数据形态（静态模板，无业务数据注入）
  const body = document.createElement('div');
  body.className = 'designer-item-body';
  body.innerHTML = buildSkeletonHtml(code);

  content.appendChild(head);
  content.appendChild(body);
  content.appendChild(codeEl);
  return el;
}

function addCardNode(card: CardLayoutVO, widthOverride?: number) {
  if (!gridStack) return;
  const code = String(card.cardCode || '');
  if (!code || inCanvasCodes.value.has(code)) return;
  const c = cardConstraint(code);
  const el = buildItemEl(card);
  const size = clampSize(
    widthOverride ?? Math.min(12, Math.max(1, Number(card.w) || 12)),
    Math.max(1, Number(card.h) || 6),
    code,
  );
  // 错峰入场动画
  const item = el.querySelector<HTMLElement>('.designer-item');
  if (item) {
    item.style.animationDelay = `${Math.min((gridStack.engine.nodes.length || 0) * 45, 360)}ms`;
  }
  gridStack.makeWidget(el as GridStackElement, {
    h: size.h,
    id: code,
    maxH: c?.maxH,
    maxW: c?.maxW,
    minH: c?.minH,
    minW: c?.minW,
    w: size.w,
    x: Math.max(0, Number(card.x) || 0),
    y: Math.max(0, Number(card.y) || 0),
  });
}

function removeCard(code: string) {
  if (!gridStack) return;
  const el = gridStack.el.querySelector(`[gs-id="${code}"]`);
  if (el) gridStack.removeWidget(el as HTMLElement);
}

function addFromPalette(card: CardLayoutVO) {
  if (Number(card.status) === 0) return;
  addCardNode(card, defaultColWidth.value);
}

/** 拖拽/缩放后同步卡片头部的 w×h 尺寸标签与 1/2/3 栏宽按钮态 */
function refreshSizeTags() {
  for (const node of gridStack?.engine.nodes || []) {
    const sizeEl = node.el?.querySelector('.designer-item-size');
    if (sizeEl && node.w && node.h) {
      sizeEl.textContent = `${node.w}×${node.h}`;
    }
    node.el
      ?.querySelectorAll<HTMLButtonElement>('.designer-item-col-btn')
      .forEach((btn) => {
        btn.classList.toggle('is-active', Number(btn.dataset.w) === node.w);
      });
  }
}

function bindGridEvents() {
  if (!gridStack) return;
  gridStack.on('added removed change', () => {
    if (editMode.value) dirty.value = true;
    refreshInCanvas();
    refreshSizeTags();
  });
  // 外部拖入不经过 addCardNode 的 inCanvasCodes 拦截，落下后若画布已有同编码卡片则移除
  gridStack.on('dropped', (_e, _old, node) => {
    const code = String(node?.id || '');
    if (!code || !node?.el) return;
    const dupes = gridStack!.engine.nodes.filter(
      (n) => String(n.id || '') === code,
    );
    if (dupes.length > 1) {
      gridStack!.removeWidget(node.el as GridStackElement);
    }
  });
}

function destroyGrid() {
  if (gridStack) {
    gridStack.destroy(false);
    gridStack = null;
  }
  if (canvasEl.value) {
    canvasEl.value.innerHTML = '';
  }
}

async function initGrid(cards: CardLayoutVO[]) {
  destroyGrid();
  await nextTick();
  if (!canvasEl.value) return;
  const { GridStack: GS } = await import('gridstack');
  // 外部拖入时 gridstack 默认把 palette 克隆元素直接当 widget（无 content 层、无骨架样式），
  // 通过 addRemoveCB 拦截，按卡片配置用 buildItemEl 重建规范的 item > content 结构
  GS.addRemoveCB = (_parent, w, add) => {
    if (!add) return undefined;
    const card = paletteCards.value.find(
      (c) => String(c.cardCode || '') === String(w.id || ''),
    );
    return buildItemEl(
      card ?? ({ cardCode: String(w.id || '') } as CardLayoutVO),
    );
  };
  gridStack = GS.init(
    {
      acceptWidgets: true,
      cellHeight: 80,
      column: 12,
      draggable: { handle: '.designer-item-head' },
      margin: 8,
      minRow: 1,
    },
    canvasEl.value,
  );
  for (const card of cards) {
    addCardNode(card);
  }
  bindGridEvents();
  refreshInCanvas();
  // 首屏必须补一次：makeWidget 早于 bindGridEvents，且 gridstack 会按 min/max 钳制尺寸，
  // 徽标需回读节点实际 w/h（否则显示的是入库值，与画布不一致）
  refreshSizeTags();
  applyEditMode();
  await nextTick();
  GS.setupDragIn('.designer-palette-item', {
    appendTo: 'body',
    helper: 'clone',
  });
}

async function loadWorkspaces() {
  try {
    const res: any = await getWorkspaceListApi();
    const list = Array.isArray(res) ? res : res?.data || [];
    workspaceOptions.value = list.map((w: any) => ({
      label: workspaceDisplayName(
        String(w?.workspaceCode || ''),
        w?.workspaceName,
      ),
      value: String(w?.workspaceCode || ''),
    }));
  } catch {
    workspaceOptions.value = [
      {
        label: $t('page.dashboard.workspace.names.default'),
        value: 'default',
      },
    ];
  }
}

async function loadLayout(reloadPalette = true) {
  const res: any = await getCardLayoutApi(pageKey.value);
  const cards: CardLayoutVO[] = Array.isArray(res) ? res : res?.data || [];
  if (reloadPalette) paletteCards.value = cards;
  await initGrid(cards);
  dirty.value = false;
}

function applyEditMode() {
  if (!gridStack) return;
  gridStack.setStatic(!editMode.value);
}

function toggleEditMode(checked: any) {
  editMode.value = checked;
  applyEditMode();
}

async function handlePageKeyChange() {
  if (dirty.value) {
    Modal.confirm({
      title: $t('page.system.dashboardDesigner.unsavedTip'),
      content: $t('page.system.dashboardDesigner.resetConfirm'),
      okText: $t('ui.button.ok'),
      cancelText: $t('ui.button.cancel'),
      onOk: async () => {
        await loadLayout();
      },
    });
    return;
  }
  await loadLayout();
}

async function handleSave() {
  if (!gridStack) return;
  saving.value = true;
  try {
    const saved: any[] = (gridStack.save(false) as any[]) || [];
    const items = saved
      .filter((n) => n && n.id)
      .map((n) => ({
        cardCode: String(n.id),
        h: Math.max(1, Math.round(Number(n.h) || 1)),
        w: Math.min(12, Math.max(1, Math.round(Number(n.w) || 12))),
        x: Math.max(0, Math.round(Number(n.x) || 0)),
        y: Math.max(0, Math.round(Number(n.y) || 0)),
      }));
    await saveCardLayoutApi({ items, pageKey: pageKey.value });
    dirty.value = false;
    message.success($t('page.system.dashboardDesigner.saveSuccess'));
  } catch {
    // 错误由全局拦截器处理
  } finally {
    saving.value = false;
  }
}

function handleReset() {
  Modal.confirm({
    title: $t('page.system.dashboardDesigner.reset'),
    content: $t('page.system.dashboardDesigner.resetConfirm'),
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: async () => {
      await loadLayout();
    },
  });
}

function handleBack() {
  if (dirty.value) {
    Modal.confirm({
      title: $t('page.system.dashboardDesigner.unsavedTip'),
      content: $t('page.system.dashboardDesigner.resetConfirm'),
      okText: $t('ui.button.ok'),
      cancelText: $t('ui.button.cancel'),
      onOk: () => router.push('/system/dashboard-card'),
    });
    return;
  }
  router.push('/system/dashboard-card');
}

loadWorkspaces();
loadLayout();

onBeforeUnmount(() => {
  destroyGrid();
});
</script>

<template>
  <Page title="" auto-content-height>
    <div class="designer-page flex h-full flex-col gap-2">
      <div
        class="designer-toolbar flex flex-wrap items-center gap-2 rounded-lg border p-2"
      >
        <Button size="small" @click="handleBack">
          {{ $t('page.system.dashboardDesigner.back') }}
        </Button>
        <div class="flex items-center gap-1">
          <span class="text-sm">
            {{ $t('page.system.dashboardDesigner.workspace') }}
          </span>
          <Select
            v-model:value="pageKey"
            :options="workspaceOptions"
            :disabled="dirty"
            class="w-44"
            size="small"
            @change="handlePageKeyChange"
          />
        </div>
        <Tooltip :title="$t('page.system.dashboardDesigner.colWidthTip')">
          <div class="flex items-center gap-1">
            <span class="text-sm">
              {{ $t('page.system.dashboardDesigner.colWidth') }}
            </span>
            <Select
              v-model:value="defaultColWidth"
              :options="colWidthOptions"
              class="w-24"
              size="small"
            />
          </div>
        </Tooltip>
        <div class="ml-auto flex items-center gap-2">
          <span v-if="dirty" class="text-xs text-orange-500">
            {{ $t('page.system.dashboardDesigner.unsavedTip') }}
          </span>
          <Switch
            :checked="editMode"
            :checked-children="$t('page.system.dashboardDesigner.editMode')"
            :un-checked-children="$t('page.system.dashboardDesigner.previewMode')"
            size="small"
            @change="toggleEditMode"
          />
          <Button size="small" @click="handleReset">
            {{ $t('page.system.dashboardDesigner.reset') }}
          </Button>
          <Button
            :loading="saving"
            size="small"
            type="primary"
            @click="handleSave"
          >
            {{ $t('page.system.dashboardDesigner.save') }}
          </Button>
        </div>
      </div>

      <div class="flex min-h-0 flex-1 gap-2">
        <aside
          class="designer-palette flex w-60 shrink-0 flex-col gap-2 overflow-y-auto rounded-lg border p-2"
        >
          <div class="text-sm font-medium">
            {{ $t('page.system.dashboardDesigner.palette') }}
          </div>
          <div
            v-for="card in paletteCards"
            :key="card.id"
            class="designer-palette-item rounded-md border p-2 text-sm"
            :class="{
              'designer-palette-item-disabled': Number(card.status) === 0,
            }"
            :gs-h="paletteSize(card).h"
            :gs-id="card.cardCode"
            :gs-max-h="cardConstraint(String(card.cardCode))?.maxH"
            :gs-max-w="cardConstraint(String(card.cardCode))?.maxW"
            :gs-min-h="cardConstraint(String(card.cardCode))?.minH"
            :gs-min-w="cardConstraint(String(card.cardCode))?.minW"
            :gs-w="paletteSize(card).w"
            :style="{
              '--designer-accent': cardAccentColor(String(card.cardCode || '')),
            }"
          >
            <div class="flex items-center justify-between gap-1">
              <span class="flex min-w-0 items-center gap-1.5">
                <span class="designer-item-dot shrink-0"></span>
                <span class="truncate">
                  {{ card.cardName || card.cardCode }}
                </span>
              </span>
              <Tag
                v-if="
                  String(card.pageKey) === 'default' && pageKey !== 'default'
                "
                class="shrink-0"
                color="geekblue"
              >
                {{ $t('page.system.dashboardDesigner.sharedTag') }}
              </Tag>
              <Tooltip
                v-if="Number(card.status) === 0"
                :title="$t('page.system.dashboardDesigner.disabledTip')"
              >
                <Tag class="shrink-0" color="default">
                  {{ $t('page.system.dashboardDesigner.disabledTag') }}
                </Tag>
              </Tooltip>
              <Tag
                v-else-if="inCanvasCodes.has(String(card.cardCode))"
                class="shrink-0"
                color="blue"
              >
                {{ $t('page.system.dashboardDesigner.inCanvas') }}
              </Tag>
            </div>
            <div class="mt-1 flex items-center justify-between">
              <span class="truncate text-xs opacity-60">
                {{ card.cardCode }}
                <span class="ml-1 opacity-80">
                  {{ paletteSize(card).w }}×{{ paletteSize(card).h }}
                </span>
              </span>
              <span class="flex shrink-0 items-center">
                <!-- 单卡配置（调研文档 8.2：显示形态/统计时间范围，存 card_config） -->
                <Popover
                  v-model:open="configPopVisible[String(card.cardCode || '')]"
                  placement="bottomRight"
                  trigger="click"
                >
                  <template #content>
                    <div
                      v-if="configDraftOf(card)"
                      class="flex flex-col gap-2 p-1"
                      style="min-width: 220px"
                    >
                      <div class="text-sm font-medium">
                        {{ card.cardName || card.cardCode }} · 单卡配置
                      </div>
                      <div class="flex items-center justify-between gap-2">
                        <span class="text-xs">显示形态</span>
                        <Select
                          v-model:value="configDraftOf(card)!.displayForm"
                          :options="[
                            { label: '数值', value: 'value' },
                            { label: '柱状图', value: 'bar' },
                            { label: '折线图', value: 'line' },
                            { label: '饼图', value: 'pie' },
                          ]"
                          size="small"
                          style="width: 110px"
                        />
                      </div>
                      <div class="flex items-center justify-between gap-2">
                        <span class="text-xs">统计范围</span>
                        <Select
                          v-model:value="configDraftOf(card)!.timeRange"
                          :options="[
                            { label: '本月', value: 'month' },
                            { label: '本季', value: 'quarter' },
                            { label: '本年', value: 'year' },
                          ]"
                          size="small"
                          style="width: 110px"
                        />
                      </div>
                      <Button
                        size="small"
                        type="primary"
                        :loading="configSavingCode === String(card.cardCode)"
                        @click="saveCardConfig(card)"
                      >
                        保存配置
                      </Button>
                    </div>
                  </template>
                  <Button size="small" type="link">配置</Button>
                </Popover>
                <Button
                  v-if="Number(card.status) !== 0"
                  :disabled="inCanvasCodes.has(String(card.cardCode))"
                  size="small"
                  type="link"
                  @click="addFromPalette(card)"
                >
                  {{ $t('page.system.dashboardDesigner.add') }}
                </Button>
              </span>
            </div>
          </div>
          <div
            v-if="paletteCards.length === 0"
            class="py-8 text-center text-xs opacity-60"
          >
            {{ $t('page.system.dashboardDesigner.paletteEmpty') }}
          </div>
        </aside>

        <main class="designer-canvas flex-1 overflow-auto rounded-lg border p-3">
          <div
            v-if="!editMode"
            class="mb-2 rounded border border-dashed px-3 py-1 text-xs opacity-70"
          >
            {{ $t('page.system.dashboardDesigner.previewTip') }}
          </div>
          <div ref="canvasEl" class="designer-grid grid-stack"></div>
        </main>
      </div>
    </div>
  </Page>
</template>

<style lang="scss">
@import 'gridstack/dist/gridstack.css';

// ============================================================
// 画布动态卡片样式（非 scoped！）
// buildItemEl 经 document.createElement + gridstack 注入的 DOM
// 不带 Vue scoped 的 data-v 属性，scoped 选择器永远无法匹配；
// 故本节必须全局，统一以 designer- 前缀做命名空间隔离。
// ============================================================
.designer-item {
  animation: designer-item-in 0.4s cubic-bezier(0.22, 1, 0.36, 1) backwards;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-left: 3px solid var(--designer-accent, hsl(var(--primary)));
  border-radius: 8px;
  color: hsl(var(--foreground));
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow:
    0 1px 2px hsl(var(--foreground) / 8%),
    0 4px 12px hsl(var(--foreground) / 6%);
  transition:
    box-shadow 0.2s ease,
    border-color 0.2s ease;

  &:hover {
    border-color: color-mix(
      in srgb,
      var(--designer-accent, hsl(var(--primary))) 45%,
      hsl(var(--border))
    );
    border-left-color: var(--designer-accent, hsl(var(--primary)));
    box-shadow:
      0 2px 4px hsl(var(--foreground) / 10%),
      0 8px 20px hsl(var(--foreground) / 12%);
  }
}

// gridstack 对 content 层默认 overflow-y: auto（特异性 0,3,0），卡片需整体裁切不出滚动条
.grid-stack > .grid-stack-item > .designer-item {
  overflow: hidden;
}

@keyframes designer-item-in {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.98);
  }

  to {
    opacity: 1;
    transform: none;
  }
}

.designer-item-head {
  align-items: center;
  background: color-mix(in srgb, var(--designer-accent, hsl(var(--primary))) 8%, hsl(var(--card)));
  cursor: move;
  display: flex;
  flex-shrink: 0;
  gap: 6px;
  padding: 6px 10px;
}

.designer-item-dot {
  background: var(--designer-accent, hsl(var(--primary)));
  border-radius: 9999px;
  display: inline-block;
  height: 8px;
  width: 8px;
}

.designer-item-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.designer-item-size {
  border: 1px solid hsl(var(--border));
  border-radius: 4px;
  color: hsl(var(--foreground) / 60%);
  flex-shrink: 0;
  font-size: 11px;
  line-height: 1;
  padding: 2px 4px;
  white-space: nowrap;
}

.designer-item-cols {
  display: inline-flex;
  flex-shrink: 0;
  gap: 2px;
}

.designer-item-col-btn {
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: hsl(var(--foreground) / 50%);
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
  height: 18px;
  line-height: 1;
  padding: 0;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
  width: 18px;

  &:hover {
    background: hsl(var(--foreground) / 10%);
    color: hsl(var(--foreground) / 90%);
  }

  &.is-active {
    background: var(--designer-accent, hsl(var(--primary)));
    color: #fff;
  }
}

.designer-item-tag {
  border: 1px solid hsl(var(--border));
  border-radius: 4px;
  font-size: 11px;
  opacity: 0.7;
  padding: 0 4px;
}

.designer-item-remove {
  background: transparent;
  border: none;
  color: hsl(var(--foreground) / 60%);
  cursor: pointer;
  flex-shrink: 0;
  font-size: 16px;
  line-height: 1;
  padding: 0 2px;

  &:hover {
    color: hsl(var(--destructive));
  }
}

// ===== 内容骨架预览（按卡片类型模拟真实数据形态，shimmer 扫光 + 错峰入场） =====
.designer-item-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 10px;
  justify-content: center;
  min-height: 0;
  overflow: hidden;
  padding: 12px 14px;

  // 行级错峰入场
  > * {
    animation: designer-sk-up 0.45s ease backwards;
  }

  @for $i from 1 through 6 {
    > :nth-child(#{$i}) {
      animation-delay: #{0.06 * $i}s;
    }
  }
}

@keyframes designer-sk-up {
  from {
    opacity: 0;
    transform: translateY(6px);
  }

  to {
    opacity: 1;
    transform: none;
  }
}

// shimmer 扫光：覆盖所有骨架块
.designer-sk-bar,
.designer-sk-big,
.designer-sk-badge,
.designer-sk-num,
.designer-sk-avatar {
  overflow: hidden;
  position: relative;

  &::after {
    animation: designer-sk-shimmer 1.8s ease-in-out infinite;
    background: linear-gradient(
      100deg,
      transparent 30%,
      hsl(var(--foreground) / 9%) 50%,
      transparent 70%
    );
    content: '';
    inset: 0;
    position: absolute;
    transform: translateX(-120%);
  }
}

@keyframes designer-sk-shimmer {
  to {
    transform: translateX(120%);
  }
}

.designer-sk-line {
  align-items: center;
  display: flex;
  gap: 10px;

  > .designer-sk-badge,
  > .designer-sk-num {
    margin-left: auto;
  }
}

.designer-sk-bar {
  background: hsl(var(--muted));
  border-radius: 4px;
  flex-shrink: 1;
  height: 10px;
  min-width: 24px;
}

.designer-sk-avatar {
  background: color-mix(
    in srgb,
    var(--designer-accent, hsl(var(--primary))) 26%,
    hsl(var(--muted))
  );
  border-radius: 9999px;
  flex-shrink: 0;
  height: 20px;
  width: 20px;
}

.designer-sk-badge {
  background: color-mix(
    in srgb,
    var(--designer-accent, hsl(var(--primary))) 18%,
    transparent
  );
  border-radius: 9999px;
  flex-shrink: 0;
  height: 16px;
  width: 34px;

  &.sm {
    width: 22px;
  }
}

.designer-sk-num {
  background: color-mix(
    in srgb,
    var(--designer-accent, hsl(var(--primary))) 28%,
    transparent
  );
  border-radius: 4px;
  flex-shrink: 0;
  height: 20px;
  width: 48px;
}

// stats：数字块 + 迷你柱状趋势
.designer-sk-stats {
  display: flex;
  gap: 14px;

  .designer-sk-stat {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 6px;
  }
}

.designer-sk-big {
  background: color-mix(
    in srgb,
    var(--designer-accent, hsl(var(--primary))) 32%,
    hsl(var(--muted))
  );
  border-radius: 6px;
  height: 22px;
  width: 64px;
}

.designer-sk-chart {
  align-items: flex-end;
  display: flex;
  flex: 1;
  gap: 6px;
  min-height: 30px;

  .designer-sk-chart-bar {
    animation: designer-sk-grow 0.6s cubic-bezier(0.22, 1, 0.36, 1) backwards;
    background: linear-gradient(
      to top,
      color-mix(
        in srgb,
        var(--designer-accent, hsl(var(--primary))) 65%,
        transparent
      ),
      color-mix(
        in srgb,
        var(--designer-accent, hsl(var(--primary))) 22%,
        transparent
      )
    );
    border-radius: 3px 3px 0 0;
    flex: 1;
    height: var(--v, 50%);
    transform-origin: bottom;
  }

  @for $i from 1 through 8 {
    .designer-sk-chart-bar:nth-child(#{$i}) {
      animation-delay: #{0.08 * $i}s;
    }
  }
}

@keyframes designer-sk-grow {
  from {
    transform: scaleY(0);
  }
}

// steps：进度条 + 步骤点
.designer-sk-progress {
  background: hsl(var(--muted));
  border-radius: 9999px;
  height: 8px;
  overflow: hidden;

  > span {
    background: linear-gradient(
      90deg,
      color-mix(
        in srgb,
        var(--designer-accent, hsl(var(--primary))) 55%,
        transparent
      ),
      var(--designer-accent, hsl(var(--primary)))
    );
    border-radius: 9999px;
    display: block;
    height: 100%;
  }
}

.designer-sk-steps {
  display: flex;
  gap: 6px;

  span {
    border: 2px solid
      color-mix(
        in srgb,
        var(--designer-accent, hsl(var(--primary))) 45%,
        transparent
      );
    border-radius: 9999px;
    flex: 1;
    height: 8px;

    &.done {
      background: var(--designer-accent, hsl(var(--primary)));
      border-color: var(--designer-accent, hsl(var(--primary)));
    }
  }
}

.designer-item-code {
  border-top: 1px dashed hsl(var(--border) / 60%);
  flex-shrink: 0;
  font-size: 11px;
  opacity: 0.5;
  padding: 4px 14px 6px;
}
</style>

<style lang="scss" scoped>
.designer-page {
  min-height: 100%;
}

.designer-toolbar,
.designer-palette {
  background: hsl(var(--card));
  border-color: hsl(var(--border));
}

// 画布用深一档台面色（--background-deep），让 card 底色的卡片在画布上凸显轮廓
.designer-canvas {
  background: hsl(var(--background-deep, var(--muted)));
  border-color: hsl(var(--border));
}

.designer-palette-item {
  background: hsl(var(--background-deep, var(--card)));
  border-color: hsl(var(--border));
  color: hsl(var(--foreground));

  &.designer-palette-item-disabled {
    opacity: 0.5;
  }
}

.designer-grid {
  // 12 列栅格背景（列宽 1/12，行高 80px 与 cellHeight 对齐），提升画布空间感
  // 栅格线用 foreground 低透明度：亮色呈深色细线、暗色呈浅色细线，双主题均可见
  background-image:
    linear-gradient(hsl(var(--foreground) / 6%) 1px, transparent 1px),
    linear-gradient(90deg, hsl(var(--foreground) / 6%) 1px, transparent 1px);
  background-size: calc(100% / 12) 80px;
  border-radius: 8px;
  min-height: 320px;
}

:deep(.grid-stack-placeholder > .placeholder-content) {
  background-color: hsl(var(--primary) / 10%);
  border: 2px dashed hsl(var(--primary) / 40%);
  border-radius: 8px;
}
</style>
