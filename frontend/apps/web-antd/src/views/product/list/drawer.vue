<script lang="ts" setup>
import { computed, ref, defineAsyncComponent, watch, onMounted } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { message, Tabs, Radio, Upload, Input, InputNumber, Select, Switch } from 'ant-design-vue';
import { LucideUpload, LucideX, LucideMaximize2 } from '@vben/icons';
import {
  createProductApi,
  getCategoryListApi,
  getProductInfoApi,
  updateProductApi,
  uploadCategoryImageApi,
  getSkuTemplateListApi,
  getSkuTemplateInfoApi,
} from '#/api';
import { getProductSpecsApi } from '#/api/core/product/spec';
import type { SkuTemplateListVO } from '#/api';
const data = ref();
const activeTab = ref('basic');
const isFullscreen = ref(false);

// SKU 表格滚动状态
const skuWrapperRef = ref<HTMLElement | null>(null);
const isLeftScrolled = ref(false);
const isRightScrolled = ref(false);

function handleSkuScroll() {
  const el = skuWrapperRef.value;
  if (!el) return;
  isLeftScrolled.value = el.scrollLeft > 0;
  isRightScrolled.value = el.scrollLeft + el.clientWidth < el.scrollWidth;
}

onMounted(() => {
  // 初始检测滚动状态
  setTimeout(() => handleSkuScroll(), 100);
});

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
  drawerApi.setState({
    class: ['product-drawer', isFullscreen.value ? 'drawer-fullscreen' : 'drawer-75'],
  });
}

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.product.list.title') })
    : $t('ui.modal.update', { moduleName: $t('page.product.list.title') }),
);

const formData = ref({
  name: '',
  productNo: '',
  categoryId: undefined as number | undefined,
  keywords: '',
  unit: '',
  barcode: '',
  salePrice: 0,
  marketPrice: 0,
  costPrice: 0,
  stock: 0,
  weight: undefined as number | undefined,
  dimensions: '',
  isActive: true,
  detail: '',
});

const categoryOptions = ref<Array<{ value: number; label: string }>>([]);

// Lazy loaded wangeditor editor
const ProductEditor = defineAsyncComponent(() => import('./product-editor.vue'));

const specType = ref<'single' | 'multiple'>('single');

// 多规格模板选择
const selectedTemplateId = ref<number | undefined>(undefined);
const templateOptions = ref<Array<{ value: number; label: string }>>([]);
const templateSpecs = ref<Array<{ name: string; values: string[] }>>([]);

const specList = ref<Array<{
  _key: number;
  id?: number | string;
  label: string;
  imageUrl: string;
  price: number;
  costPrice: number;
  originalPrice: number;
  stock: number;
  skuCode: string;
  weight: number;
  volume: number;
  isDefault: boolean;
  isActive: boolean;
}>>([{
  _key: 1,
  label: '',
  imageUrl: '',
  price: 0,
  costPrice: 0,
  originalPrice: 0,
  stock: 0,
  skuCode: '',
  weight: 0,
  volume: 0,
  isDefault: false,
  isActive: true,
}]);

let specKeyCounter = 1;



async function handleSpecImageUpload(file: File, specItem: any) {
  try {
    const url = await uploadCategoryImageApi(file);
    if (url) {
      specItem.imageUrl = url;
      message.success('图片上传成功');
    } else {
      message.error('图片上传失败');
    }
  } catch (e: any) {
    message.error(`图片上传失败: ${e?.message || '未知错误'}`);
  }
  return false;
}

function removeSpecImage(specItem: any) {
  specItem.imageUrl = '';
}

/** 默认选中规格：单选逻辑，只能有一个默认 */
function handleDefaultChange(spec: any, checked: boolean) {
  if (!checked) return; // 不允许取消唯一默认
  specList.value.forEach((s) => {
    s.isDefault = s._key === spec._key;
  });
}

// ============= 多规格模板 =============

/** 加载模板下拉列表 */
async function loadTemplateOptions() {
  try {
    const res = await getSkuTemplateListApi({ page: 1, pageSize: 999 });
    const list = (res as any)?.items || (res as any)?.rows || [];
    templateOptions.value = list.map((t: SkuTemplateListVO) => ({
      value: Number(t.id),
      label: t.templateName,
    }));
  } catch {
    // ignore
  }
}

