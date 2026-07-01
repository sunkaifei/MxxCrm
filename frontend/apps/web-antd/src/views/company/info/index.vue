<script lang="ts" setup>
import { ref, computed, onMounted } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  Empty,
  Form,
  FormItem,
  Input,
  message,
  Modal,
  Textarea,
  Popconfirm,
  Select,
  Table,
  Tag,
} from 'ant-design-vue';

import {
  deleteCompanyAccountApi,
  getCompanyAccountListApi,
  getCompanyInfoApi,
  saveCompanyAccountApi,
  updateCompanyInfoApi,
} from '#/api';

const accessStore = useAccessStore();

// 通过权限码判断是否有编辑权限（超管拥有 company:info:edit 权限）
const canEdit = computed(() => accessStore.hasAccessCode('company:info:edit'));

const loading = ref(false);
const editMode = ref(false); // 编辑模式开关
const companyInfo = ref<any>({});
const accountList = ref<any[]>([]);

// 银行账户弹窗
const accountModalVisible = ref(false);
const accountModalTitle = ref('新增银行账户');
const accountForm = ref<any>({
  id: null,
  bankName: '',
  accountName: '',
  accountNumber: '',
  accountType: 1,
  isDefault: 0,
  remark: '',
});

const accountTypeOptions = [
  { value: 1, label: '基本户' },
  { value: 2, label: '一般户' },
  { value: 3, label: '其他' },
];

const accountTypeLabel = (type: number) => {
  return accountTypeOptions.find((t) => t.value === type)?.label || '-';
};

// 法人电话脱敏（普通用户看 138****1234）
const maskedLegalPhone = computed(() => {
  const phone = companyInfo.value?.legalPhone;
  if (!phone) return '-';
  if (canEdit.value) return phone;
  const str = String(phone);
  if (str.length < 7) return str;
  return str.slice(0, 3) + '****' + str.slice(-4);
});

// 加载企业信息
const loadCompanyInfo = async () => {
  loading.value = true;
  try {
    const [info, accounts]: any = await Promise.all([
      getCompanyInfoApi(),
      getCompanyAccountListApi(),
    ]);
    companyInfo.value = info || {};
    accountList.value = accounts || [];
  } finally {
    loading.value = false;
  }
};

// 保存企业信息
const handleSaveInfo = async () => {
  try {
    await updateCompanyInfoApi(companyInfo.value);
    message.success('保存成功');
    editMode.value = false;
    loadCompanyInfo();
  } catch (e) {
    console.error(e);
  }
};

// 取消编辑
const handleCancelEdit = () => {
  editMode.value = false;
  loadCompanyInfo();
};

// 银行账户操作
const handleAddAccount = () => {
  accountModalTitle.value = '新增银行账户';
  accountForm.value = {
    id: null,
    bankName: '',
    accountName: '',
    accountNumber: '',
    accountType: 1,
    isDefault: 0,
    remark: '',
  };
  accountModalVisible.value = true;
};

const handleEditAccount = (record: any) => {
  accountModalTitle.value = '编辑银行账户';
  accountForm.value = { ...record };
  accountModalVisible.value = true;
};

const handleSaveAccount = async () => {
  if (!accountForm.value.bankName) {
    message.warning('请输入开户银行');
    return;
  }
  if (!accountForm.value.accountNumber) {
    message.warning('请输入银行账号');
    return;
  }
  try {
    await saveCompanyAccountApi(accountForm.value);
    message.success('保存成功');
    accountModalVisible.value = false;
    loadCompanyInfo();
  } catch (e) {
    console.error(e);
  }
};

const handleDeleteAccount = async (id: number) => {
  try {
    await deleteCompanyAccountApi([id]);
    message.success('删除成功');
    loadCompanyInfo();
  } catch (e) {
    console.error(e);
  }
};

// 银行账户表格列
const accountColumns = [
  { title: '开户银行', dataIndex: 'bankName', key: 'bankName' },
  { title: '账户名称', dataIndex: 'accountName', key: 'accountName' },
  { title: '银行账号', dataIndex: 'accountNumber', key: 'accountNumber' },
  {
    title: '账户类型',
    dataIndex: 'accountType',
    key: 'accountType',
    customRender: ({ value }: any) => accountTypeLabel(value),
  },
  {
    title: '默认',
    dataIndex: 'isDefault',
    key: 'isDefault',
  },
  { title: '备注', dataIndex: 'remark', key: 'remark' },
  { title: '操作', key: 'action', width: 150 },
];

onMounted(() => {
  loadCompanyInfo();
});
</script>

