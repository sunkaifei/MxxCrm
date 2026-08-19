<script lang="ts" setup>
import { onMounted, ref } from 'vue';

import { LucidePlus } from '@vben/icons';

import {
  Button,
  Card,
  DatePicker,
  Empty,
  Form,
  FormItem,
  Input,
  message,
  Modal,
  Popconfirm,
  Switch,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';
import dayjs, { type Dayjs } from 'dayjs';

import {
  deleteResumeApi,
  getMyProfileApi,
  saveResumeApi,
  updateResumeApi,
  type ResumeItem,
} from '#/api';
import { $t } from '#/locales';

const kindTabs = [
  { key: 1, label: $t('page.system.profile.kindEducation') },
  { key: 2, label: $t('page.system.profile.kindWork') },
  { key: 3, label: $t('page.system.profile.kindCert') },
];

const activeKind = ref(1);
const allResume = ref<ResumeItem[]>([]);
const loading = ref(false);

const visible = ref(false);
const editing = ref<ResumeItem | null>(null);
const saving = ref(false);
const form = ref({
  title: '',
  org: '',
  range: undefined as [Dayjs, Dayjs] | undefined,
  rangeOpen: false,
  remark: '',
  isPublic: false,
});

async function load() {
  loading.value = true;
  try {
    const data = await getMyProfileApi();
    allResume.value = data.resume || [];
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const list = () => allResume.value.filter((r) => r.kind === activeKind.value);

function kindLabel(kind: number) {
  return kindTabs.find((t) => t.key === kind)?.label || '';
}

function openCreate() {
  editing.value = null;
  form.value = {
    title: '',
    org: '',
    range: undefined,
    rangeOpen: false,
    remark: '',
    isPublic: false,
  };
  visible.value = true;
}

function openEdit(item: ResumeItem) {
  editing.value = item;
  form.value = {
    title: item.title || '',
    org: item.org || '',
    range:
      item.startDate
        ? [dayjs(item.startDate), item.endDate ? dayjs(item.endDate) : dayjs()]
        : undefined,
    rangeOpen: false,
    remark: item.remark || '',
    isPublic: item.isPublic === 1,
  };
  visible.value = true;
}

async function handleSave() {
  if (!form.value.title.trim()) {
    message.warning($t('page.system.profile.titleRequired'));
    return;
  }
  saving.value = true;
  try {
    const payload: ResumeItem = {
      id: editing.value?.id,
      kind: activeKind.value,
      title: form.value.title,
      org: form.value.org || undefined,
      startDate: form.value.range?.[0]?.format('YYYY-MM-DD'),
      endDate: form.value.range?.[1]?.format('YYYY-MM-DD'),
      remark: form.value.remark || undefined,
      isPublic: activeKind.value === 3 && form.value.isPublic ? 1 : 0,
    };
    if (editing.value) {
      await updateResumeApi(payload);
    } else {
      await saveResumeApi(payload);
    }
    message.success($t('ui.notification.update_success'));
    visible.value = false;
    await load();
  } finally {
    saving.value = false;
  }
}

async function handleDelete(item: ResumeItem) {
  if (!item.id) return;
  await deleteResumeApi(item.id);
  message.success($t('ui.notification.delete_success'));
  await load();
}
</script>

<template>
  <Card size="small">
    <div class="toolbar">
      <div class="kind-tabs">
        <div
          v-for="t in kindTabs"
          :key="t.key"
          :class="{ active: activeKind === t.key }"
          class="kind-tab"
          @click="activeKind = t.key"
        >
          {{ t.label }}
        </div>
      </div>
      <Button type="primary" size="small" @click="openCreate">
        <LucidePlus /> {{ $t('ui.button.add') }}
      </Button>
    </div>

    <div v-if="list().length" style="margin-top: 16px">
      <Timeline>
        <TimelineItem v-for="item in list()" :key="item.id">
          <div class="resume-item">
            <div class="resume-head">
              <span class="resume-title">{{ item.title }}</span>
              <Tag v-if="item.kind === 3 && item.isPublic === 1" color="blue">
                {{ $t('page.system.profile.publicTag') }}
              </Tag>
              <span class="resume-actions">
                <Button size="small" type="link" @click="openEdit(item)">
                  {{ $t('ui.button.edit') }}
                </Button>
                <Popconfirm
                  :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.system.profile.resumeModule') })"
                  @confirm="handleDelete(item)"
                >
                  <Button danger size="small" type="link">
                    {{ $t('ui.button.delete', { moduleName: $t('page.system.profile.resumeModule') }) }}
                  </Button>
                </Popconfirm>
              </span>
            </div>
            <div v-if="item.org" class="resume-org">{{ item.org }}</div>
            <div class="resume-date">
              {{ item.startDate || '?' }} ~ {{ item.endDate || $t('page.system.profile.present') }}
            </div>
            <div v-if="item.remark" class="resume-remark">{{ item.remark }}</div>
          </div>
        </TimelineItem>
      </Timeline>
    </div>
    <Empty v-else :description="$t('page.system.profile.noResume')" style="margin-top: 32px" />

    <Modal
      v-model:open="visible"
      :title="`${kindLabel(activeKind)} - ${editing ? $t('ui.button.edit') : $t('ui.button.add')}`"
      :confirm-loading="saving"
      @ok="handleSave"
    >
      <Form layout="vertical">
        <FormItem :label="$t('page.system.profile.resumeTitleLabel')" :rules="[{ required: true }]">
          <Input v-model:value="form.title" />
        </FormItem>
        <FormItem :label="$t('page.system.profile.resumeOrgLabel')">
          <Input v-model:value="form.org" />
        </FormItem>
        <FormItem :label="$t('page.system.profile.resumeRangeLabel')">
          <DatePicker.RangePicker v-model:value="form.range" style="width: 100%" />
        </FormItem>
        <FormItem :label="$t('page.system.profile.remark')">
          <Input.TextArea v-model:value="form.remark" :rows="3" />
        </FormItem>
        <FormItem v-if="activeKind === 3" :label="$t('page.system.profile.publicOnCard')">
          <Switch v-model:checked="form.isPublic" />
        </FormItem>
      </Form>
    </Modal>
  </Card>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.kind-tabs {
  display: flex;
  gap: 4px;
}

.kind-tab {
  padding: 4px 16px;
  cursor: pointer;
  border: 1px solid rgb(0 0 0 / 10%);
  border-radius: 4px;
  transition: all 0.2s;
}

.kind-tab.active {
  color: #1677ff;
  border-color: #1677ff;
}

.resume-item {
  padding-bottom: 4px;
}

.resume-head {
  display: flex;
  gap: 8px;
  align-items: center;
}

.resume-title {
  font-weight: 600;
}

.resume-actions {
  margin-left: auto;
}

.resume-org {
  margin-top: 2px;
  color: rgb(0 0 0 / 65%);
}

.resume-date {
  margin-top: 2px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}

.resume-remark {
  margin-top: 4px;
  font-size: 13px;
  color: rgb(0 0 0 / 55%);
}
</style>
