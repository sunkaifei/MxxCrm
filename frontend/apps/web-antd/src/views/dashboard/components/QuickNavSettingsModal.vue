<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Spin, Switch } from 'ant-design-vue';

import {
  getMenusRouterApi,
  getQuickNavPreferenceApi,
  getSaleSimpleModeApi,
  saveQuickNavPreferenceApi,
  saveSaleSimpleModeApi,
} from '#/api';
import type { QuickNavItem } from '#/api';
import { $t } from '#/locales';

defineOptions({
  name: 'QuickNavSettingsModal',
});

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'saved'): void;
}>();

interface MenuOption {
  id: number;
  title: string;
  icon: string;
  path: string;
}

const loading = ref(false);
const saving = ref(false);
// 销售简易模式开关
const saleSimpleMode = ref(false);
// 已选模块（按 sort 排序，前 6 个显示）
const selectedItems = ref<MenuOption[]>([]);
// 可添加模块
const availableItems = ref<MenuOption[]>([]);

const visibleProxy = computed({
  get: () => props.visible,
  set: (val: boolean) => emit('update:visible', val),
});

// 展平菜单树：保留有 path 且是叶子节点（无 children）的项
function flattenMenus(menus: any[]): MenuOption[] {
  const result: MenuOption[] = [];
  const traverse = (list: any[]) => {
    if (!Array.isArray(list)) return;
    for (const menu of list) {
      if (!menu) continue;
      const children = menu.children || [];
      // 只收集叶子菜单（有 path 且无子节点）
      if (menu.path && children.length === 0) {
        const meta = menu.meta || {};
        // 标题优先用 meta.title（国际化 key），否则用 name
        const rawTitle = meta.title || menu.name || menu.path;
        const title =
          typeof rawTitle === 'string' && rawTitle.startsWith('page.')
            ? $t(rawTitle)
            : rawTitle;
        result.push({
          id: menu.id,
          title: typeof title === 'string' ? title : String(title || ''),
          icon: meta.icon || 'lucide:menu',
          path: menu.path,
        });
      }
      if (children.length > 0) {
        traverse(children);
      }
    }
  };
  traverse(menus);
  return result;
}

async function loadData() {
  loading.value = true;
  try {
    // 并行加载菜单、已保存配置、简易模式开关
    const [menuResp, prefResp, simpleMode]: any = await Promise.all([
      getMenusRouterApi({}),
      getQuickNavPreferenceApi().catch(() => null),
      getSaleSimpleModeApi().catch(() => false),
    ]);

    saleSimpleMode.value = !!simpleMode;

    const allMenus = flattenMenus(menuResp?.items || menuResp || []);
    const savedPref: QuickNavItem[] = Array.isArray(prefResp)
      ? prefResp
      : prefResp?.items || [];

    // 按已保存的配置排序选出已选模块
    const selectedMap = new Map<number, number>();
    savedPref.forEach((p) => {
      if (p?.menuId != null) selectedMap.set(p.menuId, p.sort ?? 0);
    });

    const selected: MenuOption[] = [];
    const available: MenuOption[] = [];

    // 先按 sort 顺序填充已选
    const sortedSelectedIds = [...selectedMap.entries()]
      .sort((a, b) => a[1] - b[1])
      .map(([id]) => id);

    for (const id of sortedSelectedIds) {
      const m = allMenus.find((x) => x.id === id);
      if (m) selected.push(m);
    }

    // 未选的进 available
    for (const m of allMenus) {
      if (!selectedMap.has(m.id)) {
        available.push(m);
      }
    }

    selectedItems.value = selected;
    availableItems.value = available;
  } catch (error: any) {
    message.error(error?.message || '加载菜单失败');
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      loadData();
    }
  },
);

// ===== 原生 HTML5 拖拽实现 =====
const dragIndex = ref(-1);
const dragFrom = ref<'selected' | 'available'>('selected');

function onDragStart(
  index: number,
  from: 'selected' | 'available',
  e: DragEvent,
) {
  dragIndex.value = index;
  dragFrom.value = from;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.dropEffect = 'move';
  }
}

function onDragOver(e: DragEvent) {
  // 允许 drop
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
}

function onDropSelected(targetIndex: number, e: DragEvent) {
  e.preventDefault();
  if (dragIndex.value < 0) return;

  if (dragFrom.value === 'selected') {
    // 已选列表内排序
    const list = [...selectedItems.value];
    const [moved] = list.splice(dragIndex.value, 1);
    if (moved) list.splice(targetIndex, 0, moved);
    selectedItems.value = list;
  } else if (dragFrom.value === 'available') {
    // 从可添加模块拖入已选
    const availList = [...availableItems.value];
    const [moved] = availList.splice(dragIndex.value, 1);
    if (moved) {
      const selList = [...selectedItems.value];
      selList.splice(targetIndex, 0, moved);
      selectedItems.value = selList;
      availableItems.value = availList;
    }
  }
  dragIndex.value = -1;
}

function onDropAvailable(targetIndex: number, e: DragEvent) {
  e.preventDefault();
  if (dragIndex.value < 0) return;

  if (dragFrom.value === 'selected') {
    // 从已选拖回可添加
    const selList = [...selectedItems.value];
    const [moved] = selList.splice(dragIndex.value, 1);
    if (moved) {
      const availList = [...availableItems.value];
      availList.splice(targetIndex, 0, moved);
      selectedItems.value = selList;
      availableItems.value = availList;
    }
  }
  dragIndex.value = -1;
}

