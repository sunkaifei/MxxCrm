<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, nextTick, onMounted, ref, watch } from 'vue';

import { LucideFilePenLine, LucidePlus, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';

import {
  Button,
  Drawer,
  Form,
  Input,
  InputNumber,
  message,
  Popconfirm,
  Select,
  Switch,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getProductListApi } from '#/api';
import {
  createAlertRuleApi,
  deleteAlertRuleApi,
  getAlertRuleInfoApi,
  getAlertRuleListApi,
  updateAlertRuleApi,
} from '#/api/core/product/alert';
import { $t } from '#/locales';

import UserSelectModal from '../../crm/components/UserSelectModal.vue';
import WarehouseSelectModal from '../inventory-check/WarehouseSelectModal.vue';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const accessStore = useAccessStore();

// 最大化
const isFullscreen = ref(false);
const drawerWidth = computed(() => (isFullscreen.value ? '100%' : '75%'));

// ============ 产品列表选项 ============
const productOptions = ref<{ label: string; value: number }[]>([]);

async function loadProductOptions() {
  try {
    const res: any = await getProductListApi({ page: 1, pageSize: 999 });
    const list = res?.list || res?.items || res || [];
    productOptions.value = list.map((p: any) => ({
      label: p.name || p.productName || '',
      value: Number(p.id),
    }));
  } catch {
    productOptions.value = [];
  }
}

// ============ 通知人管理 ============
interface NotifyUser {
  id: number;
  name: string;
}

const notifyUsers = ref<NotifyUser[]>([]);
const userSelectVisible = ref(false);

function openUserSelect() {
  userSelectVisible.value = true;
}

function onUserSelected(row: any) {
  const id = Number(row.id);
  const name = row.nickName || row.realName || row.userName || `用户${id}`;
  if (!notifyUsers.value.some((u) => u.id === id)) {
    notifyUsers.value.push({ id, name });
  }
  userSelectVisible.value = false;
}

function removeNotifyUser(id: number) {
  notifyUsers.value = notifyUsers.value.filter((u) => u.id !== id);
}

function getNotifyUserIdString(): string {
  return notifyUsers.value.map((u) => u.id).join(',');
}

// ============ 仓库弹窗选择 ============
const warehouseSelectVisible = ref(false);

function openWarehouseSelect() {
  warehouseSelectVisible.value = true;
}

function onWarehouseSelected(warehouse: any) {
  editForm.value.warehouseId = Number(warehouse.id);
  editForm.value.warehouseName =
    warehouse.warehouseName ?? warehouse.name ?? '';
}

function clearWarehouse() {
  editForm.value.warehouseId = undefined;
  editForm.value.warehouseName = '';
}

// ============ 列表 ============
const gridOptions: VxeGridProps = {
  height: 'auto',
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getAlertRuleListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          productName: formValues.productName,
          warehouseName: formValues.warehouseName,
        });
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: $t('page.product.inventory.alert.field.productName'),
      field: 'productName',
      minWidth: 140,
    },
    {
      title: $t('page.product.inventory.alert.field.warehouseName'),
      field: 'warehouseName',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.minQuantity'),
      field: 'minQuantity',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.maxQuantity'),
      field: 'maxQuantity',
      width: 120,
    },
    {
      title: $t('page.product.inventory.alert.field.staleDays'),
      field: 'staleDays',
      width: 100,
    },
    {
      title: $t('page.product.inventory.alert.field.enableLowAlert'),
      field: 'enableLowAlert',
      width: 100,
      slots: { default: 'enableLowAlert' },
    },
    {
      title: $t('page.product.inventory.alert.field.enableHighAlert'),
      field: 'enableHighAlert',
      width: 100,
      slots: { default: 'enableHighAlert' },
    },
    {
      title: $t('page.product.inventory.alert.field.enableStaleAlert'),
      field: 'enableStaleAlert',
      width: 100,
      slots: { default: 'enableStaleAlert' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      width: 140,
      fixed: 'right' as const,
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

// ============ 编辑/新增抽屉（叠加在规则管理之上） ============
const editDrawerVisible = ref(false);
const editDrawerTitle = ref('');
const isEdit = ref(false);
const submitLoading = ref(false);

const editForm = ref({
  id: undefined as number | undefined,
  productId: undefined as number | undefined,
  warehouseId: undefined as number | undefined,
  warehouseName: '',
  minQuantity: 0,
  maxQuantity: 0,
  staleDays: 90,
  enableLowAlert: true,
  enableHighAlert: false,
  enableStaleAlert: false,
});

function handleCreate() {
  isEdit.value = false;
  editDrawerTitle.value = $t('page.product.inventory.alert.action.create');
  editForm.value = {
    id: undefined,
    productId: undefined,
    warehouseId: undefined,
    warehouseName: '',
    minQuantity: 0,
    maxQuantity: 0,
    staleDays: 90,
    enableLowAlert: true,
    enableHighAlert: false,
    enableStaleAlert: false,
  };
  notifyUsers.value = [];
  editDrawerVisible.value = true;
}

async function handleEdit(row: any) {
  isEdit.value = true;
  editDrawerTitle.value = $t('page.product.inventory.alert.action.edit');
  try {
    const info = await getAlertRuleInfoApi(row.id);
    const data = (info as any)?.data ?? row;
    editForm.value = {
      id: data.id,
      productId: data.productId ? Number(data.productId) : undefined,
      warehouseId: data.warehouseId ? Number(data.warehouseId) : undefined,
      warehouseName: data.warehouseName || '',
      minQuantity: data.minQuantity ?? 0,
      maxQuantity: data.maxQuantity ?? 0,
      staleDays: data.staleDays ?? 90,
      enableLowAlert: data.enableLowAlert ?? true,
      enableHighAlert: data.enableHighAlert ?? false,
      enableStaleAlert: data.enableStaleAlert ?? false,
    };
    // 解析通知人
    notifyUsers.value = [];
    if (data.notifyUsers) {
      const ids = data.notifyUsers
        .split(',')
        .map((s: string) => Number(s.trim()))
        .filter((n: number) => n > 0);
      // 尝试加载用户名（简化处理：先显示ID，后续可以加载名称）
      for (const id of ids) {
        notifyUsers.value.push({ id, name: `用户${id}` });
      }
    }
  } catch {
    editForm.value = { ...row };
  }
  editDrawerVisible.value = true;
}

async function handleSubmit() {
  if (
    !editForm.value.enableLowAlert &&
    !editForm.value.enableHighAlert &&
    !editForm.value.enableStaleAlert
  ) {
    message.warning('请至少启用一种预警类型');
    return;
  }
  if (
    editForm.value.enableLowAlert &&
    (!editForm.value.minQuantity || editForm.value.minQuantity <= 0)
  ) {
    message.warning('启用低库存预警时需设置最低数量');
    return;
  }
  if (
    editForm.value.enableHighAlert &&
    (!editForm.value.maxQuantity || editForm.value.maxQuantity <= 0)
  ) {
    message.warning('启用高库存预警时需设置最高数量');
    return;
  }

  submitLoading.value = true;
  try {
    const payload = {
      productId: editForm.value.productId || undefined,
      warehouseId: editForm.value.warehouseId || undefined,
      minQuantity: editForm.value.minQuantity || undefined,
      maxQuantity: editForm.value.maxQuantity || undefined,
      staleDays: editForm.value.staleDays ?? 90,
      enableLowAlert: editForm.value.enableLowAlert,
      enableHighAlert: editForm.value.enableHighAlert,
      enableStaleAlert: editForm.value.enableStaleAlert,
      notifyUsers: getNotifyUserIdString() || undefined,
    };

    if (isEdit.value) {
      await updateAlertRuleApi({ ...payload, id: editForm.value.id });
      message.success($t('ui.notification.update_success'));
    } else {
      await createAlertRuleApi(payload);
      message.success($t('ui.notification.create_success'));
    }
    editDrawerVisible.value = false;
    gridApi.query();
  } finally {
    submitLoading.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteAlertRuleApi([row.id]);
    message.success($t('ui.notification.delete_success'));
    gridApi.query();
  } catch {
    // ignore
  }
}

// 弹窗打开时加载数据
watch(
  () => props.visible,
  (val) => {
    if (val) {
      isFullscreen.value = false;
      nextTick(() => {
        gridApi.query();
      });
    }
  },
);

onMounted(() => {
  loadProductOptions();
});
</script>

<template>
  <!-- 外层规则管理抽屉 -->
  <Drawer
    :open="visible"
    title="预警规则管理"
    :width="drawerWidth"
    placement="right"
    :mask-closable="true"
    @close="emit('update:visible', false)"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '还原' : '最大化'">
        <Button type="text" size="small" @click="isFullscreen = !isFullscreen">
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="15 3 21 3 21 9" />
            <polyline points="9 21 3 21 3 15" />
            <line x1="21" y1="3" x2="14" y2="10" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="4 14 10 14 10 20" />
            <polyline points="20 10 14 10 14 4" />
            <line x1="14" y1="10" x2="21" y2="3" />
            <line x1="3" y1="21" x2="10" y2="14" />
          </svg>
        </Button>
      </Tooltip>
    </template>

    <div style="height: 100%">
      <Grid>
        <template #toolbar-tools>
          <Button
            v-if="accessStore.hasAccessCode('product:alert:update')"
            type="primary"
            class="mr-2"
            :icon="h(LucidePlus)"
            @click="handleCreate"
          >
            {{ $t('page.product.inventory.alert.action.create') }}
          </Button>
        </template>

        <template #enableLowAlert="{ row }">
          <Tag :color="row.enableLowAlert ? 'green' : 'default'">
            {{ row.enableLowAlert ? $t('ui.enabled') : $t('ui.disabled') }}
          </Tag>
        </template>
        <template #enableHighAlert="{ row }">
          <Tag :color="row.enableHighAlert ? 'green' : 'default'">
            {{ row.enableHighAlert ? $t('ui.enabled') : $t('ui.disabled') }}
          </Tag>
        </template>
        <template #enableStaleAlert="{ row }">
          <Tag :color="row.enableStaleAlert ? 'green' : 'default'">
            {{ row.enableStaleAlert ? $t('ui.enabled') : $t('ui.disabled') }}
          </Tag>
        </template>

        <template #action="{ row }">
          <Button
            v-if="accessStore.hasAccessCode('product:alert:update')"
            type="link"
            :icon="h(LucideFilePenLine)"
            @click="() => handleEdit(row)"
          >
            {{ $t('page.product.inventory.alert.action.edit') }}
          </Button>
          <Popconfirm
            v-if="accessStore.hasAccessCode('product:alert:update')"
            :title="$t('ui.text.do_you_want_delete')"
            @confirm="() => handleDelete(row)"
          >
            <Button type="link" danger :icon="h(LucideTrash2)">
              {{ $t('page.product.inventory.alert.action.delete') }}
            </Button>
          </Popconfirm>
        </template>
      </Grid>
    </div>
  </Drawer>

  <!-- 添加/编辑规则抽屉（叠加在规则管理之上） -->
  <Drawer
    v-model:open="editDrawerVisible"
    :title="editDrawerTitle"
    :width="500"
    placement="right"
    :mask-closable="true"
    :style="{ position: 'absolute' }"
  >
    <Form layout="vertical">
      <Form.Item label="产品（不选=全部产品）">
        <Select
          v-model:value="editForm.productId"
          placeholder="全部产品（不选则对所有产品生效）"
          allow-clear
          show-search
          :options="productOptions"
          :filter-option="
            (input: string, option: any) =>
              (option?.label ?? '').toLowerCase().includes(input.toLowerCase())
          "
          style="width: 100%"
        />
      </Form.Item>
      <Form.Item label="仓库（不选=全部仓库）">
        <Input
          :value="editForm.warehouseName || ''"
          placeholder="全部仓库（不选则对所有仓库生效）"
          readonly
          allow-clear
          style="cursor: pointer"
          @click="openWarehouseSelect"
          @change="
            (e: any) => {
              if (!e?.target?.value) clearWarehouse();
            }
          "
        />
      </Form.Item>
      <Form.Item :label="$t('page.product.inventory.alert.field.minQuantity')">
        <InputNumber
          v-model:value="editForm.minQuantity"
          style="width: 100%"
          :min="0"
          placeholder="最低库存阈值"
        />
      </Form.Item>
      <Form.Item :label="$t('page.product.inventory.alert.field.maxQuantity')">
        <InputNumber
          v-model:value="editForm.maxQuantity"
          style="width: 100%"
          :min="0"
          placeholder="最高库存阈值"
        />
      </Form.Item>
      <Form.Item :label="$t('page.product.inventory.alert.field.staleDays')">
        <InputNumber
          v-model:value="editForm.staleDays"
          style="width: 100%"
          :min="0"
          placeholder="呆滞天数（默认90）"
        />
      </Form.Item>
      <Form.Item
        :label="$t('page.product.inventory.alert.field.enableLowAlert')"
      >
        <Switch v-model:checked="editForm.enableLowAlert" />
      </Form.Item>
      <Form.Item
        :label="$t('page.product.inventory.alert.field.enableHighAlert')"
      >
        <Switch v-model:checked="editForm.enableHighAlert" />
      </Form.Item>
      <Form.Item
        :label="$t('page.product.inventory.alert.field.enableStaleAlert')"
      >
        <Switch v-model:checked="editForm.enableStaleAlert" />
      </Form.Item>

      <!-- 指定通知人 -->
      <Form.Item label="指定通知人">
        <div
          style="
            display: flex;
            flex-wrap: wrap;
            gap: 4px;
            align-items: center;
            min-height: 32px;
            padding: 4px 8px;
            border: 1px solid #d9d9d9;
            border-radius: 6px;
          "
        >
          <Tag
            v-for="user in notifyUsers"
            :key="user.id"
            closable
            @close="removeNotifyUser(user.id)"
            style="margin: 2px"
          >
            {{ user.name }}
          </Tag>
          <span
            v-if="notifyUsers.length === 0"
            style="font-size: 13px; color: #bfbfbf"
            >暂无通知人</span
          >
          <Button
            type="link"
            size="small"
            :icon="h(LucidePlus)"
            @click="openUserSelect"
            style="margin-left: auto"
          >
            添加
          </Button>
        </div>
      </Form.Item>
    </Form>
    <template #footer>
      <div style="text-align: right">
        <Button class="mr-2" @click="editDrawerVisible = false">
          {{ $t('ui.button.cancel') }}
        </Button>
        <Button type="primary" :loading="submitLoading" @click="handleSubmit">
          {{ $t('ui.button.ok') }}
        </Button>
      </div>
    </template>
  </Drawer>

  <!-- 员工选择弹窗（复用CRM组件） -->
  <UserSelectModal
    v-model:visible="userSelectVisible"
    :exclude-ids="notifyUsers.map((u) => u.id)"
    @select="onUserSelected"
  />

  <!-- 仓库选择弹窗 -->
  <WarehouseSelectModal
    :visible="warehouseSelectVisible"
    @update:visible="(val) => (warehouseSelectVisible = val)"
    @select="onWarehouseSelected"
  />
</template>
