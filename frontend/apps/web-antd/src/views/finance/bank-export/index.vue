<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Card,
  Col,
  Empty,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Modal,
  Row,
  Select,
  Statistic,
  Table,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import {
  downloadBankFileApi,
  generateBankExcelFileApi,
  generateBankFileApi,
  getBankExportListApi,
} from '#/api/core/finance';
import { $t } from '#/locales';
import { PageUsageGuide } from '#/components/PageUsageGuide';

const guideStepCount = 5;

const accessStore = useAccessStore();

const formatMoney = (val: any) => Number(val || 0).toFixed(2);

const now = new Date();
const searchForm = reactive({
  year: now.getFullYear(),
  month: now.getMonth() + 1,
  bankType: undefined as string | undefined,
});

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: `${i + 1}${$t('page.finance.common.month')}`,
}));

// ===== 银行类型 =====
const bankTypeOptions = [
  { label: $t('page.finance.bankExport.bankType.icbc'), value: 'icbc' },
  { label: $t('page.finance.bankExport.bankType.ccb'), value: 'ccb' },
  { label: $t('page.finance.bankExport.bankType.cmb'), value: 'cmb' },
  { label: $t('page.finance.bankExport.bankType.boc'), value: 'boc' },
  { label: $t('page.finance.bankExport.bankType.abc'), value: 'abc' },
];

const bankTypeLabelMap: Record<string, string> = {
  icbc: $t('page.finance.bankExport.bankType.icbc'),
  ccb: $t('page.finance.bankExport.bankType.ccb'),
  cmb: $t('page.finance.bankExport.bankType.cmb'),
  boc: $t('page.finance.bankExport.bankType.boc'),
  abc: $t('page.finance.bankExport.bankType.abc'),
};

// 状态映射：0=生成中,1=成功,2=失败
const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: $t('page.finance.bankExport.status.generating'), color: 'processing' },
  1: { label: $t('page.finance.bankExport.status.success'), color: 'success' },
  2: { label: $t('page.finance.bankExport.status.failed'), color: 'error' },
};

// ===== 列表数据 =====
const loading = ref(false);
const tableData = ref<any[]>([]);

