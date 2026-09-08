<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import {
  Button,
  Card,
  Col,
  Descriptions,
  DescriptionsItem,
  Form,
  FormItem,
  Input,
  message,
  Modal,
  Row,
  Select,
  Switch,
  Tag,
  Textarea,
} from 'ant-design-vue';

import {
  getMyProfileApi,
  sendProfileOtpApi,
  updateBasicApi,
  updateEmailApi,
  updateMobileApi,
} from '#/api';
import { $t } from '#/locales';

const emit = defineEmits<{ refresh: [] }>();

const employ = ref<any>({});
const loading = ref(false);
const saving = ref(false);

const genderOptions = [
  { label: $t('page.system.profile.genderMale'), value: 0 },
  { label: $t('page.system.profile.genderFemale'), value: 1 },
  { label: $t('page.system.profile.genderUnknown'), value: 2 },
];

const form = reactive({
  nickName: '',
  gender: undefined as number | undefined,
  intro: '',
  wechat: '',
});

const visibility = reactive({
  showMobile: false,
  showWechat: false,
  showSkills: false,
  showBirthday: false,
});

const savingVis = ref(false);

/** 名片公开开关：切换即保存（即时生效，无需等底部「保存」按钮） */
async function saveVisibility() {
  savingVis.value = true;
  try {
    await updateBasicApi({ visibility: { ...visibility } });
  } catch (e: any) {
    message.error(e?.message || $t('ui.notification.update_failed'));
    await load(); // 保存失败时还原为服务器值
  } finally {
    savingVis.value = false;
  }
}

// ========== 账号安全（邮箱/手机号走独立安全接口） ==========
const security = reactive({
  email: '',
  mobileMasked: '',
});
const hasBoundEmail = computed(() => !!security.email);

const emailModal = reactive({ visible: false, saving: false });
const emailForm = reactive({
  newEmail: '',
  oldOtp: '',
  newOtp: '',
  password: '',
});
const emailOtpState = reactive({ oldSending: false, newSending: false });

const mobileModal = reactive({ visible: false, saving: false });
const mobileForm = reactive({ mobile: '', password: '' });

async function load() {
  loading.value = true;
  try {
    const data = await getMyProfileApi();
    employ.value = data.employ || {};
    form.nickName = data.basic?.nickName || '';
    form.gender = data.basic?.gender;
    form.intro = data.basic?.intro || '';
    security.email = data.basic?.email || '';
    security.mobileMasked = data.basic?.mobileMasked || '';
    Object.assign(visibility, data.visibility || {});
  } finally {
    loading.value = false;
  }
}

onMounted(load);

async function handleSave() {
  saving.value = true;
  try {
    await updateBasicApi({
      nickName: form.nickName || undefined,
      gender: form.gender,
      intro: form.intro || undefined,
      visibility: { ...visibility },
    });
    message.success($t('ui.notification.update_success'));
    emit('refresh');
  } finally {
    saving.value = false;
  }
}

function openEmailModal() {
  emailForm.newEmail = '';
  emailForm.oldOtp = '';
  emailForm.newOtp = '';
  emailForm.password = '';
  emailModal.visible = true;
}

async function sendEmailOtp(kind: 'old' | 'new') {
  if (kind === 'new' && (!emailForm.newEmail.trim() || !emailForm.newEmail.includes('@'))) {
    message.warning($t('page.system.profile.security.newEmailFirst'));
    return;
  }
  const loadingKey = kind === 'old' ? 'oldSending' : 'newSending';
  emailOtpState[loadingKey] = true;
  try {
    const masked = await sendProfileOtpApi({
      action: kind === 'old' ? 'email_old' : 'email_new',
      email: kind === 'new' ? emailForm.newEmail.trim() : undefined,
    });
    message.success($t('page.system.profile.security.otpSent', { email: masked }));
  } catch (e: any) {
    message.error(e?.message || $t('ui.notification.update_failed'));
  } finally {
    emailOtpState[loadingKey] = false;
  }
}

