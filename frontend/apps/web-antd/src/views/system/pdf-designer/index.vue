<script lang="ts" setup>
/**
 * 可视化 PDF 模板设计器（设计文档 §8 主布局、§26 工程结构、§32 设计时数据、§33 并发与迁移）。
 *
 * 布局：① 顶部工具栏　② 左面板（元素库/数据源/图层）　③ 中央画布　④ 右属性面板　⑤ 状态栏
 *
 * 关键工程点：
 * - **软锁**：进入即抢占，15s 心跳续约，离开/关页释放（§33.1）
 * - **草稿恢复**：脏状态下写入 localStorage，重进时询问是否恢复（§24）
 * - **防呆**：占位符/样例视图下不允许设为默认，由后端二次校验 `dataMode`（§32.15）
 */
import type { PdfAssetVO, PdfDocType, PdfFieldGroup } from '#/types/pdf-layout';

import { onBeforeUnmount, onMounted, ref, watch , h } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Button, Input, Modal, Select, Tabs, Tag, message, notification } from 'ant-design-vue';
import {
  LucideLayers,
  LucideListTree,
  LucideShapes,
} from '@vben/icons';

import {
  acquirePdfLockApi,
  getPdfAssetListApi,
  getPdfDesignerTemplateApi,
  getPdfDocTypesApi,
  getPdfFieldTreeApi,
  getPdfSampleDataApi,
  getPdfVersionsApi,
  heartbeatPdfLockApi,
  importPdfBundleApi,
  pdfBundleExportUrl,
  releasePdfLockApi,
  rollbackPdfVersionApi,
  savePdfLayoutApi,
  setDefaultPdfTemplateApi,
} from '#/api/core/system/pdf-designer';
import { useDesignerStore } from '#/composables/pdf-designer/useDesignerStore';

import CanvasStage from './components/CanvasStage.vue';
import DialogImportPdf from './components/DialogImportPdf.vue';
import DialogPagePreview from './components/DialogPagePreview.vue';
import DialogPrecisePreview from './components/DialogPrecisePreview.vue';
import PanelDataSource from './components/PanelDataSource.vue';
import PanelElements from './components/PanelElements.vue';
import PanelLayers from './components/PanelLayers.vue';
import PanelProps from './components/PanelProps.vue';
import StatusBar from './components/StatusBar.vue';
import Toolbar from './components/Toolbar.vue';

const route = useRoute();
const router = useRouter();
const store = useDesignerStore();
const stage = ref<InstanceType<typeof CanvasStage>>();

const loading = ref(false);
const leftTab = ref('elements');
const docTypes = ref<PdfDocType[]>([]);
const fieldTree = ref<PdfFieldGroup[]>([]);
const assets = ref<PdfAssetVO[]>([]);

const showPrecise = ref(false);
const showPage = ref(false);
const showImportPdf = ref(false);
const showVersions = ref(false);
const versions = ref<any[]>([]);
const lockedByOther = ref<null | string>(null);

const DRAFT_KEY = 'pdf-designer-draft';

// ============================================================================
// 初始化
// ============================================================================

async function loadDocTypes() {
  const res: any = await getPdfDocTypesApi();
  docTypes.value = res?.data ?? res ?? [];
}

async function loadFieldTree() {
  loading.value = true;
  try {
    const res: any = await getPdfFieldTreeApi(store.docType);
    // 响应结构：data = { docName, docType, groups: FieldGroup[], formatters, presets }
    // 后端 FieldGroup 键为 { code, name, isArray, fields: [{path, name, dataType, sample, sampleLong, nullable}] }，
    // 与前端契约（groupCode/groupName/children + fieldPath/fieldName）不同构，此处统一映射
    const payload = res?.data ?? res ?? {};
    const rawGroups = Array.isArray(payload) ? payload : (payload.groups ?? []);
    fieldTree.value = rawGroups.map((g: any) => ({
      groupCode: g.code ?? g.groupCode ?? '',
      groupName: g.name ?? g.groupName ?? '',
      isArray: g.isArray ?? false,
      children: (g.fields ?? g.children ?? []).map((f: any) => ({
        fieldPath: f.path ?? f.fieldPath ?? '',
        fieldName: f.name ?? f.fieldName ?? '',
        dataType: f.dataType ?? 'string',
        sample: f.sample ?? '',
        sampleLong: f.sampleLong ?? '',
      })),
    }));
  } finally {
    loading.value = false;
  }
}

