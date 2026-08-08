<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  Card,
  InputNumber,
  Select,
  Switch,
} from 'ant-design-vue';

import {
  getSettingConfigApi,
  updateSettingConfigApi,
} from '#/api';
import { $t } from '#/locales';

// ==================== 系统设置表单 ====================
const form = reactive({
  multiDevice: false,
  sessionTimeout: 8,
  maxDevices: 5,
  registerEnabled: false,
});
const saving = ref(false);

// 会话超时下拉选项（小时）
const timeoutOptions = [
  { label: $t('page.system.setting.duration.2h'), value: 2 },
  { label: $t('page.system.setting.duration.4h'), value: 4 },
  { label: $t('page.system.setting.duration.8h'), value: 8 },
  { label: $t('page.system.setting.duration.12h'), value: 12 },
  { label: $t('page.system.setting.duration.24h'), value: 24 },
];

async function loadConfig() {
  try {
    const data = await getSettingConfigApi();
    if (data) {
      form.multiDevice = data.multiDevice;
      form.sessionTimeout = data.sessionTimeout || 8;
      form.maxDevices = data.maxDevices ?? 5;
      form.registerEnabled = data.registerEnabled ?? false;
    }
  } catch {
    // 忽略加载错误
  }
}

async function handleSaveConfig() {
  saving.value = true;
  try {
    await updateSettingConfigApi({
      multiDevice: form.multiDevice,
      sessionTimeout: form.sessionTimeout,
      maxDevices: form.maxDevices,
      registerEnabled: form.registerEnabled,
    });
    window.$message.success($t('page.system.setting.saveSuccess'));
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <Page>
    <div class="setting-page">
      <!-- 卡片一：会话超时 -->
      <Card :title="$t('page.system.setting.sessionTimeout')" class="mb-4">
        <div class="config-row">
          <div class="config-label">
            {{ $t('page.system.setting.sessionTimeoutDesc') }}
          </div>
          <Select
            v-model:value="form.sessionTimeout"
            style="width: 200px"
            :options="timeoutOptions"
          />
        </div>
        <div class="config-tip">{{ $t('page.system.setting.tip') }}</div>
        <div class="mt-4">
          <Button
            type="primary"
            :loading="saving"
            v-access:code="['system:setting:update']"
            @click="handleSaveConfig"
          >
            {{ $t('page.system.common.save') }}
          </Button>
        </div>
      </Card>

      <!-- 卡片二：登录策略 -->
      <Card :title="$t('page.system.setting.loginStrategy')" class="mb-4">
        <div class="config-row">
          <div>
            <div class="config-label">
              {{ $t('page.system.setting.multiDevice') }}
            </div>
            <div class="config-tip">
              {{ $t('page.system.setting.multiDeviceDesc') }}
            </div>
          </div>
          <Switch v-model:checked="form.multiDevice" />
        </div>

        <div v-if="form.multiDevice" class="config-row mt-4">
          <div>
            <div class="config-label">
              {{ $t('page.system.setting.maxDevices') }}
            </div>
            <div class="config-tip">
              {{ $t('page.system.setting.maxDevicesDesc') }}
            </div>
          </div>
          <InputNumber
            v-model:value="form.maxDevices"
            :min="0"
            :max="20"
            style="width: 160px"
          />
        </div>

        <div class="mt-4">
          <Button
            type="primary"
            :loading="saving"
            v-access:code="['system:setting:update']"
            @click="handleSaveConfig"
          >
            {{ $t('page.system.common.save') }}
          </Button>
        </div>
      </Card>

      <!-- 卡片三：注册策略 -->
      <Card :title="$t('page.system.setting.registerStrategy')" class="mb-4">
        <div class="config-row">
          <div>
            <div class="config-label">
              {{ $t('page.system.setting.registerEnabled') }}
            </div>
            <div class="config-tip">
              {{ $t('page.system.setting.registerEnabledDesc') }}
            </div>
          </div>
          <Switch v-model:checked="form.registerEnabled" />
        </div>

        <div class="mt-4">
          <Button
            type="primary"
            :loading="saving"
            v-access:code="['system:setting:update']"
            @click="handleSaveConfig"
          >
            {{ $t('page.system.common.save') }}
          </Button>
        </div>
      </Card>
    </div>
  </Page>
</template>

<style scoped>
.setting-page {
  max-width: 900px;
}

.config-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.config-label {
  font-size: 14px;
  font-weight: 500;
}

.config-tip {
  margin-top: 4px;
  font-size: 12px;
  color: rgba(0, 0, 0, 45%);
}

.mb-4 {
  margin-bottom: 16px;
}

.mt-4 {
  margin-top: 16px;
}
</style>
