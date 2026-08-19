<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Button, Card, InputNumber, Select, Switch } from 'ant-design-vue';

import { getSettingConfigApi, updateSettingConfigApi } from '#/api';
import { $t } from '#/locales';

// ==================== 系统设置表单 ====================
const form = reactive({
  multiDevice: false,
  sessionTimeout: 8,
  maxDevices: 5,
  registerEnabled: false,
  inboundAuditEnabled: true,
  outboundAuditEnabled: true,
  inboundAuditMode: 0,
  outboundAuditMode: 0,
});
const saving = ref(false);

// 审核模式选项：0=严格 1=宽松
const auditModeOptions = [
  { label: '严格模式', value: 0 },
  { label: '宽松模式', value: 1 },
];

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
      form.inboundAuditEnabled = data.inboundAuditEnabled ?? true;
      form.outboundAuditEnabled = data.outboundAuditEnabled ?? true;
      form.inboundAuditMode = data.inboundAuditMode ?? 0;
      form.outboundAuditMode = data.outboundAuditMode ?? 0;
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
      inboundAuditEnabled: form.inboundAuditEnabled,
      outboundAuditEnabled: form.outboundAuditEnabled,
      inboundAuditMode: form.inboundAuditMode,
      outboundAuditMode: form.outboundAuditMode,
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

      <!-- 卡片四：库存审批策略 -->
      <Card title="库存审批策略" class="mb-4">
        <div class="config-row">
          <div>
            <div class="config-label">入库审核</div>
            <div class="config-tip">
              开启后入库单需审核才能变更库存，关闭后保存即生效
            </div>
          </div>
          <Switch v-model:checked="form.inboundAuditEnabled" />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">出库审核</div>
            <div class="config-tip">
              开启后出库单需审核才能变更库存，关闭后保存即生效
            </div>
          </div>
          <Switch v-model:checked="form.outboundAuditEnabled" />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">入库审核模式</div>
            <div class="config-tip">
              {{
                form.inboundAuditMode === 1
                  ? '宽松模式：库管等有权限角色可打开别人的草稿，可以编辑和提交'
                  : '严格模式：仅制单人可提交，其他人只能查看草稿'
              }}
            </div>
          </div>
          <Select
            v-model:value="form.inboundAuditMode"
            style="width: 160px"
            :options="auditModeOptions"
          />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">出库审核模式</div>
            <div class="config-tip">
              {{
                form.outboundAuditMode === 1
                  ? '宽松模式：库管等有权限角色可打开别人的草稿，可以编辑和提交'
                  : '严格模式：仅制单人可提交，其他人只能查看草稿'
              }}
            </div>
          </div>
          <Select
            v-model:value="form.outboundAuditMode"
            style="width: 160px"
            :options="auditModeOptions"
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
    </div>
  </Page>
</template>

<style scoped>
.setting-page {
  max-width: 900px;
}

.config-row {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.config-label {
  font-size: 14px;
  font-weight: 500;
}

.config-tip {
  margin-top: 4px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}

.mb-4 {
  margin-bottom: 16px;
}

.mt-4 {
  margin-top: 16px;
}
</style>
