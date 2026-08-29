<script lang="ts" setup>
import type { FieldChoice } from '#/api';

import { computed, reactive, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import { LucideMaximize2, LucideMinimize2 } from '@vben/icons';

import {
  Button,
  Col,
  DatePicker,
  Form,
  Input,
  InputNumber,
  message,
  Row,
  Select,
  Switch,
} from 'ant-design-vue';

import {
  FIELD_TYPE_OPTIONS,
  getRoleOptionsApi,
  saveFieldApi,
  updateFieldApi,
} from '#/api';
import { getModuleListApi } from '#/api/core/system/field';

const data = ref<Record<string, any>>();
const isFullscreen = ref(false);
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增自定义字段' : '编辑自定义字段'));

const form = reactive({
  module: undefined as string | undefined,
  fieldKey: '',
  fieldLabel: '',
  fieldType: 1,
  precision: undefined as number | undefined,
  defaultValue: undefined as any,
  required: 0,
  visibleRoles: [] as string[],
  editableRoles: [] as string[],
  listVisible: 0,
  listSort: 0,
  filterable: 0,
  sort: 0,
  remark: '',
});

// 单选/多选选项草稿；编辑模式旧 value 锁定（不可改/删，仅可停用，P0-18）
const choices = ref<FieldChoice[]>([]);
const lockedChoiceValues = ref<Set<string>>(new Set());

const moduleOptions = ref<Array<{ label: string; value: string }>>([]);
const roleOptions = ref<Array<{ label: string; value: string }>>([]);

const isChoiceType = computed(() => form.fieldType === 6 || form.fieldType === 7);
const isNumberType = computed(() => form.fieldType === 3 || form.fieldType === 11);
// 支持配置默认值的类型：文本/多行文本/数字/日期/日期时间/单选/布尔/金额（数组型 7/9/10 暂不支持）
const isDefaultSupported = computed(() => ![7, 9, 10].includes(form.fieldType));
// 单选默认值下拉选项：取草稿中启用且填写完整的选项
const defaultValueChoiceOptions = computed(() =>
  choices.value
    .filter((c) => Number(c.active) === 1 && c.value.trim() && c.label.trim())
    .map((c) => ({ label: c.label.trim(), value: c.value.trim() })),
);

// 布尔默认值选项：value 保留 boolean（Switch 预填判定 value===true；antd Select options 类型限制需断言）
const boolDefaultOptions = [
  { label: '是', value: true },
  { label: '否', value: false },
] as Array<{ label: string; value: any }>;

const fieldKeyError = computed(() => {
  // 与后端 field_key 正则 {1,63} 对齐：小写字母开头，2-64 位
  if (isCreate.value && !/^[a-z][a-z0-9_]{1,63}$/.test(form.fieldKey)) {
    return '小写字母开头，仅含小写字母/数字/下划线，2-64 位';
  }
  return undefined;
});

async function loadModuleOptions() {
  try {
    const res: any = await getModuleListApi();
    const list = Array.isArray(res) ? res : (res?.list ?? []);
    moduleOptions.value = list.map((m: any) => ({
      label: m.label ?? m.module,
      value: m.module,
    }));
  } catch {
    /* ignore */
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

function addChoice() {
  choices.value.push({ active: 1, label: '', value: '' });
}

function removeChoice(idx: number) {
  const c = choices.value[idx];
  if (c && lockedChoiceValues.value.has(c.value)) return;
  choices.value.splice(idx, 1);
}

const activeChoiceCount = computed(() =>
  choices.value.filter((c) => Number(c.active) === 1).length,
);

function handleRequiredChange(checked: number | string | boolean) {
  if (Number(checked) === 1 && form.visibleRoles.length > 0) {
    form.visibleRoles = [];
    message.warning('必填字段必须全员可见，已自动清空可见角色限制');
  }
}

function handleVisibleRolesChange(values: unknown) {
  if (form.required === 1 && Array.isArray(values) && values.length > 0) {
    form.visibleRoles = [];
    message.warning('必填字段必须全员可见，不能限制可见角色');
  }
}

async function handleConfirm() {
  if (!form.module) {
    message.error('请选择所属模块');
    return;
  }
  if (!form.fieldLabel.trim()) {
    message.error('请输入字段显示名');
    return;
  }
  if (isCreate.value && fieldKeyError.value) {
    message.error(`字段键不合法：${fieldKeyError.value}`);
    return;
  }
  if (isChoiceType.value) {
    if (choices.value.length === 0) {
      message.error('单选/多选字段至少配置 1 个选项');
      return;
    }
    const seen = new Set<string>();
    for (const c of choices.value) {
      if (!c.value.trim() || !c.label.trim()) {
        message.error('选项的存储值与显示名均不能为空');
        return;
      }
      if (seen.has(c.value.trim())) {
        message.error(`选项存储值「${c.value}」重复`);
        return;
      }
      seen.add(c.value.trim());
    }
  }
  if (form.required === 1 && form.visibleRoles.length > 0) {
    message.error('必填字段必须全员可见，请清空可见角色限制');
    return;
  }

  const payload: Record<string, any> = {
    module: form.module,
    fieldLabel: form.fieldLabel.trim(),
    fieldType: form.fieldType,
    required: form.required,
    visibleRoles: form.visibleRoles,
    editableRoles: form.editableRoles,
    listVisible: form.listVisible,
    listSort: form.listSort,
    filterable: form.filterable,
    sort: form.sort,
    remark: form.remark,
  };
  if (isCreate.value) {
    payload.fieldKey = form.fieldKey.trim();
  }
  // options 统一构建：choices（单选/多选）、precision（数字/金额，0 合法）、defaultValue（通用预填，P0-21）
  const options: Record<string, any> = {};
  if (isChoiceType.value) {
    options.choices = choices.value.map((c) => ({
      value: c.value.trim(),
      label: c.label.trim(),
      active: Number(c.active),
    }));
  }
  if (
    isNumberType.value &&
    form.precision !== undefined &&
    form.precision !== null
  ) {
    options.precision = form.precision;
  }
  if (
    isDefaultSupported.value &&
    form.defaultValue !== undefined &&
    form.defaultValue !== null &&
    String(form.defaultValue) !== ''
  ) {
    options.defaultValue = form.defaultValue;
  }
  payload.options = Object.keys(options).length > 0 ? options : null;

  setLoading(true);
  try {
    if (isCreate.value) {
      await saveFieldApi(payload);
      message.success('新增成功');
    } else {
      // fieldKey/fieldType/module 为锁定值，原样提交由后端做一致性校验
      payload.fieldKey = form.fieldKey;
      await updateFieldApi(data.value?.row?.id, payload);
      message.success('保存成功');
    }
    drawerApi.setData({ needRefresh: true });
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
    data.value = drawerApi.getData<Record<string, any>>();
    const row = data.value?.row;
    form.module = row?.module ?? undefined;
    form.fieldKey = row?.fieldKey ?? '';
    form.fieldLabel = row?.fieldLabel ?? '';
    form.fieldType = Number(row?.fieldType ?? 1);
    form.precision = row?.options?.precision;
    form.defaultValue = row?.options?.defaultValue;
    form.required = Number(row?.required ?? 0);
    form.visibleRoles = Array.isArray(row?.visibleRoles) ? row.visibleRoles : [];
    form.editableRoles = Array.isArray(row?.editableRoles) ? row.editableRoles : [];
    form.listVisible = Number(row?.listVisible ?? 0);
    form.listSort = Number(row?.listSort ?? 0);
    form.filterable = Number(row?.filterable ?? 0);
    form.sort = Number(row?.sort ?? 0);
    form.remark = row?.remark ?? '';
    choices.value = (row?.options?.choices ?? []).map((c: any) => ({
      value: String(c.value ?? ''),
      label: String(c.label ?? ''),
      active: Number(c.active ?? 1),
    }));
    // 编辑模式锁定已存在的选项 value（保存后不可改/删，仅可停用）
    lockedChoiceValues.value = new Set(
      isCreate.value
        ? []
        : (row?.options?.choices ?? []).map((c: any) => String(c.value ?? '')),
    );
    setLoading(false);
    loadModuleOptions();
    loadRoleOptions();
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer
    :class="isFullscreen ? 'w-full! max-w-full!' : 'w-[75%]! max-w-[75%]!'"
    :title="getTitle"
  >
    <template #extra>
      <Button type="text" @click="isFullscreen = !isFullscreen">
        <LucideMinimize2 v-if="isFullscreen" class="size-4" />
        <LucideMaximize2 v-else class="size-4" />
      </Button>
    </template>
    <Form :model="form" layout="vertical">
      <div class="section-title">基础信息</div>
      <Row :gutter="16">
        <Col :span="12">
          <Form.Item label="所属模块" required>
            <Select
              v-model:value="form.module"
              :options="moduleOptions"
              :disabled="!isCreate"
              placeholder="请选择模块"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item label="字段类型" required extra="创建后类型不可修改">
            <Select
              v-model:value="form.fieldType"
              :options="FIELD_TYPE_OPTIONS"
              :disabled="!isCreate"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item
            label="字段键"
            required
            :validate-status="fieldKeyError ? 'error' : undefined"
            :help="fieldKeyError"
            extra="创建后不可修改，用于数据存储"
          >
            <Input
              v-model:value="form.fieldKey"
              :disabled="!isCreate"
              placeholder="如 assembly_line"
              :maxlength="64"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item label="显示名" required>
            <Input
              v-model:value="form.fieldLabel"
              placeholder="如 生产线"
              :maxlength="30"
            />
          </Form.Item>
        </Col>
      </Row>

      <div
        v-if="isChoiceType || isNumberType || isDefaultSupported"
        class="section-title"
      >
        {{ isChoiceType ? '选项配置' : '默认值' }}
      </div>
      <Row v-if="!isChoiceType" :gutter="16">
        <Col v-if="isNumberType" :span="12">
          <Form.Item label="小数位数">
            <InputNumber
              v-model:value="form.precision"
              :min="0"
              :max="10"
              :precision="0"
              style="width: 100%"
              placeholder="留空表示不限制"
            />
          </Form.Item>
        </Col>
        <Col v-if="isDefaultSupported" :span="12">
          <!-- 默认值配置（P0-21）：新建表单预填，存量数据不受影响；控件随字段类型切换 -->
          <Form.Item
            label="默认值"
            extra="新建表单预填；多选/附件/成员类型暂不支持"
          >
            <Input
              v-if="form.fieldType === 1 || form.fieldType === 2"
              v-model:value="form.defaultValue"
              placeholder="留空表示不设置"
              :maxlength="2000"
              allow-clear
            />
            <InputNumber
              v-else-if="form.fieldType === 3 || form.fieldType === 11"
              v-model:value="form.defaultValue"
              :precision="form.precision"
              style="width: 100%"
              placeholder="留空表示不设置"
            />
            <DatePicker
              v-else-if="form.fieldType === 4"
              v-model:value="form.defaultValue"
              value-format="YYYY-MM-DD"
              style="width: 100%"
              placeholder="留空表示不设置"
            />
            <DatePicker
              v-else-if="form.fieldType === 5"
              v-model:value="form.defaultValue"
              value-format="YYYY-MM-DD HH:mm:ss"
              show-time
              style="width: 100%"
              placeholder="留空表示不设置"
            />
            <Select
              v-else-if="form.fieldType === 6"
              v-model:value="form.defaultValue"
              :options="defaultValueChoiceOptions"
              placeholder="留空表示不设置"
              allow-clear
              style="width: 100%"
            />
            <Select
              v-else-if="form.fieldType === 8"
              v-model:value="form.defaultValue"
              :options="boolDefaultOptions"
              placeholder="留空表示不设置"
              allow-clear
              style="width: 100%"
            />
          </Form.Item>
        </Col>
      </Row>

      <div v-if="isChoiceType" class="choices-block">
        <div class="choices-header">
          <span>选项列表</span>
          <span class="choices-count">
            启用 {{ activeChoiceCount }} / {{ choices.length }}
          </span>
          <span class="choices-tip">保存后存储值锁定，仅可停用或新增</span>
        </div>
        <div
          v-for="(c, idx) in choices"
          :key="idx"
          class="choices-row"
          :class="{ 'choices-row--inactive': Number(c.active) === 0 }"
        >
          <span class="choices-index">{{ idx + 1 }}</span>
          <Input
            v-model:value="c.value"
            :disabled="lockedChoiceValues.has(c.value)"
            placeholder="存储值"
            :maxlength="50"
          />
          <Input
            v-model:value="c.label"
            placeholder="显示名"
            :maxlength="50"
          />
          <Switch
            v-model:checked="c.active"
            :checked-value="1"
            :un-checked-value="0"
            checked-children="启用"
            un-checked-children="停用"
          />
          <Button
            danger
            type="link"
            :disabled="lockedChoiceValues.has(c.value)"
            @click="removeChoice(idx)"
          >
            删除
          </Button>
        </div>
        <Button type="dashed" block @click="addChoice">+ 添加选项</Button>
      </div>

      <div class="section-title">表单与列表</div>
      <Row :gutter="16">
        <Col :span="12">
          <Form.Item
            label="必填"
            :extra="form.required === 1 ? '必填字段必须全员可见' : '开启后必须全员可见'"
          >
            <Switch
              v-model:checked="form.required"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="是"
              un-checked-children="否"
              @change="handleRequiredChange"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item label="列表显示">
            <Switch
              v-model:checked="form.listVisible"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="显示"
              un-checked-children="隐藏"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item label="列表排序权重" extra="数字越小越靠前">
            <InputNumber
              v-model:value="form.listSort"
              :min="0"
              :precision="0"
              style="width: 100%"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item label="参与筛选">
            <Switch
              v-model:checked="form.filterable"
              :checked-value="1"
              :un-checked-value="0"
              checked-children="是"
              un-checked-children="否"
            />
          </Form.Item>
        </Col>
      </Row>

      <div class="section-title">数据权限</div>
      <Row :gutter="16">
        <Col :span="12">
          <Form.Item
            label="可见角色"
            :extra="form.required === 1 ? '必填字段不可限制可见角色' : '留空表示全员可见；schema 接口按角色下发'"
          >
            <Select
              v-model:value="form.visibleRoles"
              :options="roleOptions"
              mode="multiple"
              placeholder="留空全员可见"
              :max-tag-count="3"
              allow-clear
              @change="handleVisibleRolesChange"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item
            label="可编辑角色"
            extra="留空表示全员可编辑；越权写入后端 400 拦截"
          >
            <Select
              v-model:value="form.editableRoles"
              :options="roleOptions"
              mode="multiple"
              placeholder="留空全员可编辑"
              :max-tag-count="3"
              allow-clear
            />
          </Form.Item>
        </Col>
      </Row>

      <div class="section-title">其他</div>
      <Row :gutter="16">
        <Col :span="12">
          <Form.Item label="排序权重" extra="表单内字段排列顺序，数字越小越靠前">
            <InputNumber
              v-model:value="form.sort"
              :min="0"
              :precision="0"
              style="width: 100%"
            />
          </Form.Item>
        </Col>
        <Col :span="12">
          <Form.Item label="备注">
            <Input
              v-model:value="form.remark"
              placeholder="选填"
              :maxlength="200"
            />
          </Form.Item>
        </Col>
      </Row>
    </Form>
  </Drawer>
</template>

<style scoped>
/* 紧凑化：压缩 antd Form.Item 默认 24px 底部间距 */
:deep(.ant-form-item) {
  margin-bottom: 14px;
}

/* extra 提示降为辅助字号，降低占位感 */
:deep(.ant-form-item-extra) {
  font-size: 12px;
  line-height: 18px;
}

.section-title {
  display: flex;
  gap: 8px;
  align-items: center;
  margin: 16px 0 10px;
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.section-title:first-child {
  margin-top: 0;
}

.section-title::before {
  width: 3px;
  height: 12px;
  border-radius: 2px;
  background: hsl(var(--primary));
  content: '';
}

.choices-block {
  padding: 10px;
  margin-bottom: 12px;
  background: hsl(var(--accent) / 60%);
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.choices-header {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
  font-weight: 500;
}

.choices-count {
  padding: 0 8px;
  font-size: 12px;
  font-weight: 400;
  line-height: 18px;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 12%);
  border-radius: 999px;
}

.choices-tip {
  font-size: 12px;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
}

.choices-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}

.choices-index {
  width: 20px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

.choices-row > :nth-child(2) {
  flex: 0 0 160px;
}

.choices-row > :nth-child(3) {
  flex: 1;
}

.choices-row--inactive {
  opacity: 0.55;
}
</style>
