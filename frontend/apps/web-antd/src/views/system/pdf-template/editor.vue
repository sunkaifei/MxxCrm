<script lang="ts" setup>
import { computed, reactive, ref, watch } from 'vue';

import {
  Button,
  Col,
  Drawer,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Row,
  Select,
  Switch,
} from 'ant-design-vue';

import {
  getPdfTemplateInfoApi,
  savePdfTemplateApi,
  updatePdfTemplateApi,
} from '#/api/core/system/pdf-template';
import CodeEditor from '#/components/CodeEditor/index.vue';
import { $t } from '#/locales';

const props = defineProps<{
  data?: any;
  defaultDocType?: string;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'saved'): void;
}>();

const isEdit = computed(() => !!props.data?.id);
const title = computed(() =>
  isEdit.value
    ? $t('ui.modal.update', {
        moduleName: $t('page.system.pdfTemplate.module'),
      })
    : $t('ui.modal.create', {
        moduleName: $t('page.system.pdfTemplate.module'),
      }),
);

const saving = ref(false);

// 单据类型选项
const docTypeOptions = [
  { label: $t('page.system.pdfTemplate.docTypeQuotation'), value: 'quotation' },
  { label: $t('page.system.pdfTemplate.docTypeOrder'), value: 'order' },
  { label: $t('page.system.pdfTemplate.docTypeContract'), value: 'contract' },
];

// 纸张大小选项
const paperSizeOptions = [
  { label: 'A4', value: 'A4' },
  { label: 'A3', value: 'A3' },
  { label: 'Letter', value: 'Letter' },
];

// 方向选项
const orientationOptions = [
  { label: $t('page.system.pdfTemplate.portrait'), value: 'portrait' },
  { label: $t('page.system.pdfTemplate.landscape'), value: 'landscape' },
];

// 字体选项
const fontOptions = [
  { label: '宋体 (SimSun)', value: 'SimSun' },
  { label: '黑体 (SimHei)', value: 'SimHei' },
  { label: '微软雅黑 (Microsoft YaHei)', value: 'Microsoft YaHei' },
  { label: 'Arial', value: 'Arial' },
  { label: 'Times New Roman', value: 'Times New Roman' },
  { label: 'Courier New', value: 'Courier New' },
];

const form = reactive({
  id: undefined as number | undefined,
  name: '',
  code: '',
  docType: 'quotation',
  paperSize: 'A4',
  orientation: 'portrait',
  marginTop: 20,
  marginBottom: 20,
  marginLeft: 20,
  marginRight: 20,
  font: 'SimSun',
  isDefault: 0,
  status: 1,
  sort: 0,
  remark: '',
  content: '',
  header: '',
  footer: '',
});

function resetForm() {
  Object.assign(form, {
    id: undefined,
    name: '',
    code: '',
    docType: props.defaultDocType || 'quotation',
    paperSize: 'A4',
    orientation: 'portrait',
    marginTop: 20,
    marginBottom: 20,
    marginLeft: 20,
    marginRight: 20,
    font: 'SimSun',
    isDefault: 0,
    status: 1,
    sort: 0,
    remark: '',
    content: '',
    header: '',
    footer: '',
  });
}

async function loadDetail(id: number) {
  try {
    const detail: any = await getPdfTemplateInfoApi(id);
    Object.assign(form, {
      id: detail.id,
      name: detail.name ?? '',
      code: detail.code ?? '',
      docType: detail.docType ?? 'quotation',
      paperSize: detail.paperSize ?? 'A4',
      orientation: detail.orientation ?? 'portrait',
      marginTop: detail.marginTop ?? 20,
      marginBottom: detail.marginBottom ?? 20,
      marginLeft: detail.marginLeft ?? 20,
      marginRight: detail.marginRight ?? 20,
      font: detail.font ?? 'SimSun',
      isDefault: detail.isDefault ?? 0,
      status: detail.status ?? 1,
      sort: detail.sort ?? 0,
      remark: detail.remark ?? '',
      content: detail.content ?? '',
      header: detail.header ?? '',
      footer: detail.footer ?? '',
    });
  } catch {
    // 错误由全局拦截器处理
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      if (props.data?.id) {
        loadDetail(props.data.id);
      } else {
        resetForm();
      }
    }
  },
);

function handleClose() {
  emit('update:visible', false);
}

