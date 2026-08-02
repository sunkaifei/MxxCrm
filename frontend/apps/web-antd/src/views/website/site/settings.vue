<script lang="ts" setup>
import { computed, h, onMounted, ref } from 'vue';
import { Page } from '@vben/common-ui';
import {
  Button,
  Card,
  Empty,
  Image,
  Input,
  InputNumber,
  message,
  Modal,
  Radio,
  RadioGroup,
  Select,
  Skeleton,
  Switch,
  Tabs,
  Tag,
  Upload,
} from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import {
  LucideFilePenLine,
  LucideFileText,
  LucideGlobe,
  LucideLayoutDashboard,
  LucideLink,
  LucideMegaphone,
  LucideMoreHorizontal,
  LucideSearch,
  LucideSettings,
  LucideUpload,
} from '@vben/icons';
import {
  notificationApi,
  siteApi,
  templateApi,
} from '#/api';
import type {
  NotificationConfigSaveDTO,
  NotificationConfigVO,
} from '#/api/core/website/notification';
import type { SiteVO } from '#/api/core/website/site';
import { uploadFileApi } from '#/api/core/attachment/file';
import CodeEditor from '#/components/CodeEditor/index.vue';

// ============ 加载状态 ============
const loading = ref(false);
const saving = ref(false);
const siteData = ref<Partial<SiteVO>>({});

// ============ 左侧分组菜单（DEDECMS 风格）============
interface MenuGroup {
  key: string;
  label: string;
  icon?: string;
}
const menuGroups: MenuGroup[] = [
  { key: 'basic', label: '基本设置' },
  { key: 'seo', label: 'SEO 设置' },
  { key: 'url', label: 'URL 规则' },
  { key: 'template', label: '模板设置' },
  { key: 'upload', label: '上传设置' },
  { key: 'share', label: '分享设置' },
  { key: 'notification', label: '通知配置' },
  { key: 'code', label: '代码设置' },
  { key: 'misc', label: '其他设置' },
];
const activeKey = ref('basic');

// 菜单图标映射
const menuIcons: Record<string, any> = {
  basic: LucideSettings,
  seo: LucideSearch,
  url: LucideLink,
  template: LucideLayoutDashboard,
  upload: LucideUpload,
  share: LucideGlobe,
  notification: LucideMegaphone,
  code: LucideFileText,
  misc: LucideMoreHorizontal,
};

// ============ 图片上传公共方法 ============
async function handleImageUpload(
  file: File,
  setValue: (val: string) => void,
  setList: (list: UploadFile[]) => void,
  label: string,
) {
  try {
    const res: any = await uploadFileApi(file, 'website');
    const url = res?.data?.url || res?.url;
    if (url) {
      setValue(url);
      setList([{ uid: '-1', name: label, status: 'done', url }]);
      message.success('上传成功');
    }
    return false;
  } catch {
    message.error('上传失败');
    return false;
  }
}

// Logo
const logoList = ref<UploadFile[]>([]);
const logoUrl = ref('');
async function handleLogoUpload(file: File) {
  return handleImageUpload(
    file,
    (v) => {
      logoUrl.value = v;
      formData.value.logo = v;
    },
    (l) => (logoList.value = l),
    'logo',
  );
}
function handleLogoRemove() {
  logoList.value = [];
  logoUrl.value = '';
  formData.value.logo = '';
}

// 水印图片
const watermarkImageList = ref<UploadFile[]>([]);
const watermarkImageUrl = ref('');
async function handleWatermarkImageUpload(file: File) {
  return handleImageUpload(
    file,
    (v) => {
      watermarkImageUrl.value = v;
      formData.value.watermarkImage = v;
    },
    (l) => (watermarkImageList.value = l),
    'watermark',
  );
}
function handleWatermarkImageRemove() {
  watermarkImageList.value = [];
  watermarkImageUrl.value = '';
  formData.value.watermarkImage = '';
}

// 分享图片
const shareImageList = ref<UploadFile[]>([]);
const shareImageUrl = ref('');
async function handleShareImageUpload(file: File) {
  return handleImageUpload(
    file,
    (v) => {
      shareImageUrl.value = v;
      formData.value.shareImage = v;
    },
    (l) => (shareImageList.value = l),
    'share',
  );
}
function handleShareImageRemove() {
  shareImageList.value = [];
  shareImageUrl.value = '';
  formData.value.shareImage = '';
}

// ============ 模板选择弹窗 ============
const templateModalVisible = ref(false);
const templateList = ref<any[]>([]);
const templateLoading = ref(false);
const templateSearch = ref('');
const selectedTemplateName = ref('');
const currentTemplateDetail = ref<any>(null);
const previewModalVisible = ref(false);
const previewUrl = ref('');

async function openTemplateModal() {
  templateModalVisible.value = true;
  templateLoading.value = true;
  try {
    const res: any = await templateApi.list({
      page: 1,
      pageSize: 100,
      status: 1,
      keywords: templateSearch.value,
    });
    templateList.value = res?.items || [];
  } catch {
    templateList.value = [];
  } finally {
    templateLoading.value = false;
  }
}

function selectTemplate(tpl: any) {
  formData.value.templateId = tpl.id;
  selectedTemplateName.value = tpl.name;
  currentTemplateDetail.value = tpl;
  templateModalVisible.value = false;
}

async function loadTemplateDetail(templateId?: number) {
  if (!templateId) {
    currentTemplateDetail.value = null;
    return;
  }
  try {
    const res: any = await templateApi.detail(templateId);
    currentTemplateDetail.value = res?.data || res;
    selectedTemplateName.value = currentTemplateDetail.value?.name || '';
  } catch {
    currentTemplateDetail.value = null;
  }
}

