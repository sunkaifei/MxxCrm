<script lang="ts" setup>
/**
 * 池成员管理抽屉（v3.0 公海优化 §6.1）
 * 成员表格（本地数据）+ 添加（UserSelectModal，可设角色）+ 角色调整 + 移除
 */
import type { PoolMemberVO } from '#/api/core/crm/pool';

import { computed, ref, watch } from 'vue';

import {
  Button,
  Drawer,
  message,
  Popconfirm,
  Radio,
  RadioGroup,
  Table,
  Tag,
} from 'ant-design-vue';

import {
  deletePoolMembersApi,
  getPoolMemberListApi,
  savePoolMembersApi,
} from '#/api/core/crm/pool';

import UserSelectModal from '../components/UserSelectModal.vue';

const props = defineProps<{
  poolId?: null | string;
  poolName?: string;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'changed'): void;
  (e: 'update:visible', value: boolean): void;
}>();

const innerVisible = computed({
  get: () => props.visible,
  set: (val: boolean) => emit('update:visible', val),
});

const drawerWidth = computed(() =>
  typeof window !== 'undefined' ? Math.min(window.innerWidth * 0.6, 760) : 760,
);

const loading = ref(false);
const members = ref<PoolMemberVO[]>([]);

// 待添加成员（选人后先进入暂存列表，统一保存）
const pendingMembers = ref<{ memberType: number; userName: string; userId: string }[]>([]);
const selectVisible = ref(false);
const pendingMemberType = ref<number>(2);

const memberTypeText: Record<number, string> = {
  1: '池管理员',
  2: '普通成员',
};

const columns = [
  { title: '用户名', dataIndex: 'userName', width: 160 },
  { title: '角色', dataIndex: 'memberType', width: 140 },
  { title: '加入时间', dataIndex: 'createTime', width: 170 },
  { title: '操作', dataIndex: 'action', width: 90 },
];

async function loadData() {
  if (!props.poolId) return;
  loading.value = true;
  try {
    members.value = (await getPoolMemberListApi(props.poolId!)) || [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      pendingMembers.value = [];
      pendingMemberType.value = 2;
      void loadData();
    }
  },
);

function existingUserIds(): number[] {
  const exist = members.value.map((m) => Number(m.userId));
  const pending = pendingMembers.value.map((m) => Number(m.userId));
  return [...exist, ...pending];
}

function handleSelectUser(row: any) {
  pendingMembers.value.push({
    memberType: pendingMemberType.value,
    userId: String(row.id),
    userName: row.nickName || row.realName || row.userName || String(row.id),
  });
  selectVisible.value = false;
}

async function handleSavePending() {
  if (!props.poolId || pendingMembers.value.length === 0) return;
  await savePoolMembersApi(
    props.poolId,
    pendingMembers.value.map((m) => ({
      memberType: m.memberType,
      userId: m.userId,
    })),
  );
  message.success('添加成功');
  pendingMembers.value = [];
  await loadData();
  emit('changed');
}

async function handleRemove(row: PoolMemberVO) {
  if (!props.poolId || !row.userId) return;
  await deletePoolMembersApi(props.poolId, [row.userId]);
  message.success('已移除');
  await loadData();
  emit('changed');
}
</script>

<template>
  <Drawer
    v-model:open="innerVisible"
    :title="`成员管理 - ${poolName || ''}`"
    :width="drawerWidth"
    destroy-on-close
  >
    <div class="flex items-center gap-2 mb-3">
      <span class="text-sm text-gray-500">新成员角色：</span>
      <RadioGroup v-model:value="pendingMemberType">
        <Radio :value="1">池管理员</Radio>
        <Radio :value="2">普通成员</Radio>
      </RadioGroup>
      <Button
        v-access:code="['crm:pool:member']"
        type="primary"
        size="small"
        @click="selectVisible = true"
      >
        添加成员
      </Button>
    </div>

    <Table
      :columns="columns"
      :data-source="members"
      :loading="loading"
      :pagination="false"
      row-key="id"
      size="small"
      bordered
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'memberType'">
          <Tag :color="record.memberType === 1 ? 'gold' : 'default'">
            {{ memberTypeText[record.memberType] || '-' }}
          </Tag>
        </template>
        <template v-else-if="column.key === 'action'">
          <Popconfirm
            title="移除后该成员将失去本池线索可见权限，确认移除？"
            @confirm="handleRemove(record)"
          >
            <Button
              v-access:code="['crm:pool:member']"
              type="link"
              size="small"
              class="!p-0 text-red-600"
            >
              移除
            </Button>
          </Popconfirm>
        </template>
      </template>
    </Table>

    <div
      v-if="pendingMembers.length > 0"
      class="mt-4 p-3 border border-dashed rounded"
    >
      <div class="flex items-center mb-2">
        <span class="font-medium">待添加成员（{{ pendingMembers.length }}）</span>
        <Button
          type="primary"
          size="small"
          class="ml-auto"
          @click="handleSavePending"
        >
          确认添加
        </Button>
      </div>
      <div class="flex flex-wrap gap-2">
        <Tag
          v-for="m in pendingMembers"
          :key="m.userId"
          closable
          @close.prevent="
            pendingMembers = pendingMembers.filter((p) => p.userId !== m.userId)
          "
        >
          {{ m.userName }}（{{ memberTypeText[m.memberType] }}）
        </Tag>
      </div>
    </div>

    <UserSelectModal
      v-model:visible="selectVisible"
      :exclude-ids="existingUserIds()"
      @select="handleSelectUser"
    />
  </Drawer>
</template>

<style scoped>
.border-dashed {
  border-style: dashed;
}
</style>
