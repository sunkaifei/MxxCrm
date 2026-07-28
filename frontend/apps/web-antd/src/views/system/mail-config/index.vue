<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';
import type { VbenFormProps } from '@vben/common-ui';

import { defineAsyncComponent, h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideFilePenLine, LucidePlus, LucideTrash2 } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Popconfirm,
  Switch,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createMailConfigApi,
  createMailTemplateApi,
  deleteMailConfigApi,
  deleteMailTemplateApi,
  getMailConfigInfoApi,
  getMailConfigListApi,
  getMailTemplateInfoApi,
  getMailTemplateListApi,
  setDefaultMailConfigApi,
  updateMailConfigApi,
  updateMailTemplateApi,
} from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

// 异步加载富文本编辑器
const RichTextEditor = defineAsyncComponent(
  () => import('#/components/RichTextEditor/index.vue'),
);

const activeTab = ref('config');

// ==================== Tab1：邮箱账号配置 ====================
const configFormOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '账号名称',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: $t('ui.table.status'),
      componentProps: {
        options: statusList,
        placeholder: $t('ui.placeholder.select'),
      },
    },
  ],
};

const configGridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getMailConfigListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          name: formValues.name,
          status: formValues.status,
        });
      },
    },
  },
  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 70 },
    { title: '账号名称', field: 'name', minWidth: 140 },
    { title: 'SMTP主机', field: 'host', width: 160 },
    { title: '端口', field: 'port', width: 80, align: 'center' },
    { title: '用户名', field: 'username', width: 160 },
    { title: '发件邮箱', field: 'fromEmail', width: 200 },
    { title: '发件人名称', field: 'fromName', width: 140 },
    {
      title: 'SSL',
      field: 'isSsl',
      width: 80,
      align: 'center',
      slots: { default: 'isSsl' },
    },
    {
      title: '默认',
      field: 'isDefault',
      width: 80,
      align: 'center',
      slots: { default: 'isDefault' },
    },
    {
      title: $t('ui.table.status'),
      field: 'status',
      width: 90,
      align: 'center',
      slots: { default: 'status' },
    },
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
      width: 180,
      align: 'center',
      slots: { default: 'action' },
    },
  ],
};

const [ConfigGrid, configGridApi] = useVbenVxeGrid({
  gridOptions: configGridOptions,
  formOptions: configFormOptions,
});

// 邮箱配置抽屉
const configDrawerVisible = ref(false);
const configDrawerTitle = ref('新增邮箱账号');
const configSaving = ref(false);
const configIsEdit = ref(false);

function resetConfigForm() {
  Object.assign(configForm, {
    id: undefined,
    name: '',
    host: '',
    port: 465,
    username: '',
    password: '',
    fromEmail: '',
    fromName: '',
    isSsl: true,
    isDefault: false,
    status: 1,
  });
}

const configForm = reactive({
  id: undefined as number | undefined,
  name: '',
  host: '',
  port: 465,
  username: '',
  password: '',
  fromEmail: '',
  fromName: '',
  isSsl: true,
  isDefault: false,
  status: 1,
});

function openConfigCreate() {
  configIsEdit.value = false;
  configDrawerTitle.value = '新增邮箱账号';
  resetConfigForm();
  configDrawerVisible.value = true;
}

async function openConfigEdit(row: any) {
  configIsEdit.value = true;
  configDrawerTitle.value = '编辑邮箱账号';
  try {
    const detail: any = await getMailConfigInfoApi(row.id);
    Object.assign(configForm, {
      id: detail.id,
      name: detail.name ?? '',
      host: detail.host ?? '',
      port: detail.port ?? 465,
      username: detail.username ?? '',
      password: detail.password ?? '',
      fromEmail: detail.fromEmail ?? '',
      fromName: detail.fromName ?? '',
      isSsl: !!detail.isSsl,
      isDefault: !!detail.isDefault,
      status: detail.status ?? 1,
    });
    configDrawerVisible.value = true;
  } catch {
    // 错误由全局拦截器处理
  }
}

async function handleConfigSubmit() {
  if (!configForm.name || !configForm.host || !configForm.username) {
    message.warning('请填写完整账号信息');
    return;
  }
  configSaving.value = true;
  try {
    // isSsl/isDefault 在表单中为布尔值（Switch 组件），后端期望 i32，提交前转换
    const payload = {
      ...configForm,
      isSsl: configForm.isSsl ? 1 : 0,
      isDefault: configForm.isDefault ? 1 : 0,
    };
    if (configIsEdit.value) {
      await updateMailConfigApi(payload);
    } else {
      delete payload.id;
      await createMailConfigApi(payload);
    }
    message.success(
      configIsEdit.value
        ? $t('ui.notification.update_success')
        : $t('ui.notification.create_success'),
    );
    configDrawerVisible.value = false;
    configGridApi.query();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    configSaving.value = false;
  }
}

async function handleConfigStatusChanged(row: any, checked: boolean) {
  row.pending = true;
  row.status = checked ? 1 : 0;
  try {
    await updateMailConfigApi({
      id: row.id,
      name: row.name,
      host: row.host,
      port: row.port,
      username: row.username,
      password: row.password,
      fromEmail: row.fromEmail,
      fromName: row.fromName,
      isSsl: row.isSsl ? 1 : 0,
      isDefault: row.isDefault ? 1 : 0,
      status: row.status,
    });
    window.$message.success($t('ui.notification.update_success'));
  } finally {
    row.pending = false;
    configGridApi.query();
  }
}

