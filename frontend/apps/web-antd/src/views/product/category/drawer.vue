<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { Form, message, Upload } from 'ant-design-vue';
import { LucideUpload, LucideX } from '@vben/icons';
import { createCategoryApi, getCategoryListApi, updateCategoryApi, uploadCategoryImageApi } from '#/api';

const data = ref();
const imageUrl = ref('');
const uploading = ref(false);
const parentOptions = ref<{ value: string; label: string }[]>([
  { value: '0', label: '根目录' },
]);
const currentId = ref<string | null>(null);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.product.category.title') })
    : $t('ui.modal.update', { moduleName: $t('page.product.category.title') }),
);

/** 加载可选上级分类（排除自身和子级） */
async function loadParentOptions(excludeId?: string | null) {
  try {
    const resp = await getCategoryListApi({ page: 1, pageSize: 999 });
    const list = (resp as any)?.items || (resp as any)?.rows || [];
    const items = list
      .filter((c: any) => String(c.id) !== String(excludeId))
      .map((c: any) => ({
        value: String(c.id),
        label: c.name || '',
      }));
    parentOptions.value = [{ value: '0', label: '根目录' }, ...items];
  } catch {
    parentOptions.value = [{ value: '0', label: '根目录' }];
  }
}

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '分类名称',
      rules: 'required',
      componentProps: {
        placeholder: '请输入分类名称',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'parentId',
      label: '上级分类',
      componentProps: {
        placeholder: '请选择上级分类',
        allowClear: true,
        options: parentOptions,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sortOrder',
      label: '排序号',
      defaultValue: 0,
      componentProps: {
        placeholder: '排序号（数字越小越靠前）',
        class: 'w-full',
        min: 0,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '描述',
      componentProps: {
        placeholder: '分类描述（可选）',
        rows: 3,
        allowClear: true,
      },
    },
  ],
});

/** 上传图片 */
async function handleUpload(file: File) {
  uploading.value = true;
  try {
    const url = await uploadCategoryImageApi(file);
    if (url) {
      imageUrl.value = url;
      message.success('图片上传成功');
    } else {
      message.error('图片上传失败：返回地址为空');
    }
  } catch (e: any) {
    message.error(`图片上传失败: ${e?.message || '未知错误'}`);
  } finally {
    uploading.value = false;
  }
  return false; // 阻止默认上传行为
}

function handleRemoveImage() {
  imageUrl.value = '';
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;

    setLoading(true);
    const values = await baseFormApi.getValues();

    // parentId = "0" 视为 null（根目录）
    const parentId = values.parentId && String(values.parentId) !== '0'
      ? values.parentId
      : null;

    const payload = {
      ...values,
      parentId,
      image: imageUrl.value || null,
    };

    try {
      await (data.value?.create
        ? createCategoryApi(payload)
        : updateCategoryApi({ ...payload, id: data.value.row.id }));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } catch (e: any) {
      message.error(`操作失败: ${e?.message || ''}`);
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      currentId.value = row?.id || null;

      // 加载上级分类选项
      loadParentOptions(currentId.value);

      // 设置表单值
      baseFormApi.setValues({
        name: row.name || '',
        parentId: row.parentId ? String(row.parentId) : '0',
        sortOrder: row.sortOrder ?? 0,
        description: row.description || '',
      });
      imageUrl.value = row.image || '';
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle" class="category-drawer">
    <div class="flex flex-col gap-5 px-1">
      <!-- 表单字段 -->
      <BaseForm />

      <!-- 分类图片上传 -->
      <Form.Item label="分类图片" class="mb-0">
        <div class="flex items-start gap-3">
          <!-- 图片预览 -->
          <div
            v-if="imageUrl"
            class="relative w-20 h-20 rounded-lg border border-gray-200 overflow-hidden flex-shrink-0 group"
          >
            <img :src="imageUrl" alt="分类图片" class="w-full h-full object-cover" />
            <div
              class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
              @click="handleRemoveImage"
            >
              <LucideX class="text-white w-4 h-4" />
            </div>
          </div>
          <!-- 上传区域 -->
          <Upload
            :show-upload-list="false"
            :before-upload="handleUpload"
            :disabled="uploading"
            accept="image/png,image/jpeg,image/jpg,image/gif,image/webp"
          >
            <div
              class="flex flex-col items-center justify-center w-20 h-20 rounded-lg border-2 border-dashed border-gray-300 hover:border-blue-400 cursor-pointer transition-colors"
              :class="{ 'opacity-50 cursor-not-allowed': uploading }"
            >
              <LucideUpload v-if="!uploading" class="w-4 h-4 text-gray-400" />
              <span v-if="!uploading" class="text-xs text-gray-400 mt-0.5">上传图片</span>
              <span v-else class="text-xs text-gray-400">上传中...</span>
            </div>
          </Upload>
          <div class="text-xs text-gray-400 leading-relaxed">
            <p>支持 JPG/PNG/GIF/WebP</p>
            <p>建议尺寸 200x200px</p>
          </div>
        </div>
      </Form.Item>
    </div>
  </Drawer>
</template>

<style scoped>
.category-drawer :deep(.ant-upload-select) {
  display: block !important;
}
</style>