/** 选择模板后加载规格并生成组合 */
async function handleTemplateChange(templateId: any) {
  const id = Number(templateId);
  if (!id) {
    templateSpecs.value = [];
    specList.value = [];
    return;
  }
  try {
    const res = await getSkuTemplateInfoApi(id);
    const detail = (res as any)?.data ?? res;
    const specs: Array<{ name: string; values: string[] }> = [];
    if (detail?.specs && Array.isArray(detail.specs)) {
      for (const s of detail.specs) {
        specs.push({
          name: s.name || '',
          values: (s.values || []).map((v: any) => v.value).filter(Boolean),
        });
      }
    }
    templateSpecs.value = specs;
    generateSpecCombinations(specs);
  } catch {
    message.error('加载模板规格失败');
  }
}

/** 根据规格定义生成笛卡尔积组合 */
function generateSpecCombinations(specs: Array<{ name: string; values: string[] }>) {
  if (specs.length === 0) {
    specList.value = [];
    return;
  }

  // 笛卡尔积
  let combinations: string[][] = [[]];
  for (const spec of specs) {
    const vals = spec.values.length > 0 ? spec.values : [''];
    const newCombos: string[][] = [];
    for (const combo of combinations) {
      for (const v of vals) {
        newCombos.push([...combo, v]);
      }
    }
    combinations = newCombos;
  }

  specKeyCounter = 0;
  specList.value = combinations.map((combo, index) => {
    specKeyCounter++;
    return {
      _key: specKeyCounter,
      label: combo.join(' / '),
      imageUrl: '',
      price: 0,
      costPrice: 0,
      originalPrice: 0,
      stock: 0,
      skuCode: '',
      weight: 0,
      volume: 0,
      isDefault: index === 0,
      isActive: true,
    };
  });
}

// 切换规格类型时重置
watch(specType, (val) => {
  if (val === 'single') {
    selectedTemplateId.value = undefined;
    templateSpecs.value = [];
    specList.value = [{
      _key: 1,
      label: '',
      imageUrl: '',
      price: 0,
      costPrice: 0,
      originalPrice: 0,
      stock: 0,
      skuCode: '',
      weight: 0,
      volume: 0,
      isDefault: false,
      isActive: true,
    }];
  } else {
    specList.value = [];
    selectedTemplateId.value = undefined;
    templateSpecs.value = [];
  }
});

const coverImageUrl = ref('');

async function handleCoverUpload(file: File) {
  try {
    const url = await uploadCategoryImageApi(file);
    if (url) {
      coverImageUrl.value = url;
      message.success('封面图上传成功');
    } else {
      message.error('封面图上传失败');
    }
  } catch (e: any) {
    message.error(`封面图上传失败: ${e?.message || '未知错误'}`);
  }
  return false;
}

function removeCoverImage() {
  coverImageUrl.value = '';
}

const carouselImages = ref<Array<{ url: string; uid: number }>>([]);
let carouselKeyCounter = 0;

async function handleCarouselUpload(file: File) {
  try {
    const url = await uploadCategoryImageApi(file);
    if (url) {
      carouselKeyCounter++;
      carouselImages.value.push({ url, uid: carouselKeyCounter });
      message.success('轮播图上传成功');
    } else {
      message.error('轮播图上传失败');
    }
  } catch (e: any) {
    message.error(`轮播图上传失败: ${e?.message || '未知错误'}`);
  }
  return false;
}

function removeCarouselImage(uid: number) {
  const idx = carouselImages.value.findIndex((img) => img.uid === uid);
  if (idx >= 0) {
    carouselImages.value.splice(idx, 1);
  }
}

function validateForm(): boolean {
  if (!formData.value.name || !formData.value.name.trim()) {
    message.error('请输入商品名称');
    return false;
  }
  if (!formData.value.categoryId) {
    message.error('请选择商品分类');
    return false;
  }
  return true;
}