function previewCurrentTemplate() {
  if (currentTemplateDetail.value?.previewUrl) {
    previewUrl.value = currentTemplateDetail.value.previewUrl;
    previewModalVisible.value = true;
  }
}

// ============ 主表单数据（直接响应式对象，DEDECMS 风格自定义布局）============
const formData = ref<Record<string, any>>({
  siteName: '',
  logo: '',
  domain: '',
  bindDomain: '',
  siteType: 1,
  siteMode: 1,
  client: 1,
  showBanner: 1,
  status: 1,
  closeReason: '',
  sort: 0,
  isDefault: 1,
  keywords: '',
  description: '',
  urlRule: 0,
  urlRulePattern: '',
  robotsContent: '',
  templateId: undefined,
  watermarkEnable: 0,
  watermarkType: 1,
  watermarkText: '',
  watermarkPosition: 9,
  watermarkOpacity: 50,
  uploadAllowedTypes: '',
  uploadMaxSize: 10,
  companyName: '',
  companyPhone: '',
  companyEmail: '',
  companyAddress: '',
  workDays: '',
  workTimeStart: undefined,
  workTimeEnd: undefined,
  qq: '',
  wechat: '',
  wechatQrcode: '',
  icp: '',
  copyright: '',
  shareTitle: '',
  shareDesc: '',
  shareImage: '',
  statisticsCode: '',
  customCss: '',
  customJs: '',
  remark: '',
});

// ============ 通知配置 ============
const notificationList = ref<NotificationConfigSaveDTO[]>([]);
const notificationLoading = ref(false);

// 预置场景列表，便于按场景编辑
const presetScenes: Array<{ sceneCode: string; sceneName: string }> = [
  { sceneCode: 'order_paid', sceneName: '订单支付成功' },
  { sceneCode: 'order_shipped', sceneName: '订单已发货' },
  { sceneCode: 'order_completed', sceneName: '订单完成' },
  { sceneCode: 'order_refund', sceneName: '订单退款' },
  { sceneCode: 'lead_created', sceneName: '新线索创建' },
  { sceneCode: 'message_received', sceneName: '收到新留言' },
  { sceneCode: 'member_register', sceneName: '会员注册' },
];

async function loadNotifications(websiteId?: number) {
  if (!websiteId) return;
  notificationLoading.value = true;
  try {
    const list: NotificationConfigVO[] = await notificationApi.getCurrent();
    // 将已存在配置按 sceneCode 索引
    const map = new Map<string, NotificationConfigVO>();
    list.forEach((item) => {
      if (item.sceneCode) map.set(item.sceneCode, item);
    });
    // 合并预置场景 + 已有配置（保证页面稳定显示）
    const merged: NotificationConfigSaveDTO[] = presetScenes.map((s) => {
      const exist = map.get(s.sceneCode);
      return {
        id: exist?.id,
        websiteId: exist?.websiteId ?? websiteId,
        sceneCode: s.sceneCode,
        sceneName: s.sceneName,
        channels: exist?.channels ?? 'email',
        recipientEmails: exist?.recipientEmails ?? '',
        emailSubject: exist?.emailSubject ?? '',
        emailBody: exist?.emailBody ?? '',
        enabled: exist?.enabled ?? 0,
      };
    });
    // 追加数据库中存在但不在预置列表的场景
    list.forEach((item) => {
      if (item.sceneCode && !presetScenes.some((s) => s.sceneCode === item.sceneCode)) {
        merged.push({
          id: item.id,
          websiteId: item.websiteId,
          sceneCode: item.sceneCode,
          sceneName: item.sceneName || item.sceneCode,
          channels: item.channels ?? 'email',
          recipientEmails: item.recipientEmails ?? '',
          emailSubject: item.emailSubject ?? '',
          emailBody: item.emailBody ?? '',
          enabled: item.enabled ?? 0,
        });
      }
    });
    notificationList.value = merged;
  } catch {
    console.error('通知配置加载失败');
  } finally {
    notificationLoading.value = false;
  }
}

async function saveNotifications() {
  try {
    const configs = notificationList.value.map((item) => ({
      ...item,
      channels: item.channels || 'email',
      enabled: item.enabled ?? 0,
    }));
    await notificationApi.updateCurrent(configs);
  } catch {
    console.error('通知配置保存失败');
  }
}