async function loadSampleData() {
  try {
    const res: any = await getPdfSampleDataApi({
      docType: store.docType,
      preset: store.samplePreset,
      itemRows: store.itemRows,
      templateId: store.templateId ?? undefined,
    });
    store.dataContext = res?.data ?? res ?? {};
  } catch {
    store.dataContext = {};
  }
}

async function loadAssets() {
  try {
    const res: any = await getPdfAssetListApi({ page: 1, pageSize: 200 });
    const d = res?.data ?? res;
    assets.value = d?.items ?? d ?? [];
  } catch {
    assets.value = [];
  }
}

async function loadTemplate(id: number) {
  const res: any = await getPdfDesignerTemplateApi(id);
  const t = res?.data ?? res;
  store.loadLayout(t?.layoutJson ?? null, {
    id: Number(t?.id ?? id),
    name: t?.name,
    code: t?.templateCode,
    docType: t?.docType,
    version: t?.version,
  });
}

// ============================================================================
// 软锁（§33.1）
// ============================================================================

let heartbeatTimer: any = null;

async function acquireLock() {
  if (heartbeatTimer) {
    clearInterval(heartbeatTimer);
    heartbeatTimer = null;
  }
  if (!store.templateId) return;
  try {
    await acquirePdfLockApi(store.templateId);
    lockedByOther.value = null;
  } catch (e: any) {
    const msg = e?.message ?? '';
    lockedByOther.value = msg;
    Modal.confirm({
      content: msg,
      okText: '强制接管',
      cancelText: '返回列表',
      title: '模板正被他人编辑',
      onCancel: () => router.push('/system/pdf-template'),
      onOk: async () => {
        // 强制接管：后端释放由 TTL 兜底，此处直接继续编辑
        lockedByOther.value = null;
      },
    });
  }
  heartbeatTimer = setInterval(async () => {
    if (!store.templateId || document.hidden) return;
    try {
      await heartbeatPdfLockApi(store.templateId);
    } catch {
      /* 锁已失效，下次保存会走乐观锁兜底 */
    }
  }, 15_000);
}

function releaseLockByBeacon() {
  if (!store.templateId) return;
  const { useAccessStore } = require('@vben/stores') as any;
  void useAccessStore;
  try {
    const payload = JSON.stringify({ templateId: store.templateId });
    // sendBeacon 需显式声明 JSON 类型，否则后端 web::Json 会拒绝
    const blob = new Blob([payload], { type: 'application/json' });
    navigator.sendBeacon('/api/system/pdf-designer/lock/release', blob);
  } catch {
    /* ignore */
  }
}

function onBeforeUnload(e: BeforeUnloadEvent) {
  if (!store.dirty) return;
  saveDraft();
  releaseLockByBeacon();
  e.preventDefault();
  e.returnValue = '';
}

// ============================================================================
// 草稿（§24）
// ============================================================================

function saveDraft() {
  try {
    localStorage.setItem(
      DRAFT_KEY,
      JSON.stringify({
        docType: store.docType,
        layout: store.toPayload(),
        name: store.templateName,
        templateId: store.templateId,
        time: Date.now(),
      }),
    );
  } catch {
    /* 超出配额忽略 */
  }
}

function clearDraft() {
  localStorage.removeItem(DRAFT_KEY);
}

function maybeRestoreDraft() {
  const raw = localStorage.getItem(DRAFT_KEY);
  if (!raw) return;
  try {
    const d = JSON.parse(raw);
    if (!d?.layout) return;
    Modal.confirm({
      content: `检测到 ${new Date(d.time).toLocaleString()} 的未保存草稿（${d.name || '未命名'}），是否恢复？`,
      okText: '恢复草稿',
      cancelText: '丢弃',
      title: '发现未保存的草稿',
      onCancel: clearDraft,
      onOk: () => {
        store.loadLayout(d.layout, {
          id: d.templateId,
          name: d.name,
          docType: d.docType,
        });
        store.dirty = true;
      },
    });
  } catch {
    clearDraft();
  }
}

