<script lang="ts" setup>
// 员工档案抽屉：4-tab（信息/简历/紧急联系人/变更日志），由员工列表行内"档案"按钮打开
import { computed, ref, watch } from 'vue';

import { LucideLock } from '@vben/icons';

import {
  Button,
  DatePicker,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Input,
  message,
  Spin,
  TabPane,
  Tabs,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import {
  getHrArchiveDetailApi,
  getHrArchiveLogApi,
  unlockHrArchiveApi,
  updateHrArchiveApi,
  type HrArchiveDetailVO,
  type ProfileLogVO,
} from '#/api';
import { $t } from '#/locales';

const props = defineProps<{ adminId: number | null }>();

const visible = defineModel<boolean>('open', { default: false });

// 最大化状态：默认 75% 宽度，最大化到 100%
const isMaximized = ref(false);
const drawerWidth = computed(() => (isMaximized.value ? '100%' : '75%'));

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}
function handleClose() {
  visible.value = false;
}

// 关闭时重置最大化状态，保证下次打开为 75%
watch(visible, (val) => {
  if (!val) {
    isMaximized.value = false;
  }
});

const detailLoading = ref(false);
const detail = ref<HrArchiveDetailVO | null>(null);
const saving = ref(false);

const editForm = ref({
  nickName: '',
  email: '',
  mobile: '',
  hireDate: '' as string,
  probationMonths: undefined as number | undefined,
  bankCardNo: '',
  bankName: '',
  bankAccountName: '',
});

const operateTypeMap: Record<number, { text: string; color: string }> = {
  1: { text: $t('page.system.hrArchive.opFirstFill'), color: 'blue' },
  2: { text: $t('page.system.hrArchive.opSelfEdit'), color: 'green' },
  3: { text: $t('page.system.hrArchive.opHrEdit'), color: 'orange' },
  4: { text: $t('page.system.hrArchive.opUnlock'), color: 'red' },
};

async function openDetail(id: number) {
  detailLoading.value = true;
  try {
    detail.value = await getHrArchiveDetailApi(id);
    if (detail.value) {
      editForm.value = {
        nickName: detail.value.nickName || '',
        email: detail.value.email || '',
        mobile: detail.value.mobile || '',
        hireDate: detail.value.hireDate || '',
        probationMonths: detail.value.probationMonths,
        bankCardNo: detail.value.bankCardNo || '',
        bankName: detail.value.bankName || '',
        bankAccountName: detail.value.bankAccountName || '',
      };
      logs.value = [];
    }
  } finally {
    detailLoading.value = false;
  }
}

async function handleSave() {
  if (!detail.value) return;
  saving.value = true;
  try {
    await updateHrArchiveApi(detail.value.id, {
      nickName: editForm.value.nickName || undefined,
      email: editForm.value.email || undefined,
      mobile: editForm.value.mobile || undefined,
      hireDate: editForm.value.hireDate || undefined,
      probationMonths: editForm.value.probationMonths,
      bankCardNo: editForm.value.bankCardNo || undefined,
      bankName: editForm.value.bankName || undefined,
      bankAccountName: editForm.value.bankAccountName || undefined,
    });
    message.success($t('ui.notification.update_success'));
    await openDetail(detail.value.id);
  } finally {
    saving.value = false;
  }
}

async function handleUnlock(field: 'bank' | 'id_card') {
  if (!detail.value) return;
  await unlockHrArchiveApi(detail.value.id, field);
  message.success($t('page.system.hrArchive.unlockSuccess'));
  await openDetail(detail.value.id);
}

// 变更日志
const logs = ref<ProfileLogVO[]>([]);
const logLoading = ref(false);

async function loadLogs() {
  if (!detail.value) return;
  logLoading.value = true;
  try {
    const res: any = await getHrArchiveLogApi({
      page: 1,
      pageSize: 50,
      adminId: detail.value.id,
    });
    logs.value = res?.items || [];
  } finally {
    logLoading.value = false;
  }
}

