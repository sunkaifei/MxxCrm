<script lang="ts" setup>
import { computed, h, onMounted, ref, watch } from 'vue';
import { message, Popconfirm, Select, Tag } from 'ant-design-vue';
import {
  Button,
  Input as AInput,
  InputNumber as AInputNumber,
  Select as ASelect,
  Table as ATable,
  DatePicker as ADatePicker,
} from 'ant-design-vue';
import { useVbenDrawer } from '@vben/common-ui';
import { LucideChevronDown, LucideChevronUp, LucideEye, LucidePlus, LucideTrash2 } from '@vben/icons';
import { $t } from '#/locales';
import {
  createCodeRuleApi,
  getCodeRuleInfoApi,
  previewCodeApi,
  updateCodeRuleApi,
  type SegmentConfig,
} from '#/api';
import {
  createDefaultSegment,
  dateFormatOptions,
  segmentTypeLabelMap,
  segmentTypeOptions,
  seqLengthOptions,
  separatorOptions,
  yearFormatOptions,
  yearSourceOptions,
} from './data';

const data = ref<any>();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.company.codeRule.title') })
    : $t('ui.modal.update', { moduleName: $t('page.company.codeRule.title') }),
);

// 基本字段
const formData = ref({
  id: undefined as number | undefined,
  moduleCode: '',
  moduleName: '',
  ruleName: '',
  companyAbbr: '',
  deptCode: '',
  bizTypeCode: '',
  separator: '-',
  seqLength: 4,
  enabled: 1,
  remark: '',
});

// 段位配置数组
const segments = ref<SegmentConfig[]>([]);
// 预览编号
const previewCode = ref('');
// 业务日期（用于预览）
const previewBusinessDate = ref<string>('');

const columns = [
  {
    title: '顺序',
    key: 'sort',
    width: 70,
    customRender: ({ index }: { index: number }) => index + 1,
  },
  { title: '段位类型', key: 'type', width: 130, dataIndex: 'type' },
  { title: '配置', key: 'config' },
  { title: '操作', key: 'action', width: 130 },
];

function addSegment(type: string) {
  const sort = segments.value.length + 1;
  segments.value.push(createDefaultSegment(type, sort));
}

function removeSegment(index: number) {
  segments.value.splice(index, 1);
  reassignSort();
}

function moveUp(index: number) {
  if (index === 0) return;
  const tmp = segments.value[index - 1];
  segments.value[index - 1] = segments.value[index];
  segments.value[index] = tmp;
  reassignSort();
}

function moveDown(index: number) {
  if (index === segments.value.length - 1) return;
  const tmp = segments.value[index + 1];
  segments.value[index + 1] = segments.value[index];
  segments.value[index] = tmp;
  reassignSort();
}

function reassignSort() {
  segments.value.forEach((s, i) => (s.sort = i + 1));
}

async function refreshPreview() {
  if (segments.value.length === 0) {
    previewCode.value = '';
    return;
  }
  try {
    const result = await previewCodeApi({
      segments: segments.value,
      companyAbbr: formData.value.companyAbbr || undefined,
      bizTypeCode: formData.value.bizTypeCode || undefined,
      separator: formData.value.separator,
      seqLength: formData.value.seqLength,
      deptCode: formData.value.deptCode || undefined,
      businessDate: previewBusinessDate.value || undefined,
      mockSeq: true,
    });
    previewCode.value = (result as any) ?? '';
  } catch {
    // 错误由全局拦截器处理
  }
}