// ============================================================================
// 工具栏动作
// ============================================================================

/**
 * §37：保存前补全模板元信息（名称 + 单据类型）。
 * - 新建模板必填名称；单据类型确定字段树与数据上下文，保存后锁定（换类型=换数据源）
 * - 选择类型后联动刷新字段树
 */
function promptTemplateMeta(): Promise<boolean> {
  return new Promise((resolve) => {
    let nameInput = store.templateName || '';
    let typeValue = store.docType;
    Modal.confirm({
      title: '保存模板',
      content: h('div', { class: 'space-y-3 pt-2' }, [
        h('div', [
          h('div', { class: 'mb-1 text-xs text-gray-500' }, '模板名称'),
          h(Input, {
            value: nameInput,
            placeholder: '如：销售订单-精装版',
            'onUpdate:value': (v: string) => (nameInput = v),
          }),
        ]),
        h('div', [
          h('div', { class: 'mb-1 text-xs text-gray-500' }, '单据类型（保存后锁定）'),
          h(
            Select,
            {
              value: typeValue,
              class: 'w-full',
              options: docTypes.value.map((d: any) => ({
                label: d.name,
                value: d.code,
              })),
              onChange: (v: any) => {
                typeValue = String(v);
                store.docType = String(v);
                void loadFieldTree();
              },
            },
          ),
        ]),
      ]),
      okText: '确定并保存',
      cancelText: '取消',
      onCancel: () => resolve(false),
      onOk: () => {
        const name = nameInput.trim();
        if (!name) {
          message.warning('请填写模板名称');
          resolve(false);
          return;
        }
        store.templateName = name;
        store.docType = typeValue;
        resolve(true);
      },
    });
  });
}

async function onSave() {
  if (store.dataMode !== 'sample' && store.dataMode !== 'real') {
    message.warning('请在「样例」或「真实」数据视图下保存，以免把占位符当成正式内容');
    return;
  }
  // §37：新建（无 id）或未命名模板 → 先弹窗补全「模板名称 + 单据类型」，再落库
  if (!store.templateId || !store.templateName.trim()) {
    const ok = await promptTemplateMeta();
    if (!ok) return;
  }
  store.saving = true;
  try {
    const res: any = await savePdfLayoutApi({
      id: store.templateId ?? undefined,
      name: store.templateName,
      templateCode: store.templateCode || undefined,
      docType: store.docType,
      layoutJson: store.toPayload(),
      version: store.version,
    });
    const data = res?.data ?? res;
    store.templateId = Number(data?.id ?? store.templateId);
    store.version = data?.version ?? store.version + 1;
    store.dirty = false;
    clearDraft();
    message.success('保存成功');
  } catch (e: any) {
    const msg = e?.message ?? '保存失败';
    if (msg.includes('模板已被他人修改')) {
      Modal.confirm({
        content: msg,
        okText: '覆盖',
        title: '版本冲突',
        onOk: async () => {
          await savePdfLayoutApi({
            id: store.templateId ?? undefined,
            name: store.templateName,
            docType: store.docType,
            layoutJson: store.toPayload(),
            version: store.version,
            force: true,
          });
          store.dirty = false;
          message.success('已覆盖保存');
        },
      });
    } else {
      message.error(msg);
    }
  } finally {
    store.saving = false;
  }
}

function onSaveAs() {
  Modal.confirm({
    content: '将当前版式另存为一个新模板（原模板保持不变）。',
    okText: '另存为',
    title: '另存为新模板',
    onOk: async () => {
      const name = `${store.templateName || '未命名'} 副本`;
      const res: any = await savePdfLayoutApi({
        name,
        docType: store.docType,
        layoutJson: store.toPayload(),
        basedOnTemplateId: store.templateId ?? undefined,
      });
      const data = res?.data ?? res;
      store.templateId = Number(data?.id);
      store.templateName = name;
      store.templateCode = '';
      store.version = 1;
      store.dirty = false;
      message.success('已另存为新模板');
    },
  });
}

