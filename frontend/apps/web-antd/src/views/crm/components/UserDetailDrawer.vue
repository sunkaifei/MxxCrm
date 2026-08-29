<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { LucideLock } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Button,
  DatePicker,
  Descriptions,
  Drawer,
  Empty,
  Input,
  message,
  Spin,
  Tabs,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import {
  getHrArchiveDetailApi,
  getHrArchiveLogApi,
  getUserDetailApi,
  unlockHrArchiveApi,
  updateHrArchiveApi,
  type HrArchiveDetailVO,
  type ProfileLogVO,
} from '#/api';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

const props = withDefaults(
  defineProps<{
    id?: number | string;
    visible: boolean;
  }>(),
  {
    id: undefined,
  },
);

const emit = defineEmits<{
  (e: 'update:visible', visible: boolean): void;
}>();

const TabPane = Tabs.TabPane;

const accessStore = useAccessStore();
const userStore = useUserStore();

const innerVisible = ref(false);
const loading = ref(false);
const userData = ref<any>(null);
const activeTab = ref<string>('summary');

const statusMap: Record<number, { color: string; text: string }> = {
  0: { text: '停用', color: 'default' },
  1: { text: '正常', color: 'success' },
};

const genderMap: Record<number, string> = {
  0: '男',
  1: '女',
  2: '未设置',
};

const userTypeMap: Record<number, { color: string; label: string }> = {
  1: { label: '超级管理员', color: 'red' },
  2: { label: '管理员', color: 'orange' },
  3: { label: '普通员工', color: 'blue' },
};

const { isSuperAdmin } = useSuperAdminGuard();

const isHrOrAdmin = computed(() => {
  const roles = userStore.userInfo?.roles || [];
  const permissions = accessStore.accessCodes || [];
  return (
    isSuperAdmin.value ||
    roles.includes('super_admin') ||
    roles.includes('system_admin') ||
    roles.includes('hr') ||
    permissions.includes('system:user:view')
  );
});

const fullWidth = computed(() => {
  const w = Math.min(window.innerWidth * 0.75, 1200);
  return `${w}px`;
});

const filteredDeptNames = computed(() => {
  const names = userData.value?.deptNames as string[] | undefined;
  return names ? names.filter(Boolean) : [];
});

const filteredRoleNames = computed(() => {
  const names = userData.value?.roleNames as string[] | undefined;
  return names ? names.filter(Boolean) : [];
});

const filteredPostNames = computed(() => {
  const names = userData.value?.postNames as string[] | undefined;
  return names ? names.filter(Boolean) : [];
});

const userInitial = computed(() => {
  const name = displayName.value;
  return name.charAt(0).toUpperCase();
});

/** 展示名：普通用户仅显示姓名；HR/管理员在无姓名时可回退到账号 */
const displayName = computed(() => {
  const nick = userData.value?.nickName;
  if (nick) return nick;
  return isHrOrAdmin.value ? userData.value?.userName || '未知' : '未知';
});

const formattedLastLogin = computed(() => {
  const raw = userData.value?.lastLoginTime || userData.value?.loginDate;
  return raw ? formatDateTime(raw) : '—';
});

const formattedCreateTime = computed(() => {
  const raw = userData.value?.createTime;
  return raw ? formatDateTime(raw) : '—';
});

watch(
  () => props.visible,
  (val) => {
    innerVisible.value = val;
  },
);

watch(innerVisible, (val) => {
  emit('update:visible', val);
  if (!val) {
    userData.value = null;
    archiveDetail.value = null;
    archiveLogs.value = [];
    activeTab.value = 'summary';
  }
});

watch(
  () => props.id,
  (newId) => {
    if (newId && innerVisible.value) fetchUserDetail();
  },
);

// ---- HR 档案管理（原独立档案抽屉能力并入，门槛沿用 system:hr-archive:view，后端仍逐接口校验）----
const canManageArchive = computed(() => {
  if (isSuperAdmin.value) return true;
  const permissions = accessStore.accessCodes || [];
  return permissions.includes('system:hr-archive:view');
});

