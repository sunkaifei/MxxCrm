<script lang="ts" setup>
// 发票详情抽屉：以真实票据为设计语言
// 票面（防伪线/撕票口）→ 税额计算链 → 购销双方 → 发票文件（财务审核后上传）→ 关联业务 → 备注
// 异常票据（作废/红冲）以"盖章"表达，已开票票据右下角有"发票专用章"
import { computed, ref } from 'vue';

import { LucideBuilding2, LucideCopy } from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Descriptions,
  DescriptionsItem,
  message,
  Popconfirm,
  Tag,
  Upload,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  deleteFileApi,
  downloadFileApi,
  getAttachmentsByEntityApi,
  getInvoiceInfoApi,
  uploadFileApi,
} from '#/api';

const loading = ref(false);
const detail = ref<any>({});

// ===== 发票文件（财务审核通过后上传的税控发票） =====
const attachments = ref<any[]>([]);
const uploading = ref(false);

// 附件实体类型：后端按此分目录 storage/upload/invoice/
const ENTITY_TYPE = 'invoice';

const invoiceId = computed(() => detail.value?.id);
// 简化处理：详情抽屉内均可管理附件（后端 attachment 接口有独立权限码控制）
const isSubordinateRow = ref(false);
// 审批通过后财务方可上传发票文件
const canUpload = computed(
  () => Number(detail.value?.approvalStatus) === 3 && !isSubordinateRow.value,
);

async function loadAttachments() {
  if (!invoiceId.value) return;
  try {
    const res: any = await getAttachmentsByEntityApi(
      ENTITY_TYPE,
      Number(invoiceId.value),
    );
    const list = Array.isArray(res) ? res : (res?.items ?? []);
    attachments.value = list;
  } catch {
    attachments.value = [];
  }
}

// 上传前钩子：手动上传（beforeUpload 返回 false 阻止自动上传）
async function handleUpload(file: any) {
  const rawFile: File = file.file ?? file;
  uploading.value = true;
  try {
    await uploadFileApi(rawFile, ENTITY_TYPE, Number(invoiceId.value));
    message.success('发票文件上传成功');
    // 上传后刷新附件与详情：后端在事务内回写 status 2→3（待开票→已开票，参考规则 4 / 验收 D2）
    await Promise.all([loadAttachments(), fetchDetail(Number(invoiceId.value))]);
  } catch (error: any) {
    message.error(error?.message || '上传失败');
  } finally {
    uploading.value = false;
    file.onSuccess?.({});
  }
  return false;
}

async function handleDownloadAttachment(item: any) {
  try {
    const blob: any = await downloadFileApi(item.id, 'download');
    const blobData = blob instanceof Blob ? blob : new Blob([blob]);
    const url = window.URL.createObjectURL(blobData);
    const link = document.createElement('a');
    link.href = url;
    link.download = item.name || item.fileName || '发票文件';
    document.body.append(link);
    link.click();
    link.remove();
    window.URL.revokeObjectURL(url);
  } catch (error: any) {
    message.error(error?.message || '下载失败');
  }
}

async function handleDeleteAttachment(item: any) {
  try {
    await deleteFileApi([item.id]);
    message.success('已删除');
    // 删除后刷新附件与详情：删除最后一个附件后后端回写 status 3→2（已开票→待开票，参考规则 4 / 验收 D3）
    await Promise.all([loadAttachments(), fetchDetail(Number(invoiceId.value))]);
  } catch (error: any) {
    message.error(error?.message || '删除失败');
  }
}

// ===== 字典映射（与列表页保持一致；状态：1=草稿、2=待开票、3=已开票、4=已作废、5=已红冲） =====
const statusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待开票',
  3: '已开票',
  4: '已作废',
  5: '已红冲',
};
const statusColorMap: Record<number, string> = {
  1: 'default',
  2: 'blue',
  3: 'green',
  4: 'red',
  5: 'magenta',
};
const typeLabelMap: Record<number, string> = {
  1: '增值税专用发票',
  2: '增值税普通发票',
  3: '形式发票(PI)',
  4: '商业发票(CI)',
};
// 票面左上角票种徽章字样
const badgeCharMap: Record<number, string> = {
  1: '专',
  2: '普',
  3: 'PI',
  4: 'CI',
};
const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};
const currencyLabelMap: Record<number, string> = {
  1: 'CNY 人民币',
  2: 'USD 美元',
  3: 'EUR 欧元',
  4: 'GBP 英镑',
  5: 'JPY 日元',
  6: 'HKD 港币',
};

