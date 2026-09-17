<script lang="ts" setup>
/**
 * 表单/详情布局设计器（自定义模块进阶一期）
 * 左侧字段池（系统字段+自定义字段，来自 field schema），右侧画布：
 * 选项卡（可增删改名）→ 画布区（vuedraggable 拖拽入池/排序），字段可切半行/整行。
 * 无选项卡时为单一"默认区"。未编排字段渲染端自动按 append 追加（新字段永不丢失）。
 */
import { computed, h, onMounted, reactive, ref } from 'vue';


import { IconifyIcon } from '@vben/icons';

import {
  Badge,
  Button,
  Card,
  Input,
  message,
  Modal,
  Select,
  Spin,
  TabPane,
  Tabs,
  Tag,
  Tooltip,
} from 'ant-design-vue';
import draggable from 'vuedraggable';

import {
  getFormLayoutApi,
  resetFormLayoutApi,
  saveFormLayoutApi,
} from '#/api/core/system/form-layout';
import { getFieldSchemaApi } from '#/api/core/system/field';
import { getRoleOptionsApi } from '#/api/core/system/role';

interface FieldSchemaItem {
  fieldKey: string;
  fieldLabel: string;
  fieldType: number;
  required?: number;
  isSystem?: number;
  options?: any;
}

interface CanvasField {
  key: string;
  source: 'system' | 'field_def';
  span: 1 | 2;
}

interface CanvasTab {
  key: string;
  title: string;
}

const LAYOUT_TYPES = [
  { value: 1, label: '表单布局' },
  { value: 2, label: '详情布局' },
];

// 抽屉宽度（仅表单布局）：不改=保持现用宽度
const WIDTH_OPTIONS = [
  { label: '默认宽度', value: '' },
  { label: '60%', value: '60%' },
  { label: '70%', value: '70%' },
  { label: '75%', value: '75%' },
  { label: '80%', value: '80%' },
  { label: '90%', value: '90%' },
];

const props = withDefaults(
  defineProps<{ module: string; initialLayoutType?: number }>(),
  { initialLayoutType: 1 },
);
const module = computed(() => props.module);
const activeLayout = ref<number>(props.initialLayoutType);
// D7 角色差异化：'' = 默认布局；否则为角色专属布局
const roleScope = ref('');
const roleOptions = ref<{ label: string; value: string }[]>([]);
const MODULE_NAMES: Record<string, string> = {
  crm_customer: '客户模型',
  crm_lead: '线索模型',
  crm_contact: '联系人模型',
  crm_opportunity: '商机模型',
  sale_quotation: '报价模型',
  sale_order: '订单模型',
  crm_contract: '合同模型',
};
const moduleName = computed(() => MODULE_NAMES[props.module] ?? props.module);
const loading = ref(false);
const saving = ref(false);

/** 字段元数据（两套布局共用同一字段池） */
const schemaItems = ref<FieldSchemaItem[]>([]);
/** 字段池：真实数组，与画布区共享 group 互拖（computed 会破坏 draggable 的 splice 语义） */
const poolFields = ref<CanvasField[]>([]);
/** 各布局类型的画布状态，key=layoutType */
const canvases = reactive<
  Record<
    number,
    {
      tabs: CanvasTab[];
      areas: CanvasField[][];
      version: number;
      drawerWidth: string;
    }
  >
>({
  1: { tabs: [], areas: [[]], version: 0, drawerWidth: '' },
  2: { tabs: [], areas: [[]], version: 0, drawerWidth: '' },
});

const itemMap = computed(() => {
  const m = new Map<string, FieldSchemaItem>();
  for (const it of schemaItems.value) m.set(it.fieldKey, it);
  return m;
});

/** 依据画布占用重建字段池（loadLayout/loadSchema 后调用） */
function rebuildPool() {
  const cv = canvases[activeLayout.value];
  const assigned = new Set<string>();
  for (const area of cv.areas) for (const f of area) assigned.add(f.key);
  poolFields.value = schemaItems.value
    .filter((it) => !assigned.has(it.fieldKey))
    .map((it) => ({
      key: it.fieldKey,
      source: Number(it.isSystem) === 1 ? ('system' as const) : ('field_def' as const),
      span: 1 as 1 | 2,
    }));
}

function labelOf(key: string): string {
  return itemMap.value.get(key)?.fieldLabel ?? key;
}
function isSystemField(key: string): boolean {
  return Number(itemMap.value.get(key)?.isSystem) === 1;
}