async function handleSetDefault(row: any) {
  row.pending = true;
  try {
    await setDefaultMailConfigApi(row.id);
    window.$message.success('已设为默认邮箱');
    configGridApi.query();
  } finally {
    row.pending = false;
  }
}

async function handleConfigDelete(row: any) {
  row.pending = true;
  try {
    await deleteMailConfigApi([row.id]);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    configGridApi.query();
  }
}

// ==================== Tab2：邮件模板管理 ====================
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
  cellConfig: {
    isHover: true,
  },
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
    <Tabs v-model:activeKey="activeTab" class="mb-3">
      <Tabs.TabPane key="config" tab="邮箱账号配置" />
      <Tabs.TabPane key="template" tab="邮件模板管理" />
    </Tabs>

    <!-- Tab1：邮箱账号配置 -->
    <div v-show="activeTab === 'config'">
      <ConfigGrid table-title="邮箱账号配置">
        <template #toolbar-tools>
          <Button
            class="mr-2"
            type="primary"
            :icon="h(LucidePlus)"
            @click="openConfigCreate"
          >
            新建
          </Button>
        </template>

        <template #createdAt="{ row }">
          {{ formatDateTime(row.createTime) }}
        </template>

        <template #isSsl="{ row }">
          <Tag :color="row.isSsl ? 'success' : 'default'">
            {{ row.isSsl ? '是' : '否' }}
          </Tag>
        </template>

        <template #isDefault="{ row }">
          <Tag :color="row.isDefault ? 'gold' : 'default'">
            {{ row.isDefault ? '默认' : '-' }}
          </Tag>
        </template>

        <template #status="{ row }">
          <Switch
            :checked="row.status === 1"
            :loading="row.pending"
            :checked-children="$t('ui.switch.active')"
            :un-checked-children="$t('ui.switch.inactive')"
            @change="(checked: any) => handleConfigStatusChanged(row, checked)"
          />
        </template>

        <template #action="{ row }">
          <Button
            v-if="!row.isDefault"
            type="primary"
            link
            :loading="row.pending"
            @click="() => handleSetDefault(row)"
          >
            设默认
          </Button>
          <Button
            type="primary"
            link
            :icon="h(LucideFilePenLine)"
            @click="() => openConfigEdit(row)"
          />
          <Popconfirm
            :title="$t('ui.text.do_you_want_delete', { moduleName: '邮箱账号' })"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="() => handleConfigDelete(row)"
          >
            <Button
              type="danger"
              link
              :icon="h(LucideTrash2)"
            />
          </Popconfirm>
        </template>
      </ConfigGrid>
    </div>

    <!-- Tab2：邮件模板管理 -->
    <div v-show="activeTab === 'template'">
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
              type="danger"
              link
              :icon="h(LucideTrash2)"
            />
          </Popconfirm>
        </template>
      </TemplateGrid>
    </div>

    <!-- 邮箱配置抽屉 -->
    <Drawer
      v-model:open="configDrawerVisible"
      :title="configDrawerTitle"
      :width="560"
      :destroy-on-close="true"
      :mask-closable="false"
    >
      <Form layout="vertical">
        <FormItem label="账号名称" required>
          <Input
            v-model:value="configForm.name"
            placeholder="请输入账号名称"
            allow-clear
          />
        </FormItem>
        <FormItem label="SMTP主机" required>
          <Input
            v-model:value="configForm.host"
            placeholder="例如 smtp.exmail.qq.com"
            allow-clear
          />
        </FormItem>
        <FormItem label="端口" required>
          <InputNumber
            v-model:value="configForm.port"
            :min="1"
            :max="65535"
            placeholder="例如 465"
            style="width: 100%"
          />
        </FormItem>
        <FormItem label="用户名" required>
          <Input
            v-model:value="configForm.username"
            placeholder="SMTP 登录用户名"
            allow-clear
          />
        </FormItem>
        <FormItem label="密码">
          <Input.Password
            v-model:value="configForm.password"
            placeholder="SMTP 登录密码"
            allow-clear
          />
        </FormItem>
        <FormItem label="发件邮箱">
          <Input
            v-model:value="configForm.fromEmail"
            placeholder="发件人邮箱地址"
            allow-clear
          />
        </FormItem>
        <FormItem label="发件人名称">
          <Input
            v-model:value="configForm.fromName"
            placeholder="发件人显示名称"
            allow-clear
          />
        </FormItem>
        <FormItem label="启用SSL">
          <Switch
            v-model:checked="configForm.isSsl"
            :checked-children="$t('ui.switch.active')"
            :un-checked-children="$t('ui.switch.inactive')"
          />
        </FormItem>
        <FormItem label="设为默认">
          <Switch
            v-model:checked="configForm.isDefault"
            :checked-children="$t('ui.switch.active')"
            :un-checked-children="$t('ui.switch.inactive')"
          />
        </FormItem>
        <FormItem :label="$t('ui.table.status')">
          <Switch
            v-model:checked="configForm.status"
            :checked-value="1"
            :un-checked-value="0"
            :checked-children="$t('ui.switch.active')"
            :un-checked-children="$t('ui.switch.inactive')"
          />
        </FormItem>
      </Form>

      <template #footer>
        <div class="flex justify-end gap-2">
          <Button @click="configDrawerVisible = false">
            {{ $t('ui.button.cancel') }}
          </Button>
          <Button
            type="primary"
            :loading="configSaving"
            @click="handleConfigSubmit"
          >
            {{ $t('ui.button.ok') }}
          </Button>
        </div>
      </template>
    </Drawer>

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
:deep(.vxe-table--empty-block) {
  min-height: 150px;
}

:deep(.vxe-grid) {
  overflow: hidden;
}
</style>
