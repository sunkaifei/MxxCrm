<script lang="ts" setup>
import { ref, watch, computed, h } from 'vue';
import { Button, Tag, Modal, Input, Checkbox, Spin, Empty, message } from 'ant-design-vue';
import { LucidePlus } from '@vben/icons';
import {
  getAllTagsApi,
  addTagsToEntityApi,
  removeTagsFromEntityApi,
  getTagsByEntityApi,
  getTagGroupListApi,
} from '#/api';

interface TagVO {
  id: number;
  tagName: string;
  tagColor: string;
  groupName: string;
  groupColor: string;
  groupId: number;
}

interface TagGroupVO {
  id: number;
  groupName: string;
  groupColor: string;
}

const props = withDefaults(
  defineProps<{
    entityType: string;
    entityId?: number | null;
  }>(),
  {
    entityId: null,
  },
);

const emit = defineEmits<{
  (e: 'tagsChanged', tags: TagVO[]): void;
}>();

// Current selected tags
const selectedTags = ref<TagVO[]>([]);
const selectedTagIds = computed(() => new Set(selectedTags.value.map((t) => t.id)));

// Modal state
const modalVisible = ref(false);
const modalLoading = ref(false);
const allTags = ref<TagVO[]>([]);
const tagGroups = ref<TagGroupVO[]>([]);
const searchKeyword = ref('');
const tempSelectedIds = ref<Set<number>>(new Set());
const saving = ref(false);

// 获取所有标签和分组
async function fetchAllTags() {
  modalLoading.value = true;
  try {
    const [tagsRes, groupsRes] = await Promise.all([
      getAllTagsApi(),
      getTagGroupListApi(),
    ]);
    allTags.value = (tagsRes?.data || tagsRes || []).map((t: any) => ({
      id: t.id,
      tagName: t.tagName,
      tagColor: t.tagColor || '#1890ff',
      groupName: t.groupName || '未分组',
      groupColor: t.groupColor || '#d9d9d9',
      groupId: t.groupId,
    }));
    tagGroups.value = (groupsRes?.data || groupsRes || []).map((g: any) => ({
      id: g.id,
      groupName: g.groupName,
      groupColor: g.groupColor || '#1890ff',
    }));
  } catch {
    message.error('加载标签失败');
  } finally {
    modalLoading.value = false;
  }
}

// 加载实体已有的标签
async function loadEntityTags() {
  if (!props.entityId) {
    selectedTags.value = [];
    return;
  }
  try {
    const res = await getTagsByEntityApi(props.entityType, props.entityId);
    const tags = (res?.data || res || []).map((t: any) => ({
      id: t.id,
      tagName: t.tagName,
      tagColor: t.tagColor || '#1890ff',
      groupName: t.groupName || '未分组',
      groupColor: t.groupColor || '#d9d9d9',
      groupId: t.groupId,
    }));
    selectedTags.value = tags;
    emit('tagsChanged', tags);
  } catch {
    // ignore
  }
}

// 按分组整理标签
const groupedTags = computed(() => {
  const filtered = searchKeyword.value
    ? allTags.value.filter((t) =>
        t.tagName.toLowerCase().includes(searchKeyword.value.toLowerCase()),
      )
    : allTags.value;

  const map: Record<string, { groupName: string; groupColor: string; tags: TagVO[] }> = {};
  for (const tag of filtered) {
    const key = tag.groupName;
    if (!map[key]) {
      map[key] = {
        groupName: tag.groupName,
        groupColor: tag.groupColor,
        tags: [],
      };
    }
    map[key].tags.push(tag);
  }
  return Object.values(map);
});

// 打开弹窗
function openModal() {
  tempSelectedIds.value = new Set(selectedTags.value.map((t) => t.id));
  searchKeyword.value = '';
  modalVisible.value = true;
  fetchAllTags();
}

// 关闭弹窗
function closeModal() {
  modalVisible.value = false;
}

// 切换标签选中
function toggleTag(tagId: number) {
  const newSet = new Set(tempSelectedIds.value);
  if (newSet.has(tagId)) {
    newSet.delete(tagId);
  } else {
    newSet.add(tagId);
  }
  tempSelectedIds.value = newSet;
}

