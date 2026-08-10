<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';
import type { VbenFormProps } from '@vben/common-ui';

import { defineAsyncComponent, h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  ExternalLink,
  LucideFilePenLine,
  LucidePlus,
  LucideTrash2,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Alert,
  Button,
  Drawer,
  Form,
  FormItem,
  Input,
  message,
  Popconfirm,
} from 'ant-design-vue';

import { useRouter } from 'vue-router';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createMailTemplateApi,
  deleteMailTemplateApi,
  getMailTemplateInfoApi,
  getMailTemplateListApi,
  updateMailTemplateApi,
} from '#/api';
import { $t } from '#/locales';

// 异步加载富文本编辑器
const RichTextEditor = defineAsyncComponent(
  () => import('#/components/RichTextEditor/index.vue'),
);

const router = useRouter();

function gotoIntegrationConfig() {
  router.push({
    path: '/system/integration-config',
    query: { category: 'notification' },
  });
}

// ==================== Tab：邮件模板管理 ====================
const templateFormOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '模板名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
  ],
};

const templateGridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  pagerConfig: {},
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getMailTemplateListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          name: formValues.name,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 70 },
    { title: '模板名称', field: 'name', minWidth: 200 },
    { title: '主题', field: 'subject', minWidth: 240 },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      width: 140,
      align: 'center',
      slots: { default: 'templateAction' },
    },
  ],
};

const [TemplateGrid, templateGridApi] = useVbenVxeGrid({
  gridOptions: templateGridOptions,
  formOptions: templateFormOptions,
});

// 邮件模板抽屉
const templateDrawerVisible = ref(false);
const templateDrawerTitle = ref('新增邮件模板');
const templateSaving = ref(false);
const templateIsEdit = ref(false);
const templateBody = ref('');

const templateForm = reactive({
  id: undefined as number | undefined,
  name: '',
  subject: '',
});

function resetTemplateForm() {
  Object.assign(templateForm, {
    id: undefined,
    name: '',
    subject: '',
  });
  templateBody.value = '';
}

function openTemplateCreate() {
  templateIsEdit.value = false;
  templateDrawerTitle.value = '新增邮件模板';
  resetTemplateForm();
  templateDrawerVisible.value = true;
}

async function openTemplateEdit(row: any) {
  templateIsEdit.value = true;
  templateDrawerTitle.value = '编辑邮件模板';
  try {
    const detail: any = await getMailTemplateInfoApi(row.id);
    Object.assign(templateForm, {
      id: detail.id,
      name: detail.name ?? '',
      subject: detail.subject ?? '',
    });
    templateBody.value = detail.body ?? '';
    templateDrawerVisible.value = true;
  } catch {
    // 错误由全局拦截器处理
  }
}

async function handleTemplateSubmit() {
  if (!templateForm.name) {
    message.warning('请输入模板名称');
    return;
  }
  const bodyText =
    templateBody.value?.replace(/<[^>]+>/g, '').trim() || '';
  if (!bodyText) {
    message.warning('请输入模板正文');
    return;
  }
  templateSaving.value = true;
  try {
    const payload = {
      ...(templateIsEdit.value ? { id: templateForm.id } : {}),
      name: templateForm.name,
      subject: templateForm.subject,
      body: templateBody.value,
    };
    if (templateIsEdit.value) {
      await updateMailTemplateApi(payload);
    } else {
      await createMailTemplateApi(payload);
    }
    message.success(
      templateIsEdit.value
        ? $t('ui.notification.update_success')
        : $t('ui.notification.create_success'),
    );
    templateDrawerVisible.value = false;
    templateGridApi.query();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    templateSaving.value = false;
  }
}

async function handleTemplateDelete(row: any) {
  row.pending = true;
  try {
    await deleteMailTemplateApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    templateGridApi.query();
  }
}
</script>

<template>
  <Page>
    <!-- 顶部提示：邮箱账号配置已迁移 -->
    <Alert
      class="mb-4"
      type="info"
      show-icon
      :banner="false"
      :closable="false"
    >
      <template #message>
        <div class="flex items-center justify-between">
          <div>
            <span class="font-medium">邮箱账号（SMTP）配置已统一迁移到</span>
            <span class="font-medium text-blue-600">「系统设置 → 第三方接口配置 → 通知配置 → SMTP邮件」</span>
            <span>，您可以在那里完成发送账号的配置、测试和启用 / 禁用。</span>
          </div>
          <Button
            type="link"
            size="small"
            @click="gotoIntegrationConfig"
          >
            <template #icon>
              <ExternalLink class="h-4 w-4" />
            </template>
            前往配置
          </Button>
        </div>
      </template>
    </Alert>

    <!-- 邮件模板管理 -->
    <TemplateGrid table-title="邮件模板管理">
      <template #toolbar-tools>
        <Button
          class="mr-2"
          type="primary"
          :icon="h(LucidePlus)"
          @click="openTemplateCreate"
        >
          新建
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #templateAction="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => openTemplateEdit(row)"
        />
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: '邮件模板' })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleTemplateDelete(row)"
        >
          <Button
            danger
            link
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </TemplateGrid>

    <!-- 邮件模板抽屉 -->
    <Drawer
      v-model:open="templateDrawerVisible"
      :title="templateDrawerTitle"
      :width="720"
      :destroy-on-close="true"
      :mask-closable="false"
    >
      <Form layout="vertical">
        <FormItem label="模板名称" required>
          <Input
            v-model:value="templateForm.name"
            placeholder="请输入模板名称"
            allow-clear
          />
        </FormItem>
        <FormItem label="主题">
          <Input
            v-model:value="templateForm.subject"
            placeholder="请输入邮件主题"
            allow-clear
          />
        </FormItem>
        <FormItem label="正文" required>
          <RichTextEditor
            v-model="templateBody"
            placeholder="请输入邮件正文..."
            :height="420"
          />
        </FormItem>
      </Form>

      <template #footer>
        <div class="flex justify-end gap-2">
          <Button @click="templateDrawerVisible = false">
            {{ $t('ui.button.cancel') }}
          </Button>
          <Button
            type="primary"
            :loading="templateSaving"
            @click="handleTemplateSubmit"
          >
            {{ $t('ui.button.ok') }}
          </Button>
        </div>
      </template>
    </Drawer>
  </Page>
</template>

<style scoped>
.mb-4 {
  margin-bottom: 16px;
}

.mr-2 {
  margin-right: 8px;
}
</style>
