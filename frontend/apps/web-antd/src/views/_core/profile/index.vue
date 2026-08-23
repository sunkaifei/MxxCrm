<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';

import { Profile } from '@vben/common-ui';
import { useUserStore } from '@vben/stores';

import { getMyAuditApi, updateAvatarApi } from '#/api';
import { $t } from '#/locales';
import { useAuthStore } from '#/store';

import AvatarCropper from './avatar-cropper.vue';
import AuditStatus from './audit-status.vue';
import ProfileBase from './base-info.vue';
import EmergencyContacts from './emergency-contacts.vue';
import IdFinance from './id-finance.vue';
import MyTransfers from './my-transfers.vue';
import ProfilePasswordSetting from './password-setting.vue';
import QuickEntry from './quick-entry.vue';
import ResignStatus from './resign-status.vue';
import ResumeTimeline from './resume-timeline.vue';

const userStore = useUserStore();
const authStore = useAuthStore();

const tabsValue = ref<string>('basic');
const avatarModalVisible = ref<boolean>(false);
// 入职审批状态：1=已通过。审批全部通过后「入职审批」入口无展示意义，直接隐藏
const auditStatus = ref<number>(0);

onMounted(async () => {
  if (!userStore.userInfo) {
    await authStore.fetchUserInfo();
  }
  await refreshAuditStatus();
  // 快捷入口"安全设置"跳转：监听切 tab 事件
  window.addEventListener('profile:switch-password', handleSwitchPassword);
});
onUnmounted(() => {
  window.removeEventListener('profile:switch-password', handleSwitchPassword);
});

function handleSwitchPassword() {
  tabsValue.value = 'password';
}

// 头像上传成功后：先持久化到后端用户记录（含 ?v= 缓存破坏版本号），再更新本地缓存。
const handleAvatarSuccess = async (url: string) => {
  await updateAvatarApi(url);
  if (userStore.userInfo) {
    userStore.userInfo.avatar = url;
  }
};

async function handleRefreshUserInfo() {
  await authStore.fetchUserInfo();
}

/** 刷新入职审批状态（已通过后隐藏 Tab 入口）；并处理"正停留在该 Tab"的收敛 */
async function refreshAuditStatus() {
  try {
    const res: any = await getMyAuditApi();
    const audit = res?.data?.data ?? res?.data ?? res ?? null;
    auditStatus.value = audit?.auditStatus ?? 0;
    if (auditStatus.value === 1 && tabsValue.value === 'audit') {
      tabsValue.value = 'basic';
    }
  } catch {
    // 查询失败不阻断个人中心，保持 Tab 可见
  }
}

// 审批全部通过后隐藏「入职审批」Tab（已通过再展示进度已无意义）
const tabs = computed(() => {
  const all = [
    { label: $t('page.system.profile.tabBasic'), value: 'basic' },
    { label: $t('page.system.profile.tabIdFinance'), value: 'idfinance' },
    { label: $t('page.system.profile.tabResume'), value: 'resume' },
    { label: $t('page.system.profile.tabEmergency'), value: 'emergency' },
    { label: $t('page.system.profile.tabQuick'), value: 'quick' },
    { label: $t('page.system.profile.tabAudit'), value: 'audit' },
    { label: $t('page.system.profile.tabResign'), value: 'resign' },
    { label: $t('page.system.profile.tabTransfer'), value: 'transfer' },
    { label: $t('page.system.profile.tabPassword'), value: 'password' },
  ];
  return auditStatus.value === 1 ? all.filter((t) => t.value !== 'audit') : all;
});

// 入职审批页内的档案清单项点击：跳转到对应档案 Tab
function handleSwitchTab(tab: string) {
  tabsValue.value = tab;
}
</script>

<template>
  <div class="profile-page-wrapper">
    <Profile
      v-model:model-value="tabsValue"
      :title="$t('page.system.profile.title')"
      :user-info="userStore.userInfo"
      :tabs="tabs"
    >
      <template #avatar>
        <div class="avatar-wrapper" @click="avatarModalVisible = true">
          <img
            :src="
              userStore.userInfo?.avatar ||
              'https://api.dicebear.com/7.x/avataaars/svg?seed=default'
            "
            class="avatar-img"
          />
          <div class="avatar-mask">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"
              />
              <circle cx="12" cy="13" r="4" />
            </svg>
            <span>{{ $t('page.system.profile.changeAvatar') }}</span>
          </div>
        </div>
      </template>
      <template #content>
        <ProfileBase v-if="tabsValue === 'basic'" @refresh="handleRefreshUserInfo" />
        <IdFinance v-if="tabsValue === 'idfinance'" />
        <ResumeTimeline v-if="tabsValue === 'resume'" />
        <EmergencyContacts v-if="tabsValue === 'emergency'" />
        <QuickEntry v-if="tabsValue === 'quick'" />
        <AuditStatus
          v-if="tabsValue === 'audit'"
          @switch-tab="handleSwitchTab"
          @audit-change="refreshAuditStatus"
        />
        <ResignStatus v-if="tabsValue === 'resign'" />
        <MyTransfers v-if="tabsValue === 'transfer'" />
        <ProfilePasswordSetting v-if="tabsValue === 'password'" />
      </template>
    </Profile>

    <AvatarCropper
      v-model:visible="avatarModalVisible"
      :avatar-url="userStore.userInfo?.avatar"
      @success="handleAvatarSuccess"
    />
  </div>
</template>

<style scoped>
.avatar-wrapper {
  position: relative;
  width: 80px;
  height: 80px;
  overflow: hidden;
  cursor: pointer;
  border-radius: 50%;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.avatar-mask {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  height: 28px;
  font-size: 11px;
  color: white;
  background: rgb(0 0 0 / 50%);
  opacity: 0;
  transition: opacity 0.3s;
}

.avatar-wrapper:hover .avatar-mask {
  opacity: 1;
}
</style>