function onAlign(kind: string) {
  const els = store.selectedElements;
  if (els.length === 0) return;
  store.commit('对齐');
  const boxes = els.map((e) => ({ id: e.id, x: e.x, y: e.y, w: e.w, h: e.h }));
  const minX = Math.min(...boxes.map((b) => b.x));
  const maxX = Math.max(...boxes.map((b) => b.x + b.w));
  const minY = Math.min(...boxes.map((b) => b.y));
  const maxY = Math.max(...boxes.map((b) => b.y + b.h));
  for (const b of boxes) {
    if (kind === 'left') store.patchElementLive(b.id, { x: minX });
    if (kind === 'right') store.patchElementLive(b.id, { x: maxX - b.w });
    if (kind === 'hcenter')
      store.patchElementLive(b.id, { x: (minX + maxX) / 2 - b.w / 2 });
    if (kind === 'top') store.patchElementLive(b.id, { y: minY });
    if (kind === 'bottom') store.patchElementLive(b.id, { y: maxY - b.h });
    if (kind === 'vcenter')
      store.patchElementLive(b.id, { y: (minY + maxY) / 2 - b.h / 2 });
  }
  if (kind === 'dist-h' && boxes.length >= 3) {
    const sorted = [...boxes].sort((a, b) => a.x - b.x);
    const totalW = sorted.reduce((s, b) => s + b.w, 0);
    const gap = (maxX - minX - totalW) / (sorted.length - 1);
    let cur = minX;
    for (const b of sorted) {
      store.patchElementLive(b.id, { x: Math.round(cur * 100) / 100 });
      cur += b.w + gap;
    }
  }
}

async function onSetDefault() {
  if (!store.templateId) {
    message.warning('请先保存模板');
    return;
  }
  const outCount = store.outOfPaperCount();
  if (outCount > 0) {
    Modal.confirm({
      content: `有 ${outCount} 个元素位于纸张之外，正式出图时会被裁切。确定设为默认模板？`,
      okText: '仍然设为默认',
      title: '存在纸外元素',
      onOk: () => doSetDefault(true),
    });
    return;
  }
  await doSetDefault(false);
}

async function doSetDefault(confirmOutOfPaper: boolean) {
  try {
    await setDefaultPdfTemplateApi({
      id: store.templateId!,
      confirmOutOfPaper,
    });
    message.success('已设为该单据类型的默认模板');
  } catch (e: any) {
    message.error(e?.message ?? '操作失败');
  }
}

async function onExportBundle() {
  if (!store.templateId) {
    message.warning('请先保存模板');
    return;
  }
  window.open(pdfBundleExportUrl(store.templateId), '_blank');
}

function onImportBundle() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.zip';
  input.onchange = async () => {
    const f = input.files?.[0];
    if (!f) return;
    try {
      const res: any = await importPdfBundleApi(f);
      if (res?.code !== 200) throw new Error(res?.msg ?? '导入失败');
      message.success('模板包导入成功，已生成新模板');
      const data = res?.data ?? res;
      if (data?.id) {
        await loadTemplate(Number(data.id));
      }
    } catch (e: any) {
      message.error(e?.message ?? '导入失败');
    }
  };
  input.click();
}

async function onShowVersions() {
  if (!store.templateId) return;
  showVersions.value = true;
  const res: any = await getPdfVersionsApi(store.templateId);
  versions.value = res?.data ?? res ?? [];
}

async function onRollback(v: number) {
  try {
    await rollbackPdfVersionApi({ templateId: store.templateId!, version: v });
    await loadTemplate(store.templateId!);
    showVersions.value = false;
    message.success(`已回滚到 v${v}`);
  } catch (e: any) {
    message.error(e?.message ?? '回滚失败');
  }
}

/** 导入 PDF 完成后装载生成的 layout */
async function onImportPdfDone(layoutJson: any) {
  if (!layoutJson) return;
  store.loadLayout(layoutJson, {
    id: undefined,
    name: `${store.templateName || '导入模板'}（PDF 底图）`,
    docType: store.docType,
  });
  store.dirty = true;
  notification.success({
    description: '请直接在底图上拖拽摆放字段并对齐文字，然后保存。',
    message: '已生成底图模板草稿',
  });
}

function onPickAsset(category: string) {
  leftTab.value = 'elements';
  void category;
  message.info('请到「系统 → PDF 素材」页面上传素材，再回到此处选择');
}

