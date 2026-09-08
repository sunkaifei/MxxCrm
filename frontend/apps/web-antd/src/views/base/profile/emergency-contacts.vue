<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { LucidePlus } from '@vben/icons';

import {
  Button,
  Card,
  Empty,
  Form,
  FormItem,
  Input,
  message,
  Modal,
  Popconfirm,
  Select,
} from 'ant-design-vue';

import {
  deleteEmergencyContactApi,
  getMyProfileApi,
  saveEmergencyContactApi,
  updateEmergencyContactApi,
  type EmergencyContactItem,
} from '#/api';
import { $t } from '#/locales';

const relationOptions = [
  $t('page.system.profile.relationSpouse'),
  $t('page.system.profile.relationParent'),
  $t('page.system.profile.relationChild'),
  $t('page.system.profile.relationFriend'),
  $t('page.system.profile.relationOther'),
].map((r) => ({ label: r, value: r }));

const contacts = ref<EmergencyContactItem[]>([]);
const visible = ref(false);
const editing = ref<EmergencyContactItem | null>(null);
const saving = ref(false);
const form = reactive({ name: '', relation: '', mobile: '' });

async function load() {
  const data = await getMyProfileApi();
  contacts.value = data.emergencyContacts || [];
}

onMounted(load);

function openCreate() {
  if (contacts.value.length >= 3) {
    message.warning($t('page.system.profile.contactLimit'));
    return;
  }
  editing.value = null;
  Object.assign(form, { name: '', relation: '', mobile: '' });
  visible.value = true;
}

function openEdit(item: EmergencyContactItem) {
  editing.value = item;
  Object.assign(form, {
    name: item.name,
    relation: item.relation || '',
    mobile: item.mobile,
  });
  visible.value = true;
}

async function handleSave() {
  if (!form.name.trim() || !form.mobile.trim()) {
    message.warning($t('page.system.profile.contactRequired'));
    return;
  }
  saving.value = true;
  try {
    const payload: EmergencyContactItem = {
      id: editing.value?.id,
      name: form.name,
      relation: form.relation || undefined,
      mobile: form.mobile,
    };
    if (editing.value) {
      await updateEmergencyContactApi(payload);
    } else {
      await saveEmergencyContactApi(payload);
    }
    message.success($t('ui.notification.update_success'));
    visible.value = false;
    await load();
  } finally {
    saving.value = false;
  }
}

async function handleDelete(item: EmergencyContactItem) {
  if (!item.id) return;
  await deleteEmergencyContactApi(item.id);
  message.success($t('ui.notification.delete_success'));
  await load();
}
</script>

<template>
  <Card size="small">
    <div class="toolbar">
      <span class="tip">{{ $t('page.system.profile.contactTip') }}（{{ contacts.length }}/3）</span>
      <Button
        :disabled="contacts.length >= 3"
        size="small"
        type="primary"
        @click="openCreate"
      >
        <LucidePlus /> {{ $t('ui.button.add') }}
      </Button>
    </div>

    <div v-if="contacts.length" class="contact-list">
      <div v-for="c in contacts" :key="c.id" class="contact-card">
        <div class="contact-info">
          <div class="contact-name">
            {{ c.name }}
            <span v-if="c.relation" class="contact-relation">{{ c.relation }}</span>
          </div>
          <div class="contact-mobile">{{ c.mobile }}</div>
        </div>
        <div>
          <Button size="small" type="link" @click="openEdit(c)">
            {{ $t('ui.button.edit') }}
          </Button>
          <Popconfirm
            :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.system.profile.contactModule') })"
            @confirm="handleDelete(c)"
          >
            <Button danger size="small" type="link">
              {{ $t('ui.button.delete', { moduleName: $t('page.system.profile.contactModule') }) }}
            </Button>
          </Popconfirm>
        </div>
      </div>
    </div>
    <Empty v-else :description="$t('page.system.profile.noContact')" style="margin-top: 32px" />

    <Modal
      v-model:open="visible"
      :title="editing ? $t('ui.button.edit') : $t('ui.button.add')"
      :confirm-loading="saving"
      @ok="handleSave"
    >
      <Form layout="vertical">
        <FormItem :label="$t('page.system.profile.contactName')" :rules="[{ required: true }]">
          <Input v-model:value="form.name" />
        </FormItem>
        <FormItem :label="$t('page.system.profile.contactRelation')">
          <Select
            v-model:value="form.relation"
            :options="relationOptions"
            :placeholder="$t('ui.placeholder.select')"
            allow-clear
          />
        </FormItem>
        <FormItem :label="$t('page.system.profile.contactMobile')" :rules="[{ required: true }]">
          <Input v-model:value="form.mobile" :maxlength="20" />
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

.tip {
  font-size: 13px;
  color: rgb(0 0 0 / 45%);
}

.contact-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.contact-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border: 1px solid rgb(0 0 0 / 8%);
  border-radius: 8px;
}

.contact-name {
  font-weight: 600;
}

.contact-relation {
  margin-left: 8px;
  padding: 0 8px;
  font-size: 12px;
  color: #1677ff;
  background: rgb(22 119 255 / 8%);
  border-radius: 10px;
}

.contact-mobile {
  margin-top: 4px;
  font-family: monospace;
  color: rgb(0 0 0 / 65%);
}
</style>
