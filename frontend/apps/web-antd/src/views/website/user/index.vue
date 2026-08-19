<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideEdit,
  LucideKeyRound,
  LucidePlus,
  LucideTrash2,
  LucideUserCheck,
} from '@vben/icons';

import {
  Button,
  Drawer,
  Form,
  FormItem,
  Input,
  InputPassword,
  message,
  Modal,
  Select,
  Tag,
  Textarea,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { userApi } from '#/api/core/website/user';

defineOptions({ name: 'WebsiteUser' });

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'username',
      label: '用户名',
      componentProps: {
        placeholder: '请输入用户名',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: '手机号',
      componentProps: {
        placeholder: '请输入手机号',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '正常', value: 0 },
          { label: '停用', value: 1 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  pagerConfig: {},
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await userApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          username: formValues.username || undefined,
          phone: formValues.phone || undefined,
          status: formValues.status ?? undefined,
        });
      },
      delete: async ({ body }) => {
        await userApi.batchDelete(body.removeRecords);
      },
    },
  },
  columns: [
    { title: '序号', type: 'seq', width: 70 },
    { title: '用户名', field: 'username', width: 140 },
    { title: '真实姓名', field: 'realName', width: 120 },
    { title: '手机号', field: 'phone', width: 140 },
    { title: '邮箱', field: 'email', width: 200 },
    {
      title: '会员等级',
      field: 'memberLevel',
      width: 110,
      slots: { default: 'memberLevel' },
    },
    {
      title: '状态',
      field: 'status',
      width: 90,
      slots: { default: 'status' },
    },
    { title: '累计消费', field: 'totalSpent', width: 120 },
    { title: '订单数', field: 'orderCount', width: 90 },
    { title: '最后登录', field: 'lastLoginTime', width: 170 },
    { title: '注册时间', field: 'createTime', width: 170 },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 新增/编辑抽屉
const drawerVisible = ref(false);
const isCreate = ref(true);
const editId = ref<number | undefined>(undefined);
const saving = ref(false);
const formRef = ref();

const formData = reactive({
  username: '',
  password: '',
  realName: '',
  phone: '',
  email: '',
  gender: 0,
  status: 0,
  memberLevel: 0,
  remark: '',
});

function resetForm() {
  formData.username = '';
  formData.password = '';
  formData.realName = '';
  formData.phone = '';
  formData.email = '';
  formData.gender = 0;
  formData.status = 0;
  formData.memberLevel = 0;
  formData.remark = '';
}

function handleAdd() {
  isCreate.value = true;
  editId.value = undefined;
  resetForm();
  drawerVisible.value = true;
}

function handleEdit(row: any) {
  isCreate.value = false;
  editId.value = row.id;
  formData.username = row.username || '';
  formData.password = '';
  formData.realName = row.realName || '';
  formData.phone = row.phone || '';
  formData.email = row.email || '';
  formData.gender = row.gender ?? 0;
  formData.status = row.status ?? 0;
  formData.memberLevel = row.memberLevel ?? 0;
  formData.remark = row.remark || '';
  drawerVisible.value = true;
}

async function handleSubmit() {
  try {
    await formRef.value?.validate();
  } catch {
    return;
  }
  saving.value = true;
  try {
    const payload: any = {
      username: formData.username,
      realName: formData.realName,
      phone: formData.phone,
      email: formData.email,
      gender: formData.gender,
      status: formData.status,
      memberLevel: formData.memberLevel,
      remark: formData.remark,
    };
    if (isCreate.value) {
      payload.password = formData.password;
      await userApi.create(payload);
      message.success('创建成功');
    } else {
      await userApi.update(editId.value as number, payload);
      message.success('更新成功');
    }
    drawerVisible.value = false;
    gridApi.query();
  } catch {
    message.error(isCreate.value ? '创建失败' : '更新失败');
  } finally {
    saving.value = false;
  }
}

// 重置密码弹窗
const pwdVisible = ref(false);
const pwdRow = ref<any>({});
const pwdForm = ref({ newPassword: '' });

function openPwdModal(row: any) {
  pwdRow.value = row;
  pwdForm.value = { newPassword: '' };
  pwdVisible.value = true;
}

async function handleResetPassword() {
  if (!pwdForm.value.newPassword) {
    message.warning('请输入新密码');
    return;
  }
  try {
    await userApi.resetPassword(pwdRow.value.id, pwdForm.value);
    message.success('密码重置成功');
    pwdVisible.value = false;
  } catch {
    message.error('密码重置失败');
  }
}

// 启用/停用
function handleToggleStatus(row: any) {
  const target = row.status === 0 ? 1 : 0;
  const text = target === 1 ? '停用' : '启用';
  Modal.confirm({
    title: '确认操作',
    content: `确定要${text}用户"${row.username}"吗？`,
    onOk: async () => {
      try {
        await userApi.updateStatus(row.id, target);
        message.success('操作成功');
        gridApi.query();
      } catch {
        message.error('操作失败');
      }
    },
  });
}

// 删除
async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除用户"${row.username}"吗？`,
    okType: 'danger',
    onOk: async () => {
      try {
        await userApi.batchDelete([row.id]);
        message.success('删除成功');
        gridApi.query();
      } catch {
        message.error('删除失败');
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="前台用户管理">
      <template #toolbar-tools>
        <Button type="primary" :icon="h(LucidePlus)" @click="handleAdd">
          新增用户
        </Button>
      </template>

      <template #memberLevel="{ row }">
        <Tag v-if="row.memberLevel === 0" color="default">普通会员</Tag>
        <Tag v-else-if="row.memberLevel === 1" color="cyan">银牌</Tag>
        <Tag v-else-if="row.memberLevel === 2" color="gold">金牌</Tag>
        <Tag v-else-if="row.memberLevel === 3" color="purple">钻石</Tag>
        <span v-else>—</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 0" color="success">正常</Tag>
        <Tag v-else-if="row.status === 1" color="default">停用</Tag>
        <Tag v-else color="default">未知</Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEdit)"
          @click="() => handleEdit(row)"
        >
          编辑
        </Button>
        <Button
          type="primary"
          link
          :icon="h(LucideKeyRound)"
          @click="() => openPwdModal(row)"
        >
          重置密码
        </Button>
        <Button
          type="primary"
          link
          :icon="h(LucideUserCheck)"
          @click="() => handleToggleStatus(row)"
        >
          {{ row.status === 0 ? '停用' : '启用' }}
        </Button>
        <Button
          type="primary"
          link
          danger
          :icon="h(LucideTrash2)"
          @click="() => handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>

    <!-- 新增/编辑抽屉 -->
    <Drawer
      v-model:open="drawerVisible"
      :title="isCreate ? '新增用户' : '编辑用户'"
      width="520"
    >
      <Form ref="formRef" :model="formData" layout="vertical">
        <FormItem
          name="username"
          label="用户名"
          :rules="[{ required: true, message: '请输入用户名' }]"
        >
          <Input
            v-model:value="formData.username"
            placeholder="请输入用户名"
            :disabled="!isCreate"
          />
        </FormItem>
        <FormItem
          v-if="isCreate"
          name="password"
          label="密码"
          :rules="[{ required: true, message: '请输入密码' }]"
        >
          <InputPassword
            v-model:value="formData.password"
            placeholder="请输入密码"
          />
        </FormItem>
        <FormItem name="realName" label="真实姓名">
          <Input
            v-model:value="formData.realName"
            placeholder="请输入真实姓名"
          />
        </FormItem>
        <FormItem name="phone" label="手机号">
          <Input v-model:value="formData.phone" placeholder="请输入手机号" />
        </FormItem>
        <FormItem name="email" label="邮箱">
          <Input v-model:value="formData.email" placeholder="请输入邮箱" />
        </FormItem>
        <FormItem name="gender" label="性别">
          <Select
            v-model:value="formData.gender"
            :options="[
              { label: '未知', value: 0 },
              { label: '男', value: 1 },
              { label: '女', value: 2 },
            ]"
          />
        </FormItem>
        <FormItem name="status" label="状态">
          <Select
            v-model:value="formData.status"
            :options="[
              { label: '正常', value: 0 },
              { label: '停用', value: 1 },
            ]"
          />
        </FormItem>
        <FormItem name="memberLevel" label="会员等级">
          <Select
            v-model:value="formData.memberLevel"
            :options="[
              { label: '普通会员', value: 0 },
              { label: '银牌', value: 1 },
              { label: '金牌', value: 2 },
              { label: '钻石', value: 3 },
            ]"
          />
        </FormItem>
        <FormItem name="remark" label="备注">
          <Textarea
            v-model:value="formData.remark"
            placeholder="请输入备注"
            :rows="3"
          />
        </FormItem>
      </Form>
      <template #footer>
        <div class="flex justify-end gap-3">
          <Button @click="drawerVisible = false">取消</Button>
          <Button type="primary" :loading="saving" @click="handleSubmit">
            {{ isCreate ? '创建' : '保存' }}
          </Button>
        </div>
      </template>
    </Drawer>

    <!-- 重置密码弹窗 -->
    <Modal
      v-model:open="pwdVisible"
      title="重置密码"
      width="460px"
      @ok="handleResetPassword"
    >
      <div class="space-y-4 py-2">
        <div class="flex items-center gap-3">
          <span class="w-20">用户名：</span>
          <span>{{ pwdRow.username || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-20">
            <span class="text-red-500">*</span> 新密码：
          </span>
          <InputPassword
            v-model:value="pwdForm.newPassword"
            placeholder="请输入新密码"
            style="flex: 1"
          />
        </div>
      </div>
    </Modal>
  </Page>
</template>
