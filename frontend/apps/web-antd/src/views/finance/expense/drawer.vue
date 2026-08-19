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
  TabPane,
  Tabs,
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
import { $t } from '#/locales';

import CustomerSelectModal from '../../crm/components/CustomerSelectModal.vue';
import OpportunitySelectModal from '../../crm/components/OpportunitySelectModal.vue';
import OrderSelectModal from '../../crm/components/OrderSelectModal.vue';

// drawerData 在 onOpenChange 中手动赋值，避免引用尚未定义的 drawerApi
const drawerData = ref<{ create: boolean; readonly?: boolean; row: any }>({
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
const selectedCustomer = ref<null | { id: number; name: string }>(null);
const selectedOpportunity = ref<null | { id: number; name: string }>(null);
const selectedOrder = ref<null | { id: number; name: string }>(null);

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
  } catch (error) {
    console.error('[expense] load expense types failed:', error);
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
    items.value = Array.isArray(data.items)
      ? data.items.map((it: any) => ({ ...it }))
      : [];
    // 回填附件
    attachmentList.value = Array.isArray(data.attachments)
      ? data.attachments
      : [];
    remark.value = data.remark || '';
  } catch (error) {
    console.error('[expense] load detail failed:', error);
  }
}

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: $t('page.finance.expense.drawer.expenseTitle'),
    rules: 'required',
    componentProps: {
      placeholder: $t('page.finance.expense.drawer.expenseTitlePlaceholder'),
    },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Select',
    fieldName: 'expenseType',
    label: $t('page.finance.expense.drawer.expenseType'),
    rules: 'required',
    componentProps: {
      placeholder: $t('page.finance.expense.drawer.expenseTypePlaceholder'),
      allowClear: true,
      options: expenseTypeOptions,
    },
  },
  {
    component: 'DatePicker',
    fieldName: 'applyDate',
    label: $t('page.finance.expense.drawer.applyDate'),
    componentProps: {
      placeholder: $t('page.finance.expense.drawer.applyDatePlaceholder'),
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
  {
    title: '#',
    width: 45,
    key: 'seq',
    customRender: ({ index }: any) => index + 1,
    align: 'center' as const,
  },
  {
    title: $t('page.finance.expense.drawer.itemDate'),
    key: 'itemDate',
    width: 160,
  },
  {
    title: $t('page.finance.expense.drawer.amount'),
    key: 'amount',
    width: 130,
  },
  {
    title: $t('page.finance.expense.drawer.category'),
    key: 'category',
    width: 140,
  },
  {
    title: $t('page.finance.expense.drawer.description'),
    key: 'description',
    minWidth: 180,
  },
  {
    title: $t('page.finance.expense.drawer.attachment'),
    key: 'attachment',
    width: 120,
  },
  { title: $t('page.finance.common.action'), key: 'action', width: 70 },
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
  items.value[index].attachment =
    info.fileList && info.fileList.length > 0 ? info.fileList[0] : null;
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
    } catch (error) {
      console.error('[expense] form validation error:', error);
      activeTab.value = 'basic';
      message.warning($t('page.finance.expense.drawer.improveBaseInfo'));
      return;
    }
    if (!validResult?.valid) {
      activeTab.value = 'basic';
      message.warning($t('page.finance.expense.drawer.improveRequired'));
      return;
    }

    // 2. 校验费用明细金额
    for (let i = 0; i < items.value.length; i++) {
      const it = items.value[i];
      if (Number(it.amount || 0) <= 0) {
        message.error(
          $t('page.finance.expense.drawer.rowAmountRequired', { index: i + 1 }),
        );
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
      message.success($t('page.finance.expense.drawer.updateSuccess'));
    } else {
      await createExpenseApi(submitData);
      message.success($t('page.finance.expense.drawer.createSuccess'));
    }
    closeDrawer();
  } catch (error) {
    console.error('[expense] submit failed:', error);
    message.error($t('page.finance.common.failed'));
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
        readonly?: boolean;
        row?: any;
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
    :title="
      isReadonly
        ? $t('page.finance.expense.drawer.titleDetail')
        : isEdit
          ? $t('page.finance.expense.drawer.titleEdit')
          : $t('page.finance.expense.drawer.titleCreate')
    "
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
    :show-footer="!isReadonly"
  >
    <template #extra>
      <Tooltip
        :title="
          isReadonly
            ? $t('page.finance.expense.drawer.readonlyMode')
            : $t('page.finance.expense.drawer.editMode')
        "
      >
        <span class="text-xs text-gray-400 px-2">
          {{
            isReadonly
              ? $t('page.finance.expense.drawer.readonly')
              : $t('page.finance.expense.drawer.editable')
          }}
        </span>
      </Tooltip>
    </template>
    <Tabs v-model:active-key="activeTab">
      <TabPane key="basic" :tab="$t('page.finance.expense.drawer.baseInfo')">
        <BasicForm />
        <!-- 关联业务 -->
        <div class="mt-3 px-1">
          <div class="text-sm font-medium mb-2 text-gray-700">
            {{ $t('page.finance.expense.drawer.relatedBusiness') }}
          </div>
          <div class="grid grid-cols-1 gap-3">
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 shrink-0" style="width: 82px"
                >{{ $t('page.finance.expense.drawer.customer') }}：</span
              >
              <div class="flex-1">
                <a
                  v-if="selectedCustomer"
                  class="text-blue-600 cursor-pointer"
                  @click="!isReadonly && (customerSelectVisible = true)"
                >
                  {{
                    selectedCustomer.name ||
                    $t('page.finance.expense.drawer.customerHash', {
                      id: selectedCustomer.id,
                    })
                  }}
                </a>
                <a
                  v-else-if="!isReadonly"
                  class="text-blue-600 cursor-pointer"
                  @click="customerSelectVisible = true"
                  >{{ $t('page.finance.expense.drawer.customerSelect') }}</a
                >
                <span v-else>-</span>
              </div>
              <Button
                v-if="selectedCustomer && !isReadonly"
                type="link"
                size="small"
                danger
                @click="selectedCustomer = null"
              >
                {{ $t('page.finance.expense.drawer.clear') }}
              </Button>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 shrink-0" style="width: 82px"
                >{{ $t('page.finance.expense.drawer.opportunity') }}：</span
              >
              <div class="flex-1">
                <a
                  v-if="selectedOpportunity"
                  class="text-blue-600 cursor-pointer"
                  @click="!isReadonly && (opportunitySelectVisible = true)"
                >
                  {{
                    selectedOpportunity.name ||
                    $t('page.finance.expense.drawer.opportunityHash', {
                      id: selectedOpportunity.id,
                    })
                  }}
                </a>
                <a
                  v-else-if="!isReadonly"
                  class="text-blue-600 cursor-pointer"
                  @click="opportunitySelectVisible = true"
                  >{{ $t('page.finance.expense.drawer.opportunitySelect') }}</a
                >
                <span v-else>-</span>
              </div>
              <Button
                v-if="selectedOpportunity && !isReadonly"
                type="link"
                size="small"
                danger
                @click="selectedOpportunity = null"
              >
                {{ $t('page.finance.expense.drawer.clear') }}
              </Button>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 shrink-0" style="width: 82px"
                >{{ $t('page.finance.expense.drawer.order') }}：</span
              >
              <div class="flex-1">
                <a
                  v-if="selectedOrder"
                  class="text-blue-600 cursor-pointer"
                  @click="!isReadonly && (orderSelectVisible = true)"
                >
                  {{
                    selectedOrder.name ||
                    $t('page.finance.expense.drawer.orderHash', {
                      id: selectedOrder.id,
                    })
                  }}
                </a>
                <a
                  v-else-if="!isReadonly"
                  class="text-blue-600 cursor-pointer"
                  @click="orderSelectVisible = true"
                  >{{ $t('page.finance.expense.drawer.orderSelect') }}</a
                >
                <span v-else>-</span>
              </div>
              <Button
                v-if="selectedOrder && !isReadonly"
                type="link"
                size="small"
                danger
                @click="selectedOrder = null"
              >
                {{ $t('page.finance.expense.drawer.clear') }}
              </Button>
            </div>
          </div>
        </div>
        <!-- 备注 -->
        <div class="mt-4 px-1">
          <label class="text-sm text-gray-500"
            >{{ $t('page.finance.expense.drawer.remark') }}：</label
          >
          <Input
            v-model:value="remark"
            :placeholder="$t('page.finance.expense.drawer.remarkPlaceholder')"
            type="text"
            :rows="2"
            :disabled="isReadonly"
          />
        </div>
      </TabPane>

      <TabPane
        key="items"
        :tab="$t('page.finance.expense.drawer.expenseDetail')"
      >
        <div class="mb-3 flex justify-between items-center">
          <span class="text-sm text-gray-500">
            {{
              $t('page.finance.expense.drawer.totalItems', {
                count: items.length,
              })
            }}
            <span class="font-medium text-red-500">{{
              totalAmount.toFixed(2)
            }}</span>
          </span>
          <Button
            v-if="!isReadonly"
            type="primary"
            size="small"
            @click="addItem"
          >
            {{ $t('page.finance.expense.drawer.addItem') }}
          </Button>
        </div>
        <Table
          :columns="itemColumns"
          :data-source="items"
          :pagination="false"
          size="small"
          :scroll="{ x: 850 }"
          :row-key="(_: any, index) => String(index)"
          bordered
        >
          <template #bodyCell="{ column, record, index }">
            <template v-if="column.key === 'itemDate'">
              <DatePicker
                v-model:value="record.itemDate"
                value-format="YYYY-MM-DD"
                :placeholder="
                  $t('page.finance.expense.drawer.itemDatePlaceholder')
                "
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
                :placeholder="
                  $t('page.finance.expense.drawer.amountPlaceholder')
                "
                :disabled="isReadonly"
              />
            </template>
            <template v-else-if="column.key === 'category'">
              <Input
                v-model:value="record.category"
                :placeholder="
                  $t('page.finance.expense.drawer.categoryPlaceholder')
                "
                size="small"
                :disabled="isReadonly"
              />
            </template>
            <template v-else-if="column.key === 'description'">
              <Input
                v-model:value="record.description"
                :placeholder="
                  $t('page.finance.expense.drawer.descriptionPlaceholder')
                "
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
                <Button type="link" size="small">
                  {{ $t('page.finance.expense.drawer.upload') }}
                </Button>
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
              >
                {{ $t('page.finance.common.delete') }}
              </Button>
            </template>
          </template>
        </Table>
        <!-- 金额汇总 -->
        <div class="mt-4 flex justify-end pr-4">
          <div class="flex items-center gap-2 border-t pt-2">
            <span class="font-medium">{{
              $t('page.finance.expense.drawer.detailTotal')
            }}</span>
            <span class="text-lg font-bold text-red-500">
              {{ totalAmount.toFixed(2) }}
            </span>
          </div>
        </div>
      </TabPane>

      <TabPane
        key="attachment"
        :tab="$t('page.finance.expense.drawer.attachment')"
      >
        <div class="mb-3 text-sm text-gray-500">
          {{ $t('page.finance.expense.drawer.attachmentTip') }}
        </div>
        <Upload
          v-if="!isReadonly"
          :before-upload="beforeUpload"
          :file-list="attachmentList"
          multiple
          :show-upload-list="true"
          @change="handleAttachmentChange"
        >
          <Button type="primary">
            {{ $t('page.finance.expense.drawer.clickUpload') }}
          </Button>
        </Upload>
        <div
          v-if="isReadonly && attachmentList.length === 0"
          class="text-gray-400 text-center py-8"
        >
          {{ $t('page.finance.expense.drawer.noAttachment') }}
        </div>
      </TabPane>
    </Tabs>

    <!-- 客户选择弹窗 -->
    <CustomerSelectModal
      v-model:visible="customerSelectVisible"
      @select="
        (row: any) => {
          selectedCustomer = {
            id: row.id,
            name: row.customerName || row.name || '',
          };
          customerSelectVisible = false;
        }
      "
    />
    <!-- 商机选择弹窗 -->
    <OpportunitySelectModal
      v-model:visible="opportunitySelectVisible"
      :customer-id="selectedCustomer?.id"
      @select="
        (row: any) => {
          selectedOpportunity = {
            id: row.id,
            name: row.opportunityName || row.title || '',
          };
          opportunitySelectVisible = false;
        }
      "
    />
    <!-- 订单选择弹窗 -->
    <OrderSelectModal
      v-model:visible="orderSelectVisible"
      @select="
        (row: any) => {
          selectedOrder = { id: row.id, name: row.orderNo || row.title || '' };
          orderSelectVisible = false;
        }
      "
    />
  </Drawer>
</template>

<style>
.finance-expense-drawer {
  width: 75vw !important;
}
</style>
