<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';

import { Button, Card, Input, InputNumber, Select, Switch } from 'ant-design-vue';

import {
  getIntegrationListApi,
  getSettingConfigApi,
  toggleIntegrationApi,
  updateSettingConfigApi,
} from '#/api';
import { $t } from '#/locales';

const router = useRouter();

// ==================== 系统设置表单 ====================
type CaptchaType = 'click' | 'image' | 'none';
const form = reactive({
  multiDevice: false,
  sessionTimeout: 8,
  maxDevices: 5,
  registerEnabled: false,
  inboundAuditEnabled: true,
  outboundAuditEnabled: true,
  inboundAuditMode: 0,
  outboundAuditMode: 0,
  warehouseCrossViewEnabled: false,
  loginCaptchaType: 'click' as CaptchaType,
  registerCaptchaType: 'image' as CaptchaType,
  usernameMinLength: 3,
  usernameMaxLength: 20,
  usernameCharset: 'alnum_underscore',
  usernameLetterStart: false,
  usernameNotPureNumber: true,
  usernameBannedKeywords: 'admin,root,system,administrator,test',
});
const saving = ref(false);

// 审核模式选项：0=严格 1=宽松
const auditModeOptions = [
  { label: '严格模式', value: 0 },
  { label: '宽松模式', value: 1 },
];

// 验证码设置: 形态选项（image/click/none）
const captchaTypeOptions = [
  { label: '图形验证码', value: 'image' },
  { label: '文字点选验证码', value: 'click' },
  { label: '关闭（不启用）', value: 'none' },
];

// 注册策略: 用户名允许字符集选项
const usernameCharsetOptions = [
  { label: '字母、数字、下划线（推荐）', value: 'alnum_underscore' },
  { label: '字母、数字', value: 'alnum' },
  { label: '仅字母', value: 'alpha' },
  { label: '中英文、数字、下划线', value: 'cjk_alnum_underscore' },
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
      form.loginCaptchaType =
        (data.loginCaptchaType as any) ?? 'click';
      form.registerCaptchaType =
        (data.registerCaptchaType as any) ?? 'image';
      form.warehouseCrossViewEnabled = data.warehouseCrossViewEnabled ?? false;
      form.usernameMinLength = data.usernameMinLength ?? 3;
      form.usernameMaxLength = data.usernameMaxLength ?? 20;
      form.usernameCharset = (data.usernameCharset as any) ?? 'alnum_underscore';
      form.usernameLetterStart = data.usernameLetterStart ?? false;
      form.usernameNotPureNumber = data.usernameNotPureNumber ?? true;
      form.usernameBannedKeywords =
        data.usernameBannedKeywords ?? 'admin,root,system,administrator,test';
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
      loginCaptchaType: form.loginCaptchaType,
      registerCaptchaType: form.registerCaptchaType,
      warehouseCrossViewEnabled: form.warehouseCrossViewEnabled,
      usernameMinLength: form.usernameMinLength,
      usernameMaxLength: form.usernameMaxLength,
      usernameCharset: form.usernameCharset,
      usernameLetterStart: form.usernameLetterStart,
      usernameNotPureNumber: form.usernameNotPureNumber,
      usernameBannedKeywords: form.usernameBannedKeywords,
    });
    window.$message.success($t('page.system.setting.saveSuccess'));
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  loadConfig();
  loadAuthProviders();
});

// ==================== 第三方登录开关 ====================
// auth 分类（企业微信/钉钉）开关卡片：enabled 即登录入口开关，配置完整性由前端判断必填字段
const authProviders = ref<any[]>([]);
const authToggling = reactive<Record<number, boolean>>({});

async function loadAuthProviders() {
  try {
    const res = await getIntegrationListApi('auth');
    authProviders.value = (res as any[]) || [];
  } catch {
    authProviders.value = [];
  }
}

// 各 provider 必填字段（与后端 sso_status configured 口径一致）
const AUTH_REQUIRED_FIELDS: Record<string, string[]> = {
  wecom: ['corp_id', 'agent_id', 'corp_secret'],
  dingtalk: ['app_key', 'app_secret'],
};

function parseConfigJson(item: any): Record<string, any> {
  let cj = item?.configJson;
  if (typeof cj === 'string') {
    try {
      cj = JSON.parse(cj || '{}');
    } catch {
      cj = {};
    }
  }
  return cj || {};
}

function isAuthConfigured(item: any): boolean {
  const required = AUTH_REQUIRED_FIELDS[item?.integrationCode];
  if (!required) return false;
  const cj = parseConfigJson(item);
  return required.every((k) => {
    const v = cj[k];
    return typeof v === 'string' && v.trim().length > 0;
  });
}

