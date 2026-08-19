<script lang="ts" setup>
import { onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Alert,
  Button,
  Card,
  InputNumber,
  message,
  Radio,
  RadioGroup,
  Spin,
  Switch,
} from 'ant-design-vue';

import { requestClient } from '#/api/request';

defineOptions({ name: 'PerformanceConfig' });

const loading = ref(false);
const saving = ref(false);

// ===== 配置项 =====
const config = ref({
  // 卡片显示控制
  showKpiCards: true,
  showMonthlyTrend: true,
  showCompletionRate: true,
  showDeptRanking: true,
  showEmployeeRanking: true,
  showPersonalCards: true,

  // 默认时间维度
  defaultTimeDimension: 'year',

  // 隐私控制
  showOtherEmployeeActual: true, // 是否显示其他员工的实际业绩
  showOtherEmployeeTarget: true, // 是否显示其他员工的目标

  // 业绩计算口径
  contractCalcBasis: 'sign_date', // sign_date=合同签订日, create_time=创建日
  paymentCalcBasis: 'payment_date', // payment_date=回款日, confirm_time=确认日

  // 排名显示数量
  rankingTopN: 10,

  // 是否强制要求填写销售计划
  forcePlanRequired: true,
  planDueMonth: 1, // 每年几月前必须填写
});

// 加载配置
async function loadConfig() {
  loading.value = true;
  try {
    const res = await requestClient.get('/api/system/performance-config/get');
    if (res?.data) {
      config.value = { ...config.value, ...res.data };
    }
  } catch {
    // 配置不存在时使用默认值
  } finally {
    loading.value = false;
  }
}

// 保存配置
async function handleSave() {
  saving.value = true;
  try {
    await requestClient.post(
      '/api/system/performance-config/save',
      config.value,
    );
    message.success('配置已保存');
  } catch (error: any) {
    message.error(error?.message || '保存失败');
  } finally {
    saving.value = false;
  }
}

// 重置为默认
function handleReset() {
  config.value = {
    showKpiCards: true,
    showMonthlyTrend: true,
    showCompletionRate: true,
    showDeptRanking: true,
    showEmployeeRanking: true,
    showPersonalCards: true,
    defaultTimeDimension: 'year',
    showOtherEmployeeActual: true,
    showOtherEmployeeTarget: true,
    contractCalcBasis: 'sign_date',
    paymentCalcBasis: 'payment_date',
    rankingTopN: 10,
    forcePlanRequired: true,
    planDueMonth: 1,
  };
  message.info('已重置为默认配置，请点击保存生效');
}

onMounted(() => loadConfig());

// 配置分组定义
const cardVisibilityOptions = [
  { label: 'KPI 概览卡片', value: 'showKpiCards' },
  { label: '月度趋势图', value: 'showMonthlyTrend' },
  { label: '完成率环形图', value: 'showCompletionRate' },
  { label: '部门排名表', value: 'showDeptRanking' },
  { label: '销售员排名表', value: 'showEmployeeRanking' },
  { label: '个人业绩卡（普通销售）', value: 'showPersonalCards' },
];
</script>