watch(
  () => segments.value,
  () => refreshPreview(),
  { deep: true },
);
watch(
  () => [formData.value.bizTypeCode, formData.value.separator, formData.value.seqLength, formData.value.deptCode, formData.value.companyAbbr, previewBusinessDate.value],
  () => refreshPreview(),
);

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    // 验证
    if (!formData.value.moduleCode.trim()) {
      message.error('请输入模块编码');
      return;
    }
    if (!formData.value.moduleName.trim()) {
      message.error('请输入模块名称');
      return;
    }
    if (segments.value.length === 0) {
      message.error('请至少添加一个段位');
      return;
    }

    setLoading(true);
    try {
      const payload = {
        id: formData.value.id,
        moduleCode: formData.value.moduleCode.trim(),
        moduleName: formData.value.moduleName.trim(),
        ruleName: formData.value.ruleName || undefined,
        companyAbbr: formData.value.companyAbbr || undefined,
        deptCode: formData.value.deptCode || undefined,
        bizTypeCode: formData.value.bizTypeCode || undefined,
        separator: formData.value.separator,
        segments: segments.value,
        seqLength: formData.value.seqLength,
        enabled: formData.value.enabled,
        remark: formData.value.remark || undefined,
      };
      if (isCreate.value) {
        await createCodeRuleApi(payload);
        message.success($t('ui.notification.create_success'));
      } else {
        await updateCodeRuleApi(formData.value.id as number, payload);
        message.success($t('ui.notification.update_success'));
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } catch {
      // 错误由全局拦截器处理
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row;

      if (row) {
        // 编辑：拉详情获取完整段位
        loadDetail(Number(row.id));
      } else {
        // 新增：重置
        formData.value = {
          id: undefined,
          moduleCode: '',
          moduleName: '',
          ruleName: '',
          companyAbbr: '',
          deptCode: '',
          bizTypeCode: '',
          separator: '-',
          seqLength: 4,
          enabled: 1,
          remark: '',
        };
        segments.value = [];
        previewCode.value = '';
      }
      setLoading(false);
    }
  },
});

async function loadDetail(id: number) {
  try {
    const detail = (await getCodeRuleInfoApi(id)) as any;
    if (!detail) return;
    formData.value = {
      id: Number(detail.id),
      moduleCode: detail.moduleCode || '',
      moduleName: detail.moduleName || '',
      ruleName: detail.ruleName || '',
      companyAbbr: detail.companyAbbr || '',
      deptCode: detail.deptCode || '',
      bizTypeCode: detail.bizTypeCode || '',
      separator: detail.separator || '-',
      seqLength: detail.seqLength ?? 4,
      enabled: detail.enabled ?? 1,
      remark: detail.remark || '',
    };
    segments.value = Array.isArray(detail.segments) ? detail.segments : [];
  } catch {
    // 错误由全局拦截器处理
  }
}

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

onMounted(() => {
  // 占位：避免未使用警告
  void AInput;
  void AInputNumber;
  void ASelect;
  void ADatePicker;
  void LucideEye;
});
</script>