function handleTabChange(key: string | number) {
  if (key === 'logs' && logs.value.length === 0) {
    loadLogs();
  }
}

watch(
  () => [visible.value, props.adminId] as const,
  ([open, id]) => {
    if (open && id) {
      openDetail(id);
    }
  },
);
</script>

<template>
  <Drawer
    v-model:open="visible"
    :title="$t('page.system.hrArchive.detailTitle')"
    :width="drawerWidth"
    :closable="false"
    :header-style="{ borderBottom: '1px solid #f0f0f0', padding: '16px 24px' }"
    placement="right"
  >
    <template #extra>
      <div class="flex items-center gap-1">
        <Button type="text" size="small" @click="toggleMaximize">
          {{ isMaximized ? '⤓ 还原' : '⤢' }}
        </Button>
        <Button type="text" size="small" @click="handleClose">
          <svg
            class="w-4 h-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </Button>
      </div>
    </template>
    <Spin :spinning="detailLoading">
      <Tabs default-active-key="info" @change="handleTabChange">
        <TabPane key="info" :tab="$t('page.system.hrArchive.tabInfo')">
          <template v-if="detail">
            <Descriptions :column="2" bordered size="small" style="margin-bottom: 16px">
              <DescriptionsItem :label="$t('page.system.hrArchive.employee')" :span="2">
                {{ detail.nickName }}（{{ detail.userName }}）
              </DescriptionsItem>
              <DescriptionsItem :label="$t('page.system.hrArchive.dept')" :span="2">
                <Tag v-for="d in detail.deptNames" :key="d" color="blue">{{ d }}</Tag>
              </DescriptionsItem>
              <DescriptionsItem :label="$t('page.system.profile.manager')">
                {{ detail.directManagerName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem :label="$t('page.system.hrArchive.idCard')">
                <span class="mono">{{ detail.idCardNo || '-' }}</span>
                <Tag v-if="detail.idLocked" color="orange">
                  <LucideLock :size="12" /> {{ $t('page.system.profile.locked') }}
                </Tag>
                <Button
                  v-if="detail.idLocked"
                  danger
                  size="small"
                  type="link"
                  @click="handleUnlock('id_card')"
                >
                  {{ $t('page.system.hrArchive.unlock') }}
                </Button>
              </DescriptionsItem>
              <DescriptionsItem :label="$t('page.system.hrArchive.bankCard')" :span="2">
                <span class="mono">{{ detail.bankCardNo || '-' }}</span>
                {{ detail.bankName }}
                <Tag v-if="detail.bankLocked" color="orange">
                  <LucideLock :size="12" /> {{ $t('page.system.profile.locked') }}
                </Tag>
                <Button
                  v-if="detail.bankLocked"
                  danger
                  size="small"
                  type="link"
                  @click="handleUnlock('bank')"
                >
                  {{ $t('page.system.hrArchive.unlock') }}
                </Button>
              </DescriptionsItem>
              <DescriptionsItem :label="$t('page.system.hrArchive.resumeCount')">
                {{ detail.resume?.length || 0 }}
              </DescriptionsItem>
              <DescriptionsItem :label="$t('page.system.hrArchive.contactCount')">
                {{ detail.emergencyContacts?.length || 0 }}
              </DescriptionsItem>
            </Descriptions>

            <div class="section-title">{{ $t('page.system.hrArchive.editSection') }}</div>
            <div class="edit-grid">
              <div class="edit-item">
                <span>{{ $t('page.system.profile.nickName') }}</span>
                <Input v-model:value="editForm.nickName" size="small" />
              </div>
              <div class="edit-item">
                <span>{{ $t('page.system.profile.email') }}</span>
                <Input v-model:value="editForm.email" size="small" />
              </div>
              <div class="edit-item">
                <span>{{ $t('page.system.profile.mobile') }}</span>
                <Input v-model:value="editForm.mobile" size="small" />
              </div>
              <div class="edit-item">
                <span>{{ $t('page.system.hrArchive.hireDate') }}</span>
                <DatePicker
                  :value="editForm.hireDate"
                  size="small"
                  style="width: 100%"
                  value-format="YYYY-MM-DD"
                  @change="(_: any, ds: string) => (editForm.hireDate = ds)"
                />
              </div>
              <div class="edit-item">
                <span>{{ $t('page.system.hrArchive.bankCardNo') }}</span>
                <Input v-model:value="editForm.bankCardNo" size="small" />
              </div>
              <div class="edit-item">
                <span>{{ $t('page.system.hrArchive.bankName') }}</span>
                <Input v-model:value="editForm.bankName" size="small" />
              </div>
            </div>
            <Button :loading="saving" style="margin-top: 12px" type="primary" @click="handleSave">
              {{ $t('ui.button.save') }}
            </Button>
          </template>
        </TabPane>

        <TabPane key="resume" :tab="$t('page.system.hrArchive.tabResume')">
          <template v-if="detail?.resume?.length">
            <Timeline>
              <TimelineItem v-for="r in detail.resume" :key="r.id">
                <div class="log-title">{{ r.title }}</div>
                <div class="log-sub">
                  {{ r.org }} · {{ r.startDate }} ~ {{ r.endDate || $t('page.system.profile.present') }}
                </div>
                <div v-if="r.remark" class="log-sub">{{ r.remark }}</div>
              </TimelineItem>
            </Timeline>
          </template>
          <div v-else class="empty-text">{{ $t('page.system.profile.noResume') }}</div>
        </TabPane>

        <TabPane key="contacts" :tab="$t('page.system.hrArchive.tabContacts')">
          <template v-if="detail?.emergencyContacts?.length">
            <div v-for="c in detail.emergencyContacts" :key="c.id" class="contact-row">
              <b>{{ c.name }}</b>
              <Tag v-if="c.relation">{{ c.relation }}</Tag>
              <span class="mono">{{ c.mobile }}</span>
            </div>
          </template>
          <div v-else class="empty-text">{{ $t('page.system.profile.noContact') }}</div>
        </TabPane>

        <TabPane key="logs" :tab="$t('page.system.hrArchive.tabLogs')">
          <Spin :spinning="logLoading">
            <template v-if="logs.length">
              <Timeline>
                <TimelineItem v-for="log in logs" :key="log.id">
                  <Tag :color="operateTypeMap[log.operateType || 0]?.color || 'default'">
                    {{ operateTypeMap[log.operateType || 0]?.text || '-' }}
                  </Tag>
                  <span class="log-field">{{ log.field }}</span>
                  <div v-if="log.oldValue || log.newValue" class="log-sub">
                    {{ log.oldValue || '-' }} → {{ log.newValue || '-' }}
                  </div>
                  <div class="log-meta">
                    {{ log.operatorName }} · {{ log.createDate }} {{ log.createTime }}
                  </div>
                </TimelineItem>
              </Timeline>
            </template>
            <div v-else class="empty-text">{{ $t('page.system.hrArchive.noLogs') }}</div>
          </Spin>
        </TabPane>
      </Tabs>
    </Spin>
  </Drawer>
</template>

<style scoped>
.mono {
  font-family: monospace;
}

.section-title {
  margin: 12px 0;
  font-weight: 600;
}

.edit-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.edit-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}

.log-title {
  font-weight: 600;
}

.log-sub {
  margin-top: 2px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}

.log-field {
  margin-left: 8px;
  font-family: monospace;
}

.log-meta {
  margin-top: 2px;
  font-size: 12px;
  color: rgb(0 0 0 / 35%);
}

.contact-row {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid rgb(0 0 0 / 4%);
}

.empty-text {
  padding: 24px;
  color: rgb(0 0 0 / 35%);
  text-align: center;
}
</style>
