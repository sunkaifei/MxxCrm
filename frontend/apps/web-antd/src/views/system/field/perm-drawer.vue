<script lang="ts" setup>
import type { FieldPermItem } from '#/api';

import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import { LucideMaximize2, LucideMinimize2 } from '@vben/icons';

import { Button, Select, Spin, message } from 'ant-design-vue';

import {
  getFieldPermListApi,
  getRoleOptionsApi,
  saveFieldPermApi,
} from '#/api';

// 业务模块固定为 v45 白名单登记的 3 个 CRM 模块（与 sql/v45_field_perm.sql 对齐）
const PERM_MODULES = [
  { label: '客户', value: 'crm_customer' },
  { label: '联系人', value: 'crm_contact' },
  { label: '合同', value: 'crm_contract' },
];

/** 行草稿模型：visibleRoles/editableRoles 保留后端原始值（null=未配置），Draft 供下拉编辑 */
interface PermRow extends FieldPermItem {
  editableDraft: string[];
  visibleDraft: string[];
}

const module = ref<string>('crm_customer');
const rows = ref<PermRow[]>([]);
const roleOptions = ref<Array<{ label: string; value: string }>>([]);
const listLoading = ref(false);
const isFullscreen = ref(false);

const dirtyCount = computed(() => rows.value.filter(isRowDirty).length);

function origVisible(row: PermRow): string[] {
  return Array.isArray(row.visibleRoles) ? row.visibleRoles.map(String) : [];
}

function origEditable(row: PermRow): string[] {
  return Array.isArray(row.editableRoles) ? row.editableRoles.map(String) : [];
}

function isRowDirty(row: PermRow): boolean {
  return (
    JSON.stringify([...row.visibleDraft].sort()) !==
      JSON.stringify([...origVisible(row)].sort()) ||
    JSON.stringify([...row.editableDraft].sort()) !==
      JSON.stringify([...origEditable(row)].sort())
  );
}

async function loadRows() {
  listLoading.value = true;
  try {
    const res: any = await getFieldPermListApi(module.value);
    const list = Array.isArray(res) ? res : (res?.list ?? []);
    rows.value = list.map((r: any) => ({
      editableDraft: Array.isArray(r.editableRoles)
        ? r.editableRoles.map(String)
        : [],
      editableRoles: Array.isArray(r.editableRoles)
        ? r.editableRoles.map(String)
        : null,
      fieldKey: String(r.fieldKey ?? ''),
      fieldLabel: String(r.fieldLabel ?? r.fieldKey ?? ''),
      id: r.id,
      module: String(r.module ?? ''),
      updateBy: r.updateBy ?? null,
      updateTime: r.updateTime ?? null,
      visibleDraft: Array.isArray(r.visibleRoles)
        ? r.visibleRoles.map(String)
        : [],
      visibleRoles: Array.isArray(r.visibleRoles)
        ? r.visibleRoles.map(String)
        : null,
    }));
  } catch {
    /* 错误由全局拦截器处理 */
  } finally {
    listLoading.value = false;
  }
}

async function loadRoleOptions() {
  try {
    const res: any = await getRoleOptionsApi();
    const list = Array.isArray(res) ? res : (res?.list ?? []);
    roleOptions.value = list.map((r: any) => ({
      label: String(r.label ?? r.name ?? r.value ?? r.id),
      value: String(r.value ?? r.id),
    }));
  } catch {
    /* ignore */
  }
}

/** 空数组归一为 null（未配置语义，后端同样做归一化，双保险） */
function normalizeRoles(arr: string[]): string[] | null {
  return arr.length > 0 ? [...arr] : null;
}