async function loadSchema() {
  loading.value = true;
  try {
    const res: any = await getFieldSchemaApi(module.value);
    const list = Array.isArray(res?.list) ? res.list : [];
    schemaItems.value = list;
    // schema 变化后，画布上已不存在的字段清出（字段被删等场景）
    const keys = new Set(list.map((i: any) => i.fieldKey));
    for (const lt of [1, 2]) {
      const cv = canvases[lt];
      cv.areas = cv.areas.map((area) =>
        area.filter((f) => keys.has(f.key)),
      );
    }
  } finally {
    loading.value = false;
  }
}

async function loadLayout(layoutType: number) {
  const res: any = await getFormLayoutApi(
    module.value,
    layoutType,
    roleScope.value || undefined,
  );
  const cv = canvases[layoutType];
  const layout = res?.layoutJson;
  if (!layout) {
    // 无布局：单默认区 + 字段池兜底
    cv.tabs = [];
    cv.areas = [[]];
    cv.version = 0;
    return;
  }
  cv.version = Number(res.version) || 1;
  cv.drawerWidth = String(layout.drawerWidth ?? '');
  const tabs: CanvasTab[] = Array.isArray(layout.tabs)
    ? layout.tabs.map((t: any) => ({ key: String(t.key), title: String(t.title) }))
    : [];
  const tabKeys = new Set(tabs.map((t) => t.key));
  const areas: CanvasField[][] = tabs.map(() => []);
  let defaultArea: CanvasField[] = [];
  for (const f of layout.fields ?? []) {
    const cf: CanvasField = {
      key: String(f.key),
      source: f.source === 'system' ? 'system' : 'field_def',
      span: Number(f.span) === 2 ? 2 : 1,
    };
    if (f.tab && tabKeys.has(f.tab)) {
      areas[tabs.findIndex((t) => t.key === f.tab)].push(cf);
    } else {
      defaultArea.push(cf);
    }
  }
  cv.tabs = tabs;
  cv.areas = tabs.length > 0 ? [defaultArea, ...areas] : [defaultArea];
}

/** 画布区标题：第一区为默认区（无选项卡语义），其余对应 tabs */
function areaTitle(index: number): string {
  const cv = canvases[activeLayout.value];
  if (cv.tabs.length === 0) return '默认区（表单主体）';
  return index === 0 ? '默认区（未归入选项卡的字段）' : cv.tabs[index - 1]?.title ?? `选项卡 ${index}`;
}

function addTab() {
  const cv = canvases[activeLayout.value];
  Modal.confirm({
    title: '新增选项卡',
    okText: '确定',
    cancelText: '取消',
    content: h('div', null, [
      h('p', { style: 'margin-bottom:8px' }, '选项卡标题：'),
      h(Input, {
        placeholder: '如：扩展信息',
        onChange: (e: any) => (newTabTitle = e?.target?.value ?? ''),
      }),
    ]),
    onOk: () => {
      const title = (newTabTitle || '').trim();
      if (!title) {
        message.warning('选项卡标题不能为空');
        return Promise.reject();
      }
      const key = `tab_${Date.now()}`;
      cv.tabs.push({ key, title });
      cv.areas.push([]);
      newTabTitle = '';
    },
  });
}
let newTabTitle = '';

function renameTab(index: number) {
  const cv = canvases[activeLayout.value];
  const tab = cv.tabs[index - 1];
  if (!tab) return;
  let val = tab.title;
  Modal.confirm({
    title: '重命名选项卡',
    okText: '确定',
    cancelText: '取消',
    content: h(Input, {
      value: val,
      onChange: (e: any) => (val = e?.target?.value ?? val),
    }),
    onOk: () => {
      const title = val.trim();
      if (!title) {
        message.warning('选项卡标题不能为空');
        return Promise.reject();
      }
      tab.title = title;
    },
  });
}

function removeTab(index: number) {
  const cv = canvases[activeLayout.value];
  const area = cv.areas[index];
  // 区内字段退回默认区
  cv.areas[0].push(...(area ?? []));
  cv.areas.splice(index, 1);
  cv.tabs.splice(index - 1, 1);
}

function toggleSpan(f: CanvasField) {
  f.span = f.span === 1 ? 2 : 1;
}