// ============ 主数据加载/保存 ============
async function loadSite() {
  loading.value = true;
  try {
    const data: SiteVO = await siteApi.getCurrent();
    siteData.value = data || {};
    formData.value = { ...formData.value, ...(data || {}) };
    // 图片预览
    logoUrl.value = data?.logo || '';
    logoList.value = data?.logo
      ? [{ uid: '-1', name: 'logo', status: 'done' as const, url: data.logo }]
      : [];
    watermarkImageUrl.value = data?.watermarkImage || '';
    watermarkImageList.value = data?.watermarkImage
      ? [
          {
            uid: '-1',
            name: 'watermark',
            status: 'done' as const,
            url: data.watermarkImage,
          },
        ]
      : [];
    // 分享图片
    shareImageUrl.value = data?.shareImage || '';
    shareImageList.value = data?.shareImage
      ? [
          {
            uid: '-1',
            name: 'share',
            status: 'done' as const,
            url: data.shareImage,
          },
        ]
      : [];
    // 模板详情
    if (data?.templateId) {
      await loadTemplateDetail(Number(data.templateId));
    }
    // 通知配置
    await loadNotifications(data?.id);
  } catch {
    message.error('数据加载失败，请刷新页面重试');
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  if (!formData.value.siteName) {
    activeKey.value = 'basic';
    message.warning('请输入网站名称');
    return;
  }
  saving.value = true;
  try {
    const values: Record<string, any> = { ...formData.value };
    // 合并图片URL
    values.logo = logoUrl.value;
    values.watermarkImage = watermarkImageUrl.value;
    values.shareImage = shareImageUrl.value;
    // 时间转字符串
    if (values.workTimeStart && typeof values.workTimeStart !== 'string') {
      values.workTimeStart = values.workTimeStart?.format('HH:mm');
    }
    if (values.workTimeEnd && typeof values.workTimeEnd !== 'string') {
      values.workTimeEnd = values.workTimeEnd?.format('HH:mm');
    }
    // 单站模式：默认站点必须保持为默认
    values.isDefault = 1;
    await siteApi.updateCurrent(values);
    // 同步保存通知配置
    await saveNotifications();
    message.success('保存成功');
    await loadSite();
  } catch {
    message.error('保存失败，请重试');
  } finally {
    saving.value = false;
  }
}

async function handleReset() {
  Modal.confirm({
    title: '确认重置',
    content: '将放弃当前未保存的修改并重新加载，确认继续吗？',
    onOk: async () => {
      await loadSite();
      message.success('已重置');
    },
  });
}

onMounted(loadSite);

// 顶部状态
const statusTagColor = computed(() =>
  siteData.value?.status === 1 ? 'success' : 'default',
);
const statusTagText = computed(() =>
  siteData.value?.status === 1 ? '正常' : '关闭',
);
</script>

<template>
  <Page auto-content-height>
    <div class="site-settings-page">
      <!-- 顶部状态条 -->
      <div class="site-status-bar">
        <div class="status-left">
          <Image
            v-if="logoUrl"
            :src="logoUrl"
            :width="40"
            :height="40"
            fit="cover"
            class="status-logo"
          />
          <div v-else class="status-logo-placeholder">站</div>
          <div class="status-info">
            <h2 class="status-title">
              {{ siteData.siteName || '网站设置' }}
            </h2>
            <div class="status-meta">
              <span class="meta-item">
                绑定域名：{{ siteData.bindDomain || '—' }}
              </span>
              <span class="meta-item">
                二级域名：{{ siteData.domain || '—' }}
              </span>
              <Tag :color="statusTagColor">{{ statusTagText }}</Tag>
            </div>
          </div>
        </div>
        <div class="status-right">
          <Button :loading="loading" @click="handleReset">重置</Button>
          <Button
            type="primary"
            :icon="h(LucideFilePenLine)"
            :loading="saving"
            @click="handleSave"
          >
            保存设置
          </Button>
        </div>
      </div>

      <!-- 主体：选项卡模式 -->
      <div class="site-settings-body">
        <section class="site-content">
          <Skeleton v-if="loading" active :paragraph="{ rows: 8 }" />

          <Tabs
            v-else
            v-model:activeKey="activeKey"
            class="site-tabs"
            size="large"
            :tabBarStyle="{ paddingLeft: '8px', paddingRight: '8px' }"
          >
            <!-- 基本设置 -->
            <Tabs.TabPane key="basic">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.basic" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[0]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="基础信息" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  网站的基础信息，包括名称、类型、域名和 Logo 等。
                </p>
                <div class="form-grid-2">
                  <div class="form-item">
                    <label class="form-label">
                      网站名称 <span class="req">*</span>
                    </label>
                    <Input v-model:value="formData.siteName" name="siteName" autocomplete="off" placeholder="输入网站名称…" allow-clear />
                  </div>
                  <div class="form-item">
                    <label class="form-label">网站类型</label>
                    <RadioGroup v-model:value="formData.siteType">
                      <Radio :value="1">企业官网</Radio>
                      <Radio :value="2">商城</Radio>
                      <Radio :value="3">其他</Radio>
                    </RadioGroup>
                  </div>
                  <div class="form-item">
                    <label class="form-label">
                      站点模式
                      <span class="form-label-tip">决定前台按钮渲染逻辑</span>
                    </label>
                    <RadioGroup v-model:value="formData.siteMode">
                      <Radio :value="1">展示型</Radio>
                      <Radio :value="2">交易型</Radio>
                      <Radio :value="3">混合型</Radio>
                    </RadioGroup>
                    <div class="form-extra">
                      展示型渲染"立即咨询"，交易型渲染"加入购物车/立即购买"，混合型两者都渲染
                    </div>
                  </div>
                  <div class="form-item">
                    <label class="form-label">客户端</label>
                    <RadioGroup v-model:value="formData.client">
                      <Radio :value="1">PC</Radio>
                      <Radio :value="2">WAP</Radio>
                      <Radio :value="3">CMS</Radio>
                    </RadioGroup>
                  </div>
                  <div class="form-item">
                    <label class="form-label">二级域名</label>
                    <Input v-model:value="formData.domain" name="domain" autocomplete="off" placeholder="如：demo" allow-clear />
                    <p class="field-tip">前台访问地址的子域名前缀</p>
                  </div>
                  <div class="form-item">
                    <label class="form-label">绑定域名</label>
                    <Input v-model:value="formData.bindDomain" name="bindDomain" autocomplete="off" placeholder="如：www.example.com" allow-clear />
                    <p class="field-tip">自定义域名，需要先做 DNS 解析</p>
                  </div>
                  <div class="form-item">
                    <label class="form-label">排序</label>
                    <InputNumber v-model:value="formData.sort" :min="0" style="width: 100%" />
                  </div>
                </div>
              </Card>

              <Card title="网站Logo" :bordered="false" class="cfg-card">
                <Upload
                  :file-list="logoList"
                  :before-upload="
                    (file: File) => {
                      handleLogoUpload(file);
                      return false;
                    }
                  "
                  :remove="handleLogoRemove"
                  list-type="picture-card"
                  accept="image/*"
                >
                  <div v-if="logoList.length < 1" class="upload-plus">
                    <div class="upload-icon">+</div>
                    <div class="upload-text">上传Logo</div>
                  </div>
                </Upload>
                <p class="upload-tip">建议尺寸 200×60，支持 JPG/PNG/GIF</p>
              </Card>

              <Card title="状态设置" :bordered="false" class="cfg-card">
                <div class="status-inline-grid">
                  <div class="status-inline-item">
                    <label class="form-label">网站状态</label>
                    <RadioGroup v-model:value="formData.status">
                      <Radio :value="1">正常</Radio>
                      <Radio :value="0">关闭</Radio>
                    </RadioGroup>
                  </div>
                  <div class="status-inline-item">
                    <label class="form-label">
                      首页Banner
                      <span class="form-label-tip">开启后首页顶部显示轮播图</span>
                    </label>
                    <Switch v-model:checked="formData.showBanner" />
                  </div>
                </div>
                <div class="form-item form-item-full" style="margin-top: 14px">
                  <label class="form-label">关闭原因</label>
                  <Input.TextArea v-model:value="formData.closeReason" name="closeReason" autocomplete="off" :rows="2" placeholder="网站关闭时向前台显示的提示信息…" allow-clear />
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- SEO 设置 -->
            <Tabs.TabPane key="seo">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.seo" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[1]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="SEO 优化" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  设置关键词和描述有助于搜索引擎收录和排名。
                </p>
                <div class="form-item form-item-full">
                  <label class="form-label">SEO关键词</label>
                  <Input v-model:value="formData.keywords" name="keywords" autocomplete="off" placeholder="多个关键词用逗号分隔…" allow-clear />
                  <p class="field-tip">
                    多个关键词之间用英文逗号分隔，建议 5-10 个
                  </p>
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">SEO描述</label>
                  <Input.TextArea v-model:value="formData.description" name="description" autocomplete="off" :rows="3" placeholder="网站描述，用于搜索引擎收录…" allow-clear />
                  <p class="field-tip">
                    网站简短描述，建议 50-200 字，将显示在搜索结果中
                  </p>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- URL 规则 -->
            <Tabs.TabPane key="url">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.url" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[2]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="URL 伪静态规则" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  控制前台页面的 URL 形态，影响 SEO 与可读性。
                </p>
                <div class="form-item form-item-full">
                  <label class="form-label">URL规则模式</label>
                  <RadioGroup v-model:value="formData.urlRule">
                    <Radio :value="0">动态URL（默认）</Radio>
                    <Radio :value="1">短URL</Radio>
                    <Radio :value="2">目录模式</Radio>
                    <Radio :value="3">自定义</Radio>
                  </RadioGroup>
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">自定义规则模板</label>
                  <Input.TextArea v-model:value="formData.urlRulePattern" name="urlRulePattern" autocomplete="off" :rows="3" placeholder="仅在&quot;自定义&quot;模式下生效。可用占位符：{module} {id} {page} {category}，如 /{category}/{id}.html" allow-clear />
                  <p class="field-tip">
                    仅在"自定义"模式下生效。可用占位符：
                    <code>{module}</code> <code>{id}</code> <code>{page}</code>
                    <code>{category}</code>，如
                    <code>/{category}/{id}.html</code>
                  </p>
                </div>
                <div class="url-rule-tips">
                  <div class="tip-row">
                    <strong>动态URL：</strong>
                    <span>/article/detail?id=123</span>
                  </div>
                  <div class="tip-row">
                    <strong>短URL：</strong>
                    <span>/a/123</span>
                  </div>
                  <div class="tip-row">
                    <strong>目录模式：</strong>
                    <span>/article/123/</span>
                  </div>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- 模板设置 -->
            <Tabs.TabPane key="template">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.template" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[3]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="当前模板" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  当前网站正在使用的模板，点击可更换。
                </p>
                <div class="current-template-card">
                  <div class="tpl-preview">
                    <img
                      v-if="currentTemplateDetail?.previewPic"
                      :src="currentTemplateDetail.previewPic"
                      :alt="selectedTemplateName + ' 模板预览图'"
                    />
                    <div v-else class="tpl-preview-placeholder">
                      <span>暂无预览图</span>
                    </div>
                  </div>
                  <div class="tpl-info">
                    <h4 class="tpl-name">
                      {{ selectedTemplateName || '尚未选择模板' }}
                    </h4>
                    <div v-if="currentTemplateDetail" class="tpl-meta">
                      <div class="meta-row">
                        <span class="meta-label">模板文件夹：</span>
                        <span class="meta-value">{{
                          currentTemplateDetail.templateFolder || '—'
                        }}</span>
                      </div>
                      <div class="meta-row">
                        <span class="meta-label">支持终端：</span>
                        <div class="meta-tags">
                          <Tag v-if="currentTemplateDetail.terminalPc === 1" color="blue">
                            PC端
                          </Tag>
                          <Tag
                            v-if="currentTemplateDetail.terminalMobile === 1"
                            color="green"
                          >
                            手机端
                          </Tag>
                          <Tag
                            v-if="currentTemplateDetail.terminalIpad === 1"
                            color="orange"
                          >
                            平板
                          </Tag>
                        </div>
                      </div>
                    </div>
                    <div class="tpl-actions">
                      <Button type="primary" @click="openTemplateModal">
                        更换模板
                      </Button>
                      <Button
                        v-if="currentTemplateDetail?.previewUrl"
                        @click="previewCurrentTemplate"
                      >
                        预览演示
                      </Button>
                    </div>
                  </div>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- 上传设置 -->
            <Tabs.TabPane key="upload">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.upload" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[4]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="上传设置" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  控制图片水印和文件上传的限制参数。水印将在上传图片时自动添加。
                </p>
                <h4 class="section-subtitle">图片水印</h4>
                <div class="form-grid-2">
                  <div class="form-item">
                    <label class="form-label">开启水印</label>
                    <Switch v-model:checked="formData.watermarkEnable" />
                    <p class="field-tip">开启后上传的图片将自动打上水印</p>
                  </div>
                  <div class="form-item">
                    <label class="form-label">水印类型</label>
                    <RadioGroup v-model:value="formData.watermarkType">
                      <Radio :value="1">文字水印</Radio>
                      <Radio :value="2">图片水印</Radio>
                    </RadioGroup>
                  </div>
                  <div class="form-item">
                    <label class="form-label">水印位置</label>
                    <Select v-model:value="formData.watermarkPosition" :options="[
                      { label: '左上', value: 1 },
                      { label: '上中', value: 2 },
                      { label: '右上', value: 3 },
                      { label: '左中', value: 4 },
                      { label: '居中', value: 5 },
                      { label: '右中', value: 6 },
                      { label: '左下', value: 7 },
                      { label: '下中', value: 8 },
                      { label: '右下', value: 9 },
                    ]" />
                  </div>
                  <div class="form-item">
                    <label class="form-label">透明度(%)</label>
                    <InputNumber v-model:value="formData.watermarkOpacity" :min="0" :max="100" style="width: 100%" />
                  </div>
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">水印文字</label>
                  <Input v-model:value="formData.watermarkText" name="watermarkText" autocomplete="off" placeholder="输入水印文字内容…" allow-clear />
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">水印图片</label>
                  <Upload
                    :file-list="watermarkImageList"
                    :before-upload="
                      (file: File) => {
                        handleWatermarkImageUpload(file);
                        return false;
                      }
                    "
                    :remove="handleWatermarkImageRemove"
                    list-type="picture-card"
                    accept="image/png"
                  >
                    <div v-if="watermarkImageList.length < 1" class="upload-plus">
                      <div class="upload-icon">+</div>
                      <div class="upload-text">上传水印</div>
                    </div>
                  </Upload>
                  <p class="upload-tip">图片水印时使用，建议使用透明 PNG 格式</p>
                </div>

                <div class="section-divider"></div>

                <h4 class="section-subtitle">上传限制</h4>
                <div class="form-grid-2">
                  <div class="form-item">
                    <label class="form-label">单文件最大(MB)</label>
                    <InputNumber v-model:value="formData.uploadMaxSize" name="uploadMaxSize" :min="1" :max="500" style="width: 100%" />
                    <p class="field-tip">超过此大小的文件将被拒绝上传</p>
                  </div>
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">允许上传文件类型</label>
                  <Input v-model:value="formData.uploadAllowedTypes" name="uploadAllowedTypes" autocomplete="off" placeholder="jpg,png,gif,pdf,doc 等，逗号分隔" allow-clear />
                  <p class="field-tip">
                    多个扩展名用英文逗号分隔，如：jpg,png,gif,pdf,doc,xls,zip。留空则不限制
                  </p>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- 分享设置 -->
            <Tabs.TabPane key="share">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.share" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[5]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="微信分享设置" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  设置网站分享到微信/朋友圈时显示的标题、描述和缩略图。
                </p>
                <div class="form-item form-item-full">
                  <label class="form-label">分享标题</label>
                  <Input v-model:value="formData.shareTitle" name="shareTitle" autocomplete="off" placeholder="分享到微信时的标题…" allow-clear />
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">分享描述</label>
                  <Input.TextArea v-model:value="formData.shareDesc" name="shareDesc" autocomplete="off" :rows="2" placeholder="分享到微信时的描述文字…" allow-clear />
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">分享图片</label>
                  <Upload
                    :file-list="shareImageList"
                    :before-upload="
                      (file: File) => {
                        handleShareImageUpload(file);
                        return false;
                      }
                    "
                    :remove="handleShareImageRemove"
                    list-type="picture-card"
                    accept="image/*"
                  >
                    <div v-if="shareImageList.length < 1" class="upload-plus">
                      <div class="upload-icon">+</div>
                      <div class="upload-text">上传图片</div>
                    </div>
                  </Upload>
                  <p class="upload-tip">建议尺寸 500x400，支持 JPG/PNG</p>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- 通知配置 -->
            <Tabs.TabPane key="notification">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.notification" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[6]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card :bordered="false" class="cfg-card">
                <template #title>
                  <div class="card-title-row">
                    <span>通知配置</span>
                    <span class="card-title-tip">
                      单站模式下批量保存，按场景编码自动新增/更新
                    </span>
                  </div>
                </template>
                <p class="section-desc">
                  管理各类业务事件触发的邮件通知，开关关闭则该场景不发邮件。
                </p>
                <Skeleton
                  v-if="notificationLoading"
                  active
                  :paragraph="{ rows: 6 }"
                />
                <Empty
                  v-else-if="notificationList.length === 0"
                  description="暂无通知配置"
                />
                <div v-else class="notification-list">
                  <div
                    v-for="(item, idx) in notificationList"
                    :key="item.sceneCode"
                    class="notification-item"
                  >
                    <div class="notif-header">
                      <div class="notif-title">
                        <span class="notif-name">
                          {{ item.sceneName || item.sceneCode }}
                        </span>
                        <Tag color="blue">{{ item.sceneCode }}</Tag>
                      </div>
                      <div class="notif-switch">
                        <label class="form-label">启用</label>
                        <Switch
                          :checked="item.enabled === 1"
                          @change="
                            (v: any) =>
                              (notificationList[idx]!.enabled = v ? 1 : 0)
                          "
                        />
                      </div>
                    </div>
                    <div class="notif-body">
                      <div class="form-grid-2">
                        <div class="form-item">
                          <label class="form-label">收件人邮箱</label>
                          <Input
                            v-model:value="item.recipientEmails"
                            name="recipientEmails"
                            autocomplete="off"
                            placeholder="多个邮箱用英文逗号分隔…"
                            allow-clear
                          />
                        </div>
                        <div class="form-item">
                          <label class="form-label">通知渠道</label>
                          <Input
                            v-model:value="item.channels"
                            name="channels"
                            autocomplete="off"
                            placeholder="如：email"
                            allow-clear
                          />
                        </div>
                      </div>
                      <div class="form-item form-item-full" style="margin-top: 8px">
                        <label class="form-label">邮件主题</label>
                        <Input
                          v-model:value="item.emailSubject"
                          name="emailSubject"
                          autocomplete="off"
                          placeholder="邮件主题模板…"
                          allow-clear
                        />
                      </div>
                      <div class="form-item form-item-full" style="margin-top: 8px">
                        <label class="form-label">邮件正文</label>
                        <Input.TextArea
                          v-model:value="item.emailBody"
                          name="emailBody"
                          autocomplete="off"
                          :rows="3"
                          placeholder="邮件正文模板…"
                          allow-clear
                        />
                      </div>
                    </div>
                  </div>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- 代码设置 -->
            <Tabs.TabPane key="code">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.code" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[7]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="代码设置" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  添加统计代码和自定义 CSS/JS 来扩展网站功能。统计代码将插入到 &lt;/body&gt; 标签之前。
                </p>
                <h4 class="section-subtitle">统计代码</h4>
                <div class="form-item form-item-full">
                  <CodeEditor
                    v-model="formData.statisticsCode"
                    language="html"
                    height="180px"
                  />
                  <p class="field-tip">将被插入到网站 &lt;/body&gt; 之前，如百度统计、Google Analytics 等</p>
                </div>

                <div class="section-divider"></div>

                <h4 class="section-subtitle">自定义 CSS/JS</h4>
                <p class="section-desc" style="margin-bottom: 12px">
                  高级用户可添加自定义样式和脚本，修改网站外观或行为。
                </p>
                <div class="form-item form-item-full">
                  <label class="form-label">自定义CSS</label>
                  <CodeEditor
                    v-model="formData.customCss"
                    language="css"
                    height="180px"
                  />
                </div>
                <div class="form-item form-item-full" style="margin-top: 12px">
                  <label class="form-label">自定义JS</label>
                  <CodeEditor
                    v-model="formData.customJs"
                    language="javascript"
                    height="180px"
                  />
                </div>
              </Card>
              </div>
            </Tabs.TabPane>

            <!-- 其他设置 -->
            <Tabs.TabPane key="misc">
              <template #tab>
                <span class="tab-label">
                  <component :is="menuIcons.misc" class="tab-icon" aria-hidden="true" />
                  <span>{{ menuGroups[8]?.label }}</span>
                </span>
              </template>
              <div class="panel">
              <Card title="其他设置" :bordered="false" class="cfg-card">
                <p class="section-desc">
                  内部备注和搜索引擎爬虫控制规则。
                </p>
                <h4 class="section-subtitle">内部备注</h4>
                <div class="form-item form-item-full">
                  <label class="form-label">备注内容</label>
                  <Input.TextArea v-model:value="formData.remark" name="remark" autocomplete="off" :rows="2" placeholder="输入内部备注信息…" allow-clear />
                  <p class="field-tip">仅内部可见，不对外展示</p>
                </div>

                <div class="section-divider"></div>

                <h4 class="section-subtitle">robots.txt 设置</h4>
                <p class="section-desc" style="margin-bottom: 12px">
                  自定义 robots.txt 内容，控制搜索引擎爬虫的访问权限。留空则使用默认规则。可用占位符 <code>{domain}</code> 表示站点域名。
                </p>
                <div class="form-item form-item-full">
                  <CodeEditor
                    v-model="formData.robotsContent"
                    language="plaintext"
                    height="240px"
                  />
                  <p class="field-tip">
                    留空使用默认规则。可用占位符 <code>{domain}</code> 自动替换为站点域名。
                  </p>
                </div>
              </Card>
              </div>
            </Tabs.TabPane>
          </Tabs>
        </section>
      </div>

      <!-- 底部固定保存条 -->
      <div class="site-footer-bar">
        <div class="footer-tip">
          <span v-if="saving">保存中...</span>
          <span v-else>修改后请点击右侧按钮保存</span>
        </div>
        <div class="footer-actions">
          <Button :disabled="saving" @click="handleReset">重置</Button>
          <Button
            type="primary"
            :loading="saving"
            @click="handleSave"
          >
            保存设置
          </Button>
        </div>
      </div>

      <!-- 模板预览弹窗 -->
      <Modal
        v-model:open="previewModalVisible"
        title="模板演示预览"
        width="90%"
        :footer="null"
        destroy-on-close
      >
        <div class="preview-iframe-wrap">
          <iframe
            v-if="previewUrl"
            :src="previewUrl"
            class="preview-iframe"
            frameborder="0"
          />
        </div>
      </Modal>

      <!-- 模板选择弹窗 -->
      <Modal
        v-model:open="templateModalVisible"
        title="选择模板"
        width="80%"
        :footer="null"
        destroy-on-close
      >
        <div class="template-modal-search">
          <Input
            v-model:value="templateSearch"
            placeholder="搜索模板名称..."
            allow-clear
            style="width: 240px"
            @press-enter="openTemplateModal"
          />
        </div>
        <div class="template-modal-grid">
          <div v-if="templateLoading" class="template-grid-loading">
            <Skeleton active :paragraph="{ rows: 3 }" />
          </div>
          <template v-else>
            <div
              v-for="tpl in templateList"
              :key="tpl.id"
              class="tpl-card"
              :class="{
                active:
                  String(siteData.templateId) === String(tpl.id),
              }"
              @click="selectTemplate(tpl)"
            >
              <div class="tpl-card-img">
                <img :src="tpl.previewPic" :alt="tpl.name + ' 模板预览'" />
              </div>
              <div class="tpl-card-name">{{ tpl.name }}</div>
            </div>
            <div
              v-if="templateList.length === 0"
              class="template-grid-empty"
            >
              <Empty description="暂无可用模板" />
            </div>
          </template>
        </div>
      </Modal>
    </div>
  </Page>