// ============================================================================
// 快捷键
// ============================================================================

function onKeydown(e: KeyboardEvent) {
  const mod = e.ctrlKey || e.metaKey;
  const tag = (document.activeElement?.tagName ?? '').toLowerCase();
  const editing =
    tag === 'input' || tag === 'textarea' || (document.activeElement as any)?.isContentEditable;

  if (mod && e.key.toLowerCase() === 's') {
    e.preventDefault();
    void onSave();
    return;
  }
  if (mod && e.key.toLowerCase() === 'z' && !e.shiftKey) {
    e.preventDefault();
    store.undo();
    return;
  }
  if (mod && (e.key.toLowerCase() === 'y' || (e.key.toLowerCase() === 'z' && e.shiftKey))) {
    e.preventDefault();
    store.redo();
    return;
  }
  if (editing) return;
  if (e.key === 'Delete' || e.key === 'Backspace') {
    e.preventDefault();
    store.removeElements([...store.selection]);
    return;
  }
  if (mod && e.key.toLowerCase() === 'd') {
    e.preventDefault();
    store.duplicateElements([...store.selection]);
    return;
  }
  const step = e.shiftKey ? 5 : 1;
  if (e.key === 'ArrowLeft') { e.preventDefault(); stage.value?.nudge(-step, 0); }
  if (e.key === 'ArrowRight') { e.preventDefault(); stage.value?.nudge(step, 0); }
  if (e.key === 'ArrowUp') { e.preventDefault(); stage.value?.nudge(0, -step); }
  if (e.key === 'ArrowDown') { e.preventDefault(); stage.value?.nudge(0, step); }
}

// ============================================================================
// 生命周期
// ============================================================================

/**
 * 从路由参数解析待编辑的模板 id。
 * 模板列表「可视化设计」传的是 `templateId`（`id` 为历史兼容写法）。
 */
function queryTemplateId(): number {
  const q = route.query as Record<string, any>;
  return Number(q.templateId ?? q.id ?? 0);
}

/** 按路由参数装载模板：有 templateId 走回读，无则开一张空白版式（保存时新建模板） */
async function loadFromQuery() {
  const tid = queryTemplateId();
  if (tid > 0) {
    try {
      await loadTemplate(tid);
    } catch (e: any) {
      message.error(e?.message ?? '模板加载失败');
    }
  } else {
    store.loadLayout(null, {
      docType: String((route.query as Record<string, any>).docType ?? 'order'),
    });
  }

  await Promise.all([loadFieldTree(), loadSampleData()]);
  await acquireLock();
}

onMounted(async () => {
  await Promise.all([loadDocTypes(), loadAssets()]);
  await loadFromQuery();
  maybeRestoreDraft();

  window.addEventListener('beforeunload', onBeforeUnload);
  window.addEventListener('keydown', onKeydown);
  // 首屏自动适应窗口
  setTimeout(() => {
    const el = document.querySelector('.pdf-viewport') as HTMLElement | null;
    if (el) stage.value?.fit(el.clientWidth, el.clientHeight);
  }, 120);
});

// 停留在设计器页时切换模板（如返回列表后再次点「可视化设计」）需重新装载
watch(
  () => queryTemplateId(),
  (nv, ov) => {
    if (nv !== ov) void loadFromQuery();
  },
);

onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', onBeforeUnload);
  window.removeEventListener('keydown', onKeydown);
  if (heartbeatTimer) clearInterval(heartbeatTimer);
  if (store.templateId) {
    void releasePdfLockApi(store.templateId).catch(() => {});
  }
  if (store.dirty) saveDraft();
});

// 数据视图变化时重新取样例数据
function onDataModeChange() {
  void loadSampleData();
}
</script>

