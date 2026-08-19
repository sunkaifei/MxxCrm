<script lang="ts" setup>
// 员工列表列显示配置抽屉：按四个查看级别（admin/hr/manager/employee）勾选可见列
import { reactive, ref } from 'vue';

import { useVbenDrawer } from '#/adapter/drawer';

import { $t } from '#/locales';

import { Button, Checkbox, Divider, message } from 'ant-design-vue';

interface ColumnOption {
  key: string;
  title: string;
}

interface LevelOption {
  key: string;
  label: string;
}

const saving = ref(false);
const drawerData = reactive<{
  config: Record<string, string[]>;
  defaultConfig: Record<string, string[]>;
  allColumns: ColumnOption[];
  levels: LevelOption[];
}>({
  config: {},
  defaultConfig: {},
  allColumns: [],
  levels: [],
});

// 当前角色勾选集合（levelKey -> string[]，与 config 同步）
const checkedMap = reactive<Record<string, string[]>>({});

const [Drawer, drawerApi] = useVbenDrawer({
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      const data = drawerApi.getData() as {
        config?: Record<string, string[]>;
        defaultConfig?: Record<string, string[]>;
        allColumns?: ColumnOption[];
        levels?: LevelOption[];
      };
      drawerData.config = data?.config ?? {};
      drawerData.defaultConfig = data?.defaultConfig ?? {};
      drawerData.allColumns = data?.allColumns ?? [];
      drawerData.levels = data?.levels ?? [];
      // 用配置值，缺省用默认值
      drawerData.levels.forEach((lv) => {
        const configured = drawerData.config[lv.key];
        checkedMap[lv.key] = [
          ...(configured && configured.length > 0
            ? configured
            : (drawerData.defaultConfig[lv.key] ?? [])),
        ];
      });
    }
  },
});

function handleReset(levelKey: string) {
  checkedMap[levelKey] = [...(drawerData.defaultConfig[levelKey] ?? [])];
}

async function handleSave() {
  saving.value = true;
  try {
    const config: Record<string, string[]> = {};
      drawerData.levels.forEach((lv) => {
        const selected = [...(checkedMap[lv.key] ?? [])];
        // 操作列为强制保留列：无论勾选状态如何，都保留在配置末尾
        if (!selected.includes('action')) selected.push('action');
        config[lv.key] = selected;
      });
    drawerApi.setData({ savedConfig: config });
    drawerApi.close();
    message.success($t('page.system.user.columnsConfig.saveSuccess'));
  } finally {
    saving.value = false;
  }
}

function handleCancel() {
  drawerApi.close();
}
</script>

<template>
  <Drawer
    :title="$t('page.system.user.columnsConfig.title')"
    :width="560"
    :show-footer="false"
  >
    <div class="config-hint">
      {{ $t('page.system.user.columnsConfig.hint') }}
    </div>

    <div v-for="lv in drawerData.levels" :key="lv.key" class="level-block">
      <div class="level-header">
        <span class="level-label">{{ lv.label }}</span>
        <Button size="small" type="link" @click="handleReset(lv.key)">
          {{ $t('page.system.user.columnsConfig.reset') }}
        </Button>
      </div>
      <Checkbox.Group
        v-model:value="checkedMap[lv.key]"
        class="column-group"
      >
        <Checkbox
          v-for="col in drawerData.allColumns"
          :key="col.key"
          :value="col.key"
          :disabled="col.key === 'action'"
          class="column-checkbox"
        >
          {{ col.title }}
        </Checkbox>
      </Checkbox.Group>
      <Divider v-if="lv.key !== drawerData.levels[drawerData.levels.length - 1]?.key" />
    </div>

    <div class="footer-bar">
      <Button @click="handleCancel">
        {{ $t('ui.button.cancel') }}
      </Button>
      <Button type="primary" :loading="saving" @click="handleSave">
        {{ $t('ui.button.save') }}
      </Button>
    </div>
  </Drawer>
</template>

<style scoped>
.config-hint {
  margin-bottom: 16px;
  font-size: 13px;
  color: rgb(0 0 0 / 45%);
}

.level-block {
  margin-bottom: 8px;
}

.level-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.level-label {
  font-weight: 600;
  font-size: 14px;
}

.column-group {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 0;
}

.column-checkbox {
  width: 33.333%;
  margin-right: 0;
}

.footer-bar {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
</style>