async function handleToggleAuth(item: any, checked: boolean) {
  authToggling[item.id] = true;
  try {
    await toggleIntegrationApi(item.id, checked ? 1 : 0);
    item.enabled = checked ? 1 : 0;
    window.$message.success(
      `${item.integrationName}登录已${checked ? '开启' : '关闭'}`,
    );
  } catch {
    // global interceptor
  } finally {
    authToggling[item.id] = false;
  }
}

function goAuthConfig() {
  router.push('/system/integration-config?category=auth');
}
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

        <div class="config-row mt-4">
          <div>
            <div class="config-label">登录页验证码</div>
            <div class="config-tip">登录时弹出的人机验证形态；总开关"验证码开关"关闭时全部不启用</div>
          </div>
          <Select
            v-model:value="form.loginCaptchaType"
            style="width: 200px"
            :options="captchaTypeOptions"
          />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">注册页验证码</div>
            <div class="config-tip">提交注册表单时的人机验证形态（注册开关开启时生效）</div>
          </div>
          <Select
            v-model:value="form.registerCaptchaType"
            style="width: 200px"
            :options="captchaTypeOptions"
          />
        </div>

        <div class="config-row mt-4">
          <div>
            <div class="config-label">用户名最少字符</div>
            <div class="config-tip">注册用户名最短长度（1-20）</div>
          </div>
          <InputNumber
            v-model:value="form.usernameMinLength"
            :min="1"
            :max="20"
            style="width: 120px"
          />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">用户名最多字符</div>
            <div class="config-tip">注册用户名最长长度（3-50）</div>
          </div>
          <InputNumber
            v-model:value="form.usernameMaxLength"
            :min="3"
            :max="50"
            style="width: 120px"
          />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">允许的字符集</div>
            <div class="config-tip">注册用户名允许包含的字符范围</div>
          </div>
          <Select
            v-model:value="form.usernameCharset"
            style="width: 220px"
            :options="usernameCharsetOptions"
          />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">必须以字母开头</div>
            <div class="config-tip">开启后用户名首字符必须为英文字母</div>
          </div>
          <Switch v-model:checked="form.usernameLetterStart" />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">禁止纯数字用户名</div>
            <div class="config-tip">开启后不允许用纯数字（如手机号）作为用户名</div>
          </div>
          <Switch v-model:checked="form.usernameNotPureNumber" />
        </div>
        <div class="config-row mt-4">
          <div>
            <div class="config-label">禁止保留/敏感关键字</div>
            <div class="config-tip">含这些关键字的用户名禁止注册，多个用英文逗号分隔</div>
          </div>
          <Input
            v-model:value="form.usernameBannedKeywords"
            style="width: 260px"
            placeholder="如 admin,root,system"
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

      <!-- 卡片四：第三方登录 -->
      <Card title="第三方登录" class="mb-4">
        <div class="config-tip" style="margin-bottom: 12px">
          开启后，登录页显示对应图标，员工可扫码登录。未配置参数时可点击「前往配置」到第三方接口配置中心填写。
        </div>
        <div v-if="authProviders.length === 0" class="config-tip">
          暂无第三方登录配置，请
          <a @click="goAuthConfig">前往配置</a>
        </div>
        <template v-else>
          <div
            v-for="item in authProviders"
            :key="item.id"
            class="config-row mt-4"
          >
            <div>
              <div class="config-label">
                {{ item.integrationName }}登录
              </div>
              <div class="config-tip">
                开启后登录页显示{{ item.integrationName }}图标，员工可扫码登录
              </div>
              <div class="config-tip">
                <template v-if="isAuthConfigured(item)">
                  <span style="color: #1a9">状态：已配置</span>
                </template>
                <template v-else>
                  <span style="color: #d33">状态：未配置</span>
                  <a class="ml-2" @click="goAuthConfig">前往配置</a>
                </template>
              </div>
            </div>
            <Switch
              :checked="item.enabled === 1"
              :loading="!!authToggling[item.id]"
              checked-children="开"
              un-checked-children="关"
              @change="(checked: any) => handleToggleAuth(item, checked)"
            />
          </div>
        </template>
      </Card>

      <!-- 卡片五：库存审批策略 -->
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

      <!-- 卡片五：仓库数据互看 -->
      <Card title="仓库数据互看" class="mb-4">
        <div class="config-row">
          <div>
            <div class="config-label">库管互看其他仓库数据</div>
            <div class="config-tip">
              开启后，仓库库管可以查看其他仓库的档案与库存（只读），但不能修改、删除、停用他人仓库；关闭后库管仅能看到自己负责的仓库。跨仓修改权限仅管理层持有，与本开关无关。保存后即时生效，无需重启服务。
            </div>
          </div>
          <Switch v-model:checked="form.warehouseCrossViewEnabled" />
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
