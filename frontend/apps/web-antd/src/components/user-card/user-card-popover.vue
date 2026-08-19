<script lang="ts" setup>
import { ref, watch } from 'vue';

import { LucideUser } from '@vben/icons';

import { Avatar, Button, Descriptions, DescriptionsItem, Popover, Tag } from 'ant-design-vue';

import { getCardApi, type CardVO } from '#/api';
import { $t } from '#/locales';

const props = defineProps<{ adminId: number; disabled?: boolean }>();

const open = ref(false);
const loading = ref(false);
const card = ref<CardVO | null>(null);

watch(open, async (v) => {
  if (v && props.adminId > 0 && !card.value) {
    loading.value = true;
    try {
      card.value = await getCardApi(props.adminId);
    } finally {
      loading.value = false;
    }
  }
});

async function copyText(text?: string) {
  if (!text) return;
  await navigator.clipboard.writeText(text);
  window.$message?.success($t('page.system.profile.copied'));
}
</script>

<template>
  <Popover
    v-model:open="open"
    trigger="click"
    placement="bottomLeft"
    overlay-class-name="user-card-popover"
  >
    <slot />
    <template #content>
      <div v-if="loading" class="card-loading">...</div>
      <div v-else-if="card" class="user-card">
        <div class="card-header">
          <Avatar :size="48" :src="card.avatar">
            <template #icon><LucideUser /></template>
          </Avatar>
          <div class="header-info">
            <div class="nick">{{ card.nickName }}</div>
            <div class="depts">
              <Tag v-for="d in card.deptNames" :key="d" color="blue">{{ d }}</Tag>
              <Tag v-for="p in card.postNames" :key="p" color="green">{{ p }}</Tag>
            </div>
          </div>
        </div>

        <Descriptions :column="1" size="small" class="card-body">
          <DescriptionsItem v-if="card.directManagerName" :label="$t('page.system.profile.manager')">
            {{ card.directManagerName }}
          </DescriptionsItem>
          <DescriptionsItem v-if="card.email" :label="$t('page.system.profile.email')">
            <span class="copyable" @click="copyText(card.email)">{{ card.email }}</span>
          </DescriptionsItem>
          <DescriptionsItem v-if="card.mobile" :label="$t('page.system.profile.mobile')">
            <span class="copyable" @click="copyText(card.mobile)">{{ card.mobile }}</span>
          </DescriptionsItem>
          <DescriptionsItem v-if="card.wechat" :label="$t('page.system.profile.wechat')">
            <span class="copyable" @click="copyText(card.wechat)">{{ card.wechat }}</span>
          </DescriptionsItem>
          <DescriptionsItem v-if="card.skills?.length" :label="$t('page.system.profile.skills')">
            <Tag v-for="s in card.skills" :key="s">{{ s }}</Tag>
          </DescriptionsItem>
          <DescriptionsItem v-if="card.intro" :label="$t('page.system.profile.intro')">
            {{ card.intro }}
          </DescriptionsItem>
        </Descriptions>

        <div class="card-actions">
          <Button size="small" @click="copyText(card.email)">
            {{ $t('page.system.profile.copyEmail') }}
          </Button>
        </div>
        <div class="card-privacy">{{ $t('page.system.profile.privacyNote') }}</div>
      </div>
    </template>
  </Popover>
</template>

<style scoped>
.user-card {
  width: 300px;
}

.card-loading {
  width: 300px;
  height: 100px;
}

.card-header {
  display: flex;
  gap: 12px;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px solid rgb(0 0 0 / 6%);
}

.nick {
  font-size: 15px;
  font-weight: 600;
}

.depts {
  margin-top: 4px;
}

.card-body {
  margin-top: 8px;
}

.copyable {
  cursor: pointer;
}

.copyable:hover {
  color: #1677ff;
  text-decoration: underline;
}

.card-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.card-privacy {
  margin-top: 8px;
  font-size: 11px;
  color: rgb(0 0 0 / 35%);
}
</style>