// 确认选择
async function confirmSelection() {
  const newIds = tempSelectedIds.value;
  const oldIds = selectedTagIds.value;

  // 需要新增的
  const toAdd: number[] = [];
  // 需要移除的
  const toRemove: number[] = [];

  for (const id of newIds) {
    if (!oldIds.has(id)) toAdd.push(id);
  }
  for (const id of oldIds) {
    if (!newIds.has(id)) toRemove.push(id);
  }

  saving.value = true;
  try {
    // 如果有 entityId，直接调用 API
    if (props.entityId) {
      if (toAdd.length > 0) {
        await addTagsToEntityApi({
          entityType: props.entityType,
          entityId: props.entityId,
          tagIds: toAdd,
        });
      }
      if (toRemove.length > 0) {
        await removeTagsFromEntityApi({
          entityType: props.entityType,
          entityId: props.entityId,
          tagIds: toRemove,
        });
      }
    }

    // 更新本地选中状态
    const newSelected = allTags.value.filter((t) => newIds.has(t.id));
    selectedTags.value = newSelected;
    emit('tagsChanged', newSelected);

    modalVisible.value = false;
    if (toAdd.length > 0 || toRemove.length > 0) {
      message.success('标签更新成功');
    }
  } catch {
    message.error('标签更新失败');
  } finally {
    saving.value = false;
  }
}

// 移除单个标签
async function removeTag(tag: TagVO) {
  if (props.entityId) {
    try {
      await removeTagsFromEntityApi({
        entityType: props.entityType,
        entityId: props.entityId,
        tagIds: [tag.id],
      });
    } catch {
      message.error('移除标签失败');
      return;
    }
  }
  selectedTags.value = selectedTags.value.filter((t) => t.id !== tag.id);
  emit('tagsChanged', selectedTags.value);
}

// 外部调用：保存标签到实体（用于创建模式）
async function saveToEntity(entityId: number) {
  const ids = selectedTags.value.map((t) => t.id);
  if (ids.length === 0) return;
  try {
    await addTagsToEntityApi({
      entityType: props.entityType,
      entityId,
      tagIds: ids,
    });
  } catch {
    message.error('保存标签失败');
  }
}

// 监听 entityId 变化
watch(() => props.entityId, loadEntityTags, { immediate: true });

// 暴露方法给父组件
defineExpose({ saveToEntity, loadEntityTags, selectedTags });
</script>

<template>
  <div class="tag-selector">
    <div class="flex items-center gap-2 flex-wrap">
      <Button size="small" :icon="h(LucidePlus)" @click="openModal">
        添加标签
      </Button>
      <Tag
        v-for="tag in selectedTags"
        :key="tag.id"
        :color="tag.tagColor"
        closable
        @close="removeTag(tag)"
      >
        {{ tag.tagName }}
      </Tag>
    </div>

    <Modal
      v-model:open="modalVisible"
      title="选择标签"
      width="600px"
      :confirm-loading="saving"
      :mask-closable="false"
      @ok="confirmSelection"
      @cancel="closeModal"
    >
      <div class="mb-4">
        <Input
          v-model:value="searchKeyword"
          placeholder="搜索标签..."
          allow-clear
        />
      </div>

      <Spin :spinning="modalLoading">
        <div v-if="groupedTags.length === 0 && !modalLoading" class="py-8">
          <Empty description="暂无标签" />
        </div>
        <div
          v-for="group in groupedTags"
          :key="group.groupName"
          class="mb-4"
        >
          <div class="flex items-center gap-2 mb-2">
            <span
              class="inline-block w-2 h-2 rounded-full"
              :style="{ backgroundColor: group.groupColor }"
            />
            <span class="text-sm font-medium text-gray-600">
              {{ group.groupName }}
            </span>
            <span class="text-xs text-gray-400">
              ({{ group.tags.length }})
            </span>
          </div>
          <div class="flex flex-wrap gap-2">
            <Checkbox
              v-for="tag in group.tags"
              :key="tag.id"
              :checked="tempSelectedIds.has(tag.id)"
              class="!mr-0"
              @change="toggleTag(tag.id)"
            >
              <Tag :color="tag.tagColor" class="cursor-pointer">
                {{ tag.tagName }}
              </Tag>
            </Checkbox>
          </div>
        </div>
      </Spin>
    </Modal>
  </div>
</template>