async function handleSubmit() {
  if (!form.name) {
    message.warning($t('ui.formRules.required'));
    return;
  }
  if (!form.code) {
    message.warning($t('ui.formRules.required'));
    return;
  }
  saving.value = true;
  try {
    const payload = { ...form };
    if (isEdit.value) {
      await updatePdfTemplateApi(payload);
    } else {
      delete (payload as any).id;
      await savePdfTemplateApi(payload);
    }
    message.success(
      isEdit.value
        ? $t('ui.notification.update_success')
        : $t('ui.notification.create_success'),
    );
    emit('saved');
    handleClose();
  } catch {
    // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Drawer
    :open="visible"
    :title="title"
    :width="780"
    :destroy-on-close="true"
    :mask-closable="false"
    @close="handleClose"
  >
    <Form layout="vertical">
      <Row :gutter="16">
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.name')" required>
            <Input
              v-model:value="form.name"
              :placeholder="$t('ui.placeholder.input')"
              allow-clear
            />
          </FormItem>
        </Col>
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.code')" required>
            <Input
              v-model:value="form.code"
              :placeholder="$t('ui.placeholder.input')"
              allow-clear
            />
          </FormItem>
        </Col>
      </Row>

      <Row :gutter="16">
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.docType')">
            <Select
              v-model:value="form.docType"
              :options="docTypeOptions"
              :placeholder="$t('ui.placeholder.select')"
            />
          </FormItem>
        </Col>
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.paperSize')">
            <Select
              v-model:value="form.paperSize"
              :options="paperSizeOptions"
              :placeholder="$t('ui.placeholder.select')"
            />
          </FormItem>
        </Col>
      </Row>

      <Row :gutter="16">
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.orientation')">
            <Select
              v-model:value="form.orientation"
              :options="orientationOptions"
              :placeholder="$t('ui.placeholder.select')"
            />
          </FormItem>
        </Col>
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.font')">
            <Select
              v-model:value="form.font"
              :options="fontOptions"
              :placeholder="$t('ui.placeholder.select')"
              show-search
            />
          </FormItem>
        </Col>
      </Row>

      <Row :gutter="16">
        <Col :span="6">
          <FormItem :label="$t('page.system.pdfTemplate.marginTop')">
            <InputNumber
              v-model:value="form.marginTop"
              :min="0"
              :max="100"
              style="width: 100%"
            />
          </FormItem>
        </Col>
        <Col :span="6">
          <FormItem :label="$t('page.system.pdfTemplate.marginBottom')">
            <InputNumber
              v-model:value="form.marginBottom"
              :min="0"
              :max="100"
              style="width: 100%"
            />
          </FormItem>
        </Col>
        <Col :span="6">
          <FormItem :label="$t('page.system.pdfTemplate.marginLeft')">
            <InputNumber
              v-model:value="form.marginLeft"
              :min="0"
              :max="100"
              style="width: 100%"
            />
          </FormItem>
        </Col>
        <Col :span="6">
          <FormItem :label="$t('page.system.pdfTemplate.marginRight')">
            <InputNumber
              v-model:value="form.marginRight"
              :min="0"
              :max="100"
              style="width: 100%"
            />
          </FormItem>
        </Col>
      </Row>

      <Row :gutter="16">
        <Col :span="12">
          <FormItem :label="$t('page.system.pdfTemplate.sort')">
            <InputNumber
              v-model:value="form.sort"
              :min="0"
              style="width: 100%"
            />
          </FormItem>
        </Col>
        <Col :span="12">
          <FormItem :label="$t('ui.table.status')">
            <Switch
              v-model:checked="form.status"
              :checked-value="1"
              :un-checked-value="0"
              :checked-children="$t('ui.switch.active')"
              :un-checked-children="$t('ui.switch.inactive')"
            />
          </FormItem>
        </Col>
      </Row>

      <FormItem :label="$t('ui.table.remark')">
        <Textarea
          v-model:value="form.remark"
          :rows="2"
          :placeholder="$t('ui.placeholder.input')"
          allow-clear
        />
      </FormItem>

      <FormItem :label="$t('page.system.pdfTemplate.header')">
        <Textarea
          v-model:value="form.header"
          :rows="3"
          :placeholder="$t('ui.placeholder.input')"
          allow-clear
        />
      </FormItem>

      <FormItem :label="$t('page.system.pdfTemplate.footer')">
        <Textarea
          v-model:value="form.footer"
          :rows="3"
          :placeholder="$t('ui.placeholder.input')"
          allow-clear
        />
      </FormItem>

      <FormItem :label="$t('page.system.pdfTemplate.content')">
        <CodeEditor
          v-model="form.content"
          language="plaintext"
          height="400px"
        />
      </FormItem>
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button @click="handleClose">
          {{ $t('ui.button.cancel') }}
        </Button>
        <Button type="primary" :loading="saving" @click="handleSubmit">
          {{ $t('ui.button.ok') }}
        </Button>
      </div>
    </template>
  </Drawer>
</template>