// 点击按钮添加/移除
function addItem(item: MenuOption, index: number) {
  availableItems.value.splice(index, 1);
  selectedItems.value.push(item);
}

function removeItem(item: MenuOption, index: number) {
  selectedItems.value.splice(index, 1);
  availableItems.value.push(item);
}

function close() {
  emit('update:visible', false);
}

async function handleSave() {
  saving.value = true;
  try {
    const payload: QuickNavItem[] = selectedItems.value.map(
      (item, idx) => ({
        menuId: item.id,
        sort: idx,
      }),
    );
    // 并行保存快捷导航配置和简易模式开关
    await Promise.all([
      saveQuickNavPreferenceApi(payload),
      saveSaleSimpleModeApi(saleSimpleMode.value),
    ]);
    message.success('保存成功');
    emit('saved');
    close();
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Modal
    v-model:visible="visibleProxy"
    :title="$t('page.dashboard.customQuickNav')"
    width="600px"
    :footer="null"
    destroy-on-close
    @cancel="close"
  >
    <Spin :spinning="loading">
      <div class="space-y-4">
        <!-- 销售简易模式开关 -->
        <div class="flex items-center justify-between rounded border border-blue-100 bg-blue-50 px-3 py-2.5">
          <div class="flex items-center gap-2">
            <IconifyIcon icon="lucide:zap" class="size-4 shrink-0 text-blue-500" />
            <div>
              <div class="text-sm font-medium text-gray-800">
                {{ $t('page.dashboard.saleSimpleMode') }}
              </div>
              <div class="mt-0.5 text-xs text-gray-500">
                {{ $t('page.dashboard.saleSimpleModeDesc') }}
              </div>
            </div>
          </div>
          <Switch v-model:checked="saleSimpleMode" />
        </div>

        <!-- 已选模块 -->
        <div>
          <div class="mb-2 text-sm font-medium text-gray-700">
            {{ $t('page.dashboard.selectedModules') }}
          </div>
          <div class="selected-list space-y-1.5">
            <div
              v-for="(item, idx) in selectedItems"
              :key="item.id"
              draggable="true"
              class="drag-item flex cursor-move items-center justify-between rounded border border-gray-200 bg-white px-3 py-2 transition hover:border-blue-400 hover:shadow-sm"
              @dragstart="onDragStart(idx, 'selected', $event)"
              @dragover="onDragOver"
              @drop="onDropSelected(idx, $event)"
            >
              <div class="flex min-w-0 items-center gap-2">
                <span class="text-gray-400">⠿</span>
                <IconifyIcon :icon="item.icon" class="size-4 shrink-0" />
                <span class="truncate text-sm">{{ item.title }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span
                  class="inline-flex items-center gap-1 text-xs"
                  :class="
                    idx < 6
                      ? 'text-green-600'
                      : 'text-gray-400'
                  "
                >
                  <span
                    class="inline-block size-1.5 rounded-full"
                    :class="idx < 6 ? 'bg-green-500' : 'bg-gray-300'"
                  ></span>
                  {{ idx < 6 ? $t('page.dashboard.show') : $t('page.dashboard.hide') }}
                </span>
                <a
                  class="cursor-pointer text-xs text-red-500 hover:underline"
                  @click="removeItem(item, idx)"
                >
                  {{ $t('page.dashboard.remove') }}
                </a>
              </div>
            </div>
            <div
              v-if="selectedItems.length === 0"
              class="rounded border border-dashed border-gray-200 py-4 text-center text-xs text-gray-400"
            >
              暂无已选模块
            </div>
          </div>
        </div>

        <!-- 可添加模块 -->
        <div>
          <div class="mb-2 text-sm font-medium text-gray-700">
            {{ $t('page.dashboard.availableModules') }}
          </div>
          <div class="available-list space-y-1.5">
            <div
              v-for="(item, idx) in availableItems"
              :key="item.id"
              draggable="true"
              class="flex items-center justify-between rounded border border-gray-100 bg-gray-50 px-3 py-2"
              @dragstart="onDragStart(idx, 'available', $event)"
              @dragover="onDragOver"
              @drop="onDropAvailable(idx, $event)"
            >
              <div class="flex min-w-0 items-center gap-2">
                <IconifyIcon :icon="item.icon" class="size-4 shrink-0" />
                <span class="truncate text-sm text-gray-700">{{ item.title }}</span>
              </div>
              <a
                class="cursor-pointer text-xs text-blue-500 hover:underline"
                @click="addItem(item, idx)"
              >
                {{ $t('page.dashboard.add') }}
              </a>
            </div>
            <div
              v-if="availableItems.length === 0"
              class="rounded border border-dashed border-gray-200 py-4 text-center text-xs text-gray-400"
            >
              暂无可添加模块
            </div>
          </div>
        </div>

        <!-- 底部按钮 -->
        <div class="flex justify-end gap-2 border-t border-gray-100 pt-3">
          <Button @click="close">
            {{ $t('page.dashboard.cancel') }}
          </Button>
          <Button type="primary" :loading="saving" @click="handleSave">
            {{ $t('page.dashboard.save') }}
          </Button>
        </div>
      </div>
    </Spin>
  </Modal>
</template>

<style scoped>
.drag-item {
  user-select: none;
}

.drag-item[draggable='true'] {
  cursor: grab;
}

.drag-item:active {
  cursor: grabbing;
}
</style>