const [Drawer, drawerApi] = useVbenDrawer({
  class: ['product-drawer', 'drawer-75'],
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    if (!validateForm()) {
      return;
    }

    setLoading(true);

    const specs = specList.value.map((s) => {
      let specsValue: Record<string, string> | null = null;
      if (specType.value === 'multiple' && s.label && templateSpecs.value.length > 0) {
        const values = s.label.split(' / ');
        const specsObj: Record<string, string> = {};
        templateSpecs.value.forEach((spec, index) => {
          if (values[index]) {
            specsObj[spec.name] = values[index];
          }
        });
        specsValue = specsObj;
      }
      return {
        id: s.id,
        specs: specsValue,
        imageUrl: s.imageUrl || null,
        price: s.price,
        costPrice: s.costPrice,
        originalPrice: s.originalPrice,
        stock: s.stock,
        skuCode: s.skuCode,
        weight: s.weight,
        volume: s.volume,
        isDefault: s.isDefault,
        isActive: s.isActive,
      };
    });

    const payload = {
      ...formData.value,
      currency: 'CNY',
      imageUrl: coverImageUrl.value || null,
      carouselImages: carouselImages.value.map((img) => img.url),
      specType: specType.value,
      templateId: selectedTemplateId.value || undefined,
      skus: specs.length > 0 ? specs : undefined,
    };

    try {
      await (data.value?.create
        ? createProductApi(payload)
        : updateProductApi({ ...payload, id: data.value.row.id }));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },

  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};

      formData.value = {
        name: row.name || '',
        productNo: row.productNo || '',
        categoryId: row.categoryId || undefined,
        keywords: row.keywords || '',
        unit: row.unit || '',
        barcode: row.barcode || '',
        salePrice: row.salePrice ?? 0,
        marketPrice: row.marketPrice ?? 0,
        costPrice: row.costPrice ?? 0,
        stock: row.stock ?? 0,
        weight: row.weight ?? undefined,
        dimensions: row.dimensions || '',
        isActive: row.isActive ?? true,
        detail: row.detail || '',
      };

      coverImageUrl.value = row.imageUrl || '';

      if (row.carouselImages && Array.isArray(row.carouselImages)) {
        carouselKeyCounter = 0;
        carouselImages.value = row.carouselImages.map((url: string) => {
          carouselKeyCounter++;
          return { url, uid: carouselKeyCounter };
        });
      } else {
        carouselImages.value = [];
      }

      specType.value = row.specType || 'single';
      selectedTemplateId.value = row.templateId || undefined;
      templateSpecs.value = [];

      // 加载模板下拉列表
      loadTemplateOptions();

      if (row.id && data.value?.create === false) {
        loadProductDetail(row.id);
      } else {
        specList.value = [{
          _key: 1,
          label: '',
          imageUrl: '',
          price: 0,
          costPrice: 0,
          originalPrice: 0,
          stock: 0,
          skuCode: '',
          weight: 0,
          volume: 0,
          isDefault: false,
          isActive: true,
        }];
      }

      try {
        const catRes = await getCategoryListApi({ page: 1, pageSize: 999 });
        const catList = catRes?.items || catRes?.rows || [];
        if (catList.length > 0) {
          categoryOptions.value = catList.map((c: any) => ({
            value: Number(c.id),
            label: c.name,
          }));
        }
      } catch {
        // ignore
      }

      setLoading(false);
    }
  },
});