async function submitEmail() {
  const newEmail = emailForm.newEmail.trim();
  if (!newEmail || !newEmail.includes('@')) {
    message.warning($t('page.system.profile.security.newEmailFirst'));
    return;
  }
  if (!emailForm.password) {
    message.warning($t('page.system.profile.security.passwordRequired'));
    return;
  }
  emailModal.saving = true;
  try {
    await updateEmailApi({
      password: emailForm.password,
      newEmail,
      oldOtp: emailForm.oldOtp || undefined,
      newOtp: emailForm.newOtp || undefined,
    });
    message.success($t('ui.notification.update_success'));
    emailModal.visible = false;
    load();
  } catch (e: any) {
    message.error(e?.message || $t('ui.notification.update_failed'));
  } finally {
    emailModal.saving = false;
  }
}

function openMobileModal() {
  mobileForm.mobile = '';
  mobileForm.password = '';
  mobileModal.visible = true;
}

async function submitMobile() {
  const mobile = mobileForm.mobile.trim();
  if (!mobile) {
    message.warning($t('page.system.profile.security.mobileRequired'));
    return;
  }
  if (!/^1[3-9]\d{9}$/.test(mobile)) {
    message.warning($t('page.system.profile.security.mobileFormat'));
    return;
  }
  if (!mobileForm.password) {
    message.warning($t('page.system.profile.security.passwordRequired'));
    return;
  }
  mobileModal.saving = true;
  try {
    await updateMobileApi({ password: mobileForm.password, mobile });
    message.success($t('ui.notification.update_success'));
    mobileModal.visible = false;
    load();
  } catch (e: any) {
    message.error(e?.message || $t('ui.notification.update_failed'));
  } finally {
    mobileModal.saving = false;
  }
}
</script>

