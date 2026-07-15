<script lang="ts" setup>
/**
 * 客户详情抽屉组件
 * 统一的客户详情展示抽屉，可在任何页面复用
 *
 * 用法：
 * <CustomerDetailDrawer v-model:visible="visible" :id="id" @edit="onEdit" />
 */
import { ref, watch } from 'vue';
import { Drawer } from 'ant-design-vue';
import CustomerDetail from '../customer/detail.vue';

const props = defineProps<{
  visible: boolean;
  id?: number | string;
}>();

const emit = defineEmits<{
  (e: 'update:visible', visible: boolean): void;
  (e: 'edit', customer: any): void;
}>();

const innerVisible = ref(false);

watch(() => props.visible, (val) => {
  innerVisible.value = val;
});

watch(innerVisible, (val) => {
  emit('update:visible', val);
});

function handleClose() {
  innerVisible.value = false;
}

function handleEdit(customer: any) {
  handleClose();
  emit('edit', customer);
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
    title="客户详情"
    :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }"
    @close="handleClose"
  >
    <CustomerDetail
      v-if="id"
      :id="id"
      @edit="handleEdit"
    />
  </Drawer>
</template>