<template>
  <Page :hide-footer="true">
    <Card class="company-card" :bordered="false">
      <template #title>
        <span>企业基本信息</span>
      </template>
      <template #extra>
        <template v-if="canEdit">
          <Button v-if="!editMode" type="primary" @click="editMode = true">
            编辑
          </Button>
          <template v-else>
            <Button type="primary" @click="handleSaveInfo">保存</Button>
            <Button style="margin-left: 8px" @click="handleCancelEdit">
              取消
            </Button>
          </template>
        </template>
      </template>

      <!-- 展示模式 -->
      <Descriptions
        v-if="!editMode"
        :column="2"
        bordered
        size="middle"
        :label-style="{ width: '140px' }"
      >
        <DescriptionsItem label="企业名称">
          {{ companyInfo.companyName || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="统一社会信用代码">
          {{ companyInfo.creditCode || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="法定代表人">
          {{ companyInfo.legalPerson || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="法人电话">
          {{ maskedLegalPhone }}
        </DescriptionsItem>
        <DescriptionsItem label="注册地址" :span="2">
          {{ companyInfo.registerAddress || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="联系电话">
          {{ companyInfo.contactPhone || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="联系邮箱">
          {{ companyInfo.contactEmail || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="税号">
          {{ companyInfo.taxNumber || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="发票抬头">
          {{ companyInfo.invoiceTitle || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="经营范围" :span="2">
          {{ companyInfo.businessScope || '-' }}
        </DescriptionsItem>
        <DescriptionsItem label="备注" :span="2">
          {{ companyInfo.remark || '-' }}
        </DescriptionsItem>
      </Descriptions>

      <!-- 编辑模式 -->
      <Form
        v-else
        :label-col="{ span: 6 }"
        :wrapper-col="{ span: 16 }"
      >
        <FormItem label="企业名称">
          <Input
            v-model:value="companyInfo.companyName"
            placeholder="请输入企业名称"
          />
        </FormItem>
        <FormItem label="统一社会信用代码">
          <Input
            v-model:value="companyInfo.creditCode"
            placeholder="请输入统一社会信用代码"
          />
        </FormItem>
        <FormItem label="法定代表人">
          <Input
            v-model:value="companyInfo.legalPerson"
            placeholder="请输入法定代表人"
          />
        </FormItem>
        <FormItem label="法人电话">
          <Input
            v-model:value="companyInfo.legalPhone"
            placeholder="请输入法人电话"
          />
        </FormItem>
        <FormItem label="注册地址">
          <Input
            v-model:value="companyInfo.registerAddress"
            placeholder="请输入注册地址"
          />
        </FormItem>
        <FormItem label="联系电话">
          <Input
            v-model:value="companyInfo.contactPhone"
            placeholder="请输入联系电话"
          />
        </FormItem>
        <FormItem label="联系邮箱">
          <Input
            v-model:value="companyInfo.contactEmail"
            placeholder="请输入联系邮箱"
          />
        </FormItem>
        <FormItem label="税号">
          <Input
            v-model:value="companyInfo.taxNumber"
            placeholder="请输入税号"
          />
        </FormItem>
        <FormItem label="发票抬头">
          <Input
            v-model:value="companyInfo.invoiceTitle"
            placeholder="请输入发票抬头"
          />
        </FormItem>
        <FormItem label="经营范围">
          <Textarea
            v-model:value="companyInfo.businessScope"
            :rows="3"
            placeholder="请输入经营范围"
          />
        </FormItem>
        <FormItem label="备注">
          <Textarea
            v-model:value="companyInfo.remark"
            :rows="3"
            placeholder="请输入备注"
          />
        </FormItem>
      </Form>
    </Card>

    <Card class="company-card" :bordered="false">
      <template #title>
        <span>银行账户信息</span>
      </template>
      <template #extra>
        <Button
          v-if="canEdit"
          type="primary"
          @click="handleAddAccount"
        >
          新增账户
        </Button>
      </template>

      <Table
        :columns="accountColumns"
        :data-source="accountList"
        :loading="loading"
        row-key="id"
        :pagination="false"
        size="middle"
      >
        <template #emptyText>
          <Empty description="暂无银行账户" />
        </template>
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'action'">
            <template v-if="canEdit">
              <Button type="link" @click="handleEditAccount(record)">
                编辑
              </Button>
              <Popconfirm
                title="确定删除该银行账户吗？"
                @confirm="handleDeleteAccount(record.id)"
              >
                <Button type="link" danger>删除</Button>
              </Popconfirm>
            </template>
            <span v-else>-</span>
          </template>
          <template v-else-if="column.key === 'isDefault'">
            <Tag v-if="record.isDefault === 1" color="green">默认</Tag>
            <span v-else>否</span>
          </template>
        </template>
      </Table>
    </Card>

    <!-- 银行账户弹窗 -->
    <Modal
      v-model:open="accountModalVisible"
      :title="accountModalTitle"
      :width="560"
      :footer="null"
    >
      <Form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <FormItem label="开户银行" required>
          <Input
            v-model:value="accountForm.bankName"
            placeholder="请输入开户银行"
          />
        </FormItem>
        <FormItem label="账户名称">
          <Input
            v-model:value="accountForm.accountName"
            placeholder="请输入账户名称"
          />
        </FormItem>
        <FormItem label="银行账号" required>
          <Input
            v-model:value="accountForm.accountNumber"
            placeholder="请输入银行账号"
          />
        </FormItem>
        <FormItem label="账户类型">
          <Select
            v-model:value="accountForm.accountType"
            :options="accountTypeOptions"
            placeholder="请选择账户类型"
          />
        </FormItem>
        <FormItem label="是否默认">
          <Select
            v-model:value="accountForm.isDefault"
            :options="[
              { value: 1, label: '是' },
              { value: 0, label: '否' },
            ]"
            placeholder="请选择是否默认"
          />
        </FormItem>
        <FormItem label="备注">
          <Textarea
            v-model:value="accountForm.remark"
            :rows="3"
            placeholder="请输入备注"
          />
        </FormItem>
        <FormItem :wrapper-col="{ offset: 6, span: 16 }">
          <Button type="primary" @click="handleSaveAccount">确定</Button>
          <Button
            style="margin-left: 8px"
            @click="accountModalVisible = false"
          >
            取消
          </Button>
        </FormItem>
      </Form>
    </Modal>
  </Page>
</template>

<style scoped>
.company-card {
  margin-bottom: 16px;
}
</style>
