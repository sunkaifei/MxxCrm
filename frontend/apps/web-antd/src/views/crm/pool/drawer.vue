<script lang="ts" setup>
/**
 * 公海池编辑抽屉（v3.0 公海优化 §6.1）
 * Tabs：基础信息 / 回收规则 / 领取规则 / 自动分配 / 字段脱敏
 * 保存流程：池基础（save/update）→ 池配置（config/save）
 */
import type { PoolListVO } from '#/api/core/crm/pool';

import { computed, reactive, ref, watch } from 'vue';

import {
  Button,
  CheckboxGroup,
  Input,
  InputNumber,
  message,
  Radio,
  RadioGroup,
  Drawer,
  Switch,
  TabPane,
  Tabs,
} from 'ant-design-vue';

import {
  createPoolApi,
  getPoolInfoApi,
  savePoolConfigApi,
  updatePoolApi,
} from '#/api/core/crm/pool';

const props = defineProps<{
  /** 为空 = 新建 */
  poolId?: null | string;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'saved'): void;
  (e: 'update:visible', value: boolean): void;
}>();

const innerVisible = computed({
  get: () => props.visible,
  set: (val: boolean) => emit('update:visible', val),
});

const drawerWidth = computed(() =>
  typeof window !== 'undefined' ? Math.min(window.innerWidth * 0.6, 720) : 720,
);

const saving = ref(false);
const activeTab = ref('basic');

// 基础信息
const baseForm = reactive({
  description: '',
  name: '',
  sort: 0,
});

// 池配置
const configForm = reactive({
  autoAssignEnabled: 0,
  autoAssignMode: 1,
  claimDailyLimit: 0,
  claimMode: 1,
  coolDownDays: 0,
  holdLimit: 0,
  maskFields: [] as string[],
  recycleDays: 30,
  reminderDays: 3,
});

const claimModeOptions = [
  { label: '自主领取', value: 1 },
  { label: '申领需审批', value: 2 },
  { label: '停用领取（仅分配）', value: 3 },
];

const maskFieldOptions = [
  { label: '手机号（138****5678）', value: 'mobile' },
  { label: '电话（138****5678）', value: 'phone' },
  { label: '邮箱（a***@x.com）', value: 'email' },
  { label: '微信', value: 'wechat' },
];

function resetForm() {
  baseForm.name = '';
  baseForm.description = '';
  baseForm.sort = 0;
  configForm.recycleDays = 30;
  configForm.reminderDays = 3;
  configForm.coolDownDays = 0;
  configForm.claimDailyLimit = 0;
  configForm.holdLimit = 0;
  configForm.claimMode = 1;
  configForm.autoAssignEnabled = 0;
  configForm.autoAssignMode = 1;
  configForm.maskFields = [];
  activeTab.value = 'basic';
}

async function loadData() {
  if (!props.poolId) {
    resetForm();
    return;
  }
  const info: PoolListVO = await getPoolInfoApi(props.poolId!);
  baseForm.name = info?.name ?? '';
  baseForm.description = info?.description ?? '';
  baseForm.sort = info?.sort ?? 0;
  const cfg = info?.config;
  configForm.recycleDays = cfg?.recycleDays ?? 30;
  configForm.reminderDays = cfg?.reminderDays ?? 0;
  configForm.coolDownDays = cfg?.coolDownDays ?? 0;
  configForm.claimDailyLimit = cfg?.claimDailyLimit ?? 0;
  configForm.holdLimit = cfg?.holdLimit ?? 0;
  configForm.claimMode = cfg?.claimMode ?? 1;
  configForm.autoAssignEnabled = cfg?.autoAssignEnabled ?? 0;
  configForm.autoAssignMode = cfg?.autoAssignMode ?? 1;
  configForm.maskFields = cfg?.maskFields ? [...cfg.maskFields] : [];
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      void loadData();
    }
  },
);