</template>

<style scoped>
/* ========== 页面容器 ========== */
.site-settings-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: calc(100vh - 88px);
  background: hsl(var(--background-deep));
  transition: background 0.3s ease;
}

/* ========== 顶部状态条 ========== */
.site-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-left: 4px solid hsl(var(--primary));
  border-radius: 10px;
  margin-bottom: 14px;
  box-shadow: 0 1px 3px hsl(var(--foreground) / 0.04);
  transition: background 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
  position: relative;
  overflow: hidden;
}
/* 状态条顶部装饰光晕 */
.site-status-bar::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    hsl(var(--primary) / 0.3),
    transparent
  );
  opacity: 0.6;
}
.status-left {
  display: flex;
  align-items: center;
  gap: 14px;
}
.status-logo {
  border-radius: 8px;
  background: hsl(var(--muted));
  padding: 3px;
  object-fit: contain;
  box-shadow: 0 1px 4px hsl(var(--foreground) / 0.06);
}
.status-logo-placeholder {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(
    135deg,
    hsl(var(--primary) / 0.15),
    hsl(var(--primary) / 0.08)
  );
  color: hsl(var(--primary));
  border-radius: 8px;
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 1px;
}
.status-title {
  margin: 0 0 3px;
  font-size: 18px;
  font-weight: 600;
  color: hsl(var(--foreground));
  line-height: 1.3;
}
.status-meta {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}
.meta-item {
  position: relative;
}
.meta-item + .meta-item::before {
  content: '';
  display: inline-block;
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: hsl(var(--muted-foreground) / 0.4);
  margin-right: 14px;
  vertical-align: middle;
}
.status-right {
  display: flex;
  gap: 8px;
}

