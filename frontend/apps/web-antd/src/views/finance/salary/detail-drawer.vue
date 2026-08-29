<script lang="ts" setup>
import { computed, reactive, ref } from 'vue';

import {
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  Empty,
  Form,
  FormItem,
  InputNumber,
  message,
  Modal,
  Spin,
  Table,
  Tag,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  approveSalaryApi,
  calculateSalarySingleApi,
  getSalaryDetailApi,
  getTaxDetailListApi,
  paySalaryApi,
  updateSalaryApi,
} from '#/api/core/finance';
import { $t } from '#/locales';

import ItemValuesModal from './item-values-modal.vue';

const emit = defineEmits<{ success: [] }>();

const loading = ref(false);
const detail = ref<any>(null);
const salaryId = ref<number>(0);

const statusMap: Record<number, { color: string; label: string }> = {
  0: { label: $t('page.finance.salary.status.pending'), color: 'blue' },
  1: { label: $t('page.finance.salary.status.approved'), color: 'orange' },
  2: { label: $t('page.finance.salary.status.paid'), color: 'green' },
};

const adjustVisible = ref(false);
const adjustLoading = ref(false);
const adjustForm = reactive({
  performanceBonus: 0,
  deduction: 0,
});

// ===== 自定义项值录入弹窗与明细展示 =====
const itemValuesVisible = ref(false);

function openItemValues() {
  itemValuesVisible.value = true;
}

function formatMoney(val: any) {
  if (val === null || val === undefined || val === '') return '-';
  return `¥${Number(val).toLocaleString()}`;
}

const commissionColumns = computed(() => [
  {
    title: $t('page.finance.salary.detail.column.contractName'),
    dataIndex: 'contractName',
  },
  {
    title: $t('page.finance.salary.detail.column.contractAmount'),
    dataIndex: 'contractAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.detail.column.paymentAmount'),
    dataIndex: 'paymentAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.detail.column.commissionRate'),
    dataIndex: 'commissionRate',
    customRender: ({ text }: any) =>
      text === null || text === undefined ? '-' : `${(text * 100).toFixed(2)}%`,
  },
  {
    title: $t('page.finance.salary.detail.column.commissionAmount'),
    dataIndex: 'commissionAmount',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.salary.detail.column.ruleName'),
    dataIndex: 'ruleName',
  },
]);

// 自定义项明细列（itemValues 为快照，itemType/isPretax 来自项目主表分类）
const itemValueColumns = computed(() => [
  {
    title: $t('page.finance.salary.itemValues.column.itemName'),
    dataIndex: 'itemName',
  },
  {
    title: $t('page.finance.salary.itemValues.column.itemType'),
    dataIndex: 'itemType',
    width: 90,
  },
  {
    title: $t('page.finance.salary.itemValues.column.pretax'),
    dataIndex: 'isPretax',
    width: 100,
  },
  {
    title: $t('page.finance.salary.itemValues.column.taxable'),
    dataIndex: 'isTaxable',
    width: 100,
  },
  {
    title: $t('page.finance.salary.itemValues.column.amount'),
    dataIndex: 'amount',
    width: 140,
  },
]);

async function loadDetail() {
  if (!salaryId.value) return;
  loading.value = true;
  try {
    const res: any = await getSalaryDetailApi(salaryId.value);
    detail.value = res?.data ?? res;
    await loadTaxDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.loadFailed'),
    );
  } finally {
    loading.value = false;
  }
}

// ===== 个税计算过程：数据取 salary_tax_detail 当月明细 =====
const taxDetail = ref<any>(null);
const taxDetailLoading = ref(false);

async function loadTaxDetail() {
  if (!detail.value?.employeeId || !detail.value?.year) return;
  taxDetailLoading.value = true;
  try {
    // 后端按 employeeId + year 返回全年逐月明细，此处取当月记录
    const res: any = await getTaxDetailListApi({
      employeeId: detail.value.employeeId,
      year: detail.value.year,
    });
    const list = res?.data ?? res;
    const rows = Array.isArray(list) ? list : list?.items || [];
    taxDetail.value =
      rows.find(
        (row: any) => Number(row.month) === Number(detail.value?.month),
      ) || null;
  } catch {
    taxDetail.value = null;
  } finally {
    taxDetailLoading.value = false;
  }
}

