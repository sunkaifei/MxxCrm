<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import {
  LucidePlus,
  LucideEdit,
  LucideTrash2,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Modal,
  Form,
  Input,
  Tree,
  message,
  Popconfirm,
} from 'ant-design-vue';
import type { TreeProps } from 'ant-design-vue';

const FormItem = Form.Item;

import {
  getAttachmentCategoryTreeApi,
  createAttachmentCategoryApi,
  updateAttachmentCategoryApi,
  deleteAttachmentCategoryApi,
} from '#/api';
import { $t } from '#/locales';

const accessStore = useAccessStore();

const treeData = ref<any[]>([]);
const loading = ref(false);

const modalVisible = ref(false);
const isEdit = ref(false);
const editId = ref<number | null>(null);
const formRef = ref();

const formData = ref({
  parentId: null,
  name: '',
  sortOrder: 0,
});

const loadCategoryTree = async () => {
  loading.value = true;
  try {
    const result: any = await getAttachmentCategoryTreeApi();
    const list = Array.isArray(result) ? result : [];
    treeData.value = list.map((item: any) => ({
      title: item.label,
      key: item.value,
      name: item.label,
      children: item.children?.length ? buildTreeData(item.children) : undefined,
    }));
  } catch (e) {
    console.error('加载分类树失败', e);
    message.error($t('page.attachment.category.message.loadFail'));
  } finally {
    loading.value = false;
  }
};

const buildTreeData = (list: any[]): any[] => {
  return list.map((item) => ({
    title: item.label,
    key: item.value,
    name: item.label,
    children: item.children?.length ? buildTreeData(item.children) : undefined,
  }));
};

const handleAdd = () => {
  isEdit.value = false;
  editId.value = null;
  formData.value = {
    parentId: null,
    name: '',
    sortOrder: 0,
  };
  formRef.value?.resetFields();
  modalVisible.value = true;
};

const handleEdit = (item: any) => {
  isEdit.value = true;
  editId.value = item.key;
  formData.value = {
    parentId: null,
    name: item.name,
    sortOrder: 0,
  };
  formRef.value?.setFieldsValue({
    name: item.name,
  });
  modalVisible.value = true;
};

const handleDelete = async (item: any) => {
  try {
    await deleteAttachmentCategoryApi([item.key]);
    message.success($t('ui.notification.delete_success'));
    loadCategoryTree();
  } catch (e) {
    console.error('删除失败', e);
    message.error($t('page.attachment.category.message.deleteFail'));
  }
};

const handleSubmit = () => {
  formRef.value?.validate().then(async () => {
    try {
      if (isEdit.value && editId.value) {
        await updateAttachmentCategoryApi(editId.value, {
          name: formData.value.name,
        });
        message.success($t('ui.notification.edit_success'));
      } else {
        await createAttachmentCategoryApi({
          name: formData.value.name,
          parent_id: formData.value.parentId,
          sort_order: formData.value.sortOrder,
        });
        message.success($t('ui.notification.create_success'));
      }
      modalVisible.value = false;
      loadCategoryTree();
    } catch (e) {
      console.error('提交失败', e);
      message.error($t('page.attachment.category.message.submitFail'));
    }
  });
};

const handleModalClose = () => {
  modalVisible.value = false;
  formRef.value?.resetFields();
};

const expandedKeys = ref<string[]>([]);

const onExpand: TreeProps['onExpand'] = (keys) => {
  expandedKeys.value = keys as string[];
};

onMounted(() => {
  loadCategoryTree();
});
</script>