/* ========== 主体布局（Tab 模式） ========== */
.site-settings-body {
  flex: 1;
  display: flex;
  min-height: 0;
  padding-bottom: 64px;
}

/* Tab 容器 */
.site-tabs {
  width: 100%;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  padding: 4px 4px 20px;
  transition: background 0.3s ease, border-color 0.3s ease;
}
.site-tabs :deep(.ant-tabs-nav) {
  margin: 0 0 12px;
  padding: 0 12px;
  border-bottom: 1px solid hsl(var(--border));
}
.site-tabs :deep(.ant-tabs-nav-list) {
  gap: 4px;
}
.site-tabs :deep(.ant-tabs-tab) {
  padding: 12px 18px;
  margin: 0;
  border-radius: 8px 8px 0 0;
  color: hsl(var(--foreground) / 0.7);
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
}
.site-tabs :deep(.ant-tabs-tab:hover) {
  color: hsl(var(--foreground));
  background: hsl(var(--accent));
}
.site-tabs :deep(.ant-tabs-tab.ant-tabs-tab-active) {
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.06);
}
.site-tabs :deep(.ant-tabs-ink-bar) {
  background: hsl(var(--primary));
  height: 3px;
  border-radius: 3px 3px 0 0;
}
.site-tabs :deep(.ant-tabs-tab-btn) {
  outline: none;
}
.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
}
.tab-icon {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
  opacity: 0.85;
}