async function loadProductDetail(id: number) {
  try {
    const [productRes, specsRes] = await Promise.all([
      getProductInfoApi(id),
      getProductSpecsApi(id),
    ]);

    const productData = productRes as any;
    const specsData = specsRes as any;

    if (productData) {
      formData.value = {
        name: productData.name || '',
        productNo: productData.productNo || '',
        categoryId: productData.categoryId || undefined,
        keywords: productData.keywords || '',
        unit: productData.unit || '',
        barcode: productData.barcode || '',
        salePrice: productData.salePrice ?? 0,
        marketPrice: productData.marketPrice ?? 0,
        costPrice: productData.costPrice ?? 0,
        stock: productData.stock ?? 0,
        weight: productData.weight ?? undefined,
        dimensions: productData.dimensions || '',
        isActive: productData.isActive ?? true,
        detail: productData.detail || '',
      };

      coverImageUrl.value = productData.imageUrl || '';

      if (productData.carouselImages && Array.isArray(productData.carouselImages)) {
        carouselKeyCounter = 0;
        carouselImages.value = productData.carouselImages.map((url: string) => {
          carouselKeyCounter++;
          return { url, uid: carouselKeyCounter };
        });
      }

      specType.value = productData.specType || 'single';
      selectedTemplateId.value = productData.templateId || undefined;
    }

    const skuSource = specsData?.skus && Array.isArray(specsData.skus) ? specsData.skus :
                      productData?.skus && Array.isArray(productData.skus) ? productData.skus : [];
    
    if (skuSource.length > 0) {
      specKeyCounter = 0;
      specList.value = skuSource.map((s: any) => {
        specKeyCounter++;
        let label = '';
        if (s.label) {
          label = s.label;
        } else if (typeof s.specs === 'string') {
          label = s.specs;
        } else if (s.specs && typeof s.specs === 'object' && !Array.isArray(s.specs)) {
          label = Object.values(s.specs).join(' / ');
        }
        return {
          _key: specKeyCounter,
          id: s.id,
          label,
          imageUrl: s.imageUrl || s.image || '',
          price: s.price ?? 0,
          costPrice: s.costPrice ?? 0,
          originalPrice: s.originalPrice ?? 0,
          stock: s.stock ?? 0,
          skuCode: s.skuCode || '',
          weight: s.weight ?? 0,
          volume: s.volume ?? 0,
          isDefault: s.isDefault ?? false,
          isActive: s.isActive ?? true,
        };
      });
    } else if (specType.value === 'single') {
      specList.value = [{
        _key: 1,
        label: '',
        imageUrl: '',
        price: productData.salePrice ?? 0,
        costPrice: productData.costPrice ?? 0,
        originalPrice: productData.marketPrice ?? 0,
        stock: productData.stock ?? 0,
        skuCode: '',
        weight: productData.weight ?? 0,
        volume: 0,
        isDefault: false,
        isActive: productData.isActive ?? true,
      }];
    }

    if (specsData?.specs && Array.isArray(specsData.specs)) {
      templateSpecs.value = specsData.specs.map((s: any) => ({
        name: s.name || '',
        values: (s.values || []).map((v: any) => String(v.value || '')).filter(v => v !== ''),
      }));
    }
  } catch {
    // ignore
  }
}

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <template #extra>
      <button
        class="ml-2 p-1.5 rounded-full opacity-80 transition-opacity hover:opacity-100 hover:bg-gray-100"
        @click="toggleFullscreen"
        title="全屏"
      >
        <LucideMaximize2 class="w-4 h-4 text-gray-600" />
      </button>
    </template>
    <Tabs v-model:activeKey="activeTab" class="mt-2">
      <Tabs.TabPane key="basic" tab="基本信息">
        <div class="pt-6 space-y-5">
          <div class="basic-form-row">
            <label class="basic-form-label required">
              <span class="required-star">*</span>
              商品名称
            </label>
            <div class="basic-form-control">
              <Input
                v-model:value="formData.name"
                placeholder="请输入商品名称"
                allow-clear
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label required">
              <span class="required-star">*</span>
              商品分类
            </label>
            <div class="basic-form-control">
              <Select
                v-model:value="formData.categoryId"
                placeholder="请选择商品分类"
                allow-clear
                :options="categoryOptions"
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">关键字</label>
            <div class="basic-form-control">
              <Input
                v-model:value="formData.keywords"
                placeholder="请输入关键字"
                allow-clear
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">单位名</label>
            <div class="basic-form-control">
              <Input
                v-model:value="formData.unit"
                placeholder="请输入单位名"
                allow-clear
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">产品编号</label>
            <div class="basic-form-control">
              <Input
                v-model:value="formData.productNo"
                placeholder="请输入产品编号"
                allow-clear
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">条码</label>
            <div class="basic-form-control">
              <Input
                v-model:value="formData.barcode"
                placeholder="请输入条码"
                allow-clear
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label required">
              <span class="required-star">*</span>
              商品价格
            </label>
            <div class="basic-form-control">
              <InputNumber
                v-model:value="formData.salePrice"
                :min="0"
                :precision="2"
                placeholder="0"
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">成本价</label>
            <div class="basic-form-control">
              <InputNumber
                v-model:value="formData.costPrice"
                :min="0"
                :precision="2"
                placeholder="0"
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">市场价</label>
            <div class="basic-form-control">
              <InputNumber
                v-model:value="formData.marketPrice"
                :min="0"
                :precision="2"
                placeholder="0"
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">库存</label>
            <div class="basic-form-control">
              <InputNumber
                v-model:value="formData.stock"
                :min="0"
                placeholder="0"
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">重量 (KG)</label>
            <div class="basic-form-control">
              <InputNumber
                v-model:value="formData.weight"
                :min="0"
                :precision="3"
                placeholder="0"
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">尺寸</label>
            <div class="basic-form-control">
              <Input
                v-model:value="formData.dimensions"
                placeholder="如 30x20x10cm"
                allow-clear
                style="width: 100%"
              />
            </div>
          </div>

          <div class="basic-form-row items-start">
            <label class="basic-form-label required pt-1.5">
              <span class="required-star">*</span>
              封面图
            </label>
            <div class="basic-form-control">
              <div class="flex items-start gap-3">
                <div v-if="coverImageUrl" class="relative w-20 h-20 rounded-lg border border-gray-200 overflow-hidden flex-shrink-0">
                  <img :src="coverImageUrl" alt="封面图" class="w-full h-full object-cover" />
                  <div class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity cursor-pointer" @click="removeCoverImage">
                    <LucideX class="text-white w-4 h-4" />
                  </div>
                </div>
                <Upload v-else :show-upload-list="false" :before-upload="handleCoverUpload" accept="image/png,image/jpeg,image/jpg,image/gif,image/webp">
                  <div class="flex flex-col items-center justify-center w-20 h-20 rounded-lg border-2 border-dashed border-gray-300 hover:border-blue-400 cursor-pointer transition-colors">
                    <LucideUpload class="w-4 h-4 text-gray-400" />
                    <span class="text-xs text-gray-400 mt-0.5">上传图片</span>
                  </div>
                </Upload>
              </div>
            </div>
          </div>

          <div class="basic-form-row items-start">
            <label class="basic-form-label pt-1.5">轮播图</label>
            <div class="basic-form-control">
              <div class="flex flex-wrap gap-3">
                <div
                  v-for="img in carouselImages"
                  :key="img.uid"
                  class="relative w-20 h-20 rounded-lg border border-gray-200 overflow-hidden flex-shrink-0"
                >
                  <img :src="img.url" alt="轮播图" class="w-full h-full object-cover" />
                  <div class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity cursor-pointer" @click="removeCarouselImage(img.uid)">
                    <LucideX class="text-white w-4 h-4" />
                  </div>
                </div>
                <Upload :show-upload-list="false" :before-upload="handleCarouselUpload" accept="image/png,image/jpeg,image/jpg,image/gif,image/webp">
                  <div class="flex flex-col items-center justify-center w-20 h-20 rounded-lg border-2 border-dashed border-gray-300 hover:border-blue-400 cursor-pointer transition-colors">
                    <LucideUpload class="w-4 h-4 text-gray-400" />
                    <span class="text-xs text-gray-400 mt-0.5">上传图片</span>
                  </div>
                </Upload>
              </div>
            </div>
          </div>

          <div class="basic-form-row">
            <label class="basic-form-label">商品状态</label>
            <div class="basic-form-control">
              <Radio.Group v-model:value="formData.isActive">
                <Radio :value="true">上架</Radio>
                <Radio :value="false">下架</Radio>
              </Radio.Group>
            </div>
          </div>
        </div>
      </Tabs.TabPane>

      <Tabs.TabPane key="spec" tab="商品规格">
        <div class="space-y-5 pt-4">
          <div class="flex items-center gap-6">
            <label class="w-24 text-sm font-medium text-gray-700 text-right pr-3">商品规格</label>
            <Radio.Group v-model:value="specType" class="flex gap-4">
              <Radio value="single">单规格</Radio>
              <Radio value="multiple">多规格</Radio>
            </Radio.Group>
          </div>

          <!-- 多规格：模板选择 -->
          <div v-if="specType === 'multiple'" class="flex items-center gap-6">
            <label class="w-24 text-sm font-medium text-gray-700 text-right pr-3">规格模板</label>
            <div class="flex-1 max-w-md">
              <Select
                v-model:value="selectedTemplateId"
                placeholder="请选择规格模板"
                allow-clear
                :options="templateOptions"
                style="width: 100%"
                @change="handleTemplateChange"
              />
            </div>
          </div>

          <!-- 多规格：模板规格预览 -->
          <div v-if="specType === 'multiple' && templateSpecs.length > 0" class="flex flex-wrap gap-3">
            <div
              v-for="ts in templateSpecs"
              :key="ts.name"
              class="bg-blue-50 border border-blue-200 rounded-lg px-3 py-2"
            >
              <span class="text-sm font-medium text-blue-700">{{ ts.name }}：</span>
              <span class="text-sm text-blue-600">{{ ts.values.join('、') }}</span>
            </div>
          </div>

          <div
            ref="skuWrapperRef"
            class="border rounded-lg sku-table-wrapper"
            :class="{ 'scrolled-right': isLeftScrolled, 'scrolled-left': isRightScrolled }"
            @scroll="handleSkuScroll"
          >
            <table class="sku-table">
              <thead class="bg-gray-50">
                <tr>
                  <th v-if="specType === 'multiple'" class="px-4 py-3 text-left text-sm font-medium text-gray-600 whitespace-nowrap sticky-col-spec">规格组合</th>
                  <th :class="['px-4 py-3 text-left text-sm font-medium text-gray-600', { 'sticky-col-image': specType === 'multiple' }]">图片</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-130">售价</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-130">成本价</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-130">原价</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-110">库存</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-150">商品编号</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-140">重量 (KG)</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 min-w-140">体积(m³)</th>
                  <th v-if="specType === 'multiple'" class="px-4 py-3 text-left text-sm font-medium text-gray-600 whitespace-nowrap sticky-col-right-group sticky-col-default">默认选中规格</th>
                  <th v-if="specType === 'multiple'" class="px-4 py-3 text-left text-sm font-medium text-gray-600 whitespace-nowrap sticky-col-right-group sticky-col-action">操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="spec in specList" :key="spec._key" class="border-t">
                  <td v-if="specType === 'multiple'" class="px-4 py-3 sticky-col-spec">
                    <span class="text-sm font-medium text-gray-700 whitespace-nowrap">{{ spec.label }}</span>
                  </td>
                  <td :class="['px-4 py-3', { 'sticky-col-image': specType === 'multiple' }]">
                    <div v-if="spec.imageUrl" class="relative w-12 h-12 rounded-lg border border-gray-200 overflow-hidden">
                      <img :src="spec.imageUrl" alt="规格图片" class="w-full h-full object-cover" />
                      <div class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity cursor-pointer" @click="removeSpecImage(spec)">
                        <LucideX class="text-white w-3 h-3" />
                      </div>
                    </div>
                    <Upload v-else :show-upload-list="false" :before-upload="(file: File) => handleSpecImageUpload(file, spec)" accept="image/png,image/jpeg,image/jpg,image/gif,image/webp">
                      <div class="flex items-center justify-center w-12 h-12 rounded-lg border-2 border-dashed border-gray-300 hover:border-blue-400 cursor-pointer transition-colors">
                        <LucideUpload class="w-4 h-4 text-gray-400" />
                      </div>
                    </Upload>
                  </td>
                  <td class="px-4 py-3">
                    <InputNumber v-model:value="spec.price" :min="0" :precision="2" size="small" style="width: 100%" />
                  </td>
                  <td class="px-4 py-3">
                    <InputNumber v-model:value="spec.costPrice" :min="0" :precision="2" size="small" style="width: 100%" />
                  </td>
                  <td class="px-4 py-3">
                    <InputNumber v-model:value="spec.originalPrice" :min="0" :precision="2" size="small" style="width: 100%" />
                  </td>
                  <td class="px-4 py-3">
                    <InputNumber v-model:value="spec.stock" :min="0" size="small" style="width: 100%" />
                  </td>
                  <td class="px-4 py-3">
                    <Input v-model:value="spec.skuCode" placeholder="商品编号" size="small" style="width: 100%" />
                  </td>
                  <td class="px-4 py-3">
                    <InputNumber v-model:value="spec.weight" :min="0" :precision="3" size="small" style="width: 100%" />
                  </td>
                  <td class="px-4 py-3">
                    <InputNumber v-model:value="spec.volume" :min="0" :precision="6" size="small" style="width: 100%" />
                  </td>
                  <td v-if="specType === 'multiple'" class="px-4 py-3 sticky-col-right-group sticky-col-default">
                    <div class="flex items-center">
                      <Switch :checked="spec.isDefault" @change="(val: any) => handleDefaultChange(spec, val ?? false)" />
                    </div>
                  </td>
                  <td v-if="specType === 'multiple'" class="px-4 py-3 sticky-col-right-group sticky-col-action">
                    <div class="flex items-center">
                       <Switch
                        v-model:checked="spec.isActive"
                        checked-children="显示"
                        un-checked-children="隐藏"
                      />
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </Tabs.TabPane>

      <Tabs.TabPane key="detail" tab="商品详情">
        <div class="pt-4">
          <div class="flex items-start mb-2">
            <label class="basic-form-label pt-1.5">产品描述</label>
          </div>
          <ProductEditor v-model="formData.detail" />
        </div>
      </Tabs.TabPane>
    </Tabs>
  </Drawer>