// 累计减除 = 累计收入 - 累计应纳税所得额（明细表未直接存储，按公式推导）
const taxCumulativeDeduction = computed(() => {
  const row = taxDetail.value;
  if (!row) return null;
  const income = row.cumulativeIncome;
  const taxable = row.cumulativeTaxable;
  if (
    income === null ||
    income === undefined ||
    taxable === null ||
    taxable === undefined
  ) {
    return null;
  }
  return Number(income) - Number(taxable);
});

function openAdjust() {
  adjustForm.performanceBonus = detail.value?.performanceBonus ?? 0;
  adjustForm.deduction = detail.value?.deductionAmount ?? 0;
  adjustVisible.value = true;
}

async function handleAdjustSubmit() {
  adjustLoading.value = true;
  try {
    await updateSalaryApi({
      id: salaryId.value,
      performanceBonus: adjustForm.performanceBonus,
      deductionAmount: adjustForm.deduction,
    });
    message.success($t('page.finance.salary.message.adjustSuccess'));
    adjustVisible.value = false;
    emit('success');
    await loadDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.adjustFailed'),
    );
  } finally {
    adjustLoading.value = false;
  }
}

// ===== 单员工重新核算：删除该员工当月待审核记录及明细后按最新配置重新生成 =====
function handleRecalculate() {
  Modal.confirm({
    title: $t('page.finance.salary.detail.recalcConfirmTitle'),
    content: $t('page.finance.salary.detail.recalcConfirmContent'),
    okText: $t('page.finance.salary.detail.recalcButton'),
    cancelText: $t('page.finance.common.cancel'),
    onOk: async () => {
      const res: any = await calculateSalarySingleApi(salaryId.value);
      message.success($t('page.finance.salary.message.recalcSuccess'));
      emit('success');
      // 重算“先删后建”会生成新记录 ID，需切换新 ID 刷新详情，否则后端报“工资记录不存在”
      const newId = Number(res?.data ?? 0);
      if (newId > 0) {
        salaryId.value = newId;
        await loadDetail();
      } else {
        drawerApi.close();
      }
    },
  });
}

function handleItemValuesSaved() {
  emit('success');
  void loadDetail();
}

async function handleApprove() {
  try {
    await approveSalaryApi(salaryId.value);
    message.success($t('page.finance.salary.message.approveSuccess'));
    emit('success');
    await loadDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.approveFailed'),
    );
  }
}

async function handlePay() {
  try {
    await paySalaryApi(salaryId.value);
    message.success($t('page.finance.salary.message.paySuccess'));
    emit('success');
    await loadDetail();
  } catch (error: any) {
    message.error(
      error?.message || $t('page.finance.salary.message.payFailed'),
    );
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onOpenChange(isOpen) {
    if (isOpen) {
      const data = drawerApi.getData() as { id?: number };
      detail.value = null;
      taxDetail.value = null;
      if (data?.id) {
        salaryId.value = Number(data.id);
        void loadDetail();
      }
    }
  },
});
</script>