/* Tab 内容区 */
.site-content {
  flex: 1;
  min-width: 0;
}
.panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 0 16px;
}
.cfg-card {
  border-radius: 10px;
  transition: background 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
}
.cfg-card :deep(.ant-card) {
  border-radius: 10px;
  overflow: hidden;
}
.cfg-card :deep(.ant-card-head) {
  border-bottom-color: hsl(var(--border));
  padding: 0 20px;
  min-height: 48px;
}
.cfg-card :deep(.ant-card-head-title) {
  font-size: 15px;
  font-weight: 600;
  padding: 12px 0;
  color: hsl(var(--foreground));
}
.cfg-card :deep(.ant-card-body) {
  padding: 20px;
  color: hsl(var(--foreground));
}
.section-desc {
  margin: 0 0 14px;
  font-size: 13px;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
}

/* 卡片内子标题与分割线 */
.section-subtitle {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground) / 0.85);
  padding-bottom: 6px;
  border-bottom: 1px solid hsl(var(--border) / 0.5);
}
.section-divider {
  height: 1px;
  background: hsl(var(--border) / 0.4);
  margin: 18px 0;
}

/* ========== 表单 ========== */
.form-grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px 24px;
}

/* 状态设置：水平紧凑布局 */
.status-inline-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 20px 40px;
}
.status-inline-item {
  display: flex;
  align-items: center;
  gap: 12px;
}
.status-inline-item .form-label {
  white-space: nowrap;
  flex-shrink: 0;
}
.form-item-full {
  width: 100%;
}
.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-label {
  font-size: 14px;
  font-weight: 500;
  color: hsl(var(--foreground) / 0.88);
  transition: color 0.3s ease;
}
.form-label .req {
  color: hsl(var(--destructive));
  margin-left: 2px;
}
.form-label-tip {
  margin-left: 6px;
  font-size: 12px;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
}
.form-extra {
  margin-top: 4px;
  font-size: 12px;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
}
.field-tip {
  margin: 4px 0 0;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}
