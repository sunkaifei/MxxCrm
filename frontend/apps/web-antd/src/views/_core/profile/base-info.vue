<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

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
  Row,
  Select,
  Switch,
  Tag,
  Textarea,
} from 'ant-design-vue';

import { getMyProfileApi, updateBasicApi } from '#/api';
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
  email: '',
  intro: '',
  wechat: '',
});

const visibility = reactive({
  showMobile: false,
  showWechat: false,
  showSkills: false,
  showBirthday: false,
});

async function load() {
  loading.value = true;
  try {
    const data = await getMyProfileApi();
    employ.value = data.employ || {};
    form.nickName = data.basic?.nickName || '';
    form.gender = data.basic?.gender;
    form.email = data.basic?.email || '';
    form.intro = data.basic?.intro || '';
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
      email: form.email || undefined,
      intro: form.intro || undefined,
      visibility: { ...visibility },
    });
    message.success($t('ui.notification.update_success'));
    emit('refresh');
  } finally {
    saving.value = false;
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
          <FormItem :label="$t('page.system.profile.email')">
            <Input v-model:value="form.email" :placeholder="$t('ui.placeholder.input')" />
          </FormItem>
          <FormItem :label="$t('page.system.profile.intro')">
            <Textarea v-model:value="form.intro" :rows="3" />
          </FormItem>
        </Form>
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
          <Switch v-model:checked="visibility.showMobile" size="small" />
        </div>
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showWechat') }}</span>
          <Switch v-model:checked="visibility.showWechat" size="small" />
        </div>
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showSkills') }}</span>
          <Switch v-model:checked="visibility.showSkills" size="small" />
        </div>
        <div class="vis-row">
          <span>{{ $t('page.system.profile.showBirthday') }}</span>
          <Switch v-model:checked="visibility.showBirthday" size="small" />
        </div>
        <div class="readonly-tip">{{ $t('page.system.profile.visibilityTip') }}</div>
      </Card>
    </Col>
  </Row>
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

.vis-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}
</style>