async function handleSave() {
  const cv = canvases[activeLayout.value];
  // 组装 fields：默认区在前，随后各选项卡区；sort 即数组序
  const fields: any[] = [];
  cv.areas.forEach((area, areaIndex) => {
    const tab = cv.tabs.length > 0 && areaIndex > 0 ? cv.tabs[areaIndex - 1]?.key : undefined;
    area.forEach((f, i) => {
      fields.push({
        key: f.key,
        source: f.source,
        tab: tab ?? undefined,
        span: f.span,
        sort: fields.length + i + 1,
      });
    });
  });
  if (fields.length === 0) {
    message.warning('请先将字段拖入画布');
    return;
  }
  saving.value = true;
  try {
    await saveFormLayoutApi({
      module: module.value,
      layoutType: activeLayout.value,
      roleKey: roleScope.value || undefined,
      version: cv.version || undefined,
      layoutJson: {
        version: 1,
        tabs: cv.tabs,
        fields,
        unassignedPolicy: 'append',
        ...(activeLayout.value === 1 ? { drawerWidth: cv.drawerWidth || undefined } : {}),
      },
    });
    message.success('布局已保存，前端即时生效');
    await loadLayout(activeLayout.value);
    rebuildPool();
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  } finally {
    saving.value = false;
  }
}

async function handleReset() {
  Modal.confirm({
    title: '恢复默认布局',
    content: '将删除当前布局，页面回到系统默认渲染。确定继续？',
    okText: '确定',
    cancelText: '取消',
    onOk: async () => {
      await resetFormLayoutApi(
        module.value,
        activeLayout.value,
        roleScope.value || undefined,
      );
      message.success('已恢复默认布局');
      canvases[activeLayout.value] = { tabs: [], areas: [[]], version: 0, drawerWidth: '' };
      await loadLayout(activeLayout.value);
      rebuildPool();
    },
  });
}

/** 预览：按画布顺序 + 半行/整行模拟 24 栅格 */
const previewVisible = ref(false);
const previewRows = computed(() => {
  const cv = canvases[activeLayout.value];
  const rows: Array<{ label: string; span: number }[]> = [];
  let current: { label: string; span: number }[] = [];
  let used = 0;
  cv.areas.forEach((area) => {
    for (const f of area) {
      const weight = f.span === 2 ? 24 : 12;
      if (used + weight > 24) {
        rows.push(current);
        current = [];
        used = 0;
      }
      current.push({ label: labelOf(f.key), span: weight });
      used += weight;
    }
  });
  if (current.length > 0) rows.push(current);
  return rows;
});


/** 表单/详情页签切换：两套画布独立，切走时按当前画布重建池 */
function onTabChange() {
  rebuildPool();
}

onMounted(async () => {
  try {
    const opts: any = await getRoleOptionsApi();
    const list = Array.isArray(opts) ? opts : (opts?.list ?? []);
    roleOptions.value = list.map((r: any) => ({
      label: r.label || r.roleName || r.name,
      value: r.value || r.roleKey || r.key,
    }));
  } catch {
    roleOptions.value = [];
  }
  await loadSchema();
  await loadLayout(1);
  await loadLayout(2);
  rebuildPool();
});

async function onRoleScopeChange() {
  await loadLayout(1);
  await loadLayout(2);
  rebuildPool();
}
</script>

