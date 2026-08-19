<script lang="ts" setup>
import type { TemplateTagVO } from '#/api/core/website/template-data';

import { computed, h, onMounted, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';
import {
  LucideClock,
  LucideEye,
  LucideMaximize2,
  LucideMinimize2,
  LucideTag,
} from '@vben/icons';

import {
  Button,
  Collapse,
  CollapsePanel,
  Empty,
  Input,
  message,
  Modal,
  TabPane,
  Tabs,
} from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  addTemplateDataApi,
  getTemplateDataDetailApi,
  getTemplateTagsApi,
  previewTemplateDataApi,
  updateTemplateDataApi,
} from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import RevisionModal from '../template-data/revision-modal.vue';

// 类型选项
const typeOptions = [
  { value: 1, label: '首页' },
  { value: 2, label: '列表页' },
  { value: 3, label: '内容页' },
  { value: 4, label: '栏目封面' },
  { value: 5, label: '报价页' },
  { value: 6, label: '专题' },
  { value: 7, label: '产品列表' },
  { value: 8, label: '产品详情' },
  { value: 14, label: '页头' },
  { value: 15, label: '页脚' },
];

const drawerData = ref<{ row?: any; templateId: number }>({ templateId: 0 });
const isCreate = computed(() => !drawerData.value?.row?.id);
const drawerTitle = computed(() =>
  isCreate.value
    ? '新增页面'
    : `编辑页面 - ${drawerData.value?.row?.name || ''}`,
);

// 全屏切换
const isFullscreen = ref(false);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
  drawerApi.setState({ class: isFullscreen.value ? 'w-full' : 'w-[75vw]' });
}

// 版本历史
const revisionVisible = ref(false);
const revisionTemplateDataId = ref<null | number>(null);

function openRevisionModal() {
  if (drawerData.value?.row?.id) {
    revisionTemplateDataId.value = drawerData.value.row.id;
    revisionVisible.value = true;
  }
}

function handleRollback(temptext: string) {
  baseFormApi.setValues({ temptext });
}

// 标签文档面板
const tagsPanelActive = ref<string[]>([]);
const allTags = ref<TemplateTagVO[]>([]);
const tagsLoading = ref(false);
const tagSearchKeyword = ref('');

const tagsByCategory = computed(() => {
  const keyword = tagSearchKeyword.value.trim().toLowerCase();
  const filtered = keyword
    ? allTags.value.filter(
        (t) =>
          t.name.toLowerCase().includes(keyword) ||
          t.description.toLowerCase().includes(keyword) ||
          t.syntax.toLowerCase().includes(keyword),
      )
    : allTags.value;

  const map = new Map<string, TemplateTagVO[]>();
  for (const tag of filtered) {
    const list = map.get(tag.category) ?? [];
    list.push(tag);
    map.set(tag.category, list);
  }
  return [...map.entries()].map(([category, tags]) => ({
    category,
    tags,
  }));
});

async function loadTags() {
  if (allTags.value.length > 0) return;
  tagsLoading.value = true;
  try {
    const res = await getTemplateTagsApi();
    allTags.value = Array.isArray(res) ? res : [];
  } catch {
    // 全局拦截器处理
  } finally {
    tagsLoading.value = false;
  }
}

function toggleTagsPanel() {
  if (tagsPanelActive.value.length > 0) {
    tagsPanelActive.value = [];
  } else {
    tagsPanelActive.value = ['tags'];
    loadTags();
  }
}

// 把标签示例代码插入到编辑器光标位置
async function insertTagExample(example: string) {
  // 通过表单 API 获取 CodeEditor 组件实例
  const comp = baseFormApi.getFieldComponentRef<any>('temptext');
  if (comp && typeof comp.insertText === 'function') {
    comp.insertText(example);
    message.success('已插入到光标位置');
  } else {
    // 降级：追加到末尾
    const values = await baseFormApi.getValues();
    const current = values?.temptext || '';
    await baseFormApi.setValues({ temptext: `${current}\n${example}` });
    message.success('已追加到末尾');
  }
}