async function loadList() {
  loading.value = true;
  try {
    const res: any = await getBankExportListApi(searchForm);
    const data = res?.data || res;
    tableData.value = Array.isArray(data) ? data : data?.items || data?.list || [];
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.bankExport.message.loadFailed'));
    tableData.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 生成代发文件弹窗 =====
const generateVisible = ref(false);
const generateLoading = ref(false);
const generateForm = reactive({
  year: now.getFullYear(),
  month: now.getMonth() + 1,
  bankType: 'icbc',
  fileFormat: 'txt' as 'txt' | 'xlsx', // V9: 支持 TXT 与 Excel 双格式
});

// V9: 文件格式选项
const fileFormatOptions = [
  { label: $t('page.finance.bankExport.fileFormat.txt'), value: 'txt' },
  { label: $t('page.finance.bankExport.fileFormat.xls'), value: 'xlsx' },
];

// ===== 生成结果弹窗 =====
const resultVisible = ref(false);
const resultData = ref<any>({});

// V9: 下载 blob 文件辅助函数
function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

async function handleGenerate() {
  if (!generateForm.bankType) {
    message.warning($t('page.finance.bankExport.message.bankTypeRequired'));
    return;
  }
  generateLoading.value = true;
  try {
    const params = {
      year: generateForm.year,
      month: generateForm.month,
      bankType: generateForm.bankType,
    };
    // V9: Excel 格式走新接口，直接下载二进制；TXT 格式走原接口，弹窗预览
    if (generateForm.fileFormat === 'xlsx') {
      const res: any = await generateBankExcelFileApi(params);
      const blob = res instanceof Blob
        ? res
        : new Blob([res as any], {
            type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
          });
      const fileName = `bank_${params.year}-${params.month}-${params.bankType}.xlsx`;
      downloadBlob(blob, fileName);
      message.success($t('page.finance.bankExport.modal.exportSuccess'));
      generateVisible.value = false;
      loadList();
    } else {
      const res: any = await generateBankFileApi(params);
      const data = res?.data || res || {};
      // 补充 bankType 用于结果弹窗展示（后端 GenerateResult 未返回该字段）
      if (!data.bankType) data.bankType = generateForm.bankType;
      resultData.value = data;
      generateVisible.value = false;
      resultVisible.value = true;
      loadList();
    }
  } catch (e: any) {
    message.error(e?.message || $t('page.finance.bankExport.message.generateFailed'));
  } finally {
    generateLoading.value = false;
  }
}

function handleDownload(id: number) {
  window.open(downloadBankFileApi(id), '_blank');
}

const columns: ColumnsType = [
  {
    title: $t('page.finance.bankExport.column.yearMonth'),
    key: 'yearMonth',
    width: 110,
    customRender: ({ record }: any) => `${record.year}-${record.month}`,
  },
  {
    title: $t('page.finance.bankExport.column.bankType'),
    dataIndex: 'bankType',
    width: 120,
    customRender: ({ text }: any) => bankTypeLabelMap[text] || text || '-',
  },
  { title: $t('page.finance.bankExport.column.fileName'), dataIndex: 'fileName', ellipsis: true },
  {
    title: $t('page.finance.bankExport.column.totalCount'),
    dataIndex: 'totalCount',
    width: 100,
    align: 'right',
  },
  {
    title: $t('page.finance.bankExport.column.totalAmount'),
    dataIndex: 'totalAmount',
    width: 130,
    align: 'right',
    customRender: ({ text }: any) => formatMoney(text),
  },
  {
    title: $t('page.finance.bankExport.column.status'),
    dataIndex: 'status',
    width: 100,
  },
  {
    title: $t('page.finance.bankExport.column.createTime'),
    dataIndex: 'createTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.finance.bankExport.column.operator'),
    dataIndex: 'creatorName',
    width: 100,
  },
  {
    title: $t('page.finance.common.action'),
    key: 'action',
    width: 100,
    fixed: 'right',
  },
];

onMounted(() => {
  loadList();
});
</script>

<template>
  <Page auto-content-height>
    <PageUsageGuide
      :title="$t('page.finance.bankExport.guide.title')"
      :brief="$t('page.finance.bankExport.guide.brief')"
      :expand-text="$t('page.finance.bankExport.guide.expand')"
      :collapse-text="$t('page.finance.bankExport.guide.collapse')"
    >
      <div
        v-for="i in guideStepCount"
        :key="i"
        class="page-guide-step-item"
      >
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.finance.bankExport.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.finance.bankExport.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <!-- 搜索栏 -->
    <Card class="mb-4" :bordered="false">
      <Form layout="inline">
        <FormItem :label="$t('page.finance.common.year')">
          <InputNumber
            v-model:value="searchForm.year"
            :min="2020"
            :max="2099"
            style="width: 120px"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.common.month')">
          <Select
            v-model:value="searchForm.month"
            :options="monthOptions"
            allow-clear
            :placeholder="$t('page.finance.common.all')"
            style="width: 120px"
          />
        </FormItem>
        <FormItem :label="$t('page.finance.bankExport.column.bankType')">
          <Select
            v-model:value="searchForm.bankType"
            :options="bankTypeOptions"
            allow-clear
            :placeholder="$t('page.finance.common.all')"
            style="width: 160px"
          />
        </FormItem>
        <FormItem>
          <Button type="primary" @click="loadList">{{ $t('page.finance.common.query') }}</Button>
        </FormItem>
      </Form>
    </Card>

    <Card :bordered="false" :title="$t('page.finance.bankExport.modal.title')">
      <template #extra>
        <Button
          v-if="accessStore.hasAccessCode('finance:bank-export:manage')"
          type="primary"
          @click="generateVisible = true"
        >
          {{ $t('page.finance.bankExport.button.generate') }}
        </Button>
      </template>

      <Table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        row-key="id"
        :pagination="{ pageSize: 20 }"
        size="middle"
        :scroll="{ x: 1100 }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'status'">
            <Tag :color="statusMap[record.status]?.color ?? 'default'">
              {{ statusMap[record.status]?.label ?? '-' }}
            </Tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <Button
              v-if="record.status === 1"
              type="link"
              size="small"
              @click="handleDownload(record.id)"
            >
              {{ $t('page.finance.bankExport.button.download') }}
            </Button>
            <span v-else class="text-gray-400">-</span>
          </template>
        </template>
        <template #emptyText>
          <Empty :description="$t('page.finance.bankExport.message.noData')" />
        </template>
      </Table>
    </Card>

    <!-- 生成代发文件弹窗 -->
    <Modal
      v-model:open="generateVisible"
      :title="$t('page.finance.bankExport.modal.generateTitle')"
      :confirm-loading="generateLoading"
      @ok="handleGenerate"
    >
      <div class="py-4">
        <Row :gutter="16">
          <Col :span="8">
            <FormItem :label="$t('page.finance.common.year')" required>
              <InputNumber
                v-model:value="generateForm.year"
                :min="2020"
                :max="2099"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.common.month')" required>
              <Select
                v-model:value="generateForm.month"
                :options="monthOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
          <Col :span="8">
            <FormItem :label="$t('page.finance.bankExport.modal.bankShort')" required>
              <Select
                v-model:value="generateForm.bankType"
                :options="bankTypeOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
        <Row :gutter="16">
          <Col :span="24">
            <FormItem :label="$t('page.finance.bankExport.modal.fileFormat')" required>
              <Select
                v-model:value="generateForm.fileFormat"
                :options="fileFormatOptions"
                style="width: 100%"
              />
            </FormItem>
          </Col>
        </Row>
      </div>
    </Modal>

    <!-- 生成结果弹窗 -->
    <Modal
      v-model:open="resultVisible"
      :title="$t('page.finance.bankExport.modal.resultTitle')"
      :footer="null"
      width="640px"
    >
      <div class="py-4">
        <Row :gutter="16" class="mb-4">
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.bankExport.column.totalCount')"
              :value="resultData.totalCount || 0"
            />
          </Col>
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.bankExport.column.totalAmount')"
              :value="formatMoney(resultData.totalAmount)"
              :value-style="{ color: '#1890ff' }"
            />
          </Col>
          <Col :span="8">
            <Statistic
              :title="$t('page.finance.bankExport.modal.bankShort')"
              :value="bankTypeLabelMap[resultData.bankType] || resultData.bankType || '-'"
            />
          </Col>
        </Row>

        <div class="mb-2 font-semibold">{{ $t('page.finance.bankExport.modal.previewTitle') }}</div>
        <Input.TextArea
          :value="resultData.fileContent || resultData.content || ''"
          :rows="12"
          readonly
          class="mb-4"
        />

        <div class="flex justify-end">
          <Button
            v-if="resultData.fileId || resultData.id"
            type="primary"
            @click="handleDownload(resultData.fileId || resultData.id)"
          >
            {{ $t('page.finance.bankExport.button.downloadFile') }}
          </Button>
        </div>
      </div>
    </Modal>
  </Page>
</template>