<template>
  <Page auto-content-height>
    <Spin :spinning="loading">
      <!-- 顶部说明 -->
      <Card class="mb-4">
        <div class="flex items-start gap-3">
          <IconifyIcon
            icon="lucide:settings"
            class="text-xl text-primary mt-1"
          />
          <div>
            <div class="text-base font-semibold">业绩概览页面配置</div>
            <div class="text-sm text-gray-500 mt-1">
              配置"业绩概览"页面显示哪些数据卡片、默认时间维度、隐私控制和业绩计算口径。
              此配置对全公司生效。
            </div>
          </div>
        </div>
      </Card>

      <!-- 卡片显示控制 -->
      <Card title="页面卡片显示控制" class="mb-4">
        <Alert
          message="勾选的卡片将在业绩概览页面显示，未勾选的将隐藏"
          type="info"
          show-icon
          class="mb-4"
        />
        <div class="grid grid-cols-2 gap-3">
          <div
            v-for="opt in cardVisibilityOptions"
            :key="opt.value"
            class="flex items-center gap-2 p-3 border rounded hover:bg-gray-50"
          >
            <Switch
              :checked="(config as any)[opt.value]"
              @change="(val: any) => ((config as any)[opt.value] = val)"
            />
            <span>{{ opt.label }}</span>
          </div>
        </div>
      </Card>

      <!-- 默认时间维度 -->
      <Card title="默认时间维度" class="mb-4">
        <div class="flex items-center gap-4">
          <span class="text-gray-600">页面打开时默认显示的时间维度：</span>
          <RadioGroup v-model:value="config.defaultTimeDimension">
            <Radio value="year">按年</Radio>
            <Radio value="month">按月</Radio>
            <Radio value="day">按日</Radio>
          </RadioGroup>
        </div>
      </Card>

      <!-- 隐私控制 -->
      <Card title="隐私控制" class="mb-4">
        <Alert
          message="控制普通销售员能否看到其他同事的业绩数据（部门经理和公司管理层不受此限制）"
          type="warning"
          show-icon
          class="mb-4"
        />
        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 border rounded">
            <div>
              <div class="font-medium">显示其他员工的实际业绩</div>
              <div class="text-xs text-gray-500">
                关闭后，普通销售只能看到自己的实际金额
              </div>
            </div>
            <Switch v-model:checked="config.showOtherEmployeeActual" />
          </div>
          <div class="flex items-center justify-between p-3 border rounded">
            <div>
              <div class="font-medium">显示其他员工的目标金额</div>
              <div class="text-xs text-gray-500">
                关闭后，普通销售只能看到自己的目标
              </div>
            </div>
            <Switch v-model:checked="config.showOtherEmployeeTarget" />
          </div>
        </div>
      </Card>

      <!-- 业绩计算口径 -->
      <Card title="业绩计算口径" class="mb-4">
        <Alert
          message="决定实际业绩按哪个日期归入月份。例如：1月25日签订合同，按签订日归入1月；按创建日可能归入上月。"
          type="info"
          show-icon
          class="mb-4"
        />
        <div class="grid grid-cols-2 gap-4">
          <div>
            <div class="text-sm font-medium mb-2">合同业绩计算口径</div>
            <RadioGroup
              v-model:value="config.contractCalcBasis"
              direction="vertical"
            >
              <Radio value="sign_date">按合同签订日（推荐）</Radio>
              <Radio value="create_time">按合同创建日</Radio>
            </RadioGroup>
          </div>
          <div>
            <div class="text-sm font-medium mb-2">回款业绩计算口径</div>
            <RadioGroup
              v-model:value="config.paymentCalcBasis"
              direction="vertical"
            >
              <Radio value="payment_date">按回款日（推荐）</Radio>
              <Radio value="confirm_time">按确认日</Radio>
            </RadioGroup>
          </div>
        </div>
      </Card>

      <!-- 排名显示数量 -->
      <Card title="排名显示设置" class="mb-4">
        <div class="flex items-center gap-4">
          <span class="text-gray-600">排名表默认显示前</span>
          <InputNumber
            v-model:value="config.rankingTopN"
            :min="3"
            :max="100"
            :step="1"
            style="width: 100px"
          />
          <span class="text-gray-600">名</span>
        </div>
      </Card>

      <!-- 销售计划强制要求 -->
      <Card title="销售计划填写要求" class="mb-4">
        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 border rounded">
            <div>
              <div class="font-medium">强制要求填写销售计划</div>
              <div class="text-xs text-gray-500">
                开启后，未填写计划的普通销售员标题栏将显示红色"设置销售计划"按钮
              </div>
            </div>
            <Switch v-model:checked="config.forcePlanRequired" />
          </div>
          <div
            v-if="config.forcePlanRequired"
            class="flex items-center gap-4 p-3 border rounded"
          >
            <span class="text-gray-600">每年</span>
            <InputNumber
              v-model:value="config.planDueMonth"
              :min="1"
              :max="12"
              :step="1"
              style="width: 80px"
            />
            <span class="text-gray-600">月前必须填写（超期显示红色提醒）</span>
          </div>
        </div>
      </Card>

      <!-- 底部操作按钮 -->
      <div class="flex justify-end gap-3 mt-6">
        <Button @click="handleReset">
          <template #icon><IconifyIcon icon="lucide:rotate-ccw" /></template>
          重置默认
        </Button>
        <Button type="primary" :loading="saving" @click="handleSave">
          <template #icon><IconifyIcon icon="lucide:save" /></template>
          保存配置
        </Button>
      </div>
    </Spin>
  </Page>
</template>
