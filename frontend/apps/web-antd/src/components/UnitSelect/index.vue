<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import type { ProductUnitOption } from '#/api/core/product/product-unit';

import { LucideChevronDown, LucideTrash2, LucideX } from '@vben/icons';

import {
  Button,
  Checkbox,
  Input,
  Popconfirm,
  Popover,
  message,
} from 'ant-design-vue';

import { useProductUnits } from './use-product-units';

const props = withDefaults(
  defineProps<{
    placeholder?: string;
    value?: string;
  }>(),
  {
    value: '',
    placeholder: '选择单位',
  },
);

const emit = defineEmits<{
  (e: 'change', value: string): void;
  (e: 'update:value', value: string): void;
}>();

const { addUnit, ensureUnits, precisionOf, removeUnit, units, updateUnit } =
  useProductUnits();

const open = ref(false);
const manage = ref(false);
const adding = ref(false);
const savingId = ref('');
const removingId = ref('');
const newName = ref('');
const newDecimal = ref(false);

watch(open, (val) => {
  if (val) {
    ensureUnits();
  } else {
    manage.value = false;
  }
});

const currentPrecision = computed(() => precisionOf(props.value));

const hintText = computed(() => {
  if (!props.value) return '选择单位后，出入库数量自动按单位精度校验';
  return currentPrecision.value === 0
    ? `${props.value}：数量按整数录入`
    : `${props.value}：数量最多 ${currentPrecision.value} 位小数`;
});

function setValue(name: string) {
  emit('update:value', name);
  emit('change', name);
}

function pick(name: string) {
  setValue(name);
  open.value = false;
}

function clear() {
  setValue('');
}

async function submitAdd() {
  const name = newName.value.trim();
  if (!name) {
    message.warning('请输入单位名称');
    return;
  }
  if (units.value.some((item) => item.name === name)) {
    newName.value = '';
    newDecimal.value = false;
    message.info(`单位「${name}」已存在，已为你选中`);
    pick(name);
    return;
  }
  adding.value = true;
  try {
    await addUnit(name, newDecimal.value ? 2 : 0);
    message.success(`单位「${name}」已添加`);
    newName.value = '';
    newDecimal.value = false;
    pick(name);
  } catch (error: any) {
    message.error(error?.message || '添加失败');
  } finally {
    adding.value = false;
  }
}

async function toggleDecimal(item: ProductUnitOption, decimal: boolean) {
  if (savingId.value) return;
  const target = decimal ? 2 : 0;
  if ((item.decimalPlaces ?? 0) === target) return;
  savingId.value = item.id;
  try {
    await updateUnit(item.id, target);
    message.success(`已保存：${item.name}${decimal ? '可小数' : '仅整数'}`);
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  } finally {
    savingId.value = '';
  }
}

async function submitDelete(item: ProductUnitOption) {
  removingId.value = item.id;
  try {
    await removeUnit(item.id);
    message.success(`单位「${item.name}」已删除`);
    if (props.value === item.name) {
      setValue('');
    }
  } catch (error: any) {
    message.error(error?.message || '删除失败');
  } finally {
    removingId.value = '';
  }
}
</script>

<template>
  <Popover
    v-model:open="open"
    trigger="click"
    placement="bottomLeft"
    :arrow="false"
    :overlay-style="{ width: '320px' }"
  >
    <template #content>
      <div class="unit-panel">
        <div class="unit-panel-head">
          <span class="unit-panel-title">
            {{ manage ? '管理单位' : '选择单位' }}
          </span>
          <button
            type="button"
            class="unit-manage-toggle"
            @click="manage = !manage"
          >
            {{ manage ? '完成' : '管理' }}
          </button>
        </div>
        <div v-if="!manage && units.length > 0" class="unit-chips">
          <button
            v-for="item in units"
            :key="item.id"
            type="button"
            class="unit-chip"
            :class="{ 'unit-chip--active': item.name === value }"
            :title="precisionOf(item.name) === 0 ? '整数录入' : '可两位小数'"
            @click="pick(item.name || '')"
          >
            {{ item.name }}
          </button>
        </div>
        <div v-if="!manage && units.length === 0" class="unit-empty">
          暂无单位，请在下方添加
        </div>
        <div v-if="manage" class="unit-manage-list">
          <div v-for="item in units" :key="item.id" class="unit-manage-row">
            <span class="unit-manage-name">
              {{ item.name }}
              <span v-if="item.isDefault" class="unit-manage-default">默认</span>
            </span>
            <Checkbox
              class="unit-manage-decimal"
              :checked="(item.decimalPlaces ?? 0) > 0"
              :disabled="savingId === item.id"
              @change="(e: any) => toggleDecimal(item, !!e.target?.checked)"
            >
              可小数
            </Checkbox>
            <Popconfirm
              title="确定删除该单位？"
              :disabled="item.isDefault"
              @confirm="submitDelete(item)"
            >
              <button
                type="button"
                class="unit-manage-delete"
                :disabled="item.isDefault || removingId === item.id"
                :title="item.isDefault ? '默认单位不能删除' : '删除单位'"
              >
                <LucideTrash2 />
              </button>
            </Popconfirm>
          </div>
          <div v-if="units.length === 0" class="unit-empty">暂无单位</div>
        </div>
        <div class="unit-hint">{{ hintText }}</div>
        <div class="unit-add">
          <Input
            v-model:value="newName"
            size="small"
            placeholder="新单位，如：公斤"
            :maxlength="20"
            @press-enter="submitAdd"
          />
          <Checkbox v-model:checked="newDecimal" class="unit-add-decimal">
            可小数
          </Checkbox>
          <Button
            size="small"
            type="primary"
            :loading="adding"
            @click="submitAdd"
          >
            添加
          </Button>
        </div>
      </div>
    </template>

    <div
      class="unit-trigger"
      :class="{ 'unit-trigger--empty': !value, 'unit-trigger--open': open }"
    >
      <span class="unit-trigger-text">{{ value || placeholder }}</span>
      <span class="unit-trigger-icons">
        <LucideX
          v-if="value"
          class="unit-trigger-clear"
          @click.stop="clear"
        />
        <LucideChevronDown
          class="unit-trigger-arrow"
          :class="{ 'unit-trigger-arrow--open': open }"
        />
      </span>
    </div>
  </Popover>