<template>
  <Drawer
    class="w-[75%]! max-w-[75%]!"
    :title="$t('page.finance.salary.detail.drawerTitle')"
    :destroy-on-close="true"
    :footer="false"
    width="75%"
  >
    <Spin :spinning="loading">
      <Empty
        v-if="!detail && !loading"
        :description="$t('page.finance.salary.message.loadFailed')"
      />
      <div v-else-if="detail" class="salary-detail-wrap">
        <!-- 操作按钮行 -->
        <div class="mb-4 flex items-center justify-end gap-2">
          <Button v-if="detail.status === 0" @click="handleRecalculate">
            {{ $t('page.finance.salary.detail.recalcButton') }}
          </Button>
          <!-- 缺口1：录入自定义项值（仅待审核记录） -->
          <Button v-if="detail.status === 0" @click="openItemValues">
            {{ $t('page.finance.salary.action.itemValues') }}
          </Button>
          <Button v-if="detail.status === 0" type="primary" @click="openAdjust">
            {{ $t('page.finance.salary.button.adjust') }}
          </Button>
          <Button
            v-if="detail.status === 0"
            type="primary"
            @click="handleApprove"
          >
            {{ $t('page.finance.salary.detail.approveButton') }}
          </Button>
          <Button v-if="detail.status === 1" type="primary" @click="handlePay">
            {{ $t('page.finance.salary.detail.payButton') }}
          </Button>
        </div>

        <Card
          :title="$t('page.finance.salary.detail.salaryInfo')"
          class="mb-4"
          size="small"
        >
          <Descriptions :column="3" bordered size="small">
            <DescriptionsItem
              :label="$t('page.finance.salary.column.employeeName')"
            >
              {{ detail.employeeName }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.department')"
            >
              {{ detail.departmentName }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.yearMonth')"
            >
              {{ detail.year }}{{ $t('page.finance.common.year')
              }}{{ detail.month }}{{ $t('page.finance.common.month') }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.baseSalary')"
            >
              {{ formatMoney(detail.baseSalary) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.commissionAmount')"
            >
              {{ formatMoney(detail.commissionAmount) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.teamCommissionAmount')"
            >
              {{ formatMoney(detail.teamCommissionAmount) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.bonusAmount')"
            >
              {{ formatMoney(detail.bonusAmount) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.allocatedCommission')"
            >
              {{ formatMoney(detail.allocatedCommission) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.deferredCommission')"
            >
              {{ formatMoney(detail.deferredCommission) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.performanceBonus')"
            >
              {{ formatMoney(detail.performanceBonus) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.deductionAmount')"
            >
              {{ formatMoney(detail.deductionAmount) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.totalSalary')"
            >
              <span class="font-medium text-blue-600">
                {{ formatMoney(detail.totalSalary) }}
              </span>
            </DescriptionsItem>
            <!-- Bug#15：补全社保/公积金/个税/实发展示，便于财务核对 -->
            <DescriptionsItem
              :label="$t('page.finance.salary.column.socialInsurancePersonal')"
            >
              {{ formatMoney(detail.socialInsurancePersonal) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.housingFundPersonal')"
            >
              {{ formatMoney(detail.housingFundPersonal) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.taxAmount')"
            >
              <span class="font-medium text-orange-600">
                {{ formatMoney(detail.taxAmount) }}
              </span>
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.column.netSalary')"
            >
              <span class="font-medium text-green-600">
                {{ formatMoney(detail.netSalary) }}
              </span>
            </DescriptionsItem>
            <DescriptionsItem :label="$t('page.finance.common.status')">
              <Tag :color="statusMap[detail.status]?.color || 'default'">
                {{ statusMap[detail.status]?.label || detail.status }}
              </Tag>
            </DescriptionsItem>
          </Descriptions>
        </Card>

        <!-- 个税计算过程：累计预扣法各环节逐项展示，数据取 salary_tax_detail -->
        <Card
          :title="$t('page.finance.salary.detail.taxProcess')"
          class="mb-4"
          size="small"
          :loading="taxDetailLoading"
        >
          <Descriptions v-if="taxDetail" :column="4" bordered size="small">
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.monthlyIncome')"
            >
              {{ formatMoney(taxDetail.monthlyIncome) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.cumulativeIncome')"
            >
              {{ formatMoney(taxDetail.cumulativeIncome) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.cumulativeDeduction')"
            >
              {{
                taxCumulativeDeduction === null
                  ? '-'
                  : formatMoney(taxCumulativeDeduction)
              }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.applicableRate')"
            >
              {{
                taxDetail.applicableRate === null ||
                taxDetail.applicableRate === undefined
                  ? '-'
                  : `${(Number(taxDetail.applicableRate) * 100).toFixed(2)}%`
              }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.quickDeduction')"
            >
              {{ formatMoney(taxDetail.quickDeduction) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.cumulativeTaxShould')"
            >
              {{ formatMoney(taxDetail.cumulativeTaxShould) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.cumulativeTaxPaid')"
            >
              {{ formatMoney(taxDetail.cumulativeTaxPaid) }}
            </DescriptionsItem>
            <DescriptionsItem
              :label="$t('page.finance.salary.detail.tax.monthlyTax')"
            >
              <span class="font-medium text-red-600">
                {{ formatMoney(taxDetail.monthlyTax) }}
              </span>
            </DescriptionsItem>
          </Descriptions>
          <div v-else class="py-4 text-center text-gray-400">
            {{ $t('page.finance.salary.detail.tax.noData') }}
          </div>
        </Card>

        <Card
          :title="$t('page.finance.salary.detail.commissionDetail')"
          class="mb-4"
          size="small"
        >
          <!-- 后端 SalaryDetailDTO 明细字段名为 details -->
          <Table
            :data-source="detail.details || []"
            :columns="commissionColumns"
            :pagination="false"
            row-key="id"
            size="small"
          />
        </Card>

        <!-- 自定义项明细（含公式项快照与手动项，展示增减/税前税后/应税口径） -->
        <Card
          :title="$t('page.finance.salary.detail.itemValues')"
          size="small"
        >
          <Table
            :data-source="detail.itemValues || []"
            :columns="itemValueColumns"
            :pagination="false"
            row-key="id"
            size="small"
          >
            <template #bodyCell="{ column, record: item }">
              <template v-if="column.dataIndex === 'itemName'">
                {{ item.itemName }}
                <Tag v-if="item.enabled === 0" color="default">
                  {{ $t('page.finance.salary.itemValues.disabled') }}
                </Tag>
              </template>
              <template v-else-if="column.dataIndex === 'itemType'">
                <Tag :color="item.itemType === 2 ? 'red' : 'green'">
                  {{
                    item.itemType === 2
                      ? $t('page.finance.salary.itemValues.type.deduction')
                      : $t('page.finance.salary.itemValues.type.addition')
                  }}
                </Tag>
              </template>
              <template v-else-if="column.dataIndex === 'isPretax'">
                {{
                  item.isPretax === 1
                    ? $t('page.finance.salary.itemValues.pretax.before')
                    : $t('page.finance.salary.itemValues.pretax.after')
                }}
              </template>
              <template v-else-if="column.dataIndex === 'isTaxable'">
                {{
                  item.isTaxable === 1
                    ? $t('page.finance.salary.itemValues.taxable.yes')
                    : $t('page.finance.salary.itemValues.taxable.no')
                }}
              </template>
              <template v-else-if="column.dataIndex === 'amount'">
                <span
                  :class="item.itemType === 2 ? 'text-red-600' : 'text-green-600'"
                >
                  {{ item.itemType === 2 ? '-' : '+' }}{{ formatMoney(item.amount) }}
                </span>
              </template>
            </template>
            <template #emptyText>
              {{ $t('page.finance.salary.itemValues.empty') }}
            </template>
          </Table>
        </Card>
      </div>
    </Spin>

    <Modal
      v-model:open="adjustVisible"
      :title="$t('page.finance.salary.detail.adjustPerformance')"
      :confirm-loading="adjustLoading"
      @ok="handleAdjustSubmit"
    >
      <Form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }" class="py-4">
        <FormItem :label="$t('page.finance.salary.detail.performanceBonus')">
          <InputNumber
            v-model:value="adjustForm.performanceBonus"
            :min="0"
            :precision="2"
            style="width: 100%"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.salary.detail.deductionAmount')">
          <InputNumber
            v-model:value="adjustForm.deduction"
            :min="0"
            :precision="2"
            style="width: 100%"
          />
        </FormItem>
      </Form>
    </Modal>

    <!-- 手动录入自定义项值弹窗，保存成功后刷新详情 -->
    <ItemValuesModal
      v-model:open="itemValuesVisible"
      :record="detail"
      @success="handleItemValuesSaved"
    />
  </Drawer>
</template>

<style scoped>
.salary-detail-wrap {
  padding: 4px 0 16px;
}
</style>