async function handleConfirm() {
  const dirty = rows.value.filter(isRowDirty);
  if (dirty.length === 0) {
    message.info('没有需要保存的修改');
    return;
  }
  setLoading(true);
  try {
    for (const row of dirty) {
      await saveFieldPermApi({
        editableRoles: normalizeRoles(row.editableDraft),
        fieldKey: row.fieldKey,
        module: module.value,
        visibleRoles: normalizeRoles(row.visibleDraft),
      });
    }
    message.success(`已保存 ${dirty.length} 项字段权限配置`);
    drawerApi.close();
  } catch {
    // 错误由全局拦截器处理，保留抽屉以便修改后重试
  } finally {
    setLoading(false);
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  onConfirm: handleConfirm,
  onOpenChange(isOpen) {
    if (!isOpen) return;
    loadRoleOptions();
    loadRows();
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer
    :class="isFullscreen ? 'w-full! max-w-full!' : 'w-[880px]! max-w-[880px]!'"
    title="标准字段权限配置"
  >
    <template #extra>
      <Button type="text" @click="isFullscreen = !isFullscreen">
        <LucideMinimize2 v-if="isFullscreen" class="size-4" />
        <LucideMaximize2 v-else class="size-4" />
      </Button>
    </template>

    <div class="perm-toolbar">
      <Select
        v-model:value="module"
        :options="PERM_MODULES"
        style="width: 200px"
        @change="loadRows"
      />
      <span class="perm-toolbar-tip">
        {{
          dirtyCount > 0
            ? `有 ${dirtyCount} 项修改待保存（点击下方确定保存）`
            : '未配置=全部可见/跟随可见；配置后无权限角色看不到对应字段，也无法越权编辑'
        }}
      </span>
    </div>

    <Spin :spinning="listLoading" size="small">
      <div class="perm-list">
        <div class="perm-head">
          <span>字段</span>
          <span>可见角色</span>
          <span>可编辑角色</span>
          <span class="perm-head-state">状态</span>
        </div>
        <div
          v-for="row in rows"
          :key="row.fieldKey"
          class="perm-row"
          :class="{ 'perm-row--dirty': isRowDirty(row) }"
        >
          <div class="perm-name">
            <div class="perm-name-label">{{ row.fieldLabel }}</div>
            <div class="perm-name-key">{{ row.fieldKey }}</div>
          </div>
          <Select
            v-model:value="row.visibleDraft"
            :options="roleOptions"
            mode="multiple"
            placeholder="留空=全部可见"
            :max-tag-count="2"
            allow-clear
            size="small"
          />
          <Select
            v-model:value="row.editableDraft"
            :options="roleOptions"
            mode="multiple"
            placeholder="留空=跟随可见"
            :max-tag-count="2"
            allow-clear
            size="small"
          />
          <span class="perm-state">
            {{
              isRowDirty(row)
                ? '待保存'
                : row.visibleRoles
                  ? '已限制'
                  : '未配置'
            }}
          </span>
        </div>
        <div v-if="!listLoading && rows.length === 0" class="perm-empty">
          当前模块暂无敏感字段白名单
        </div>
      </div>
    </Spin>
  </Drawer>
</template>

<style scoped>
.perm-toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
}

.perm-toolbar-tip {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.perm-list {
  padding: 8px 10px;
  background: hsl(var(--accent) / 60%);
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.perm-head,
.perm-row {
  display: grid;
  grid-template-columns: 170px 1fr 1fr 70px;
  gap: 10px;
  align-items: center;
}

.perm-head {
  padding: 4px 0 8px;
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  border-bottom: 1px solid hsl(var(--border));
}

.perm-head-state {
  text-align: center;
}

.perm-row {
  padding: 8px 0;
  border-bottom: 1px dashed hsl(var(--border));
}

.perm-row:last-of-type {
  border-bottom: none;
}

.perm-row--dirty {
  padding-right: 6px;
  padding-left: 6px;
  margin: 0 -6px;
  background: hsl(var(--primary) / 8%);
  border-radius: 4px;
}

.perm-name-label {
  font-size: 13px;
  font-weight: 500;
  line-height: 18px;
}

.perm-name-key {
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 16px;
  color: hsl(var(--muted-foreground));
}

.perm-state {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

.perm-row--dirty .perm-state {
  font-weight: 500;
  color: hsl(var(--primary));
}

.perm-empty {
  padding: 24px 0;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  text-align: center;
}
</style>