.field-tip code {
  background: hsl(var(--muted));
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 12px;
  color: hsl(var(--primary));
  margin: 0 2px;
  font-family: 'Consolas', 'Monaco', 'SF Mono', monospace;
}

/* URL 规则提示 */
.url-rule-tips {
  margin-top: 16px;
  padding: 14px 18px;
  background: hsl(var(--muted));
  border-radius: 8px;
  font-size: 13px;
  transition: background 0.3s ease;
}
.tip-row {
  display: flex;
  gap: 8px;
  padding: 4px 0;
}
.tip-row strong {
  color: hsl(var(--foreground) / 0.75);
  flex-shrink: 0;
  width: 90px;
}
.tip-row span {
  color: hsl(var(--muted-foreground));
  font-family: 'Consolas', 'Monaco', 'SF Mono', monospace;
  font-size: 12px;
}

/* 代码编辑 */
.code-editor :deep(.ant-input) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
}

/* ========== 上传 ========== */
.upload-plus {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: hsl(var(--muted-foreground));
  transition: color 0.2s ease;
}
.upload-plus:hover {
  color: hsl(var(--primary));
}
.upload-icon {
  font-size: 22px;
  line-height: 1;
  margin-bottom: 2px;
}
.upload-text {
  font-size: 12px;
}
.upload-tip {
  margin: 6px 0 0;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ========== 模板卡片（当前模板详情区域） ========== */
.current-template-card {
  display: flex;
  gap: 20px;
  padding: 20px;
  background: hsl(var(--accent));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  transition: background 0.3s ease, border-color 0.3s ease;
}
.tpl-preview {
  width: 240px;
  flex-shrink: 0;
  aspect-ratio: 4 / 3;
  border-radius: 8px;
  overflow: hidden;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  box-shadow: 0 1px 4px hsl(var(--foreground) / 0.06);
}
.tpl-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}
.tpl-preview:hover img {
  transform: scale(1.03);
}
.tpl-preview-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: hsl(var(--muted-foreground));
  font-size: 14px;
}
.tpl-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}
.tpl-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: hsl(var(--foreground));
}
.tpl-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.meta-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
}
.meta-label {
  color: hsl(var(--muted-foreground));
  flex-shrink: 0;
}
.meta-value {
  color: hsl(var(--foreground) / 0.75);
  word-break: break-all;
}
.meta-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.tpl-actions {
  display: flex;
  gap: 10px;
  margin-top: auto;
}

