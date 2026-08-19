<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { LucideLock } from '@vben/icons';

import {
  Alert,
  Button,
  Card,
  Col,
  Form,
  FormItem,
  Input,
  message,
  Row,
  Tag,
} from 'ant-design-vue';

import { getMyProfileApi, submitBankApi, submitIdCardApi } from '#/api';
import { $t } from '#/locales';

const emit = defineEmits<{ refresh: [] }>();

const idCard = reactive({ masked: '', locked: false });
const bank = reactive({
  maskedCardNo: '',
  bankName: '',
  maskedAccountName: '',
  locked: false,
});

const idForm = reactive({ idCardNo: '' });
const bankForm = reactive({ bankCardNo: '', bankName: '', bankAccountName: '' });
const submittingId = ref(false);
const submittingBank = ref(false);

async function load() {
  const data = await getMyProfileApi();
  idCard.masked = data.idCard?.masked || '';
  idCard.locked = data.idCard?.locked || false;
  bank.maskedCardNo = data.bank?.maskedCardNo || '';
  bank.bankName = data.bank?.bankName || '';
  bank.maskedAccountName = data.bank?.maskedAccountName || '';
  bank.locked = data.bank?.locked || false;
}

onMounted(load);

async function handleSubmitId() {
  submittingId.value = true;
  try {
    await submitIdCardApi(idForm.idCardNo.trim());
    message.success($t('page.system.profile.submitLockedSuccess'));
    await load();
    emit('refresh');
  } finally {
    submittingId.value = false;
  }
}

async function handleSubmitBank() {
  submittingBank.value = true;
  try {
    await submitBankApi({
      bankCardNo: bankForm.bankCardNo.trim(),
      bankName: bankForm.bankName || undefined,
      bankAccountName: bankForm.bankAccountName || undefined,
    });
    message.success($t('page.system.profile.submitLockedSuccess'));
    await load();
    emit('refresh');
  } finally {
    submittingBank.value = false;
  }
}
</script>

<template>
  <Alert
    :message="$t('page.system.profile.lockPolicyTitle')"
    :description="$t('page.system.profile.lockPolicyDesc')"
    type="info"
    show-icon
    style="margin-bottom: 16px"
  />
  <Row :gutter="16">
    <Col :xs="24" :md="12">
      <Card :title="$t('page.system.profile.idCardTitle')" size="small">
        <template v-if="idCard.locked">
          <div class="locked-value">
            <LucideLock :size="14" />
            <span class="masked">{{ idCard.masked }}</span>
            <Tag color="orange">{{ $t('page.system.profile.locked') }}</Tag>
          </div>
          <div class="locked-tip">{{ $t('page.system.profile.lockedTip') }}</div>
        </template>
        <Form v-else layout="vertical" @submit.prevent>
          <FormItem
            :label="$t('page.system.profile.idCardNo')"
            :rules="[{ required: true }]"
          >
            <Input
              v-model:value="idForm.idCardNo"
              :maxlength="18"
              :placeholder="$t('page.system.profile.idCardPlaceholder')"
            />
          </FormItem>
          <Button
            :loading="submittingId"
            :disabled="idForm.idCardNo.trim().length !== 18"
            type="primary"
            @click="handleSubmitId"
          >
            {{ $t('page.system.profile.submitAndLock') }}
          </Button>
        </Form>
      </Card>
    </Col>
    <Col :xs="24" :md="12">
      <Card :title="$t('page.system.profile.bankTitle')" size="small">
        <template v-if="bank.locked">
          <div class="locked-value">
            <LucideLock :size="14" />
            <span class="masked">{{ bank.maskedCardNo }}</span>
            <Tag color="orange">{{ $t('page.system.profile.locked') }}</Tag>
          </div>
          <div v-if="bank.bankName" class="locked-sub">{{ bank.bankName }}</div>
          <div class="locked-tip">{{ $t('page.system.profile.lockedTip') }}</div>
        </template>
        <Form v-else layout="vertical" @submit.prevent>
          <FormItem :label="$t('page.system.profile.bankCardNo')" :rules="[{ required: true }]">
            <Input v-model:value="bankForm.bankCardNo" :placeholder="$t('page.system.profile.bankCardPlaceholder')" />
          </FormItem>
          <FormItem :label="$t('page.system.profile.bankName')">
            <Input v-model:value="bankForm.bankName" />
          </FormItem>
          <FormItem :label="$t('page.system.profile.bankAccountName')">
            <Input v-model:value="bankForm.bankAccountName" />
          </FormItem>
          <Button
            :loading="submittingBank"
            :disabled="bankForm.bankCardNo.trim().length < 12"
            type="primary"
            @click="handleSubmitBank"
          >
            {{ $t('page.system.profile.submitAndLock') }}
          </Button>
        </Form>
      </Card>
    </Col>
  </Row>
</template>

<style scoped>
.locked-value {
  display: flex;
  gap: 8px;
  align-items: center;
}

.masked {
  font-family: monospace;
  font-size: 16px;
  letter-spacing: 2px;
}

.locked-sub {
  margin-top: 4px;
  color: rgb(0 0 0 / 65%);
}

.locked-tip {
  margin-top: 8px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}
</style>
