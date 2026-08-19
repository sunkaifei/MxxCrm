<script lang="ts" setup>
import { h, ref, watch } from 'vue';

import { LucideSearch, LucideUserPlus } from '@vben/icons';

import { Avatar, Button, Empty, Input, List, Modal } from 'ant-design-vue';

import { searchUsersApi, startSessionApi } from '#/api/core/message/chat';

const props = withDefaults(
  defineProps<{
    visible?: boolean;
  }>(),
  {
    visible: false,
  },
);

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'select', user: any): void;
}>();

const innerVisible = ref(props.visible);
const keyword = ref('');
const users = ref<any[]>([]);
const loading = ref(false);
const searchTimer = ref<any>(null);

watch(
  () => props.visible,
  (val) => {
    innerVisible.value = val;
    if (val) {
      keyword.value = '';
      users.value = [];
    }
  },
);

watch(innerVisible, (val) => {
  emit('update:visible', val);
});

watch(keyword, () => {
  if (searchTimer.value) clearTimeout(searchTimer.value);
  searchTimer.value = setTimeout(() => {
    if (keyword.value.trim()) {
      handleSearch();
    } else {
      users.value = [];
    }
  }, 300);
});

async function handleSearch() {
  if (!keyword.value.trim()) return;
  loading.value = true;
  try {
    const res = await searchUsersApi({
      keyword: keyword.value,
      page: 1,
      pageSize: 20,
    });
    users.value = res.list || [];
  } catch (error) {
    console.error(error);
  } finally {
    loading.value = false;
  }
}

async function handleSelect(user: any) {
  try {
    const res = await startSessionApi({ receiverId: user.userId || user.id });
    emit('select', res || user);
    innerVisible.value = false;
  } catch (error) {
    console.error(error);
  }
}

function getAvatar(user: any) {
  return (
    user.avatar ||
    user.avatarUrl ||
    `https://api.dicebear.com/7.x/avataaars/svg?seed=${user.userId || user.id || user.userName}`
  );
}

function getName(user: any) {
  return (
    user.nickName || user.realName || user.name || user.userName || '未知用户'
  );
}
</script>

<template>
  <Modal
    v-model:open="innerVisible"
    title="新建消息"
    :footer="null"
    :width="480"
    :destroy-on-close="true"
  >
    <div class="mb-4">
      <Input
        v-model:value="keyword"
        placeholder="搜索用户姓名/用户名"
        allow-clear
        size="large"
      >
        <template #prefix>
          <LucideSearch class="w-5 h-5 text-gray-400" />
        </template>
      </Input>
    </div>

    <div style="max-height: 400px; overflow-y: auto">
      <List :data-source="users" :loading="loading">
        <template #renderItem="{ item }">
          <List.Item
            class="cursor-pointer hover:bg-gray-50 rounded-lg px-2 transition-colors"
            @click="handleSelect(item)"
          >
            <List.Item.Meta>
              <template #avatar>
                <Avatar :src="getAvatar(item)" size="large" />
              </template>
              <template #title>
                <span class="font-medium">{{ getName(item) }}</span>
              </template>
              <template #description>
                <span class="text-sm text-gray-500">
                  {{ item.userName || item.username || '' }}
                  <span v-if="item.deptName || item.departmentName">
                    · {{ item.deptName || item.departmentName }}
                  </span>
                </span>
              </template>
            </List.Item.Meta>
            <Button type="primary" size="small" :icon="h(LucideUserPlus)">
              发消息
            </Button>
          </List.Item>
        </template>
      </List>

      <div
        v-if="!loading && !keyword.trim()"
        class="flex flex-col items-center justify-center py-12 text-gray-400"
      >
        <LucideSearch class="w-12 h-12 mb-2 opacity-30" />
        <p class="text-sm">输入关键词搜索用户</p>
      </div>

      <div
        v-if="!loading && keyword.trim() && users.length === 0"
        class="flex flex-col items-center justify-center py-12"
      >
        <Empty description="未找到相关用户" />
      </div>
    </div>
  </Modal>
</template>