onMounted(() => {
  loadTags();
});

// 表单
const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '页面名称',
      componentProps: {
        placeholder: '请输入页面名称',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入页面名称' }),
    },
    {
      component: 'Select',
      fieldName: 'typeId',
      label: '页面类型',
      componentProps: {
        options: typeOptions,
        placeholder: '请选择页面类型',
      },
      rules: z.string().min(1, { message: '请选择页面类型' }),
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      componentProps: {
        placeholder: '排序值（越小越靠前）',
        allowClear: true,
        min: 0,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: 1,
      label: '状态',
      rules: 'selectRequired',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: statusList,
      },
    },
    {
      component: 'CodeEditor',
      fieldName: 'temptext',
      label: '模板内容',
      componentProps: {
        language: 'html',
        height: '400px',
      },
    },
  ],
});

// 预览
const previewVisible = ref(false);
const previewHtml = ref('');
const previewLoading = ref(false);

async function handlePreview() {
  const values = await baseFormApi.getValues();
  const temptext = values?.temptext;
  if (!temptext) {
    message.warning('请先输入模板内容');
    return;
  }
  previewLoading.value = true;
  try {
    const html = await previewTemplateDataApi({
      temptext,
      typeId: values?.typeId,
    });
    previewHtml.value = html || '';
    previewVisible.value = true;
  } catch {
    // 错误由全局拦截器处理
  } finally {
    previewLoading.value = false;
  }
}

// 保存
async function handleSave() {
  const validate = await baseFormApi.validate();
  if (!validate.valid) return;
  setLoading(true);
  const values = await baseFormApi.getValues();
  // 确保 templateId 传入
  values.templateId = drawerData.value.templateId;
  try {
    if (isCreate.value) {
      await addTemplateDataApi(values);
      message.success($t('ui.notification.create_success'));
    } else {
      await updateTemplateDataApi(drawerData.value.row.id, values);
      message.success($t('ui.notification.update_success'));
    }
    drawerApi.close();
    // 通知父组件刷新
    drawerApi.setData({ needRefresh: true });
  } catch {
    // 错误由全局拦截器处理
  } finally {
    setLoading(false);
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[75vw]',
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleSave();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{ row?: any; templateId: number }>();
      if (drawerData.value?.row?.id) {
        // 如果是编辑模式，获取详情
        loadDetail(drawerData.value.row.id);
      } else {
        baseFormApi.setValues({ templateId: drawerData.value.templateId });
      }
      setLoading(false);
    }
  },
});

async function loadDetail(id: number) {
  try {
    const res: any = await getTemplateDataDetailApi(id);
    const detail = res?.data || res;
    if (detail) {
      baseFormApi.setValues(detail);
    }
  } catch {
    // 全局拦截器处理
  }
}

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

// 提供给外部调用的 API
function open(data: { row?: any; templateId: number }) {
  drawerApi.setData(data);
  drawerApi.open();
}

function close() {
  drawerApi.close();
}

defineExpose({ open, close });
</script>

