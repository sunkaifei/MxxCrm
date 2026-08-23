<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Descriptions,
  Drawer,
  Empty,
  Spin,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { getUserDetailApi } from '#/api';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';

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

const workExperiences = computed(() => {
  const list = userData.value?.workExperiences;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const educations = computed(() => {
  const list = userData.value?.educations;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const skills = computed(() => {
  const list = userData.value?.skills;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const languages = computed(() => {
  const list = userData.value?.languages;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const certificates = computed(() => {
  const list = userData.value?.certificates;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const hobbies = computed(() => {
  const list = userData.value?.hobbies;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const family = computed(() => {
  const list = userData.value?.familyMembers;
  if (Array.isArray(list) && list.length > 0) return list;
  return [];
});

const emergencyContact = computed(() => {
  return {
    name: userData.value?.emergencyName || '',
    relation: userData.value?.emergencyRelation || '',
    phone: userData.value?.emergencyPhone || '',
  };
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
    activeTab.value = 'summary';
  }
});

watch(
  () => props.id,
  (newId) => {
    if (newId && innerVisible.value) fetchUserDetail();
  },
);

async function fetchUserDetail() {
  if (!props.id) return;
  loading.value = true;
  try {
    const result: any = await getUserDetailApi(props.id);
    userData.value = result;
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
                  {{ userData.idCard || '—' }}
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
              </Descriptions>
            </div>
          </TabPane>

          <!-- 工作履历 -->
          <TabPane key="work" tab="工作履历">
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
                      {{ item.position }}
                    </div>
                    <div class="user-drawer__timeline-subtitle">
                      {{ item.company }}
                    </div>
                    <p
                      v-if="item.description"
                      class="user-drawer__timeline-desc"
                    >
                      {{ item.description }}
                    </p>
                  </div>
                </div>
              </div>
              <Empty v-else description="暂无工作履历" />
            </div>
          </TabPane>

          <!-- 教育背景 -->
          <TabPane key="education" tab="教育背景">
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
                      {{ item.school }}
                    </div>
                    <div class="user-drawer__timeline-subtitle">
                      {{ item.degree }} · {{ item.major }}
                    </div>
                  </div>
                </div>
              </div>
              <Empty v-else description="暂无教育背景" />
            </div>
          </TabPane>

          <!-- 技能特长 -->
          <TabPane key="skills" tab="技能特长">
            <div class="user-drawer__pane">
              <div v-if="skills.length > 0" class="user-drawer__chip-list">
                <Tag
                  v-for="(s, i) in skills"
                  :key="`s-${i}`"
                  color="blue"
                  class="user-drawer__chip"
                >
                  {{ s }}
                </Tag>
              </div>
              <Empty v-else description="暂未填写技能特长" />
            </div>
          </TabPane>

          <!-- 语言能力 -->
          <TabPane key="languages" tab="语言能力">
            <div class="user-drawer__pane">
              <div v-if="languages.length > 0" class="user-drawer__lang-grid">
                <div
                  v-for="(l, i) in languages"
                  :key="`l-${i}`"
                  class="user-drawer__lang-card"
                >
                  <div class="user-drawer__lang-name">{{ l.name }}</div>
                  <div class="user-drawer__lang-level">{{ l.level }}</div>
                </div>
              </div>
              <Empty v-else description="暂未填写语言能力" />
            </div>
          </TabPane>

          <!-- 证书资质 -->
          <TabPane key="certificates" tab="证书资质">
            <div class="user-drawer__pane">
              <ul v-if="certificates.length > 0" class="user-drawer__cert-list">
                <li
                  v-for="(c, i) in certificates"
                  :key="`c-${i}`"
                  class="user-drawer__cert-item"
                >
                  <Tag color="gold" class="user-drawer__cert-tag">证书</Tag>
                  <span>{{ c }}</span>
                </li>
              </ul>
              <Empty v-else description="暂未填写证书资质" />
            </div>
          </TabPane>

          <!-- 家庭信息 -->
          <TabPane key="family" tab="家庭信息">
            <div class="user-drawer__pane">
              <Descriptions
                v-if="family.length > 0"
                :column="1"
                bordered
                size="small"
              >
                <Descriptions.Item
                  v-for="(f, i) in family"
                  :key="`f-${i}`"
                  :label="f.relation"
                >
                  {{ f.name }}
                  <span
                    v-if="f.phone && f.phone !== '—'"
                    class="user-drawer__phone"
                  >
                    · {{ f.phone }}</span
                  >
                </Descriptions.Item>
              </Descriptions>
              <Empty v-else description="暂无家庭信息" />
            </div>
          </TabPane>

          <!-- 兴趣爱好 -->
          <TabPane key="hobbies" tab="兴趣爱好">
            <div class="user-drawer__pane">
              <div v-if="hobbies.length > 0" class="user-drawer__chip-list">
                <Tag
                  v-for="(hobby, i) in hobbies"
                  :key="`h-${i}`"
                  color="purple"
                  class="user-drawer__chip"
                >
                  {{ hobby }}
                </Tag>
              </div>
              <Empty v-else description="暂未填写兴趣爱好" />
            </div>
          </TabPane>

          <!-- 紧急联系人 -->
          <TabPane key="emergency" tab="紧急联系人">
            <div class="user-drawer__pane">
              <Descriptions :column="1" bordered size="small">
                <Descriptions.Item label="姓名">
                  {{ emergencyContact.name || '—' }}
                </Descriptions.Item>
                <Descriptions.Item label="关系">
                  {{ emergencyContact.relation || '—' }}
                </Descriptions.Item>
                <Descriptions.Item label="联系电话">
                  <a
                    v-if="emergencyContact.phone"
                    :href="`tel:${emergencyContact.phone}`"
                    >{{ emergencyContact.phone }}</a
                  >
                  <span v-else class="user-drawer__muted">—</span>
                </Descriptions.Item>
              </Descriptions>
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

.user-drawer__phone {
  font-family: 'SF Mono', Consolas, monospace;
  color: var(--text-color-secondary, rgb(0 0 0 / 65%));
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

.user-drawer__chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.user-drawer__chip {
  margin: 0;
}

.user-drawer__lang-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}

.user-drawer__lang-card {
  padding: 12px 16px;
  background-color: var(--background-color-light, #fafafa);
  border: 1px solid var(--border-color-base, #f0f0f0);
  border-radius: 4px;
}

.user-drawer__lang-name {
  margin-bottom: 4px;
  font-size: 14px;
  font-weight: 500;
  color: var(--heading-color, rgb(0 0 0 / 88%));
}

.user-drawer__lang-level {
  font-size: 12px;
  color: var(--primary-color, #1677ff);
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