</template>

<style scoped>
.product-drawer :deep(.ant-modal-body) {
  max-height: 70vh;
  overflow-y: auto;
}

.product-drawer :deep(.ant-tabs-content) {
  padding: 0;
}

.basic-form-row {
  display: flex;
  align-items: center;
  min-height: 32px;
}

.basic-form-row.items-start {
  align-items: flex-start;
}

.basic-form-label {
  width: 120px;
  padding-right: 16px;
  text-align: right;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  flex-shrink: 0;
  line-height: 32px;
}

.basic-form-label.required {
  padding-left: 12px;
  position: relative;
}

.required-star {
  color: #ef4444;
  position: absolute;
  left: 0;
  top: 0;
  line-height: 32px;
}

.basic-form-control {
  width: 360px;
  flex-shrink: 0;
}

/* SKU 表格：水平滚动 + 左右列固定 */
.sku-table-wrapper {
  overflow-x: auto;
  overflow-y: hidden;
  padding-bottom: 16px;
}

.sku-table {
  min-width: 1300px;
  width: 100%;
}

/* 左侧固定列 - 规格组合 */
.sticky-col-spec {
  position: sticky;
  left: 0;
  z-index: 2;
  background: #fff;
  min-width: 160px;
}

/* 左侧固定列 - 图片 */
.sticky-col-image {
  position: sticky;
  left: 160px;
  z-index: 2;
  background: #fff;
}