// ===== 派生值 =====
const statusValue = computed(() => Number(detail.value.status) || 0);
const typeValue = computed(() => Number(detail.value.invoiceType) || 0);
const currencySymbol = computed(
  () => currencySymbolMap[Number(detail.value.currency)] || '¥',
);
const currencyCode = computed(
  () => currencyLabelMap[Number(detail.value.currency)] || 'CNY 人民币',
);

// 金额链条：不含税金额 → 税额 → 价税合计
const amountExcl = computed(() => Number(detail.value.amount || 0));
const taxAmountValue = computed(() => Number(detail.value.taxAmount || 0));
const totalAmount = computed(() => amountExcl.value + taxAmountValue.value);
const taxRateText = computed(() => {
  const rate = Number(detail.value.taxRate ?? 0);
  return rate > 0 ? `${rate}%` : '免税';
});

// 异常票据印章（作废/红冲）：盖在票面文字之上，模拟真实票据的盖章效果
const seal = computed(() => {
  if (statusValue.value === 4) {
    return { text: '作废', cls: 'invoice-detail__seal--void' };
  }
  if (statusValue.value === 5) {
    return { text: '红冲', cls: 'invoice-detail__seal--reverse' };
  }
  return null;
});

// 到期日预警：草稿/待开票状态下，逾期标红、临期标橙（作废/红冲不提示）
const dueTag = computed(() => {
  const due = detail.value.dueDate;
  if (
    !due ||
    statusValue.value === 0 ||
    statusValue.value === 4 ||
    statusValue.value === 5
  ) {
    return null;
  }
  const dueTime = new Date(`${due} 23:59:59`).getTime();
  if (Number.isNaN(dueTime)) return null;
  const diffDays = Math.ceil((dueTime - Date.now()) / 86_400_000);
  if (diffDays < 0) {
    return { text: `已逾期 ${Math.abs(diffDays)} 天`, color: 'red' };
  }
  if (diffDays <= 7) {
    return { text: `${diffDays} 天后到期`, color: 'orange' };
  }
  return null;
});

