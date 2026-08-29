<script lang="ts" setup>
/**
 * 工作台设计器（方案 5.3-M2：全屏画布 + 卡片库 + 工作台切换 + 保存/预览/重置）
 * 权限：路由菜单 system:dashboard:design（d30 已注册），保存接口后端二次校验
 * 画布节点采用 gridstack JS 模式动态增删（与 index.vue 的声明式模式隔离，5.4-1）
 */
import type { GridStack, GridStackElement } from 'gridstack';

import { nextTick, onBeforeUnmount, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Button,
  Modal,
  Select,
  Tag,
  Tooltip,
  message,
} from 'ant-design-vue';

import {
  getCardLayoutApi,
  saveCardLayoutApi,
} from '#/api/core/system/dashboard-card';
import { getWorkspaceListApi } from '#/api/core/system/workspace';
import { $t } from '#/locales';

import { WORKSPACE_CANVAS_CARDS } from '../../dashboard/workspace/canvas';

interface CardLayoutVO {
  cardCode?: null | string;
  cardName?: null | string;
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

let gridStack: GridStack | null = null;

function cardConstraint(code: string) {
  return WORKSPACE_CANVAS_CARDS.find((c) => c.code === code);
}

function refreshInCanvas() {
  const codes = new Set<string>();
  for (const node of gridStack?.engine.nodes || []) {
    if (node.id) codes.add(String(node.id));
  }
  inCanvasCodes.value = codes;
}

function buildItemEl(card: CardLayoutVO): HTMLElement {
  const el = document.createElement('div');
  el.className = 'designer-item';
  const disabled = Number(card.status) === 0;

  const head = document.createElement('div');
  head.className = 'designer-item-head';
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
  const removeBtn = document.createElement('button');
  removeBtn.type = 'button';
  removeBtn.className = 'designer-item-remove';
  removeBtn.title = $t('page.system.dashboardDesigner.removeCard');
  removeBtn.textContent = '×';
  removeBtn.addEventListener('click', () => removeCard(card.cardCode || ''));
  head.appendChild(removeBtn);

  const code = document.createElement('div');
  code.className = 'designer-item-code';
  code.textContent = card.cardCode || '';

  el.appendChild(head);
  el.appendChild(code);
  return el;
}

function addCardNode(card: CardLayoutVO) {
  if (!gridStack) return;
  const code = String(card.cardCode || '');
  if (!code || inCanvasCodes.value.has(code)) return;
  const c = cardConstraint(code);
  gridStack.makeWidget(buildItemEl(card) as GridStackElement, {
    h: Math.max(1, Number(card.h) || 6),
    id: code,
    maxH: c?.maxH,
    maxW: c?.maxW,
    minH: c?.minH,
    minW: c?.minW,
    w: Math.min(12, Math.max(1, Number(card.w) || 12)),
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
  addCardNode(card);
}

function bindGridEvents() {
  if (!gridStack) return;
  gridStack.on('added removed change', () => {
    if (editMode.value) dirty.value = true;
    refreshInCanvas();
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
      label: w?.workspaceName || w?.workspaceCode,
      value: String(w?.workspaceCode || ''),
    }));
  } catch {
    workspaceOptions.value = [{ label: '默认工作台', value: 'default' }];
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
          <IconifyIcon icon="lucide:arrow-left" class="size-4" />
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
            :gs-h="card.h || 6"
            :gs-w="card.w || 12"
          >
            <div class="flex items-center justify-between gap-1">
              <span class="truncate">
                {{ card.cardName || card.cardCode }}
              </span>
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
              </span>
              <Button
                v-if="Number(card.status) !== 0"
                :disabled="inCanvasCodes.has(String(card.cardCode))"
                size="small"
                type="link"
                @click="addFromPalette(card)"
              >
                {{ $t('page.system.dashboardDesigner.add') }}
              </Button>
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
</style>

<style lang="scss" scoped>
.designer-page {
  min-height: 100%;
}

.designer-toolbar,
.designer-palette,
.designer-canvas {
  background: hsl(var(--card));
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

.designer-item {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  color: hsl(var(--foreground));
  overflow: hidden;
}

.designer-item-head {
  align-items: center;
  background: hsl(var(--accent, hsl(var(--primary) / 8%)));
  cursor: move;
  display: flex;
  gap: 6px;
  padding: 6px 10px;
}

.designer-item-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
  font-size: 16px;
  line-height: 1;
  padding: 0 2px;

  &:hover {
    color: hsl(var(--destructive));
  }
}

.designer-item-code {
  font-size: 12px;
  opacity: 0.5;
  padding: 6px 10px;
}

.designer-grid {
  background: hsl(var(--background-deep, transparent));
  border-radius: 8px;
  min-height: 320px;
}

:deep(.grid-stack-placeholder > .placeholder-content) {
  background-color: hsl(var(--primary) / 10%);
  border: 2px dashed hsl(var(--primary) / 40%);
  border-radius: 8px;
}
</style>