<template>
  <div class="pdf-designer">
    <Toolbar
      @align="onAlign"
      @back="router.push('/system/pdf-template')"
      @data-change="onDataModeChange"
      @export="onExportBundle"
      @import="onImportBundle"
      @import-pdf="showImportPdf = true"
      @page-preview="showPage = true"
      @precise-preview="showPrecise = true"
      @save="onSave"
      @save-as="onSaveAs"
      @set-default="onSetDefault"
      @versions="onShowVersions"
    />

    <div class="pd-body">
      <!-- ② 左面板 -->
      <div class="pd-left">
        <Tabs v-model:activeKey="leftTab" size="small" class="pd-left-tabs">
          <Tabs.TabPane key="elements">
            <template #tab><LucideShapes class="pd-tab-icon" />元素库</template>
            <PanelElements />
          </Tabs.TabPane>
          <Tabs.TabPane key="data">
            <template #tab><LucideListTree class="pd-tab-icon" />数据源</template>
            <PanelDataSource :loading="loading" :tree="fieldTree" />
          </Tabs.TabPane>
          <Tabs.TabPane key="layers">
            <template #tab><LucideLayers class="pd-tab-icon" />图层</template>
            <PanelLayers />
          </Tabs.TabPane>
        </Tabs>
      </div>

      <!-- ③ 中央画布 -->
      <div class="pd-center">
        <CanvasStage ref="stage" />
      </div>

      <!-- ④ 右属性面板 -->
      <div class="pd-right">
        <div class="pd-right-head">
          <span>属性</span>
          <Tag v-if="!store.designable" color="orange">该类型不支持自定义版式</Tag>
        </div>
        <PanelProps :assets="assets" :field-tree="fieldTree" @pick-asset="onPickAsset" />
      </div>
    </div>

    <!-- ⑤ 状态栏 -->
    <StatusBar />

    <!-- 弹层 -->
    <DialogPrecisePreview v-model:open="showPrecise" />
    <DialogPagePreview v-model:open="showPage" />
    <DialogImportPdf
      v-model:open="showImportPdf"
      :doc-type="store.docType"
      @done="onImportPdfDone"
    />

    <Modal
      v-model:open="showVersions"
      :footer="null"
      title="版本历史"
      width="720px"
    >
      <div v-if="versions.length === 0" class="pd-ver-empty">暂无历史版本</div>
      <div v-for="v in versions" :key="v.id" class="pd-ver-item">
        <div class="pd-ver-main">
          <Tag color="blue">v{{ v.version }}</Tag>
          <span class="pd-ver-note">{{ v.changeNote || '—' }}</span>
        </div>
        <div class="pd-ver-side">
          <span class="pd-ver-time">{{ v.createTime }}</span>
          <Button size="small" type="link" @click="onRollback(v.version)">回滚</Button>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.pdf-designer {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: #f0f2f5;
}
.pd-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.pd-left {
  display: flex;
  flex-direction: column;
  width: 280px;
  background: #fff;
  border-right: 1px solid #f0f0f0;
}
.pd-left-tabs {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.pd-left-tabs :deep(.ant-tabs-nav) {
  margin: 0;
  padding: 0 6px;
}
.pd-left-tabs :deep(.ant-tabs-content-holder) {
  flex: 1;
  overflow: hidden;
}
.pd-left-tabs :deep(.ant-tabs-content) {
  height: 100%;
}
.pd-left-tabs :deep(.ant-tabs-tabpane) {
  height: 100%;
  overflow: hidden;
}
.pd-tab-icon {
  width: 13px;
  height: 13px;
  margin-right: 2px;
}
.pd-center {
  flex: 1;
  overflow: hidden;
}
.pd-right {
  display: flex;
  flex-direction: column;
  width: 320px;
  background: #fff;
  border-left: 1px solid #f0f0f0;
}
.pd-right-head {
  display: flex;
  gap: 8px;
  align-items: center;
  height: 34px;
  padding: 0 10px;
  font-size: 13px;
  font-weight: 500;
  color: #262626;
  border-bottom: 1px solid #f0f0f0;
}
.pd-right > :deep(.panel-props) {
  flex: 1;
  padding-top: 6px;
}
.pd-ver-empty {
  padding: 30px;
  font-size: 12px;
  color: #bfbfbf;
  text-align: center;
}
.pd-ver-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px;
  border-bottom: 1px solid #f5f5f5;
}
.pd-ver-main {
  display: flex;
  gap: 8px;
  align-items: center;
}
.pd-ver-note {
  font-size: 12px;
  color: #595959;
}
.pd-ver-side {
  display: flex;
  gap: 8px;
  align-items: center;
}
.pd-ver-time {
  font-size: 11px;
  color: #bfbfbf;
}
</style>