const archiveDetail = ref<HrArchiveDetailVO | null>(null);
const archiveLoading = ref(false);
const saving = ref(false);
const archiveLogs = ref<ProfileLogVO[]>([]);
const logLoading = ref(false);

// 履历/紧急联系人统一来自 HR 档案详情（resume.kind：1教育 2工作 3证书）
const resumeItems = computed(() => archiveDetail.value?.resume || []);

const workExperiences = computed(() =>
  resumeItems.value.filter((item) => item.kind === 2),
);

const educations = computed(() =>
  resumeItems.value.filter((item) => item.kind === 1),
);

const certificates = computed(() =>
  resumeItems.value.filter((item) => item.kind === 3),
);

const emergencyContacts = computed(
  () => archiveDetail.value?.emergencyContacts || [],
);

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

function fillEditForm(detail: HrArchiveDetailVO) {
  editForm.value = {
    nickName: detail.nickName || '',
    email: detail.email || '',
    mobile: detail.mobile || '',
    hireDate: detail.hireDate || '',
    probationMonths: detail.probationMonths,
    bankCardNo: detail.bankCardNo || '',
    bankName: detail.bankName || '',
    bankAccountName: detail.bankAccountName || '',
  };
}

async function loadArchiveDetail(id: number) {
  archiveLoading.value = true;
  try {
    const detail = await getHrArchiveDetailApi(id);
    archiveDetail.value = detail;
    if (detail) {
      fillEditForm(detail);
      archiveLogs.value = [];
    }
  } catch {
    archiveDetail.value = null;
  } finally {
    archiveLoading.value = false;
  }
}