<template>
  <Page auto-content-height class="attachment-category-page">
    <div class="category-header">
      <div class="header-title">
        <h2>{{ $t('page.attachment.category.title') }}</h2>
        <p class="title-desc">{{ $t('page.attachment.category.description') }}</p>
      </div>
      <div class="header-actions">
        <Button type="primary" @click="handleAdd" :disabled="!accessStore.hasAccessCode('attachment:category:add')">
          <template #icon>
            <component :is="LucidePlus" />
          </template>
          {{ $t('page.attachment.category.button.add') }}
        </Button>
      </div>
    </div>

    <div class="category-tree-wrapper">
      <div class="tree-container">
        <Tree
          :tree-data="treeData"
          :expanded-keys="expandedKeys"
          :loading="loading"
          show-line
          default-expand-all
          @expand="onExpand"
        >
          <template #title="{ dataRef }">
            <div class="tree-node-content">
              <svg v-if="dataRef.children?.length" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="node-icon">
                <path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9Z"/>
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="node-icon">
                <path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>
              </svg>
              <span class="node-name">{{ dataRef.title }}</span>
              <div class="node-actions">
                <Button
                  type="text"
                  size="small"
                  @click.stop="handleEdit(dataRef)"
                  v-if="accessStore.hasAccessCode('attachment:category:edit')"
                >
                  <template #icon>
                    <component :is="LucideEdit" />
                  </template>
                </Button>
                <Popconfirm
                  :title="$t('page.attachment.category.message.deleteConfirm')"
                  @confirm="handleDelete(dataRef)"
                >
                  <Button
                    type="text"
                    size="small"
                    danger
                    v-if="accessStore.hasAccessCode('attachment:category:delete')"
                  >
                    <template #icon>
                      <component :is="LucideTrash2" />
                    </template>
                  </Button>
                </Popconfirm>
              </div>
            </div>
          </template>
        </Tree>

        <div v-if="!loading && treeData.length === 0" class="empty-tree">
          <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="empty-icon">
            <path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9Z"/>
          </svg>
          <p>{{ $t('page.attachment.category.message.empty') }}</p>
          <Button type="primary" @click="handleAdd">
            {{ $t('page.attachment.category.button.add') }}
          </Button>
        </div>
      </div>
    </div>

    <Modal
      v-model:open="modalVisible"
      :title="isEdit ? $t('page.attachment.category.modal.edit') : $t('page.attachment.category.modal.add')"
      :footer="null"
      width="480px"
      :centered="true"
      :mask-closable="false"
      @cancel="handleModalClose"
    >
      <Form ref="formRef" :model="formData" layout="vertical">
        <FormItem
          :label="$t('page.attachment.category.form.name')"
          name="name"
          :rules="[{ required: true, message: $t('page.attachment.category.form.nameRequired') }]"
        >
          <Input v-model:value="formData.name" :placeholder="$t('page.attachment.category.form.namePlaceholder')" />
        </FormItem>
        <div class="modal-footer">
          <Button @click="handleModalClose">
            {{ $t('ui.button.cancel') }}
          </Button>
          <Button type="primary" @click="handleSubmit">
            {{ $t('ui.button.confirm') }}
          </Button>
        </div>
      </Form>
    </Modal>
  </Page>
</template>

<style>
.attachment-category-page {
  padding: 20px;
}

.category-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #1f1f1f;
}

.title-desc {
  margin: 4px 0 0;
  font-size: 14px;
  color: #8c8c8c;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.category-tree-wrapper {
  background: #fff;
  border-radius: 8px;
  border: 1px solid #f0f0f0;
}

.tree-container {
  padding: 24px;
}

.tree-node-content {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.node-icon {
  font-size: 14px;
  color: #faad14;
}

.node-name {
  flex: 1;
  font-size: 14px;
  color: #262626;
}

.node-actions {
  opacity: 0;
  transition: opacity 0.2s;
}

.ant-tree-title:hover .node-actions {
  opacity: 1;
}

.empty-tree {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
}

.empty-icon {
  font-size: 48px;
  color: #d9d9d9;
  margin-bottom: 16px;
}

.empty-tree p {
  font-size: 14px;
  color: #8c8c8c;
  margin: 0 0 16px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