<template>
  <Drawer :title="getTitle" :width="900">
    <div class="code-rule-drawer space-y-4">
      <!-- 基本信息表单 -->
      <div class="rounded border p-4">
        <div class="mb-3 font-semibold">{{ $t('page.company.codeRule.basicInfo') }}</div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <div class="mb-1 text-sm">模块编码 *</div>
            <AInput
              v-model:value="formData.moduleCode"
              placeholder="如 customer / order / tech_doc"
              :disabled="!isCreate"
            />
          </div>
          <div>
            <div class="mb-1 text-sm">模块名称 *</div>
            <AInput v-model:value="formData.moduleName" placeholder="如 客户管理" />
          </div>
          <div>
            <div class="mb-1 text-sm">规则名称</div>
            <AInput v-model:value="formData.ruleName" placeholder="如 客户编号规则" />
          </div>
          <div>
            <div class="mb-1 text-sm">企业简称</div>
            <AInput v-model:value="formData.companyAbbr" placeholder="如 XYH" />
          </div>
          <div>
            <div class="mb-1 text-sm">部门编码</div>
            <AInput v-model:value="formData.deptCode" placeholder="如 XS" />
          </div>
          <div>
            <div class="mb-1 text-sm">业务类型编码</div>
            <AInput v-model:value="formData.bizTypeCode" placeholder="如 KH / HT / JS" />
          </div>
          <div>
            <div class="mb-1 text-sm">分隔符</div>
            <ASelect v-model:value="formData.separator" :options="separatorOptions" />
          </div>
          <div>
            <div class="mb-1 text-sm">流水号位数</div>
            <ASelect v-model:value="formData.seqLength" :options="seqLengthOptions" />
          </div>
          <div>
            <div class="mb-1 text-sm">状态</div>
            <ASelect
              v-model:value="formData.enabled"
              :options="[
                { label: '启用', value: 1 },
                { label: '停用', value: 0 },
              ]"
            />
          </div>
          <div>
            <div class="mb-1 text-sm">备注</div>
            <AInput v-model:value="formData.remark" />
          </div>
        </div>
      </div>

      <!-- 段位配置 -->
      <div class="rounded border p-4">
        <div class="mb-3 flex items-center justify-between">
          <span class="font-semibold">{{ $t('page.company.codeRule.segmentsTitle') }}</span>
          <Select
            placeholder="添加段位"
            style="width: 160px"
            :options="segmentTypeOptions.map((o) => ({ label: o.label, value: o.value }))"
            @change="(val: string) => { if (val) { addSegment(val); } }"
            :value="undefined"
            allow-clear
          />
        </div>
        <ATable
          :columns="columns"
          :data-source="segments"
          :pagination="false"
          row-key="sort"
          size="small"
          :locale="{ emptyText: '暂无段位，请点击右上角添加段位' }"
        >
          <template #bodyCell="{ column, index, record }">
            <template v-if="column.key === 'type'">
              <Tag color="blue">{{ segmentTypeLabelMap[record.type] || record.type }}</Tag>
            </template>
            <template v-else-if="column.key === 'config'">
              <div class="flex flex-wrap items-center gap-2">
                <template v-if="record.type === 'fixed' || record.type === 'biz_type'">
                  <span class="text-xs text-gray-500">值:</span>
                  <AInput
                    v-model:value="record.value"
                    placeholder="段位值"
                    style="width: 120px"
                    size="small"
                  />
                </template>
                <template v-if="record.type === 'year'">
                  <span class="text-xs text-gray-500">格式:</span>
                  <ASelect
                    v-model:value="record.format"
                    :options="yearFormatOptions"
                    style="width: 130px"
                    size="small"
                  />
                  <span class="text-xs text-gray-500">来源:</span>
                  <ASelect
                    v-model:value="record.source"
                    :options="yearSourceOptions"
                    style="width: 160px"
                    size="small"
                  />
                </template>
                <template v-if="record.type === 'date'">
                  <span class="text-xs text-gray-500">格式:</span>
                  <ASelect
                    v-model:value="record.format"
                    :options="dateFormatOptions"
                    style="width: 150px"
                    size="small"
                  />
                </template>
                <template v-if="record.type === 'seq'">
                  <span class="text-xs text-gray-500">位数:</span>
                  <AInputNumber
                    v-model:value="record.length"
                    :min="3"
                    :max="8"
                    style="width: 100px"
                    size="small"
                  />
                </template>
                <template v-if="record.type === 'version'">
                  <span class="text-xs text-gray-500">默认值:</span>
                  <AInput
                    v-model:value="record.value"
                    placeholder="V1"
                    style="width: 80px"
                    size="small"
                  />
                </template>
              </div>
            </template>
            <template v-else-if="column.key === 'action'">
              <Button type="link" size="small" :icon="h(LucideChevronUp)" :disabled="index === 0" @click="moveUp(index)" />
              <Button type="link" size="small" :icon="h(LucideChevronDown)" :disabled="index === segments.length - 1" @click="moveDown(index)" />
              <Popconfirm title="确定删除此段位？" @confirm="removeSegment(index)">
                <Button type="link" size="small" danger :icon="h(LucideTrash2)" />
              </Popconfirm>
            </template>
          </template>
        </ATable>
      </div>

      <!-- 实时预览 -->
      <div class="rounded border p-4">
        <div class="mb-3 flex items-center gap-2 font-semibold">
          <component :is="h(LucideEye)" />
          {{ $t('page.company.codeRule.previewTitle') }}
        </div>
        <div class="mb-3 flex flex-wrap items-center gap-4">
          <div class="flex items-center gap-2">
            <span class="text-xs text-gray-500">业务日期:</span>
            <ADatePicker
              v-model:value="previewBusinessDate"
              placeholder="补录历史文件时填"
              style="width: 180px"
              size="small"
              value-format="YYYY-MM-DD"
            />
          </div>
        </div>
        <div class="rounded bg-gray-50 p-3">
          <span class="text-xs text-gray-500">预览编号:</span>
          <span class="ml-2 font-mono text-lg font-bold text-blue-600">{{ previewCode || '—' }}</span>
        </div>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
.code-rule-drawer {
  display: flex;
  flex-direction: column;
}
</style>
