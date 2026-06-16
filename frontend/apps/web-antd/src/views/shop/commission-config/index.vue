<script lang="ts" setup>
import { ref, reactive, onMounted } from 'vue';
import { Page } from '@vben/common-ui';
import {
  Button,
  Form,
  Input,
  message,
  Card,
  Table,
  Space,
} from 'ant-design-vue';
import { commissionApi } from '#/api';

const loading = ref(false);

const formData = reactive({
  defaultRate: 5,
  shopRates: [] as Array<{ shopId: number; shopName: string; rate: number }>,
});

const tableColumns = [
  {
    title: '店铺ID',
    dataIndex: 'shopId',
    key: 'shopId',
    width: 100,
  },
  {
    title: '店铺名称',
    dataIndex: 'shopName',
    key: 'shopName',
  },
  {
    title: '佣金比例(%)',
    dataIndex: 'rate',
    key: 'rate',
    width: 120,
  },
  {
    title: '操作',
    key: 'action',
    width: 80,
  },
];

function addShopRate() {
  formData.shopRates.push({ shopId: 0, shopName: '', rate: 5 });
}

function removeShopRate(index: number) {
  formData.shopRates.splice(index, 1);
}

async function loadConfig() {
  loading.value = true;
  try {
    const config = await commissionApi.getConfig();
    Object.assign(formData, config);
  } catch (e) {
    console.error('Load config error:', e);
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  try {
    await commissionApi.saveConfig({
      defaultRate: formData.defaultRate,
      shopRates: formData.shopRates.filter(
        (item) => item.shopId && item.shopName,
      ),
    });
    message.success('保存成功');
  } catch (e) {
    console.error('Save config error:', e);
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <Page auto-content-height>
    <Card title="佣金设置" :loading="loading">
      <Form
        :model="formData"
        :label-col="{ span: 8 }"
        :wrapper-col="{ span: 16 }"
      >
        <Form.Item label="默认佣金比例(%)" name="defaultRate" required>
          <Input
            type="number"
            v-model="formData.defaultRate"
            :min="0"
            :max="100"
            placeholder="请输入默认佣金比例"
          />
          <span class="ml-2 text-gray-500 text-sm"
            >所有未设置单独佣金的店铺将使用此比例</span
          >
        </Form.Item>

        <Form.Item label="店铺单独佣金" name="shopRates">
          <div>
            <Space class="mb-4">
              <Button type="primary" @click="addShopRate">添加店铺佣金</Button>
            </Space>
            <Table
              :data-source="formData.shopRates"
              :columns="tableColumns"
              row-key="shopId"
              :pagination="false"
              bordered
            >
              <template #bodyCell="{ column, record, index }">
                <template v-if="column.dataIndex === 'shopId'">
                  <Input
                    type="number"
                    v-model="record.shopId"
                    placeholder="店铺ID"
                  />
                </template>
                <template v-else-if="column.dataIndex === 'shopName'">
                  <Input v-model="record.shopName" placeholder="店铺名称" />
                </template>
                <template v-else-if="column.dataIndex === 'rate'">
                  <Input
                    type="number"
                    v-model="record.rate"
                    :min="0"
                    :max="100"
                  />
                </template>
                <template v-else-if="column.key === 'action'">
                  <Button type="link" danger @click="removeShopRate(index)"
                    >删除</Button
                  >
                </template>
              </template>
            </Table>
            <p class="text-gray-500 text-sm mt-2">
              设置店铺单独佣金后，该店铺的商品将使用此比例计算佣金，优先级高于默认佣金
            </p>
          </div>
        </Form.Item>

        <Form.Item :wrapper-col="{ span: 16 }">
          <Button type="primary" @click="handleSave">保存设置</Button>
        </Form.Item>
      </Form>
    </Card>
  </Page>
</template>
