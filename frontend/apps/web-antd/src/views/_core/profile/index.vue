<script setup lang="ts">
import { onMounted, ref } from 'vue';

import { Profile } from '@vben/common-ui';
import { useUserStore } from '@vben/stores';

import { updateAvatarApi } from '#/api';
import { useAuthStore } from '#/store';

import ProfileBase from './base-setting.vue';
import ProfileNotificationSetting from './notification-setting.vue';
import ProfilePasswordSetting from './password-setting.vue';
import ProfileSecuritySetting from './security-setting.vue';
import AvatarCropper from './avatar-cropper.vue';

const userStore = useUserStore();
const authStore = useAuthStore();

const tabsValue = ref<string>('basic');
const avatarModalVisible = ref<boolean>(false);

onMounted(async () => {
  if (!userStore.userInfo) {
    await authStore.fetchUserInfo();
  }
});

// 头像上传成功后：先持久化到后端用户记录（含 ?v= 缓存破坏版本号），再更新本地缓存。
// 这样刷新页面重新拉取 getUserInfo 时仍能拿到最新头像地址，且浏览器不会命中旧缓存。
const handleAvatarSuccess = async (url: string) => {
  await updateAvatarApi(url);
  if (userStore.userInfo) {
    userStore.userInfo.avatar = url;
  }
};

const tabs = ref([
  {
    label: '基本设置',
    value: 'basic',
  },
  {
    label: '安全设置',
    value: 'security',
  },
  {
    label: '修改密码',
    value: 'password',
  },
  {
    label: '新消息提醒',
    value: 'notice',
  },
]);
</script>

<template>
  <div class="profile-page-wrapper">
    <Profile
      v-model:model-value="tabsValue"
      title="个人中心"
      :user-info="userStore.userInfo"
      :tabs="tabs"
    >
      <template #avatar>
        <div
          class="avatar-wrapper"
          @click="avatarModalVisible = true"
        >
          <img
            :src="userStore.userInfo?.avatar || 'https://api.dicebear.com/7.x/avataaars/svg?seed=default'"
            class="avatar-img"
          />
          <div class="avatar-mask">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
              <circle cx="12" cy="13" r="4"/>
            </svg>
            <span>更换头像</span>
          </div>
        </div>
      </template>
      <template #content>
        <ProfileBase v-if="tabsValue === 'basic'" />
        <ProfileSecuritySetting v-if="tabsValue === 'security'" />
        <ProfilePasswordSetting v-if="tabsValue === 'password'" />
        <ProfileNotificationSetting v-if="tabsValue === 'notice'" />
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
  border-radius: 50%;
  overflow: hidden;
  cursor: pointer;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
}

.avatar-mask {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 28px;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  color: white;
  font-size: 11px;
  opacity: 0;
  transition: opacity 0.3s;
}

.avatar-wrapper:hover .avatar-mask {
  opacity: 1;
}
</style>
