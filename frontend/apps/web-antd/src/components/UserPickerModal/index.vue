<script lang="ts" setup>
/**
 * 用户选择器弹窗组件
 *
 * 用于替代直接输入 userId 的 InputNumber。外部展示只读 Input（显示已选用户名），
 * 点击打开 Modal 用户列表，支持搜索、分页、单选。
 *
 * 用法：
 * <UserPickerModal v-model:value="userId" @change="onUserChange" />
 */
import type { TableColumnsType } from 'ant-design-vue';

import { computed, h, reactive, ref, watch } from 'vue';

import { LucideSearch, LucideUser, LucideX } from '@vben/icons';

import { Button, Input, Modal, Table } from 'ant-design-vue';

import { getUserDetailApi, getUserListApi } from '#/api/core/system/user';

/** 用户信息 */
interface UserVO {
  id: number | string;
  userName?: string;
  nickName?: string;
  realName?: string;
  name?: string;
  depts?: { deptName?: string }[];
  deptName?: string;
  departmentName?: string;
  mobile?: string;
  phone?: string;
  email?: string;
  status?: number;
}

const props = withDefaults(
  defineProps<{
    /** 是否禁用 */
    disabled?: boolean;
    /** 占位文本 */
    placeholder?: string;
    /** v-model 绑定的用户ID */
    value?: number;
  }>(),
  {
    value: undefined,
    placeholder: '点击选择用户',
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: 'update:value', value: number | undefined): void;
  (e: 'change', user: undefined | UserVO): void;
}>();

const modalVisible = ref(false);
const keyword = ref('');
const tableData = ref<UserVO[]>([]);
const loading = ref(false);
/** 当前选中的用户完整信息（用于外部 Input 回显名称） */
const selectedUser = ref<UserVO>();
let searchTimer: ReturnType<typeof setTimeout> | undefined;

const pagination = reactive({
  current: 1,
  pageSize: 10,
  total: 0,
  showSizeChanger: true,
  showTotal: (total: number) => `共 ${total} 条`,
});

function getDisplayName(user?: UserVO): string {
  if (!user) return '';
  return user.nickName || user.realName || user.name || user.userName || '';
}

function getDeptName(row: UserVO): string {
  if (row.depts && row.depts.length > 0) {
    const names = row.depts.map((d) => d.deptName).filter(Boolean);
    if (names.length > 0) return names.join('、');
  }
  return row.deptName || row.departmentName || '-';
}

const columns: TableColumnsType = [
  { title: '用户名', dataIndex: 'userName', width: 120, ellipsis: true },
  {
    title: '姓名',
    key: 'nickName',
    width: 120,
    ellipsis: true,
    customRender: ({ record }: any) => getDisplayName(record) || '-',
  },
  {
    title: '部门',
    key: 'dept',
    width: 140,
    ellipsis: true,
    customRender: ({ record }: any) => getDeptName(record),
  },
  {
    title: '邮箱',
    dataIndex: 'email',
    width: 180,
    ellipsis: true,
    customRender: ({ value }: any) => value || '-',
  },
  {
    title: '操作',
    key: 'action',
    width: 80,
    fixed: 'right',
    align: 'center',
    customRender: ({ record }: any) =>
      h(
        Button,
        {
          type: 'primary',
          size: 'small',
          onClick: (e: Event) => {
            e.stopPropagation();
            handleSelect(record);
          },
        },
        () => '选择',
      ),
  },
];

const displayName = computed(() => getDisplayName(selectedUser.value));

function openModal() {
  if (props.disabled) return;
  modalVisible.value = true;
  keyword.value = '';
  pagination.current = 1;
  loadData();
}

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getUserListApi({
      page: pagination.current,
      pageSize: pagination.pageSize,
      nickName: keyword.value || undefined,
    });
    tableData.value = res?.items || [];
    pagination.total = res?.total || 0;
  } catch (error) {
    console.error('加载用户列表失败:', error);
    tableData.value = [];
    pagination.total = 0;
  } finally {
    loading.value = false;
  }
}

// 关键词搜索防抖
watch(keyword, () => {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    pagination.current = 1;
    loadData();
  }, 300);
});

function handleSearchEnter() {
  if (searchTimer) clearTimeout(searchTimer);
  pagination.current = 1;
  loadData();
}

function handleTableChange(pag: any) {
  pagination.current = pag.current;
  pagination.pageSize = pag.pageSize;
  loadData();
}

function handleSelect(row: UserVO) {
  selectedUser.value = row;
  emit(
    'update:value',
    row.id !== undefined && row.id !== null ? Number(row.id) : undefined,
  );
  emit('change', row);
  modalVisible.value = false;
}

function handleClear(e: Event) {
  e.stopPropagation();
  selectedUser.value = undefined;
  emit('update:value', undefined);
  emit('change', undefined);
}

function customRow(record: UserVO) {
  return {
    onClick: () => handleSelect(record),
    style: { cursor: 'pointer' },
  };
}

function rowClassName(record: UserVO) {
  if (
    selectedUser.value &&
    Number(selectedUser.value.id) === Number(record.id)
  ) {
    return 'user-picker-row-selected';
  }
  return '';
}

// 反向查询：当 value 外部变化且与当前选中不一致时，拉取用户详情用于回显名称
watch(
  () => props.value,
  async (val) => {
    if (val === undefined || val === null) {
      selectedUser.value = undefined;
      return;
    }
    if (selectedUser.value && Number(selectedUser.value.id) === Number(val)) {
      return;
    }
    try {
      const detail: any = await getUserDetailApi(val);
      selectedUser.value = detail as UserVO;
    } catch (error) {
      console.error('加载用户详情失败:', error);
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="user-picker-modal">
    <Input
      :value="displayName"
      :placeholder="placeholder"
      :disabled="disabled"
      readonly
      class="user-picker-input"
      @click="openModal"
    >
      <template #prefix>
        <LucideUser class="h-4 w-4 text-gray-400" />
      </template>
      <template #suffix>
        <LucideX
          v-if="displayName && !disabled"
          class="user-picker-clear"
          @click="handleClear"
        />
      </template>
    </Input>

    <Modal
      v-model:open="modalVisible"
      title="选择用户"
      :width="780"
      :footer="null"
      :destroy-on-close="true"
    >
      <div class="mb-3">
        <Input
          v-model:value="keyword"
          placeholder="输入姓名/昵称搜索"
          allow-clear
          @press-enter="handleSearchEnter"
        >
          <template #prefix>
            <LucideSearch class="h-4 w-4 text-gray-400" />
          </template>
        </Input>
      </div>

      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        :pagination="pagination"
        row-key="id"
        :custom-row="customRow"
        :row-class-name="rowClassName"
        size="small"
        :scroll="{ x: 640 }"
        @change="handleTableChange"
      />

      <div class="mt-2 text-right text-xs text-gray-400">
        提示：点击行可快速选择
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.user-picker-input {
  cursor: pointer;
}

.user-picker-input:disabled {
  cursor: not-allowed;
}

.user-picker-clear {
  width: 14px;
  height: 14px;
  color: #9ca3af;
  cursor: pointer;
  transition: color 0.2s;
}

.user-picker-clear:hover {
  color: #ef4444;
}

:deep(.user-picker-row-selected) td {
  background-color: #e6f4ff !important;
}
</style>