async function handleArchiveSave() {
  if (!archiveDetail.value) return;
  saving.value = true;
  try {
    await updateHrArchiveApi(archiveDetail.value.id, {
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
    await loadArchiveDetail(archiveDetail.value.id);
  } finally {
    saving.value = false;
  }
}

async function handleUnlockArchive(field: 'bank' | 'id_card') {
  if (!archiveDetail.value) return;
  await unlockHrArchiveApi(archiveDetail.value.id, field);
  message.success($t('page.system.hrArchive.unlockSuccess'));
  await loadArchiveDetail(archiveDetail.value.id);
}

async function loadArchiveLogs() {
  if (!archiveDetail.value) return;
  logLoading.value = true;
  try {
    const res: any = await getHrArchiveLogApi({
      page: 1,
      pageSize: 50,
      adminId: archiveDetail.value.id,
    });
    archiveLogs.value = res?.items || [];
  } catch {
    archiveLogs.value = [];
  } finally {
    logLoading.value = false;
  }
}

function handleTabChange(key: string | number) {
  if (key === 'logs' && archiveLogs.value.length === 0) {
    loadArchiveLogs();
  }
}

async function fetchUserDetail() {
  if (!props.id) return;
  loading.value = true;
  archiveDetail.value = null;
  archiveLogs.value = [];
  try {
    const result: any = await getUserDetailApi(props.id);
    userData.value = result;
    if (result?.id && canManageArchive.value) {
      loadArchiveDetail(result.id);
    }
  } catch {
    userData.value = null;
  } finally {
    loading.value = false;
  }
}

function handleClose() {
  innerVisible.value = false;
}

watch(
  () => innerVisible.value,
  (val) => {
    if (val && props.id) {
      fetchUserDetail();
    }
  },
  { immediate: true },
);
</script>

<template>
  <Drawer
    :open="innerVisible"
    :width="fullWidth"
    placement="right"
    :destroy-on-close="true"
    :mask-closable="true"
    :closable="true"
    :title="userData ? `${displayName} - 员工简历` : '员工简历'"
    :body-style="{
      padding: '0',
      maxHeight: 'calc(100vh - 55px)',
      overflow: 'hidden',
    }"
    @close="handleClose"
  >
    <div v-if="loading" class="user-drawer__loading">
      <Spin size="large" />
    </div>

    <div v-else-if="!userData" class="user-drawer__empty">
      <Empty description="暂无员工信息" />
    </div>

    <div v-else class="user-drawer">
      <!-- 头部信息卡 -->
      <div class="user-drawer__header">
        <Avatar :size="72" class="user-drawer__avatar">
          {{ userInitial }}
        </Avatar>
        <div class="user-drawer__headline">
          <div class="user-drawer__name-row">
            <h2 class="user-drawer__name">
              {{ userData.nickName || userData.userName || '未知' }}
            </h2>
            <Tag :color="statusMap[userData.status]?.color || 'default'">
              {{ statusMap[userData.status]?.text || '未知' }}
            </Tag>
            <Tag
              v-if="userData.userType !== undefined"
              :color="userTypeMap[userData.userType]?.color || 'blue'"
            >
              {{ userTypeMap[userData.userType]?.label || '普通员工' }}
            </Tag>
          </div>
          <div class="user-drawer__meta">
            <span v-if="isHrOrAdmin">用户名：{{ userData.userName || '—' }}</span>
            <span v-if="filteredDeptNames.length > 0"
              >部门：{{ filteredDeptNames.join('、') }}</span
            >
            <span v-if="filteredPostNames.length > 0"
              >岗位：{{ filteredPostNames.join('、') }}</span
            >
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <Tabs
        v-model:active-key="activeTab"
        class="user-drawer__tabs"
        type="line"
        size="large"
        @change="handleTabChange"
      >
        <!-- 概览 - 所有人可见 -->
        <TabPane key="summary" tab="概览">
          <div class="user-drawer__pane">
            <Descriptions title="账号信息" :column="2" bordered size="small">
              <Descriptions.Item v-if="isHrOrAdmin" label="用户ID">
                {{ userData.id }}
              </Descriptions.Item>
              <Descriptions.Item v-if="isHrOrAdmin" label="登录账号">
                {{ userData.userName || '—' }}
              </Descriptions.Item>
              <Descriptions.Item v-if="isHrOrAdmin" label="员工编号">
                {{ userData.employeeNo || '—' }}
              </Descriptions.Item>
              <Descriptions.Item label="姓名">
                {{ userData.nickName || '—' }}
              </Descriptions.Item>
              <Descriptions.Item label="账号状态">
                <Tag :color="statusMap[userData.status]?.color || 'default'">
                  {{ statusMap[userData.status]?.text || '未知' }}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="用户类型">
                <Tag :color="userTypeMap[userData.userType]?.color || 'blue'">
                  {{ userTypeMap[userData.userType]?.label || '普通员工' }}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="创建时间">
                {{ formattedCreateTime }}
              </Descriptions.Item>
            </Descriptions>

            <Descriptions
              title="组织归属"
              :column="1"
              bordered
              size="small"
              class="user-drawer__desc"
            >
              <Descriptions.Item label="所属部门">
                <template v-if="filteredDeptNames.length > 0">
                  <Tag
                    v-for="(dept, i) in filteredDeptNames"
                    :key="`d-${i}`"
                    color="blue"
                    class="user-drawer__tag"
                  >
                    {{ dept }}
                  </Tag>
                </template>
                <span v-else class="user-drawer__muted">—</span>
              </Descriptions.Item>
              <Descriptions.Item label="所属岗位">
                <template v-if="filteredPostNames.length > 0">
                  <Tag
                    v-for="(p, i) in filteredPostNames"
                    :key="`p-${i}`"
                    color="cyan"
                    class="user-drawer__tag"
                  >
                    {{ p }}
                  </Tag>
                </template>
                <span v-else class="user-drawer__muted">—</span>
              </Descriptions.Item>
              <Descriptions.Item v-if="isHrOrAdmin" label="角色权限">
                <template v-if="filteredRoleNames.length > 0">
                  <Tag
                    v-for="(r, i) in filteredRoleNames"
                    :key="`r-${i}`"
                    color="purple"
                    class="user-drawer__tag"
                  >
                    {{ r }}
                  </Tag>
                </template>
                <span v-else class="user-drawer__muted">—</span>
              </Descriptions.Item>
            </Descriptions>

            <Descriptions
              v-if="isHrOrAdmin"
              title="登录信息"
              :column="2"
              bordered
              size="small"
              class="user-drawer__desc"
            >
              <Descriptions.Item label="最后登录时间" :span="2">
                {{ formattedLastLogin }}
              </Descriptions.Item>
              <Descriptions.Item label="最后登录IP" :span="2">
                {{ userData.lastLoginIp || userData.loginIp || '—' }}
              </Descriptions.Item>
            </Descriptions>
          </div>
        </TabPane>

        <!-- 联系方式 - 所有人可见 -->
        <TabPane key="contact" tab="联系方式">
          <div class="user-drawer__pane">
            <Descriptions :column="2" bordered size="small">
              <Descriptions.Item label="邮箱" :span="2">
                <a v-if="userData.email" :href="`mailto:${userData.email}`">{{
                  userData.email
                }}</a>
                <span v-else class="user-drawer__muted">—</span>
              </Descriptions.Item>
              <Descriptions.Item label="手机号" :span="2">
                <a v-if="userData.mobile" :href="`tel:${userData.mobile}`">{{
                  userData.mobile
                }}</a>
                <span v-else class="user-drawer__muted">—</span>
              </Descriptions.Item>
              <Descriptions.Item label="性别">
                {{ genderMap[userData.gender as number] || '未设置' }}
              </Descriptions.Item>
              <Descriptions.Item label="归属地">
                {{ userData.nativePlace || '—' }}
              </Descriptions.Item>
            </Descriptions>
          </div>
        </TabPane>

        <!-- 人事/管理员可见的 Tab -->
        <template v-if="isHrOrAdmin">
          <!-- 个人信息 -->
          <TabPane key="personal" tab="个人信息">
            <div class="user-drawer__pane">
              <Descriptions :column="2" bordered size="small">
                <Descriptions.Item label="出生日期">
                  {{ userData.birthday || '—' }}
                </Descriptions.Item>
                <Descriptions.Item label="身份证号">
                  <span class="user-drawer__mono">{{
                    archiveDetail?.idCardNo || userData.idCard || '—'
                  }}</span>
                  <template v-if="archiveDetail">
                    <Tag v-if="archiveDetail.idLocked" color="orange">
                      <LucideLock :size="12" />
                      {{ $t('page.system.profile.locked') }}
                    </Tag>
                    <Button
                      v-if="archiveDetail.idLocked"
                      danger
                      size="small"
                      type="link"
                      @click="handleUnlockArchive('id_card')"
                    >
                      {{ $t('page.system.hrArchive.unlock') }}
                    </Button>
                  </template>
                </Descriptions.Item>
                <Descriptions.Item label="户籍地址" :span="2">
                  {{ userData.nativePlace || '—' }}
                </Descriptions.Item>
                <Descriptions.Item label="现居地址" :span="2">
                  {{ userData.address || '—' }}
                </Descriptions.Item>
                <Descriptions.Item label="政治面貌">
                  {{ userData.politicalStatus || '—' }}
                </Descriptions.Item>
                <Descriptions.Item label="婚姻状况">
                  {{ userData.maritalStatus || '—' }}
                </Descriptions.Item>
                <template v-if="archiveDetail">
                  <Descriptions.Item label="入职日期">
                    {{ archiveDetail.hireDate || '—' }}
                  </Descriptions.Item>
                  <Descriptions.Item label="试用期（月）">
                    {{ archiveDetail.probationMonths ?? '—' }}
                  </Descriptions.Item>
                </template>
              </Descriptions>

              <Spin :spinning="archiveLoading">
                <template v-if="archiveDetail">
                  <div class="user-drawer__section-title">
                    {{ $t('page.system.hrArchive.editSection') }}
                  </div>
                  <div class="user-drawer__edit-grid">
                    <div class="user-drawer__edit-item">
                      <span>{{ $t('page.system.profile.nickName') }}</span>
                      <Input v-model:value="editForm.nickName" size="small" />
                    </div>
                    <div class="user-drawer__edit-item">
                      <span>{{ $t('page.system.profile.email') }}</span>
                      <Input v-model:value="editForm.email" size="small" />
                    </div>
                    <div class="user-drawer__edit-item">
                      <span>{{ $t('page.system.profile.mobile') }}</span>
                      <Input v-model:value="editForm.mobile" size="small" />
                    </div>
                    <div class="user-drawer__edit-item">
                      <span>{{ $t('page.system.hrArchive.hireDate') }}</span>
                      <DatePicker
                        :value="editForm.hireDate"
                        size="small"
                        style="width: 100%"
                        value-format="YYYY-MM-DD"
                        @change="
                          (_: any, ds: string) => (editForm.hireDate = ds)
                        "
                      />
                    </div>
                  </div>
                  <Button
                    :loading="saving"
                    style="margin-top: 12px"
                    type="primary"
                    @click="handleArchiveSave"
                  >
                    {{ $t('ui.button.save') }}
                  </Button>
                </template>
              </Spin>
            </div>
          </TabPane>

          <!-- 财务信息（银行卡三件套，来自 HR 档案，需档案权限） -->
          <TabPane v-if="canManageArchive" key="finance" tab="财务信息">
            <div class="user-drawer__pane">
              <Spin :spinning="archiveLoading">
                <template v-if="archiveDetail">
                  <Descriptions :column="1" bordered size="small">
                    <Descriptions.Item label="银行卡号">
                      <span class="user-drawer__mono">{{
                        archiveDetail.bankCardNo || '—'
                      }}</span>
                      <Tag v-if="archiveDetail.bankLocked" color="orange">
                        <LucideLock :size="12" />
                        {{ $t('page.system.profile.locked') }}
                      </Tag>
                      <Button
                        v-if="archiveDetail.bankLocked"
                        danger
                        size="small"
                        type="link"
                        @click="handleUnlockArchive('bank')"
                      >
                        {{ $t('page.system.hrArchive.unlock') }}
                      </Button>
                    </Descriptions.Item>
                    <Descriptions.Item label="开户行">
                      {{ archiveDetail.bankName || '—' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="开户名">
                      {{ archiveDetail.bankAccountName || '—' }}
                    </Descriptions.Item>
                  </Descriptions>
                  <div class="user-drawer__section-title">
                    {{ $t('page.system.hrArchive.editSection') }}
                  </div>
                  <div class="user-drawer__edit-grid">
                    <div class="user-drawer__edit-item">
                      <span>{{ $t('page.system.hrArchive.bankCardNo') }}</span>
                      <Input v-model:value="editForm.bankCardNo" size="small" />
                    </div>
                    <div class="user-drawer__edit-item">
                      <span>{{ $t('page.system.hrArchive.bankName') }}</span>
                      <Input v-model:value="editForm.bankName" size="small" />
                    </div>
                    <div class="user-drawer__edit-item">
                      <span>{{
                        $t('page.system.profile.bankAccountName')
                      }}</span>
                      <Input
                        v-model:value="editForm.bankAccountName"
                        size="small"
                      />
                    </div>
                  </div>
                  <Button
                    :loading="saving"
                    style="margin-top: 12px"
                    type="primary"
                    @click="handleArchiveSave"
                  >
                    {{ $t('ui.button.save') }}
                  </Button>
                </template>
                <Empty v-else description="暂无档案数据" />
              </Spin>
            </div>
          </TabPane>

          <!-- 工作履历 -->
          <TabPane
            v-if="canManageArchive"
            key="work"
            tab="工作履历"
          >
            <div class="user-drawer__pane">
              <div
                v-if="workExperiences.length > 0"
                class="user-drawer__timeline"
              >
                <div
                  v-for="(item, i) in workExperiences"
                  :key="`work-${i}`"
                  class="user-drawer__timeline-item"
                >
                  <div class="user-drawer__timeline-date">
                    {{ item.startDate }} — {{ item.endDate || '至今' }}
                  </div>
                  <div class="user-drawer__timeline-body">
                    <div class="user-drawer__timeline-title">
                      {{ item.title }}
                    </div>
                    <div class="user-drawer__timeline-subtitle">
                      {{ item.org }}
                    </div>
                    <p
                      v-if="item.remark"
                      class="user-drawer__timeline-desc"
                    >
                      {{ item.remark }}
                    </p>
                  </div>
                </div>
              </div>
              <Empty v-else description="暂无工作履历" />
            </div>
          </TabPane>

          <!-- 教育背景 -->
          <TabPane
            v-if="canManageArchive"
            key="education"
            tab="教育背景"
          >
            <div class="user-drawer__pane">
              <div v-if="educations.length > 0" class="user-drawer__timeline">
                <div
                  v-for="(item, i) in educations"
                  :key="`edu-${i}`"
                  class="user-drawer__timeline-item"
                >
                  <div class="user-drawer__timeline-date">
                    {{ item.startDate }} — {{ item.endDate || '至今' }}
                  </div>
                  <div class="user-drawer__timeline-body">
                    <div class="user-drawer__timeline-title">
                      {{ item.title }}
                    </div>
                    <div class="user-drawer__timeline-subtitle">
                      {{ item.org }}
                    </div>
                  </div>
                </div>
              </div>
              <Empty v-else description="暂无教育背景" />
            </div>
          </TabPane>

          <!-- 证书资质 -->
          <TabPane
            v-if="canManageArchive"
            key="certificates"
            tab="证书资质"
          >
            <div class="user-drawer__pane">
              <ul v-if="certificates.length > 0" class="user-drawer__cert-list">
                <li
                  v-for="(c, i) in certificates"
                  :key="`c-${i}`"
                  class="user-drawer__cert-item"
                >
                  <Tag color="gold" class="user-drawer__cert-tag">证书</Tag>
                  <span>{{ c.title }}</span>
                  <span v-if="c.org" class="user-drawer__muted">{{
                    c.org
                  }}</span>
                  <span class="user-drawer__cert-date">
                    {{ c.startDate || '?' }} ~
                    {{ c.endDate || $t('page.system.profile.present') }}
                  </span>
                </li>
              </ul>
              <Empty v-else description="暂未填写证书资质" />
            </div>
          </TabPane>

          <!-- 紧急联系人 -->
          <TabPane
            v-if="canManageArchive"
            key="emergency"
            tab="紧急联系人"
          >
            <div class="user-drawer__pane">
              <Descriptions
                v-if="emergencyContacts.length > 0"
                :column="1"
                bordered
                size="small"
              >
                <Descriptions.Item
                  v-for="(c, i) in emergencyContacts"
                  :key="`ec-${i}`"
                  :label="`${c.relation || '联系人'}${i + 1}`"
                >
                  {{ c.name }}
                  <a
                    v-if="c.mobile"
                    :href="`tel:${c.mobile}`"
                    class="user-drawer__cert-date"
                    >{{ c.mobile }}</a
                  >
                </Descriptions.Item>
              </Descriptions>
              <Empty v-else description="暂无紧急联系人" />
            </div>
          </TabPane>

          <!-- 变更日志（HR 审计留痕，首次切到该 tab 时懒加载） -->
          <TabPane
            v-if="canManageArchive"
            key="logs"
            :tab="$t('page.system.hrArchive.tabLogs')"
          >
            <div class="user-drawer__pane">
              <Spin :spinning="logLoading">
                <Timeline v-if="archiveLogs.length > 0">
                  <TimelineItem v-for="log in archiveLogs" :key="log.id">
                    <Tag
                      :color="
                        operateTypeMap[log.operateType || 0]?.color || 'default'
                      "
                    >
                      {{ operateTypeMap[log.operateType || 0]?.text || '-' }}
                    </Tag>
                    <span class="user-drawer__log-field">{{ log.field }}</span>
                    <div
                      v-if="log.oldValue || log.newValue"
                      class="user-drawer__log-sub"
                    >
                      {{ log.oldValue || '-' }} → {{ log.newValue || '-' }}
                    </div>
                    <div class="user-drawer__log-meta">
                      {{ log.operatorName }} · {{ log.createDate }}
                      {{ log.createTime }}
                    </div>
                  </TimelineItem>
                </Timeline>
                <Empty
                  v-else
                  :description="$t('page.system.hrArchive.noLogs')"
                />
              </Spin>
            </div>
          </TabPane>

          <!-- 备注 -->
          <TabPane v-if="userData.remark" key="remark" tab="备注">
            <div class="user-drawer__pane">
              <div class="user-drawer__remark">{{ userData.remark }}</div>
            </div>
          </TabPane>
        </template>
      </Tabs>
    </div>
  </Drawer>
</template>

<style scoped>
.user-drawer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--layout-body-background, #f5f7fa);
}

.user-drawer__loading,
.user-drawer__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 60vh;
}

.user-drawer__header {
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 20px 24px;
  background-color: var(--component-background, #fff);
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
}

.user-drawer__avatar {
  flex-shrink: 0;
  font-size: 28px;
  font-weight: 600;
  color: #fff;
  background-color: var(--primary-color, #1677ff);
}

.user-drawer__headline {
  flex: 1;
  min-width: 0;
}

.user-drawer__name-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-bottom: 6px;
}

.user-drawer__name {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  line-height: 1.3;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.user-drawer__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  font-size: 13px;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.user-drawer__tabs {
  flex: 1;
  padding: 0 24px;
  overflow-y: auto;
  background-color: var(--component-background, #fff);
}

.user-drawer__tabs :deep(.ant-tabs-nav) {
  padding-top: 4px;
  margin: 0;
}

.user-drawer__pane {
  padding: 16px 0 24px;
}

.user-drawer__desc {
  margin-top: 16px;
}

.user-drawer__tag {
  margin-right: 4px;
  margin-bottom: 4px;
}

.user-drawer__muted {
  color: var(--disabled-color, rgb(0 0 0 / 25%));
}

.user-drawer__timeline {
  display: flex;
  flex-direction: column;
}

.user-drawer__timeline-item {
  display: flex;
  padding: 12px 0;
  border-bottom: 1px dashed var(--border-color-base, #f0f0f0);
}

.user-drawer__timeline-item:last-child {
  border-bottom: none;
}

.user-drawer__timeline-date {
  flex-shrink: 0;
  width: 140px;
  font-size: 13px;
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
}

.user-drawer__timeline-body {
  flex: 1;
  min-width: 0;
}

.user-drawer__timeline-title {
  margin-bottom: 4px;
  font-size: 14px;
  font-weight: 500;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.user-drawer__timeline-subtitle {
  font-size: 13px;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.user-drawer__timeline-desc {
  margin: 6px 0 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.user-drawer__cert-list {
  padding: 0;
  margin: 0;
  list-style: none;
}

.user-drawer__cert-item {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 0;
  font-size: 13px;
  color: var(--heading-color, rgb(0 0 0 / 88%));
  border-bottom: 1px dashed var(--border-color-base, #f0f0f0);
}

.user-drawer__cert-item:last-child {
  border-bottom: none;
}

.user-drawer__cert-tag {
  margin: 0;
}

.user-drawer__cert-date {
  font-family: 'SF Mono', Consolas, monospace;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.user-drawer__remark {
  padding: 12px 16px;
  font-size: 14px;
  line-height: 1.8;
  color: var(--heading-color, rgb(0 0 0 / 88%));
  overflow-wrap: break-word;
  white-space: pre-wrap;
  background-color: var(--background-color-light, #fafafa);
  border-left: 3px solid var(--primary-color, #1677ff);
  border-radius: 0 4px 4px 0;
}

.user-drawer__mono {
  font-family: 'SF Mono', Consolas, monospace;
}

.user-drawer__section-title {
  margin: 20px 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.user-drawer__edit-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.user-drawer__edit-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}

.user-drawer__log-field {
  margin-left: 8px;
  font-family: 'SF Mono', Consolas, monospace;
}

.user-drawer__log-sub {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
}

.user-drawer__log-meta {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-color-secondary, rgb(0 0 0 / 45%));
}

@media (max-width: 768px) {
  .user-drawer__header {
    padding: 16px;
  }

  .user-drawer__name {
    font-size: 18px;
  }

  .user-drawer__meta {
    flex-direction: column;
    gap: 4px;
  }

  .user-drawer__tabs {
    padding: 0 16px;
  }

  .user-drawer__timeline-item {
    flex-direction: column;
    gap: 4px;
  }

  .user-drawer__timeline-date {
    width: auto;
  }
}
</style>
