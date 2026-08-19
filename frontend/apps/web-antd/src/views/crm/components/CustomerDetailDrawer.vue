<script lang="ts" setup>
/**
 * 客户详情抽屉组件
 * 统一的客户详情/新建展示抽屉，可在任何页面复用
 *
 * 用法：
 *   查看/编辑：<CustomerDetailDrawer v-model:visible="visible" :id="id" />
 *   新建：    <CustomerDetailDrawer v-model:visible="visible" :customer-type="1" @created="onCreated" />
 */
import { computed, ref, watch } from 'vue';

import { Drawer } from 'ant-design-vue';

import CustomerDetail from '../customer/detail.vue';

const props = defineProps<{
  customerType?: number; // 新建模式：1=企业, 2=个人
  id?: number | string;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', visible: boolean): void;
  (e: 'created', id: number | string): void;
}>();

const innerVisible = ref(false);

// 是否新建模式（无 id 但传了 customerType）
const isCreate = computed(
  () =>
    !props.id &&
    props.customerType !== null &&
    props.customerType !== undefined,
);

const drawerTitle = computed(() => {
  if (isCreate.value) {
    return Number(props.customerType) === 2 ? '新建个人客户' : '新建企业客户';
  }
  return '客户详情';
});

watch(
  () => props.visible,
  (val) => {
    innerVisible.value = val;
  },
);

watch(innerVisible, (val) => {
  emit('update:visible', val);
});

function handleClose() {
  innerVisible.value = false;
}

function handleCreated(id: number | string) {
  emit('created', id);
  handleClose();
}
</script>

<template>
  <Drawer
    :open="innerVisible"
    :width="1000"
    placement="right"
    :destroy-on-close="true"
    :mask-closable="true"
    :closable="true"
    :title="drawerTitle"
    :body-style="{
      padding: 0,
      maxHeight: 'calc(100vh - 110px)',
      overflow: 'auto',
    }"
    @close="handleClose"
  >
    <!-- 编辑模式：传 id -->
    <CustomerDetail v-if="!isCreate && id" :id="id" @created="handleCreated" />
    <!-- 新建模式：传 customerType -->
    <CustomerDetail
      v-else-if="isCreate"
      :customer-type="customerType"
      @created="handleCreated"
    />
  </Drawer>
</template>