</template>

<style scoped>
.unit-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.unit-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.unit-chip {
  padding: 3px 12px;
  font-size: 13px;
  line-height: 20px;
  color: hsl(var(--foreground));
  cursor: pointer;
  background: hsl(var(--accent));
  border: 1px solid transparent;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.unit-chip:hover {
  color: hsl(var(--primary));
  border-color: hsl(var(--primary));
}

.unit-chip--active {
  color: hsl(var(--primary-foreground));
  background: hsl(var(--primary));
}

.unit-chip--active:hover {
  color: hsl(var(--primary-foreground));
}

.unit-empty {
  padding: 8px 0;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}

.unit-hint {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.unit-add {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 10px;
  border-top: 1px solid hsl(var(--border));
}

.unit-add-decimal {
  flex-shrink: 0;
  font-size: 13px;
}

.unit-panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.unit-panel-title {
  font-size: 13px;
  font-weight: 600;
}

.unit-manage-toggle {
  padding: 0;
  font-size: 12px;
  color: hsl(var(--primary));
  cursor: pointer;
  user-select: none;
  background: none;
  border: none;
}

.unit-manage-toggle:hover {
  opacity: 0.8;
}

.unit-manage-list {
  display: flex;
  flex-direction: column;
  max-height: 240px;
  overflow-y: auto;
}

.unit-manage-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}

.unit-manage-row + .unit-manage-row {
  border-top: 1px solid hsl(var(--border));
}

.unit-manage-name {
  flex: 1;
  overflow: hidden;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unit-manage-default {
  margin-left: 4px;
  padding: 0 4px;
  font-size: 11px;
  line-height: 16px;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 10%);
  border-radius: 4px;
}

.unit-manage-decimal {
  flex-shrink: 0;
  font-size: 12px;
}

.unit-manage-delete {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  background: none;
  border: none;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.unit-manage-delete svg {
  width: 14px;
  height: 14px;
}

.unit-manage-delete:hover:not(:disabled) {
  color: hsl(var(--destructive));
  background: hsl(var(--destructive) / 10%);
}

.unit-manage-delete:disabled {
  cursor: not-allowed;
  opacity: 0.35;
}

.unit-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
  width: 100%;
  height: 32px;
  padding: 4px 11px;
  font-size: 14px;
  color: hsl(var(--foreground));
  cursor: pointer;
  user-select: none;
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.unit-trigger:hover,
.unit-trigger--open {
  border-color: hsl(var(--primary));
}

.unit-trigger--empty {
  color: hsl(var(--muted-foreground));
}

.unit-trigger-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unit-trigger-icons {
  display: flex;
  flex-shrink: 0;
  gap: 2px;
  align-items: center;
}

.unit-trigger-clear {
  width: 14px;
  height: 14px;
  color: hsl(var(--muted-foreground));
  opacity: 0;
  transition: opacity 0.15s;
}

.unit-trigger-clear:hover {
  color: hsl(var(--foreground));
}

.unit-trigger:hover .unit-trigger-clear,
.unit-trigger--open .unit-trigger-clear {
  opacity: 1;
}

.unit-trigger-arrow {
  width: 14px;
  height: 14px;
  color: hsl(var(--muted-foreground));
  transition: transform 0.2s;
}

.unit-trigger-arrow--open {
  transform: rotate(180deg);
}
</style>