/* 右侧固定列分组层：默认选中规格 + 操作 */
.sticky-col-right-group {
  background: #fff;
  z-index: 2;
}

/* 默认选中规格列（固定列，紧邻操作列左侧） */
.sticky-col-default {
  position: sticky;
  right: 78px;
  width: 78px;
  box-sizing: border-box;
}

/* 操作列（固定在最右侧） */
.sticky-col-action {
  position: sticky;
  right: 0;
  width: 78px;
  box-sizing: border-box;
}

/* 向右滚动时，左侧固定列右侧加阴影和边框 */
.sku-table-wrapper.scrolled-right .sticky-col-spec,
.sku-table-wrapper.scrolled-right .sticky-col-image {
  box-shadow: 3px 0 6px -3px rgba(0,0,0,0.12);
  border-right: 1px solid #e5e7eb;
}

/* 右侧固定列分组左侧始终显示渐变阴影和边框（参照 vxe-table fixed right 效果） */
.sticky-col-right-group {
  box-shadow: -8px 0 12px -6px rgba(0,0,0,0.15);
  border-left: 1px solid #e5e7eb;
}

/* thead 中的左侧固定列需要更高层级 */
thead .sticky-col-spec,
thead .sticky-col-image {
  z-index: 3;
}

/* thead 中的右侧固定列分组 */
thead .sticky-col-right-group {
  z-index: 3;
}

/* 中间可滚动列最小宽度 */
.min-w-110 { min-width: 110px; }
.min-w-130 { min-width: 130px; }
.min-w-140 { min-width: 140px; }
.min-w-150 { min-width: 150px; }
</style>

<style>
/* 非 scoped：product-drawer、drawer-75/drawer-fullscreen、w-130 在同一个元素上 */
.product-drawer.drawer-75.w-130 {
  width: 75% !important;
}
.product-drawer.drawer-fullscreen.w-130 {
  width: 100% !important;
}
</style>