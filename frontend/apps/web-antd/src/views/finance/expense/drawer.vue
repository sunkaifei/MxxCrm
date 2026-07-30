<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import {
  Button,
  DatePicker,
  Input,
  InputNumber,
  message,
  Table,
  Tabs,
  TabPane,
  Tooltip,
  Upload,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  createExpenseApi,
  getExpenseDetailApi,
  getExpenseTypeListApi,
  updateExpenseApi,
} from '#/api';
import CustomerSelectModal from '../../crm/components/CustomerSelectModal.vue';
import OpportunitySelectModal from '../../crm/components/OpportunitySelectModal.vue';
import OrderSelectModal from '../../crm/components/OrderSelectModal.vue';

// drawerData 在 onOpenChange 中手动赋值，避免引用尚未定义的 drawerApi
const drawerData = ref<{ create: boolean; row: any; readonly?: boolean }>({
  create: true,
  row: {},
});
const isEdit = computed(() => !drawerData.value.create);
const isReadonly = computed(() => !!drawerData.value.readonly);

const activeTab = ref('basic');
const submitting = ref(false);

// ===== 费用类型 =====
const expenseTypeOptions = ref<{ label: string; value: number }[]>([]);

// ===== 关联业务 =====
const selectedCustomer = ref<{ id: number; name: string } | null>(null);
const selectedOpportunity = ref<{ id: number; name: string } | null>(null);
const selectedOrder = ref<{ id: number; name: string } | null>(null);

// ===== 选择器弹窗可见性 =====
const customerSelectVisible = ref(false);
const opportunitySelectVisible = ref(false);
const orderSelectVisible = ref(false);

// ===== 费用明细 =====
const items = ref<any[]>([]);

// ===== 主表附件 =====
const attachmentList = ref<any[]>([]);

// ===== 备注 =====
const remark = ref('');

const drawerClass = computed(() => 'finance-expense-drawer');

// 加载费用类型
async function loadExpenseTypes() {
  try {
    const res: any = await getExpenseTypeListApi({
      page: 1,
      pageSize: 100,
      enabled: 1,
    });
    const data = res?.data ?? res ?? {};
    const list = data.list || data.items || data.rows || data || [];
    const arr = Array.isArray(list) ? list : [];
    expenseTypeOptions.value = arr.map((t: any) => ({
      label: t.typeName || t.name || '',
      value: t.id,
    }));
  } catch (e) {
    console.error('[费用申请] 加载费用类型失败:', e);
    expenseTypeOptions.value = [];
  }
}

// 加载费用申请详情
async function loadExpenseDetail(id: number) {
  try {
    const res: any = await getExpenseDetailApi(id);
    const data = res?.data ?? res ?? {};
    // 回填基本信息
    await basicFormApi.setValues({
      title: data.title || '',
      expenseType: data.expenseType ?? data.expenseTypeId,
      applyDate: data.applyDate,
    });
    // 回填关联业务
    if (data.customerId) {
      selectedCustomer.value = {
        id: data.customerId,
        name: data.customerName || '',
      };
    }
    if (data.opportunityId) {
      selectedOpportunity.value = {
        id: data.opportunityId,
        name: data.opportunityName || data.opportunityTitle || '',
      };
    }
    if (data.orderId) {
      selectedOrder.value = {
        id: data.orderId,
        name: data.orderNo || data.orderTitle || '',
      };
    }
    // 回填费用明细
    items.value = Array.isArray(data.items) ? data.items.map((it: any) => ({ ...it })) : [];
    // 回填附件
    attachmentList.value = Array.isArray(data.attachments) ? data.attachments : [];
    remark.value = data.remark || '';
  } catch (e) {
    console.error('[费用申请] 加载详情失败:', e);
  }
}

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '费用标题',
    rules: 'required',
    componentProps: { placeholder: '请输入费用标题' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'expenseType',
    label: '费用类型',
    rules: 'required',
    componentProps: {
      placeholder: '请选择费用类型',
      allowClear: true,
      options: expenseTypeOptions,
    },
  },
  {
    component: 'DatePicker',
    fieldName: 'applyDate',
    label: '申请日期',
    componentProps: {
      placeholder: '请选择申请日期',
      valueFormat: 'YYYY-MM-DD',
      style: 'width:100%',
    },
  },
];

