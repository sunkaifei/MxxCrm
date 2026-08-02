<script lang="ts" setup>
import { computed, h, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import {
  LucideMaximize2,
  LucideMinimize2,
  LucideEye,
  LucideClock,
} from '@vben/icons';
import { Button, message, Modal } from 'ant-design-vue';
import {
  addTemplateDataApi,
  updateTemplateDataApi,
  previewTemplateDataApi,
  getTemplateDataDetailApi,
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

const drawerData = ref<{ templateId: number; row?: any }>({ templateId: 0 });
const isCreate = computed(() => !drawerData.value?.row?.id);
const drawerTitle = computed(() => (isCreate.value ? '新增页面' : '编辑页面 - ' + (drawerData.value?.row?.name || '')));

// 全屏切换
const isFullscreen = ref(false);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
  drawerApi.setState({ width: isFullscreen.value ? '100%' : '75%' });
}

// 版本历史
const revisionVisible = ref(false);
const revisionTemplateDataId = ref<number | null>(null);

function openRevisionModal() {
  if (drawerData.value?.row?.id) {
    revisionTemplateDataId.value = drawerData.value.row.id;
    revisionVisible.value = true;
  }
}

function handleRollback(temptext: string) {
  baseFormApi.setValues({ temptext });
}

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
  width: '75%',
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleSave();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      drawerData.value = drawerApi.getData<{ templateId: number; row?: any }>();
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
function open(data: { templateId: number; row?: any }) {
  drawerApi.setData(data);
  drawerApi.open();
}

function close() {
  drawerApi.close();
}

defineExpose({ open, close });
</script>

<template>
  <Drawer :title="drawerTitle" :class="isFullscreen ? 'fullscreen-editor' : 'normal-editor'">
    <!-- 顶部工具栏 -->
    <div class="editor-toolbar">
      <div class="editor-toolbar-left">
        <Button
          type="text"
          size="small"
          :icon="h(isFullscreen ? LucideMinimize2 : LucideMaximize2)"
          @click="toggleFullscreen"
        >
          {{ isFullscreen ? '退出全屏' : '全屏编辑' }}
        </Button>
      </div>
      <div class="editor-toolbar-right">
        <Button
          v-if="!isCreate"
          type="link"
          size="small"
          :icon="h(LucideClock)"
          @click="openRevisionModal"
        >
          版本历史
        </Button>
      </div>
    </div>

    <!-- 表单内容 -->
    <BaseForm />

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
      />
    </Modal>
  </Drawer>
</template>

<style scoped>
.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color, #f0f0f0);
}

.editor-toolbar-left,
.editor-toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

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

/* 暗黑模式适配 */
:root.dark .editor-toolbar {
  --border-color: #333;
}

/* 响应式：小屏幕全宽 */
@media (max-width: 767px) {
  :deep(.ant-drawer-content-wrapper) {
    width: 100% !important;
  }
}
</style>