async function handleSave() {
  if (!baseForm.name.trim()) {
    message.warning('请填写池名称');
    activeTab.value = 'basic';
    return;
  }
  saving.value = true;
  try {
    let poolId: number | string;
    if (props.poolId) {
      await updatePoolApi({
        description: baseForm.description,
        id: props.poolId,
        name: baseForm.name.trim(),
        sort: baseForm.sort,
      });
      poolId = props.poolId;
    } else {
      const newId = await createPoolApi({
        description: baseForm.description,
        name: baseForm.name.trim(),
        sort: baseForm.sort,
      });
      poolId = newId as unknown as number | string;
    }
    await savePoolConfigApi({
      autoAssignEnabled: configForm.autoAssignEnabled,
      autoAssignMode: configForm.autoAssignMode,
      claimDailyLimit: configForm.claimDailyLimit,
      claimMode: configForm.claimMode,
      coolDownDays: configForm.coolDownDays,
      holdLimit: configForm.holdLimit,
      maskFields: configForm.maskFields,
      poolId,
      recycleDays: configForm.recycleDays,
      reminderDays: configForm.reminderDays,
    });
    message.success('保存成功');
    innerVisible.value = false;
    emit('saved');
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Drawer
    v-model:open="innerVisible"
    :title="poolId ? '编辑公海池' : '新建公海池'"
    :width="drawerWidth"
    destroy-on-close
  >
    <Tabs v-model:activeKey="activeTab">
      <!-- Tab 1: 基础信息 -->
      <TabPane key="basic" tab="基础信息">
        <div class="form-row">
          <div class="form-label required">池名称</div>
          <div class="form-content">
            <Input
              v-model:value="baseForm.name"
              placeholder="请输入池名称"
              :maxlength="50"
              style="width: 320px"
            />
          </div>
        </div>
        <div class="form-row">
          <div class="form-label">描述</div>
          <div class="form-content">
            <Input.TextArea
              v-model:value="baseForm.description"
              placeholder="请输入池描述"
              :rows="3"
              :maxlength="200"
              style="width: 420px"
            />
          </div>
        </div>
        <div class="form-row">
          <div class="form-label">排序</div>
          <div class="form-content">
            <InputNumber
              v-model:value="baseForm.sort"
              :min="0"
              :precision="0"
              style="width: 160px"
            />
            <span class="ml-2 text-xs text-gray-400">数值越小越靠前</span>
          </div>
        </div>
      </TabPane>

      <!-- Tab 2: 回收规则 -->
      <TabPane key="recycle" tab="回收规则">
        <div class="form-row">
          <div class="form-label">回收天数</div>
          <div class="form-content">
            <InputNumber
              v-model:value="configForm.recycleDays"
              :min="0"
              :precision="0"
              style="width: 160px"
            />
            <span class="ml-2 text-xs text-gray-400">
              私海线索未跟进超过 N 天自动回收，0 = 不回收
            </span>
          </div>
        </div>
        <div class="form-row">
          <div class="form-label">提前提醒</div>
          <div class="form-content">
            <InputNumber
              v-model:value="configForm.reminderDays"
              :min="0"
              :precision="0"
              style="width: 160px"
            />
            <span class="ml-2 text-xs text-gray-400">
              回收前 N 天站内提醒，0 = 不提醒
            </span>
          </div>
        </div>
        <div class="form-row">
          <div class="form-label">冷却期</div>
          <div class="form-content">
            <InputNumber
              v-model:value="configForm.coolDownDays"
              :min="0"
              :precision="0"
              style="width: 160px"
            />
            <span class="ml-2 text-xs text-gray-400">
              退回公海后 N 天内不可再领取同池线索，0 = 不启用
            </span>
          </div>
        </div>
      </TabPane>

      <!-- Tab 3: 领取规则 -->
      <TabPane key="claim" tab="领取规则">
        <div class="form-row">
          <div class="form-label">领取模式</div>
          <div class="form-content">
            <RadioGroup v-model:value="configForm.claimMode">
              <Radio
                v-for="opt in claimModeOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </Radio>
            </RadioGroup>
          </div>
        </div>
        <div class="form-row">
          <div class="form-label">每日领取上限</div>
          <div class="form-content">
            <InputNumber
              v-model:value="configForm.claimDailyLimit"
              :min="0"
              :precision="0"
              style="width: 160px"
            />
            <span class="ml-2 text-xs text-gray-400">0 = 不限制</span>
          </div>
        </div>
        <div class="form-row">
          <div class="form-label">保有量上限</div>
          <div class="form-content">
            <InputNumber
              v-model:value="configForm.holdLimit"
              :min="0"
              :precision="0"
              style="width: 160px"
            />
            <span class="ml-2 text-xs text-gray-400">
              私海线索总量超过 N 条后不可领取，0 = 不限制
            </span>
          </div>
        </div>
      </TabPane>

      <!-- Tab 4: 自动分配 -->
      <TabPane key="autoAssign" tab="自动分配">
        <div class="form-row">
          <div class="form-label">自动分配</div>
          <div class="form-content">
            <Switch
              :checked="configForm.autoAssignEnabled === 1"
              checked-children="开"
              un-checked-children="关"
              @change="(checked: any) => (configForm.autoAssignEnabled = checked ? 1 : 0)"
            />
            <span class="ml-2 text-xs text-gray-400">
              入池线索按策略自动分配给池成员
            </span>
          </div>
        </div>
        <div v-if="configForm.autoAssignEnabled === 1" class="form-row">
          <div class="form-label">分配模式</div>
          <div class="form-content">
            <RadioGroup v-model:value="configForm.autoAssignMode">
              <Radio :value="1">轮询</Radio>
              <Radio :value="2">负载均衡（优先分配给持有量少的成员）</Radio>
            </RadioGroup>
          </div>
        </div>
      </TabPane>

      <!-- Tab 5: 字段脱敏 -->
      <TabPane key="mask" tab="字段脱敏">
        <div class="form-row">
          <div class="form-label">脱敏字段</div>
          <div class="form-content">
            <CheckboxGroup
              v-model:value="configForm.maskFields"
              :options="maskFieldOptions"
            />
          </div>
        </div>
        <div class="form-row">
          <div class="form-label"></div>
          <div class="form-content text-xs text-gray-400 leading-6">
            普通池成员查看线索时，勾选字段按规则打码；池管理员与超管不受影响。未勾选 = 不脱敏。
          </div>
        </div>
      </TabPane>
    </Tabs>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="innerVisible = false">取消</Button>
        <Button
          v-access:code="['crm:pool:save', 'crm:pool:update']"
          type="primary"
          :loading="saving"
          @click="handleSave"
        >
          保存
        </Button>
      </div>
    </template>
  </Drawer>
</template>

<style scoped>
.form-row {
  display: flex;
  align-items: flex-start;
  margin-bottom: 16px;
}

.form-label {
  flex-shrink: 0;
  width: 110px;
  padding-top: 6px;
  padding-right: 12px;
  color: #666;
  text-align: right;
}

.form-label.required::before {
  content: '*';
  margin-right: 2px;
  color: #ff4d4f;
}

.form-content {
  display: flex;
  flex: 1;
  flex-wrap: wrap;
  align-items: center;
}
</style>