function formatMoney(val: number) {
  return val.toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

// 复制到剪贴板（http 非安全上下文下降级为 execCommand）
async function copyText(text?: string) {
  if (text === null || text === undefined || text === '') return;
  try {
    await navigator.clipboard.writeText(text);
    window.$message?.success('已复制到剪贴板');
    return;
  } catch {
    // Clipboard API 不可用时走降级方案
  }
  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.style.position = 'fixed';
  textarea.style.opacity = '0';
  document.body.append(textarea);
  textarea.select();
  const copied = document.execCommand('copy');
  textarea.remove();
  if (copied) window.$message?.success('已复制到剪贴板');
}

async function fetchDetail(id: number) {
  loading.value = true;
  try {
    const info = await getInvoiceInfoApi(id);
    detail.value = info ?? {};
  } catch {
    // 详情接口失败时退回行内数据，保证详情仍可读
    detail.value = rowBackup.value;
  } finally {
    loading.value = false;
  }
}

// 行数据兜底（详情接口失败时展示列表字段）
const rowBackup = ref<any>({});

const [Drawer, drawerApi] = useVbenDrawer({
  // 详情仅展示，隐藏底部确认/取消
  footer: false,
  onOpenChange(isOpen) {
    if (isOpen) {
      // connectedComponent 模式：数据经 setData 传入，须用 getData() 获取（而非 props）
      const data = drawerApi.getData() as any;
      const row = data?.row ?? {};
      rowBackup.value = row;
      detail.value = row;
      isSubordinateRow.value = !!data?.isSubordinate;
      attachments.value = [];
      if (row?.id !== null && row?.id !== undefined) {
        fetchDetail(Number(row.id));
        loadAttachments();
      }
    }
  },
});
</script>

<template>
  <Drawer
    title="发票详情"
    class="sale-invoice-detail-drawer"
    content-class="sale-invoice-detail-content"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <div class="invoice-detail">
      <!-- ===== 票面 ===== -->
      <section
        class="invoice-detail__paper"
        :class="{ 'invoice-detail__paper--void': statusValue === 4 }"
      >
        <!-- 异常票据印章：覆盖在票面文字之上 -->
        <div v-if="seal" class="invoice-detail__seal" :class="seal.cls">
          {{ seal.text }}
        </div>

        <div class="invoice-detail__paper-main">
          <div class="invoice-detail__head">
            <span class="invoice-detail__badge" :data-type="typeValue">
              {{ badgeCharMap[typeValue] || '票' }}
            </span>
            <div class="invoice-detail__head-text">
              <div class="invoice-detail__title-row">
                <h2 class="invoice-detail__title">
                  {{ detail.title || '未命名发票' }}
                </h2>
                <Tag
                  :color="statusColorMap[statusValue] || 'default'"
                  class="!m-0"
                >
                  {{ statusLabelMap[statusValue] || '未知' }}
                </Tag>
              </div>
              <div class="invoice-detail__no">
                <span class="invoice-detail__no-label">NO.</span>
                <span class="invoice-detail__no-value">
                  {{ detail.invoiceNo || '尚未分配票号' }}
                </span>
                <button
                  v-if="detail.invoiceNo"
                  type="button"
                  class="invoice-detail__copy"
                  title="复制发票号"
                  @click="copyText(detail.invoiceNo)"
                >
                  <LucideCopy />
                </button>
              </div>
            </div>
          </div>

          <div class="invoice-detail__paper-meta">
            <div class="invoice-detail__meta-item">
              <span class="invoice-detail__meta-label">开票日期</span>
              <span class="invoice-detail__meta-value">
                {{ detail.invoiceDate || '—' }}
              </span>
            </div>
            <div class="invoice-detail__meta-item">
              <span class="invoice-detail__meta-label">到期日</span>
              <span class="invoice-detail__meta-value">
                {{ detail.dueDate || '—' }}
              </span>
              <Tag v-if="dueTag" :color="dueTag.color" class="!m-0 !ml-1">
                {{ dueTag.text }}
              </Tag>
            </div>
            <div class="invoice-detail__meta-item">
              <span class="invoice-detail__meta-label">发票类型</span>
              <span class="invoice-detail__meta-value">
                {{ typeLabelMap[typeValue] || '—' }}
              </span>
            </div>
          </div>
        </div>

        <div class="invoice-detail__paper-total">
          <div class="invoice-detail__total-label">价税合计</div>
          <div class="invoice-detail__total-value">
            <span class="invoice-detail__total-symbol">
              {{ currencySymbol }}
            </span>
            <span class="invoice-detail__total-number">
              {{ formatMoney(totalAmount) }}
            </span>
          </div>
          <div class="invoice-detail__total-code">{{ currencyCode }}</div>
          <!-- 已开票：票面右下角的椭圆"发票专用章" -->
          <div
            v-if="statusValue === 3"
            class="invoice-detail__seal-mini"
            aria-hidden="true"
          >
            发票专用章
          </div>
        </div>
      </section>

      <!-- ===== 税额计算链 ===== -->
      <section class="invoice-detail__chain">
        <div class="invoice-detail__chain-item">
          <div class="invoice-detail__chain-label">金额（不含税）</div>
          <div class="invoice-detail__chain-value">
            {{ currencySymbol }}{{ formatMoney(amountExcl) }}
          </div>
        </div>
        <div class="invoice-detail__chain-item">
          <div class="invoice-detail__chain-label">税率</div>
          <div class="invoice-detail__chain-value">{{ taxRateText }}</div>
        </div>
        <div class="invoice-detail__chain-item">
          <div class="invoice-detail__chain-label">税额</div>
          <div class="invoice-detail__chain-value">
            {{ currencySymbol }}{{ formatMoney(taxAmountValue) }}
          </div>
        </div>
        <div
          class="invoice-detail__chain-item invoice-detail__chain-item--total"
        >
          <div class="invoice-detail__chain-label">价税合计</div>
          <div class="invoice-detail__chain-value">
            {{ currencySymbol }}{{ formatMoney(totalAmount) }}
          </div>
        </div>
      </section>

      <!-- ===== 购销双方 ===== -->
      <section class="invoice-detail__parties">
        <div class="invoice-detail__party">
          <div class="invoice-detail__party-head">
            <LucideBuilding2 />
            <span>购买方</span>
            <span class="invoice-detail__party-en">BUYER</span>
          </div>
          <div class="invoice-detail__party-row">
            <span class="invoice-detail__party-label">名称</span>
            <span class="invoice-detail__party-value">
              {{ detail.buyerName || detail.customerName || '—' }}
            </span>
          </div>
          <div class="invoice-detail__party-row">
            <span class="invoice-detail__party-label">税号</span>
            <span class="invoice-detail__party-value invoice-detail__mono">
              {{ detail.buyerTaxNo || '—' }}
            </span>
            <button
              v-if="detail.buyerTaxNo"
              type="button"
              class="invoice-detail__copy"
              title="复制税号"
              @click="copyText(detail.buyerTaxNo)"
            >
              <LucideCopy />
            </button>
          </div>
          <div class="invoice-detail__party-row">
            <span class="invoice-detail__party-label">开户行</span>
            <span class="invoice-detail__party-value">
              {{ detail.buyerBank || '—' }}
            </span>
          </div>
          <div class="invoice-detail__party-row">
            <span class="invoice-detail__party-label">地址电话</span>
            <span class="invoice-detail__party-value">
              {{ detail.buyerAddress || '—' }}
            </span>
          </div>
        </div>

        <div class="invoice-detail__party">
          <div
            class="invoice-detail__party-head invoice-detail__party-head--seller"
          >
            <LucideBuilding2 />
            <span>销售方</span>
            <span class="invoice-detail__party-en">SELLER</span>
          </div>
          <div class="invoice-detail__party-row">
            <span class="invoice-detail__party-label">名称</span>
            <span class="invoice-detail__party-value">—</span>
          </div>
          <div class="invoice-detail__party-row">
            <span class="invoice-detail__party-label">税号</span>
            <span class="invoice-detail__party-value invoice-detail__mono">
              {{ detail.taxNo || '—' }}
            </span>
            <button
              v-if="detail.taxNo"
              type="button"
              class="invoice-detail__copy"
              title="复制税号"
              @click="copyText(detail.taxNo)"
            >
              <LucideCopy />
            </button>
          </div>
        </div>
      </section>

      <!-- ===== 发票文件（财务审核通过后上传） ===== -->
      <Card size="small" class="invoice-detail__card">
        <template #title>
          <div class="invoice-detail__card-title">
            <span class="invoice-detail__card-bar"></span>发票文件
            <span class="invoice-detail__card-hint">
              财务审核通过后上传税控发票（PDF / OFD / 图片）
            </span>
          </div>
        </template>
        <div v-if="attachments.length > 0" class="invoice-detail__files">
          <div
            v-for="item in attachments"
            :key="item.id"
            class="invoice-detail__file"
          >
            <span class="invoice-detail__file-name" :title="item.name">
              {{ item.name || item.fileName }}
            </span>
            <span class="invoice-detail__file-size">
              {{
                item.fileSize ? `${Math.round(item.fileSize / 1024)} KB` : ''
              }}
            </span>
            <Button
              type="link"
              size="small"
              class="invoice-detail__file-btn"
              @click="handleDownloadAttachment(item)"
            >
              下载
            </Button>
            <Popconfirm
              title="确定删除该发票文件？"
              ok-text="删除"
              cancel-text="取消"
              @confirm="handleDeleteAttachment(item)"
            >
              <Button
                type="link"
                size="small"
                danger
                class="invoice-detail__file-btn"
              >
                删除
              </Button>
            </Popconfirm>
          </div>
        </div>
        <div v-else class="invoice-detail__files-empty">尚未上传发票文件</div>

        <Upload
          v-if="canUpload"
          :show-upload-list="false"
          :custom-request="handleUpload"
          accept=".pdf,.ofd,.jpg,.jpeg,.png"
        >
          <Button size="small" :loading="uploading">上传发票文件</Button>
        </Upload>
        <div v-else class="invoice-detail__files-tip">
          {{
            Number(detail.approvalStatus) === 3
              ? '无上传权限'
              : '财务审核通过后可上传发票文件'
          }}
        </div>
      </Card>

      <!-- ===== 关联业务 ===== -->
      <Card size="small" class="invoice-detail__card">
        <template #title>
          <div class="invoice-detail__card-title">
            <span class="invoice-detail__card-bar"></span>关联业务
          </div>
        </template>
        <Descriptions :column="2" size="small" :colon="false">
          <DescriptionsItem label="客户名称">
            {{ detail.customerName || '—' }}
          </DescriptionsItem>
          <DescriptionsItem label="负责人">
            {{ detail.ownerUserName || '—' }}
          </DescriptionsItem>
          <DescriptionsItem label="关联订单">
            {{ detail.orderId ? `#${detail.orderId}` : '—' }}
          </DescriptionsItem>
          <DescriptionsItem label="关联合同">
            {{ detail.contractId ? `#${detail.contractId}` : '—' }}
          </DescriptionsItem>
          <DescriptionsItem label="创建人">
            {{ detail.createBy || '—' }}
          </DescriptionsItem>
          <DescriptionsItem label="创建时间">
            {{ formatDateTime(detail.createTime) }}
          </DescriptionsItem>
        </Descriptions>
      </Card>

      <!-- ===== 备注 ===== -->
      <Card v-if="detail.remark" size="small" class="invoice-detail__card">
        <template #title>
          <div class="invoice-detail__card-title">
            <span class="invoice-detail__card-bar"></span>备注
          </div>
        </template>
        <p class="invoice-detail__remark">{{ detail.remark }}</p>
      </Card>
    </div>
  </Drawer>
</template>

<style>
/* 抽屉容器（非 scoped：content-class 挂在 Drawer 内部节点上） */
.sale-invoice-detail-drawer {
  width: 720px !important;
  max-width: 94vw;
}

.sale-invoice-detail-content {
  --invoice-bg: hsl(var(--background));
  background: hsl(var(--background)) !important;
  padding: 16px !important;
}
</style>

<style scoped>
/* ===== 设计 tokens：等宽数字（财务）+ 宋体标题（票据正式感） ===== */
.invoice-detail {
  --inv-mono:
    ui-monospace, 'Cascadia Mono', 'SF Mono', Consolas, 'Courier New', monospace;
  --inv-serif: 'Songti SC', 'STSong', SimSun, 'Noto Serif SC', serif;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* ===== 票面 ===== */
.invoice-detail__paper {
  position: relative;
  display: flex;
  gap: 20px;
  padding: 20px 22px;
  overflow: hidden;
  background:
    repeating-linear-gradient(
      90deg,
      hsl(var(--primary) / 0.03) 0 1px,
      transparent 1px 26px
    ),
    linear-gradient(165deg, hsl(var(--primary) / 0.06), transparent 55%),
    hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px 8px 0 0;
  border-bottom: 1px dashed hsl(var(--border) / 1.2);
  box-shadow:
    0 1px 2px rgb(0 0 0 / 4%),
    0 6px 16px -8px rgb(0 0 0 / 8%);
}

/* 撕票口：票面底边两端的半圆缺口 */
.invoice-detail__paper::before,
.invoice-detail__paper::after {
  position: absolute;
  bottom: -7px;
  width: 14px;
  height: 14px;
  content: '';
  background: var(--invoice-bg, hsl(var(--background)));
  border-radius: 50%;
  box-shadow: inset 0 2px 3px rgb(0 0 0 / 7%);
}

.invoice-detail__paper::before {
  left: 14px;
}

.invoice-detail__paper::after {
  right: 14px;
}

/* 作废票据：票面整体褪色 */
.invoice-detail__paper--void {
  filter: grayscale(0.6);
}

.invoice-detail__paper-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  justify-content: space-between;
  gap: 14px;
  min-width: 0;
}

/* 票头：徽章 + 标题 + 票号 */
.invoice-detail__head {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.invoice-detail__badge {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  font-family: var(--inv-serif);
  font-size: 17px;
  font-weight: 700;
  border: 1px solid;
  border-radius: 6px;
}

.invoice-detail__badge[data-type='1'] {
  color: hsl(212 100% 40%);
  background: hsl(212 100% 40% / 0.09);
  border-color: hsl(212 100% 40% / 0.3);
}

.invoice-detail__badge[data-type='2'] {
  color: hsl(187 85% 33%);
  background: hsl(187 85% 33% / 0.09);
  border-color: hsl(187 85% 33% / 0.3);
}

.invoice-detail__badge[data-type='3'] {
  color: hsl(28 92% 42%);
  background: hsl(28 92% 42% / 0.1);
  border-color: hsl(28 92% 42% / 0.3);
}

.invoice-detail__badge[data-type='4'] {
  color: hsl(271 60% 48%);
  background: hsl(271 60% 48% / 0.09);
  border-color: hsl(271 60% 48% / 0.3);
}

.invoice-detail__badge[data-type='0'] {
  color: hsl(var(--foreground) / 60%);
  background: hsl(var(--muted));
  border-color: hsl(var(--border));
}

.invoice-detail__head-text {
  flex: 1;
  min-width: 0;
}

.invoice-detail__title-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.invoice-detail__title {
  overflow: hidden;
  font-family: var(--inv-serif);
  font-size: 17px;
  font-weight: 600;
  line-height: 1.4;
  color: hsl(var(--foreground));
  text-overflow: ellipsis;
  white-space: nowrap;
}

.invoice-detail__no {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-top: 4px;
}

.invoice-detail__no-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 1px;
  color: hsl(var(--foreground) / 38%);
}

.invoice-detail__no-value {
  font-family: var(--inv-mono);
  font-size: 13px;
  letter-spacing: 0.5px;
  color: hsl(var(--foreground) / 75%);
}

/* 票面 meta 行 */
.invoice-detail__paper-meta {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.invoice-detail__meta-item {
  display: flex;
  gap: 6px;
  align-items: center;
}

.invoice-detail__meta-label {
  font-size: 12px;
  color: hsl(var(--foreground) / 45%);
}

.invoice-detail__meta-value {
  font-family: var(--inv-mono);
  font-size: 12.5px;
  color: hsl(var(--foreground) / 85%);
}

/* 价税合计（票面右栏） */
.invoice-detail__paper-total {
  position: relative;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  gap: 2px;
  align-items: flex-end;
  justify-content: center;
  padding-left: 20px;
  border-left: 1px solid hsl(var(--border) / 0.8);
}

.invoice-detail__total-label {
  font-size: 11px;
  letter-spacing: 2px;
  color: hsl(var(--foreground) / 45%);
}

.invoice-detail__total-value {
  display: flex;
  gap: 2px;
  align-items: baseline;
  color: hsl(var(--foreground));
}

.invoice-detail__total-symbol {
  font-size: 15px;
  font-weight: 600;
}

.invoice-detail__total-number {
  font-family: var(--inv-mono);
  font-size: 27px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.5px;
}

.invoice-detail__total-code {
  font-size: 11px;
  color: hsl(var(--foreground) / 40%);
}

/* ===== 印章 ===== */
/* 异常票据大圆章：双环 + 旋转，覆盖票面文字（pointer-events 关闭） */
.invoice-detail__seal {
  position: absolute;
  top: 22px;
  left: 42%;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 92px;
  height: 92px;
  font-family: var(--inv-serif);
  font-size: 27px;
  font-weight: 800;
  letter-spacing: 7px;
  text-indent: 7px;
  pointer-events: none;
  opacity: 0.82;
  border: 3px solid currentColor;
  border-radius: 50%;
  outline: 1px solid currentColor;
  outline-offset: -9px;
  transform: rotate(-14deg);
  mix-blend-mode: multiply;
}

:global(.dark) .invoice-detail__seal {
  mix-blend-mode: normal;
}

.invoice-detail__seal--void {
  color: hsl(354 74% 44%);
}

.invoice-detail__seal--reverse {
  color: hsl(271 62% 46%);
}

/* 已开票：右下角椭圆"发票专用章" */
.invoice-detail__seal-mini {
  margin-top: 10px;
  padding: 3px 14px;
  font-family: var(--inv-serif);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 3px;
  text-indent: 3px;
  color: hsl(13 78% 42% / 0.85);
  pointer-events: none;
  border: 1.5px solid currentColor;
  border-radius: 50%;
  outline: 1px solid currentColor;
  outline-offset: 2px;
  transform: rotate(-8deg);
}

/* ===== 税额计算链 ===== */
.invoice-detail__chain {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  overflow: hidden;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.invoice-detail__chain-item {
  padding: 12px 16px;
  border-left: 1px solid hsl(var(--border) / 0.7);
}

.invoice-detail__chain-item:first-child {
  border-left: none;
}

.invoice-detail__chain-label {
  margin-bottom: 4px;
  font-size: 11px;
  color: hsl(var(--foreground) / 45%);
}

.invoice-detail__chain-value {
  font-family: var(--inv-mono);
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: hsl(var(--foreground) / 90%);
}

.invoice-detail__chain-item--total {
  background: hsl(var(--primary) / 0.07);
}

.invoice-detail__chain-item--total .invoice-detail__chain-label {
  color: hsl(var(--primary) / 75%);
}

.invoice-detail__chain-item--total .invoice-detail__chain-value {
  color: hsl(var(--primary));
}

/* ===== 购销双方 ===== */
.invoice-detail__parties {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.invoice-detail__party {
  padding: 14px 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.invoice-detail__party-head {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 10px;
  font-size: 13px;
  font-weight: 600;
  color: hsl(212 100% 40%);
}

.invoice-detail__party-head :deep(svg) {
  width: 14px;
  height: 14px;
}

.invoice-detail__party-head--seller {
  color: hsl(187 85% 33%);
}

.invoice-detail__party-en {
  margin-left: auto;
  font-size: 10px;
  font-weight: 500;
  letter-spacing: 1.5px;
  color: hsl(var(--foreground) / 28%);
}

.invoice-detail__party-row {
  display: flex;
  gap: 8px;
  align-items: baseline;
  padding: 4px 0;
}

.invoice-detail__party-label {
  flex-shrink: 0;
  width: 52px;
  font-size: 12px;
  color: hsl(var(--foreground) / 45%);
}

.invoice-detail__party-value {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  word-break: break-all;
  color: hsl(var(--foreground) / 88%);
}

.invoice-detail__mono {
  font-family: var(--inv-mono);
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.3px;
}

/* 复制按钮：默认隐藏，行 hover 时浮现 */
.invoice-detail__copy {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 4px;
  opacity: 0;
  transition: all 0.15s;
}

.invoice-detail__copy :deep(svg) {
  width: 12px;
  height: 12px;
  color: hsl(var(--foreground) / 45%);
}

.invoice-detail__party-row:hover .invoice-detail__copy,
.invoice-detail__no:hover .invoice-detail__copy {
  opacity: 1;
}

.invoice-detail__copy:hover {
  background: hsl(var(--accent));
  opacity: 1;
}

.invoice-detail__copy:hover :deep(svg) {
  color: hsl(var(--primary));
}

/* ===== 卡片 ===== */
.invoice-detail__card {
  border-radius: 8px;
}

.invoice-detail__card-title {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 13px;
}

.invoice-detail__card-bar {
  width: 3px;
  height: 12px;
  background: hsl(var(--primary));
  border-radius: 2px;
}

.invoice-detail__card-hint {
  font-size: 11px;
  font-weight: 400;
  color: hsl(var(--foreground) / 40%);
}

/* ===== 发票文件 ===== */
.invoice-detail__files {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
}

.invoice-detail__file {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 6px 10px;
  background: hsl(var(--muted) / 50%);
  border-radius: 6px;
}

.invoice-detail__file-name {
  flex: 1;
  overflow: hidden;
  min-width: 0;
  font-size: 13px;
  color: hsl(var(--foreground) / 88%);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.invoice-detail__file-size {
  flex-shrink: 0;
  font-size: 11px;
  color: hsl(var(--foreground) / 40%);
}

.invoice-detail__file-btn {
  flex-shrink: 0;
  padding: 0 4px;
}

.invoice-detail__files-empty {
  margin-bottom: 10px;
  font-size: 12.5px;
  color: hsl(var(--foreground) / 40%);
}

.invoice-detail__files-tip {
  margin-top: 6px;
  font-size: 12px;
  color: hsl(var(--foreground) / 40%);
}

.invoice-detail__remark {
  margin: 0;
  font-size: 13px;
  line-height: 1.7;
  color: hsl(var(--foreground) / 80%);
  white-space: pre-wrap;
}
</style>