const [BasicForm, basicFormApi] = useVbenForm({
  schema: basicFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

// 费用明细列
const itemColumns = [
  { title: '#', width: 45, key: 'seq', customRender: ({ index }: any) => index + 1, align: 'center' },
  { title: '日期', key: 'itemDate', width: 160 },
  { title: '金额', key: 'amount', width: 130 },
  { title: '类别', key: 'category', width: 140 },
  { title: '说明', key: 'description', minWidth: 180 },
  { title: '附件', key: 'attachment', width: 120 },
  { title: '操作', key: 'action', width: 70 },
];

function addItem() {
  items.value.push({
    itemDate: '',
    amount: 0,
    category: '',
    description: '',
    attachment: null,
  });
}

function removeItem(index: number) {
  items.value.splice(index, 1);
}

// 明细合计
const totalAmount = computed(() => {
  return items.value.reduce((sum, it) => sum + (Number(it.amount) || 0), 0);
});

// 处理明细附件（单文件）
function handleItemAttachmentChange(index: number, info: any) {
  if (info.fileList && info.fileList.length > 0) {
    items.value[index].attachment = info.fileList[0];
  } else {
    items.value[index].attachment = null;
  }
}

// 主表附件上传（多文件）
function handleAttachmentChange(info: any) {
  attachmentList.value = info.fileList || [];
}

// 上传前阻止自动上传，仅收集文件对象
function beforeUpload() {
  return false;
}

async function handleSubmit() {
  try {
    // 1. 表单验证
    let validResult;
    try {
      validResult = await basicFormApi.validate();
    } catch (e) {
      console.error('[费用申请提交] 表单验证异常:', e);
      activeTab.value = 'basic';
      message.warning('请完善基本信息');
      return;
    }
    if (!validResult?.valid) {
      activeTab.value = 'basic';
      message.warning('请完善必填项');
      return;
    }

    // 2. 校验费用明细金额
    for (let i = 0; i < items.value.length; i++) {
      const it = items.value[i];
      if (Number(it.amount || 0) <= 0) {
        message.error(`第 ${i + 1} 行金额必须大于0`);
        activeTab.value = 'items';
        return;
      }
    }

    // 3. 收集数据
    const basicValues = await basicFormApi.getValues();
    const submitItems = items.value.map((it) => ({
      itemDate: it.itemDate || undefined,
      amount: Number(it.amount) || 0,
      category: it.category || undefined,
      description: it.description || undefined,
      attachment: it.attachment
        ? {
            name: it.attachment.name,
            size: it.attachment.size,
            uid: it.attachment.uid,
          }
        : undefined,
    }));

    const data = {
      title: basicValues.title,
      expenseType: basicValues.expenseType,
      applyDate: basicValues.applyDate || undefined,
      customerId: selectedCustomer.value?.id || undefined,
      customerName: selectedCustomer.value?.name || undefined,
      opportunityId: selectedOpportunity.value?.id || undefined,
      opportunityName: selectedOpportunity.value?.name || undefined,
      orderId: selectedOrder.value?.id || undefined,
      orderName: selectedOrder.value?.name || undefined,
      totalAmount: totalAmount.value,
      remark: remark.value || undefined,
      items: submitItems,
      attachments: attachmentList.value.map((f: any) => ({
        name: f.name,
        size: f.size,
        uid: f.uid,
      })),
    };

    submitting.value = true;
    const submitData = isEdit.value
      ? { ...data, id: drawerData.value.row.id }
      : data;

    if (isEdit.value) {
      await updateExpenseApi(submitData);
      message.success('更新成功');
    } else {
      await createExpenseApi(submitData);
      message.success('创建成功');
    }
    closeDrawer();
  } catch (e) {
    console.error('[费用申请提交] 提交失败:', e);
    message.error('操作失败');
  } finally {
    submitting.value = false;
  }
}

function closeDrawer() {
  drawerApi.close();
  drawerApi.setData({ needRefresh: true });
}

const [Drawer, drawerApi] = useVbenDrawer({
  onConfirm: handleSubmit,
  onOpenChange(isOpen) {
    if (isOpen) {
      const data = drawerApi.getData() as {
        create?: boolean;
        row?: any;
        readonly?: boolean;
      };
      drawerData.value = {
        create: data?.create ?? true,
        row: data?.row ?? {},
        readonly: data?.readonly,
      };
      activeTab.value = 'basic';
      // 重置
      selectedCustomer.value = null;
      selectedOpportunity.value = null;
      selectedOrder.value = null;
      customerSelectVisible.value = false;
      opportunitySelectVisible.value = false;
      orderSelectVisible.value = false;
      items.value = [];
      attachmentList.value = [];
      remark.value = '';
      basicFormApi.resetForm();
      // 加载费用类型
      loadExpenseTypes();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadExpenseDetail(Number(drawerData.value.row.id));
      }
    }
  },
});
</script>

<template>
  <Drawer
    :title="isReadonly ? '费用申请详情' : isEdit ? '修改费用申请' : '新建费用申请'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
    :show-footer="!isReadonly"
  >
    <template #extra>
      <Tooltip :title="isReadonly ? '只读模式' : '编辑模式'">
        <span class="text-xs text-gray-400 px-2">
          {{ isReadonly ? '只读' : '可编辑' }}
        </span>
      </Tooltip>
    </template>
    <Tabs v-model:activeKey="activeTab">
      <TabPane key="basic" tab="基本信息">
        <BasicForm />
        <!-- 关联业务 -->
        <div class="mt-3 px-1">
          <div class="text-sm font-medium mb-2 text-gray-700">关联业务（可选）</div>
          <div class="grid grid-cols-1 gap-3">
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 shrink-0" style="width: 82px">客户：</span>
              <div class="flex-1">
                <a
                  v-if="selectedCustomer"
                  class="text-blue-600 cursor-pointer"
                  @click="!isReadonly && (customerSelectVisible = true)"
                >
                  {{ selectedCustomer.name || `客户 #${selectedCustomer.id}` }}
                </a>
                <a
                  v-else-if="!isReadonly"
                  class="text-blue-600 cursor-pointer"
                  @click="customerSelectVisible = true"
                >选择客户</a>
                <span v-else>-</span>
              </div>
              <Button
                v-if="selectedCustomer && !isReadonly"
                type="link"
                size="small"
                danger
                @click="selectedCustomer = null"
              >清除</Button>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 shrink-0" style="width: 82px">商机：</span>
              <div class="flex-1">
                <a
                  v-if="selectedOpportunity"
                  class="text-blue-600 cursor-pointer"
                  @click="!isReadonly && (opportunitySelectVisible = true)"
                >
                  {{ selectedOpportunity.name || `商机 #${selectedOpportunity.id}` }}
                </a>
                <a
                  v-else-if="!isReadonly"
                  class="text-blue-600 cursor-pointer"
                  @click="opportunitySelectVisible = true"
                >选择商机</a>
                <span v-else>-</span>
              </div>
              <Button
                v-if="selectedOpportunity && !isReadonly"
                type="link"
                size="small"
                danger
                @click="selectedOpportunity = null"
              >清除</Button>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 shrink-0" style="width: 82px">订单：</span>
              <div class="flex-1">
                <a
                  v-if="selectedOrder"
                  class="text-blue-600 cursor-pointer"
                  @click="!isReadonly && (orderSelectVisible = true)"
                >
                  {{ selectedOrder.name || `订单 #${selectedOrder.id}` }}
                </a>
                <a
                  v-else-if="!isReadonly"
                  class="text-blue-600 cursor-pointer"
                  @click="orderSelectVisible = true"
                >选择订单</a>
                <span v-else>-</span>
              </div>
              <Button
                v-if="selectedOrder && !isReadonly"
                type="link"
                size="small"
                danger
                @click="selectedOrder = null"
              >清除</Button>
            </div>
          </div>
        </div>
        <!-- 备注 -->
        <div class="mt-4 px-1">
          <label class="text-sm text-gray-500">备注：</label>
          <Input
            v-model:value="remark"
            placeholder="备注信息"
            type="textarea"
            :rows="2"
            :disabled="isReadonly"
          />
        </div>
      </TabPane>

      <TabPane key="items" tab="费用明细">
        <div class="mb-3 flex justify-between items-center">
          <span class="text-sm text-gray-500">
            共 {{ items.length }} 项，合计：
            <span class="font-medium text-red-500">{{ totalAmount.toFixed(2) }}</span>
          </span>
          <Button v-if="!isReadonly" type="primary" size="small" @click="addItem">
            + 添加明细
          </Button>
        </div>
        <Table
          :columns="itemColumns"
          :data-source="items"
          :pagination="false"
          size="small"
          :scroll="{ x: 850 }"
          :row-key="(_: any, index: number) => index"
          bordered
        >
          <template #bodyCell="{ column, record, index }">
            <template v-if="column.key === 'itemDate'">
              <DatePicker
                v-model:value="record.itemDate"
                value-format="YYYY-MM-DD"
                placeholder="选择日期"
                size="small"
                style="width: 100%"
                :disabled="isReadonly"
              />
            </template>
            <template v-else-if="column.key === 'amount'">
              <InputNumber
                v-model:value="record.amount"
                :min="0"
                :precision="2"
                style="width: 100%"
                size="small"
                placeholder="金额"
                :disabled="isReadonly"
              />
            </template>
            <template v-else-if="column.key === 'category'">
              <Input
                v-model:value="record.category"
                placeholder="类别"
                size="small"
                :disabled="isReadonly"
              />
            </template>
            <template v-else-if="column.key === 'description'">
              <Input
                v-model:value="record.description"
                placeholder="说明"
                size="small"
                :disabled="isReadonly"
              />
            </template>
            <template v-else-if="column.key === 'attachment'">
              <Upload
                v-if="!isReadonly"
                :before-upload="beforeUpload"
                :max-count="1"
                :file-list="record.attachment ? [record.attachment] : []"
                :show-upload-list="true"
                @change="(info: any) => handleItemAttachmentChange(index, info)"
              >
                <Button type="link" size="small">上传</Button>
              </Upload>
              <span v-else>{{ record.attachment?.name || '-' }}</span>
            </template>
            <template v-else-if="column.key === 'action'">
              <Button
                v-if="!isReadonly"
                type="link"
                danger
                size="small"
                @click="removeItem(index)"
              >删除</Button>
            </template>
          </template>
        </Table>
        <!-- 金额汇总 -->
        <div class="mt-4 flex justify-end pr-4">
          <div class="flex items-center gap-2 border-t pt-2">
            <span class="font-medium">明细合计：</span>
            <span class="text-lg font-bold text-red-500">
              {{ totalAmount.toFixed(2) }}
            </span>
          </div>
        </div>
      </TabPane>

      <TabPane key="attachment" tab="附件">
        <div class="mb-3 text-sm text-gray-500">
          上传费用申请相关附件（支持多文件）
        </div>
        <Upload
          v-if="!isReadonly"
          :before-upload="beforeUpload"
          :file-list="attachmentList"
          multiple
          :show-upload-list="true"
          @change="handleAttachmentChange"
        >
          <Button type="primary">点击上传</Button>
        </Upload>
        <div v-if="isReadonly && attachmentList.length === 0" class="text-gray-400 text-center py-8">
          暂无附件
        </div>
      </TabPane>
    </Tabs>

    <!-- 客户选择弹窗 -->
    <CustomerSelectModal
      v-model:visible="customerSelectVisible"
      @select="(row: any) => {
        selectedCustomer = { id: row.id, name: row.customerName || row.name || '' };
        customerSelectVisible = false;
      }"
    />
    <!-- 商机选择弹窗 -->
    <OpportunitySelectModal
      v-model:visible="opportunitySelectVisible"
      :customer-id="selectedCustomer?.id"
      @select="(row: any) => {
        selectedOpportunity = { id: row.id, name: row.opportunityName || row.title || '' };
        opportunitySelectVisible = false;
      }"
    />
    <!-- 订单选择弹窗 -->
    <OrderSelectModal
      v-model:visible="orderSelectVisible"
      @select="(row: any) => {
        selectedOrder = { id: row.id, name: row.orderNo || row.title || '' };
        orderSelectVisible = false;
      }"
    />
  </Drawer>
</template>

<style>
.finance-expense-drawer {
  width: 75vw !important;
}
</style>