<template>
  <Row :gutter="16">
    <Col :xs="24" :md="12">
      <Card :title="$t('page.system.profile.basicInfo')" :loading="loading" size="small">
        <Form layout="vertical">
          <FormItem :label="$t('page.system.profile.nickName')">
            <Input v-model:value="form.nickName" :placeholder="$t('ui.placeholder.input')" />
          </FormItem>
          <FormItem :label="$t('page.system.profile.gender')">
            <Select
              v-model:value="form.gender"
              :options="genderOptions"
              :placeholder="$t('ui.placeholder.select')"
            />
          </FormItem>
          <FormItem :label="$t('page.system.profile.intro')">
            <Textarea v-model:value="form.intro" :rows="3" />
          </FormItem>
        </Form>
      </Card>

      <Card
        :title="$t('page.system.profile.security.title')"
        size="small"
        style="margin-top: 16px"
      >
        <Descriptions :column="1" size="small">
          <DescriptionsItem :label="$t('page.system.profile.security.account')">
            {{ employ.userName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.profile.email')">
            {{ security.email || '-' }}
            <a style="margin-left: 8px" @click="openEmailModal">
              {{ $t('page.system.profile.security.modify') }}
            </a>
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.profile.mobile')">
            {{ security.mobileMasked || '-' }}
            <a style="margin-left: 8px" @click="openMobileModal">
              {{ $t('page.system.profile.security.modify') }}
            </a>
          </DescriptionsItem>
        </Descriptions>
        <div class="readonly-tip">{{ $t('page.system.profile.security.tip') }}</div>
      </Card>
    </Col>
    <Col :xs="24" :md="12">
      <Card
        :title="$t('page.system.profile.employInfo')"
        size="small"
        style="margin-bottom: 16px"
      >
        <Descriptions :column="1" size="small">
          <DescriptionsItem :label="$t('page.system.profile.userName')">
            {{ employ.userName }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.profile.dept')">
            <template v-if="employ.deptNames?.length">
              <Tag v-for="d in employ.deptNames" :key="d" color="blue">{{ d }}</Tag>
            </template>
            <span v-else>-</span>
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.profile.post')">
            <template v-if="employ.postNames?.length">
              <Tag v-for="p in employ.postNames" :key="p" color="green">{{ p }}</Tag>
            </template>
            <span v-else>-</span>
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.profile.manager')">
            {{ employ.directManagerName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem :label="$t('page.system.profile.hireDate')">
            {{ employ.hireDate || '-' }}
          </DescriptionsItem>
        </Descriptions>
        <div class="readonly-tip">{{ $t('page.system.profile.employReadonlyTip') }}</div>
      </Card>

      <Card :title="$t('page.system.profile.visibilityTitle')" size="small">
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showMobile') }}</span>
          <Switch
            v-model:checked="visibility.showMobile"
            :loading="savingVis"
            size="small"
            @change="saveVisibility"
          />
        </div>
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showWechat') }}</span>
          <Switch
            v-model:checked="visibility.showWechat"
            :loading="savingVis"
            size="small"
            @change="saveVisibility"
          />
        </div>
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showSkills') }}</span>
          <Switch
            v-model:checked="visibility.showSkills"
            :loading="savingVis"
            size="small"
            @change="saveVisibility"
          />
        </div>
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showBirthday') }}</span>
          <Switch
            v-model:checked="visibility.showBirthday"
            :loading="savingVis"
            size="small"
            @change="saveVisibility"
          />
        </div>
        <div class="readonly-tip">{{ $t('page.system.profile.visibilityTip') }}</div>
      </Card>
    </Col>
  </Row>

  <Modal
    :open="emailModal.visible"
    :title="$t('page.system.profile.security.modifyEmail')"
    :confirm-loading="emailModal.saving"
    :ok-text="$t('ui.button.ok')"
    :cancel-text="$t('ui.button.cancel')"
    @ok="submitEmail"
    @cancel="emailModal.visible = false"
  >
    <Form layout="vertical">
      <FormItem :label="$t('page.system.profile.security.newEmail')">
        <Input
          v-model:value="emailForm.newEmail"
          :placeholder="$t('page.system.profile.security.newEmail')"
        />
      </FormItem>
      <template v-if="hasBoundEmail">
        <FormItem :label="$t('page.system.profile.security.oldEmailOtp')">
          <div class="otp-row">
            <Input
              v-model:value="emailForm.oldOtp"
              :placeholder="$t('page.system.profile.security.otpPlaceholder')"
            />
            <Button :loading="emailOtpState.oldSending" size="small" @click="sendEmailOtp('old')">
              {{ $t('page.system.profile.security.sendOtp') }}
            </Button>
          </div>
        </FormItem>
      </template>
      <FormItem :label="$t('page.system.profile.security.newEmailOtp')">
        <div class="otp-row">
          <Input
            v-model:value="emailForm.newOtp"
            :placeholder="$t('page.system.profile.security.otpPlaceholder')"
          />
          <Button :loading="emailOtpState.newSending" size="small" @click="sendEmailOtp('new')">
            {{ $t('page.system.profile.security.sendOtp') }}
          </Button>
        </div>
      </FormItem>
      <FormItem :label="$t('page.system.profile.security.password')">
        <Input.Password
          v-model:value="emailForm.password"
          :placeholder="$t('page.system.profile.security.passwordPlaceholder')"
        />
      </FormItem>
      <div class="readonly-tip">{{ $t('page.system.profile.security.emailTip') }}</div>
    </Form>
  </Modal>

  <Modal
    :open="mobileModal.visible"
    :title="$t('page.system.profile.security.modifyMobile')"
    :confirm-loading="mobileModal.saving"
    :ok-text="$t('ui.button.ok')"
    :cancel-text="$t('ui.button.cancel')"
    @ok="submitMobile"
    @cancel="mobileModal.visible = false"
  >
    <Form layout="vertical">
      <FormItem :label="$t('page.system.profile.security.newMobile')">
        <Input
          v-model:value="mobileForm.mobile"
          :placeholder="$t('page.system.profile.security.mobilePlaceholder')"
        />
      </FormItem>
      <FormItem :label="$t('page.system.profile.security.password')">
        <Input.Password
          v-model:value="mobileForm.password"
          :placeholder="$t('page.system.profile.security.passwordPlaceholder')"
        />
      </FormItem>
      <div class="readonly-tip">{{ $t('page.system.profile.security.mobileTip') }}</div>
    </Form>
  </Modal>

  <div style="margin-top: 16px; text-align: center">
    <Button :loading="saving" type="primary" @click="handleSave">
      {{ $t('ui.button.save') }}
    </Button>
  </div>
</template>

<style scoped>
.readonly-tip {
  margin-top: 8px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}

.otp-row {
  display: flex;
  gap: 8px;
}

.otp-row .ant-input {
  flex: 1;
}

.vis-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}
</style>