<template>
  <Drawer :title="drawerTitle">
    <!-- 右上角：标签文档 + 版本历史 + 全屏按钮 -->
    <template #extra>
      <Button
        type="link"
        size="small"
        :icon="h(LucideTag)"
        @click="toggleTagsPanel"
      >
        标签文档
      </Button>
      <Button
        v-if="!isCreate"
        type="link"
        size="small"
        :icon="h(LucideClock)"
        @click="openRevisionModal"
      >
        版本历史
      </Button>
      <Button
        type="text"
        size="small"
        :icon="h(isFullscreen ? LucideMinimize2 : LucideMaximize2)"
        @click="toggleFullscreen"
      />
    </template>

    <!-- 表单内容 -->
    <BaseForm />

    <!-- 标签文档面板（可折叠） -->
    <Collapse v-model:active-key="tagsPanelActive" ghost class="tags-panel">
      <CollapsePanel key="tags" header="模板标签文档（点击标签插入到光标位置）">
        <Input
          v-model:value="tagSearchKeyword"
          placeholder="搜索标签名称、说明或语法..."
          allow-clear
          size="small"
          class="mb-3"
        />
        <Empty v-if="tagsByCategory.length === 0" description="暂无标签" />
        <Tabs v-else type="card" size="small">
          <TabPane
            v-for="group in tagsByCategory"
            :key="group.category"
            :tab="`${group.category} (${group.tags.length})`"
          >
            <div class="tag-list">
              <div v-for="tag in group.tags" :key="tag.name" class="tag-card">
                <div class="tag-card-header">
                  <span class="tag-name">{{ tag.name }}</span>
                  <Button
                    type="primary"
                    size="small"
                    @click="insertTagExample(tag.example)"
                  >
                    插入
                  </Button>
                </div>
                <p class="tag-desc">{{ tag.description }}</p>
                <pre class="tag-syntax">{{ tag.syntax }}</pre>
                <div
                  v-if="tag.params && tag.params.length > 0"
                  class="tag-params"
                >
                  <span class="tag-params-title">参数：</span>
                  <span v-for="p in tag.params" :key="p[0]" class="tag-param">
                    <code>{{ p[0] }}</code> {{ p[1] }}
                  </span>
                </div>
              </div>
            </div>
          </TabPane>
        </Tabs>
      </CollapsePanel>
    </Collapse>

    <!-- 底部操作栏 -->
    <template #footer>
      <div class="editor-footer">
        <Button
          type="default"
          :loading="previewLoading"
          :icon="h(LucideEye)"
          @click="handlePreview"
        >
          预览
        </Button>
        <div>
          <Button @click="close()">取消</Button>
          <Button type="primary" @click="handleSave">保存</Button>
        </div>
      </div>
    </template>

    <!-- 版本历史弹窗 -->
    <RevisionModal
      v-model:visible="revisionVisible"
      :template-data-id="revisionTemplateDataId"
      @rollback="handleRollback"
    />

    <!-- 预览弹窗 -->
    <Modal
      v-model:open="previewVisible"
      title="模板预览"
      width="90%"
      wrap-class-name="full-modal"
      :footer="null"
      :destroy-on-close="true"
    >
      <iframe
        :srcdoc="previewHtml"
        class="preview-iframe"
        title="template-preview"
      ></iframe>
    </Modal>
  </Drawer>
</template>

<style scoped>
.editor-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.editor-footer > div {
  display: flex;
  gap: 8px;
}

.preview-iframe {
  width: 100%;
  height: 80vh;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
}

/* 标签文档面板 */
.tags-panel {
  padding-top: 4px;
  margin-top: 12px;
  border-top: 1px dashed #e8e8e8;
}

.tag-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 12px;
  padding: 4px 0;
}

.tag-card {
  padding: 10px 12px;
  background: #fafafa;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.tag-card:hover {
  border-color: #2563eb;
  box-shadow: 0 2px 8px rgb(37 99 235 / 10%);
}

.tag-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.tag-name {
  font-family: SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px;
  font-weight: 600;
  color: #0f172a;
}

.tag-desc {
  margin: 0 0 8px;
  font-size: 12px;
  line-height: 1.5;
  color: #64748b;
}

.tag-syntax {
  padding: 8px 10px;
  margin: 0;
  overflow-x: auto;
  font-family: SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  color: #e2e8f0;
  word-break: break-all;
  white-space: pre-wrap;
  background: #0f172a;
  border-radius: 4px;
}

.tag-params {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
  font-size: 12px;
  color: #475569;
}

.tag-params-title {
  font-weight: 600;
  color: #334155;
}

.tag-param code {
  padding: 1px 5px;
  font-size: 11px;
  color: #1d4ed8;
  background: #e2e8f0;
  border-radius: 3px;
}

/* 响应式：小屏幕全宽 */
@media (max-width: 767px) {
  :deep(.ant-drawer-content-wrapper) {
    width: 100% !important;
  }

  .tag-list {
    grid-template-columns: 1fr;
  }
}
</style>