<template>
  <div>
    <Card class="mb-4">
      <div class="flex flex-wrap items-center gap-3">
        <span class="text-base font-medium">{{ moduleName }} · 模板设置</span>
        <template v-if="activeLayout === 1">
          <span class="ml-2">抽屉宽度：</span>
          <Select
            v-model:value="canvases[1].drawerWidth"
            :options="WIDTH_OPTIONS"
            class="w-32"
          />
        </template>
        <span class="ml-2">适用范围：</span>
        <Select
          v-model:value="roleScope"
          :options="[
            { label: '默认布局（全部用户）', value: '' },
            ...roleOptions,
          ]"
          class="w-56"
          @change="onRoleScopeChange"
        />
        <Tooltip title="未拖入画布的字段，渲染时按默认规则自动追加在布局之后，新字段永不丢失；用户命中角色专属布局时优先于默认布局生效">
          <IconifyIcon icon="lucide:info" class="text-muted-foreground" />
        </Tooltip>
        <div class="flex-1"></div>
        <Button :loading="saving" type="primary" @click="handleSave">
          <template #icon><IconifyIcon icon="lucide:save" /></template>
          保存布局
        </Button>
        <Button danger @click="handleReset">
          <template #icon><IconifyIcon icon="lucide:rotate-ccw" /></template>
          恢复默认
        </Button>
        <Button @click="previewVisible = true">
          <template #icon><IconifyIcon icon="lucide:eye" /></template>
          预览
        </Button>
      </div>
    </Card>

    <Card>
      <Tabs :activeKey="String(activeLayout)" @change="onTabChange">
        <TabPane v-for="lt in LAYOUT_TYPES" :key="String(lt.value)" :tab="lt.label" />
      </Tabs>
      <Spin :spinning="loading">
        <div class="flex flex-col gap-4 md:flex-row">
          <!-- 字段池 -->
          <div class="w-full shrink-0 md:w-72">
            <Card size="small" title="字段池（拖入右侧画布）">
              <draggable
                :list="poolFields"
                item-key="fieldKey"
                group="formLayoutFields"
                :animation="200"
                class="min-h-24 space-y-2"
              >
                <template #item="{ element }">
                  <div
                    class="pool-item flex items-center gap-2 rounded border border-border px-2 py-1.5 transition-colors hover:border-primary/50"
                  >
                    <IconifyIcon icon="lucide:grip-vertical" class="text-muted-foreground" />
                    <span class="flex-1 truncate">{{ labelOf(element.key) }}</span>
                    <Tag v-if="isSystemField(element.key)" color="blue">系统</Tag>
                    <Tag v-if="Number(itemMap.get(element.key)?.required) === 1" color="red">必填</Tag>
                  </div>
                </template>
              </draggable>
            </Card>
          </div>

          <!-- 画布 -->
          <div class="min-w-0 flex-1 space-y-4">
            <div
              v-for="(area, areaIndex) in canvases[activeLayout].areas"
              :key="areaIndex"
            >
              <div class="mb-2 flex items-center gap-2">
                <span class="font-medium">{{ areaTitle(areaIndex) }}</span>
                <span class="text-xs text-muted-foreground">（点「半行/整行」切换该字段占一行还是半行）</span>
                <template v-if="areaIndex > 0">
                  <Button size="small" type="link" @click="renameTab(areaIndex)">改名</Button>
                  <Button size="small" type="link" danger @click="removeTab(areaIndex)">删除选项卡</Button>
                </template>
              </div>
              <div class="rounded border border-dashed border-border p-3">
                <draggable
                  :list="canvases[activeLayout].areas[areaIndex]"
                  item-key="key"
                  group="formLayoutFields"
                  :animation="200"
                  class="grid grid-cols-2 gap-2"
                >
                  <template #item="{ element }">
                    <div
                      class="flex items-center gap-2 rounded border border-primary/40 bg-primary/10 px-2 py-1.5"
                      :class="element.span === 2 ? 'col-span-2' : 'col-span-1'"
                    >
                      <IconifyIcon icon="lucide:grip-vertical" class="text-muted-foreground" />
                      <span class="flex-1 truncate">{{ labelOf(element.key) }}</span>
                      <Badge v-if="isSystemField(element.key)" text="系统" color="blue" />
                      <Button
                        size="small"
                        :type="element.span === 2 ? 'primary' : 'default'"
                        class="px-1"
                        @click="toggleSpan(element)"
                      >
                        {{ element.span === 1 ? '半行' : '整行' }}
                      </Button>
                    </div>
                  </template>
                </draggable>
                <div
                  v-if="(!canvases[activeLayout].areas[areaIndex] || canvases[activeLayout].areas[areaIndex].length === 0)"
                  class="py-6 text-center text-muted-foreground/60"
                >
                  拖字段到这里
                </div>
              </div>
            </div>
            <Button dashed block @click="addTab">
              <template #icon><IconifyIcon icon="lucide:plus" /></template>
              添加选项卡
            </Button>
          </div>
        </div>
      </Spin>
    </Card>

    <Modal v-model:open="previewVisible" :footer="null" title="布局预览" width="640px">
      <div class="space-y-2">
        <template v-if="canvases[activeLayout].tabs.length > 0">
          <Tabs>
            <TabPane
              v-for="(tab, i) in canvases[activeLayout].tabs"
              :key="tab.key"
              :tab="tab.title"
            >
              <div class="grid grid-cols-2 gap-3 rounded border p-3">
                <div
                  v-for="f in canvases[activeLayout].areas[i + 1] ?? []"
                  :key="f.key"
                  class="rounded bg-muted/50 px-3 py-2 text-sm"
                  :class="f.span === 2 ? 'col-span-2' : 'col-span-1'"
                >
                  {{ labelOf(f.key) }}
                </div>
                <div
                  v-if="(canvases[activeLayout].areas[i + 1] ?? []).length === 0"
                  class="col-span-2 py-4 text-center text-muted-foreground/60"
                >
                  无字段
                </div>
              </div>
            </TabPane>
          </Tabs>
        </template>
        <div
          v-for="(row, ri) in previewRows"
          :key="ri"
          class="grid grid-cols-2 gap-3 rounded border border-border p-3"
        >
          <div
            v-for="(cell, ci) in row"
            :key="ci"
            class="rounded bg-muted/50 px-3 py-2 text-sm"
            :class="cell.span === 24 ? 'col-span-2' : 'col-span-1'"
          >
            {{ cell.label }}
          </div>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.pool-item {
  cursor: move;
}
</style>