/* ========== 通知配置 ========== */
.card-title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.card-title-tip {
  font-size: 12px;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
}
.notification-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.notification-item {
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  padding: 18px;
  background: hsl(var(--card));
  transition: background 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
  position: relative;
}
.notification-item:hover {
  border-color: hsl(var(--border));
  box-shadow: 0 2px 8px hsl(var(--foreground) / 0.05);
}
.notif-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
  padding-bottom: 10px;
  border-bottom: 1px dashed hsl(var(--border));
}
.notif-title {
  display: flex;
  align-items: center;
  gap: 8px;
}
.notif-name {
  font-size: 14px;
  font-weight: 500;
  color: hsl(var(--foreground) / 0.88);
}
.notif-switch {
  display: flex;
  align-items: center;
  gap: 8px;
}
.notif-body {
  display: flex;
  flex-direction: column;
}

/* ========== 底部固定条 ========== */
.site-footer-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: hsl(var(--card));
  border-top: 1px solid hsl(var(--border));
  box-shadow: 0 -2px 12px hsl(var(--foreground) / 0.06);
  z-index: 100;
  transition: background 0.3s ease, border-color 0.3s ease;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.footer-tip {
  font-size: 13px;
  color: hsl(var(--muted-foreground));
}
.footer-actions {
  display: flex;
  gap: 8px;
}

/* 预览iframe */
.preview-iframe-wrap {
  min-height: 600px;
}
.preview-iframe {
  width: 100%;
  height: 75vh;
  border: none;
  background: hsl(var(--background));
  border-radius: 6px;
}

/* 模板弹窗 */
.template-modal-search {
  margin-bottom: 16px;
  display: flex;
  justify-content: flex-end;
}
.template-modal-grid {
  min-height: 300px;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
.tpl-card {
  display: flex;
  flex-direction: column;
  width: calc(25% - 9px);
  border: 2px solid transparent;
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.25s ease;
  background: hsl(var(--card));
}
.tpl-card:hover {
  border-color: hsl(var(--primary) / 0.35);
  box-shadow: 0 6px 20px hsl(var(--foreground) / 0.08);
  transform: translateY(-2px);
}
.tpl-card.active {
  border-color: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 0.15);
}
.tpl-card-img {
  aspect-ratio: 4 / 3;
  overflow: hidden;
  background: hsl(var(--muted));
}
.tpl-card-img img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}
.tpl-card:hover .tpl-card-img img {
  transform: scale(1.05);
}
.tpl-card-name {
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 500;
  text-align: center;
  color: hsl(var(--foreground));
  border-top: 1px solid hsl(var(--border));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.template-grid-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  width: 100%;
}

/* ========== 响应式 ========== */
@media (max-width: 1200px) {
  .form-grid-2 {
    grid-template-columns: 1fr;
  }
  .tpl-card {
    width: calc(33.333% - 8px);
  }
}
@media (max-width: 768px) {
  .site-status-bar {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  .status-right {
    width: 100%;
    justify-content: flex-end;
  }
  .site-tabs :deep(.ant-tabs-nav) {
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0 8px;
  }
  .site-tabs :deep(.ant-tabs-nav-list) {
    flex-wrap: nowrap;
  }
  .site-tabs :deep(.ant-tabs-tab) {
    padding: 10px 14px;
    font-size: 13px;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .tab-icon {
    display: none;
  }
  .panel {
    padding: 0 8px;
  }
  .tpl-card {
    width: calc(50% - 6px);
  }
  .current-template-card {
    flex-direction: column;
  }
  .tpl-preview {
    width: 100%;
  }
}
</style>
