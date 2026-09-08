<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { useAccess } from '@vben/access';
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

import {
  getPerformanceConfigApi,
  savePerformanceConfigApi,
} from '#/api/core/statistics/performance';
import { $t } from '#/locales';

defineOptions({ name: 'PerformanceConfig' });

const { hasAccessByCodes } = useAccess();
const canManage = computed(() =>
  hasAccessByCodes(['system:performance-config:manage']),
);

const loading = ref(false);
const saving = ref(false);

const SCOPE_ALL_USERS = 'all_users';
const SCOPE_PLAN_SALES = 'plan_sales';

const defaultConfig = () => ({
  statScope: SCOPE_ALL_USERS,
  showZeroTargetRows: false,
  hideOtherActual: false,
  hideOtherTarget: false,
  rankingTopN: 10,
});

const config = ref(defaultConfig());

async function loadConfig() {
  loading.value = true;
  try {
    const res: any = await getPerformanceConfigApi();
    if (res) {
      config.value = { ...config.value, ...res };
    }
  } catch {
    // 配置不存在时使用默认值
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await savePerformanceConfigApi({
      statScope: config.value.statScope,
      showZeroTargetRows: config.value.showZeroTargetRows,
      hideOtherActual: config.value.hideOtherActual,
      hideOtherTarget: config.value.hideOtherTarget,
      rankingTopN: config.value.rankingTopN,
    });
    message.success($t('page.system.performanceConfig.saveSuccess'));
  } catch (error: any) {
    message.error(error?.message || $t('page.system.common.saveFailed'));
  } finally {
    saving.value = false;
  }
}

function handleReset() {
  config.value = defaultConfig();
  message.info($t('page.system.performanceConfig.resetTip'));
}

onMounted(() => loadConfig());
</script>

<template>
  <Page>
    <Spin :spinning="loading">
      <!-- 顶部说明 -->
      <Card class="mb-4 config-card">
        <div class="flex items-start gap-3">
          <IconifyIcon icon="lucide:settings" class="mt-1 text-xl text-primary" />
          <div>
            <div class="text-base font-semibold">
              {{ $t('page.system.performanceConfig.title') }}
            </div>
            <div class="mt-1 text-sm text-gray-500">
              {{ $t('page.system.performanceConfig.desc') }}
            </div>
          </div>
        </div>
      </Card>

      <!-- 统计口径 -->
      <Card
        :title="$t('page.system.performanceConfig.scopeTitle')"
        class="mb-4 config-card"
      >
        <div class="mb-4">
          <div class="mb-2 font-medium">
            {{ $t('page.system.performanceConfig.statScope') }}
          </div>
          <RadioGroup
            v-model:value="config.statScope"
            :disabled="!canManage"
            class="w-full"
          >
            <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
              <label
                class="flex cursor-pointer items-start gap-2 rounded border p-3 hover:bg-gray-50"
              >
                <Radio :value="'all_users'" class="mt-1" />
                <span>
                  <span class="font-medium">
                    {{ $t('page.system.performanceConfig.statScopeAllUsers') }}
                  </span>
                  <div class="text-xs text-gray-500">
                    {{
                      $t(
                        'page.system.performanceConfig.statScopeAllUsersDesc',
                      )
                    }}
                  </div>
                </span>
              </label>
              <label
                class="flex cursor-pointer items-start gap-2 rounded border p-3 hover:bg-gray-50"
              >
                <Radio :value="SCOPE_PLAN_SALES" class="mt-1" />
                <span>
                  <span class="font-medium">
                    {{ $t('page.system.performanceConfig.statScopePlanSales') }}
                  </span>
                  <div class="text-xs text-gray-500">
                    {{
                      $t(
                        'page.system.performanceConfig.statScopePlanSalesDesc',
                      )
                    }}
                  </div>
                </span>
              </label>
            </div>
          </RadioGroup>
          <div class="mt-2 text-xs text-gray-400">
            {{ $t('page.system.performanceConfig.statScopeTip') }}
          </div>
        </div>
        <div
          class="flex items-center justify-between rounded border p-3"
          :class="{ 'opacity-60': config.statScope !== SCOPE_ALL_USERS }"
        >
          <div>
            <div class="font-medium">
              {{ $t('page.system.performanceConfig.showZeroTargetRows') }}
            </div>
            <div class="text-xs text-gray-500">
              {{ $t('page.system.performanceConfig.showZeroTargetRowsTip') }}
            </div>
          </div>
          <Switch
            v-model:checked="config.showZeroTargetRows"
            :disabled="!canManage || config.statScope !== 'all_users'"
          />
        </div>
      </Card>

      <!-- 金额隐私 -->
      <Card
        :title="$t('page.system.performanceConfig.privacyTitle')"
        class="mb-4 config-card"
      >
        <Alert
          :message="$t('page.system.performanceConfig.privacyAlert')"
          type="warning"
          show-icon
          class="mb-4"
        />
        <div class="space-y-3">
          <div class="flex items-center justify-between rounded border p-3">
            <div>
              <div class="font-medium">
                {{ $t('page.system.performanceConfig.hideOtherActual') }}
              </div>
              <div class="text-xs text-gray-500">
                {{ $t('page.system.performanceConfig.hideOtherActualTip') }}
              </div>
            </div>
            <Switch
              v-model:checked="config.hideOtherActual"
              :disabled="!canManage"
            />
          </div>
          <div class="flex items-center justify-between rounded border p-3">
            <div>
              <div class="font-medium">
                {{ $t('page.system.performanceConfig.hideOtherTarget') }}
              </div>
              <div class="text-xs text-gray-500">
                {{ $t('page.system.performanceConfig.hideOtherTargetTip') }}
              </div>
            </div>
            <Switch
              v-model:checked="config.hideOtherTarget"
              :disabled="!canManage"
            />
          </div>
        </div>
      </Card>

      <!-- 榜单展示 -->
      <Card
        :title="$t('page.system.performanceConfig.rankTitle')"
        class="mb-4 config-card"
      >
        <div class="flex flex-wrap items-center gap-4">
          <span class="text-gray-600">
            {{ $t('page.system.performanceConfig.rankingTopN') }}
          </span>
          <InputNumber
            v-model:value="config.rankingTopN"
            :min="3"
            :max="100"
            :step="1"
            :precision="0"
            :disabled="!canManage"
            style="width: 120px"
          />
          <span class="text-gray-600">
            {{ $t('page.system.performanceConfig.rankingTopNUnit') }}
          </span>
          <span class="text-xs text-gray-400">
            {{ $t('page.system.performanceConfig.rankingTopNTip') }}
          </span>
        </div>
      </Card>

      <!-- 底部操作按钮 -->
      <div class="mt-6 flex justify-end gap-3">
        <Button :disabled="!canManage" @click="handleReset">
          <template #icon><IconifyIcon icon="lucide:rotate-ccw" /></template>
          {{ $t('page.system.performanceConfig.reset') }}
        </Button>
        <Button
          type="primary"
          :loading="saving"
          :disabled="!canManage"
          @click="handleSave"
        >
          <template #icon><IconifyIcon icon="lucide:save" /></template>
          {{ $t('page.system.performanceConfig.button.manage') }}
        </Button>
      </div>
    </Spin>
  </Page>
</template>

<style scoped>
  .config-card {
    margin-bottom: 16px;
  }
</style>
