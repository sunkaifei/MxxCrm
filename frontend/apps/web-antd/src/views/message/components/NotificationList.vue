<script lang="ts" setup>
import type { NotificationDTO } from '#/api/core/message/notification';

import { h, onMounted, ref, watch } from 'vue';

import {
  LucideFileText,
  LucideCheckCircle,
  LucideXCircle,
  LucideList,
  LucideTrash2,
  LucideCheck,
} from '@vben/icons';

import {
  Button,
  Empty,
  List,
  Tabs,
  Tag,
} from 'ant-design-vue';
import dayjs from 'dayjs';

import {
  deleteNotificationApi,
  getNotificationListApi,
  readAllNotificationApi,
  readNotificationApi,
} from '#/api/core/message/notification';

const { TabPane } = Tabs;

const emit = defineEmits<{
  (e: 'unreadChange', count: number): void;
}>();

const activeType = ref<string>('all');
const notifications = ref<NotificationDTO[]>([]);
const loading = ref(false);
const page = ref(1);
const pageSize = ref(20);
const total = ref(0);

const typeMap: Record<number, { label: string; icon: any; color: string }> = {
  1: { label: '公告', icon: LucideFileText, color: 'blue' },
  2: { label: '审批', icon: LucideCheckCircle, color: 'green' },
  3: { label: '业务提醒', icon: LucideXCircle, color: 'orange' },
  4: { label: '任务', icon: LucideList, color: 'purple' },
};

async function loadNotifications() {
  loading.value = true;
  try {
    const params: any = {
      page: page.value,
      pageSize: pageSize.value,
    };
    if (activeType.value !== 'all') {
      params.type = parseInt(activeType.value);
    }
    const res = await getNotificationListApi(params);
    notifications.value = res.list || [];
    total.value = res.total || 0;
  } finally {
    loading.value = false;
  }
}

async function handleRead(item: NotificationDTO) {
  if (item.isRead) return;
  try {
    await readNotificationApi({ id: item.id });
    item.isRead = true;
    emit('unreadChange', -1);
  } catch (e) {
    console.error(e);
  }
}

async function handleReadAll() {
  try {
    await readAllNotificationApi();
    const unreadCount = notifications.value.filter((n) => !n.isRead).length;
    notifications.value.forEach((n) => (n.isRead = true));
    emit('unreadChange', -unreadCount);
    window.$message?.success('已全部标记为已读');
  } catch (e) {
    console.error(e);
  }
}

async function handleDelete(item: NotificationDTO) {
  try {
    await deleteNotificationApi({ id: item.id });
    notifications.value = notifications.value.filter((n) => n.id !== item.id);
    if (!item.isRead) {
      emit('unreadChange', -1);
    }
    window.$message?.success('删除成功');
  } catch (e) {
    console.error(e);
  }
}

function handleClick(item: NotificationDTO) {
  handleRead(item);
  if (item.linkUrl) {
    if (item.linkUrl.startsWith('http://') || item.linkUrl.startsWith('https://')) {
      window.open(item.linkUrl, '_blank');
    } else {
      window.location.href = item.linkUrl;
    }
  }
}

function formatTime(time: string) {
  const now = dayjs();
  const msgTime = dayjs(time);
  if (now.isSame(msgTime, 'day')) {
    return msgTime.format('HH:mm');
  }
  if (now.diff(msgTime, 'day') < 7) {
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    return weekdays[msgTime.day()];
  }
  return msgTime.format('YYYY-MM-DD');
}

watch(activeType, () => {
  page.value = 1;
  loadNotifications();
});

onMounted(() => {
  loadNotifications();
});

defineExpose({ loadNotifications });
</script>

<template>
  <div class="flex flex-col h-full bg-white">
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200">
      <div class="font-medium text-gray-800 text-lg flex items-center gap-2">
        <LucideFileText class="w-5 h-5 text-[#1677ff]" />
        系统通知
      </div>
      <Button
        type="link"
        size="small"
        :icon="h(LucideCheck)"
        @click="handleReadAll"
      >
        全部已读
      </Button>
    </div>

    <div class="px-4 pt-2">
      <Tabs
        v-model:active-key="activeType"
        size="small"
      >
        <TabPane tab="全部" key="all" />
        <TabPane tab="公告" key="1" />
        <TabPane tab="审批" key="2" />
        <TabPane tab="业务提醒" key="3" />
        <TabPane tab="任务" key="4" />
      </Tabs>
    </div>

    <div class="flex-1 overflow-y-auto px-2">
      <List
        :data-source="notifications"
        :loading="loading"
        item-layout="horizontal"
      >
        <template #renderItem="{ item }">
          <List.Item
            class="cursor-pointer hover:bg-gray-50 rounded-lg mx-2 transition-colors"
            :class="{ 'bg-gray-50': !item.isRead }"
            @click="handleClick(item)"
          >
            <List.Item.Meta>
              <template #avatar>
                <div
                  class="w-10 h-10 rounded-full flex items-center justify-center bg-blue-100"
                >
                  <component
                    :is="typeMap[item.type]?.icon || LucideFileText"
                    class="w-5 h-5 text-blue-500"
                  />
                </div>
              </template>
              <template #title>
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="font-medium" :class="item.isRead ? 'text-gray-600' : 'text-gray-900'">
                      {{ item.title }}
                    </span>
                    <Tag v-if="!item.isRead" color="red" style="margin-left: 8px;">新</Tag>
                  </div>
                  <span class="text-xs text-gray-400">{{ formatTime(item.createTime) }}</span>
                </div>
              </template>
              <template #description>
                <div class="flex items-start justify-between">
                  <p class="text-sm text-gray-500 line-clamp-2 flex-1 mr-4">
                    {{ item.content }}
                  </p>
                  <Button
                    type="text"
                    size="small"
                    danger
                    class="flex-shrink-0 opacity-0 hover:opacity-100 transition-opacity"
                    :icon="h(LucideTrash2)"
                    @click.stop="handleDelete(item)"
                  />
                </div>
              </template>
            </List.Item.Meta>
          </List.Item>
        </template>
      </List>
      <div
        v-if="!loading && notifications.length === 0"
        class="flex flex-col items-center justify-center py-16"
      >
        <Empty description="暂无通知" />
      </div>
    </div>
  </div>
</template>
