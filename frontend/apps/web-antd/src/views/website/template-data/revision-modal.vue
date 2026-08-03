<script lang="ts" setup>
import { ref, watch } from 'vue';
import { Modal, Button, List, Tag, message, Spin } from 'ant-design-vue';
import { getTemplateRevisionListApi } from '#/api';
import CodeEditor from '#/components/CodeEditor/index.vue';

const props = defineProps<{
  visible: boolean;
  templateDataId: number | null;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'rollback', temptext: string): void;
}>();

const loading = ref(false);
const revisions = ref<any[]>([]);
const previewVisible = ref(false);
const previewContent = ref('');
const previewTitle = ref('');

async function loadRevisions() {
  if (!props.templateDataId) return;
  loading.value = true;
  try {
    const res: any = await getTemplateRevisionListApi(props.templateDataId);
    const data = res?.data || res;
    revisions.value = Array.isArray(data) ? data : data?.items || data?.list || [];
  } catch {
    revisions.value = [];
  } finally {
    loading.value = false;
  }
}

watch(() => props.visible, (val) => {
  if (val) loadRevisions();
});

function handlePreview(item: any) {
  previewContent.value = item.temptext || '';
  previewTitle.value = `版本 #${item.id} - ${item.createTime}`;
  previewVisible.value = true;
}

function handleRollback(item: any) {
  Modal.confirm({
    title: '确认回滚',
    content: `确定要回滚到版本 #${item.id} 吗？当前内容将被替换。`,
    onOk: () => {
      emit('rollback', item.temptext);
      emit('update:visible', false);
      message.success('已回滚，请点击保存以应用更改');
    },
  });
}

function handleClose() {
  emit('update:visible', false);
}
</script>

<template>
  <Modal
    :open="visible"
    title="模板版本历史"
    :width="700"
    :footer="null"
    @cancel="handleClose"
  >
    <Spin :spinning="loading">
      <List
        v-if="revisions.length > 0"
        :data-source="revisions"
        :pagination="{ pageSize: 5 }"
      >
        <template #renderItem="{ item }">
          <List.Item>
            <List.Item.Meta>
              <template #title>
                <span>版本 #{{ item.id }}</span>
                <Tag v-if="item.revisionNote" color="blue" style="margin-left: 8px;">
                  {{ item.revisionNote }}
                </Tag>
              </template>
              <template #description>
                {{ item.createTime }}
              </template>
            </List.Item.Meta>
            <template #actions>
              <Button type="link" size="small" @click="handlePreview(item)">查看</Button>
              <Button type="link" size="small" danger @click="handleRollback(item)">回滚</Button>
            </template>
          </List.Item>
        </template>
      </List>
      <div v-else style="text-align: center; padding: 40px; color: #999;">
        暂无版本历史
      </div>
    </Spin>

    <!-- 预览弹窗 -->
    <Modal
      v-model:open="previewVisible"
      :title="previewTitle"
      :width="800"
      :footer="null"
    >
      <CodeEditor
        :model-value="previewContent"
        :read-only="true"
        language="html"
        height="500px"
      />
    </Modal>
  </Modal>
</template>
