<script lang="ts" setup>
import { computed, defineAsyncComponent, ref } from 'vue';
import { message, Select as ASelect, Tag, Button, Popconfirm } from 'ant-design-vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import {
  createNoticeApi,
  updateNoticeApi,
  publishNoticeApi,
  revokeNoticeApi,
  NOTICE_TYPE_OPTIONS,
  NOTICE_LEVEL_OPTIONS,
  NOTICE_TARGET_TYPE_OPTIONS,
  NOTICE_PUBLISH_STATUS,
} from '#/api';
import { getColleagueListApi } from '#/api/core/message/chat';

// 异步加载富文本编辑器
const RichTextEditor = defineAsyncComponent(
  () => import('#/components/RichTextEditor/index.vue'),
);

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.notice.title') })
    : $t('ui.modal.update', { moduleName: $t('page.system.notice.title') }),
);

// 当前编辑行的发布状态（编辑模式才有）
const rowPublishStatus = ref<number | undefined>(undefined);
const rowPublisherName = ref<string>('');
const rowPublishTime = ref<string>('');
const rowRevokeTime = ref<string>('');
const rowCreateTime = ref<string>('');

// 是否已发布（用于控制可编辑性）
const isPublished = computed(() => rowPublishStatus.value === NOTICE_PUBLISH_STATUS.PUBLISHED);
// 是否已撤回
const isRevoked = computed(() => rowPublishStatus.value === NOTICE_PUBLISH_STATUS.REVOKED);
// 是否禁止编辑（已发布状态不可编辑）
const isReadonly = computed(() => isPublished.value);

// 富文本内容单独管理
const richContent = ref('');
const targetType = ref<number>(1);
const targetUserIds = ref<string[]>([]);
// 本地 loading 状态（用于 footer 按钮反馈）
const saving = ref(false);

// 同事列表
const colleagueOptions = ref<Array<{ label: string; value: string }>>([]);
const colleagueLoading = ref(false);

async function loadColleagues() {
  colleagueLoading.value = true;
  try {
    const res: any = await getColleagueListApi({ page: 1, pageSize: 500 });
    const list = Array.isArray(res) ? res : (res?.list || res?.records || []);
    colleagueOptions.value = list.map((u: any) => {
      const name = u.nickName || u.userName || `用户${u.id}`;
      return {
        label: u.depts?.[0]?.deptName ? `${name}（${u.depts[0].deptName}）` : name,
        value: String(u.id),
      };
    });
  } catch (e) {
    console.error('加载同事列表失败', e);
  } finally {
    colleagueLoading.value = false;
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
      fieldName: 'title',
      label: '公告标题',
      componentProps: {
        placeholder: '请输入公告标题',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入公告标题' }),
    },
    {
      component: 'Select',
      fieldName: 'type',
      label: '公告类型',
      defaultValue: 2,
      componentProps: {
        placeholder: '请选择公告类型',
        allowClear: true,
        options: NOTICE_TYPE_OPTIONS,
      },
      rules: z.any().refine((v) => v != null, { message: '请选择公告类型' }),
    },
    {
      component: 'Select',
      fieldName: 'level',
      label: '公告等级',
      defaultValue: 'normal',
      componentProps: {
        placeholder: '请选择公告等级',
        allowClear: true,
        options: NOTICE_LEVEL_OPTIONS,
      },
      rules: z.any().refine((v) => v != null, { message: '请选择公告等级' }),
    },
    {
      component: 'RadioGroup',
      fieldName: 'targetType',
      label: '目标用户',
      defaultValue: 1,
      componentProps: {
        optionType: 'button',
        options: NOTICE_TARGET_TYPE_OPTIONS,
        // ant-design-vue RadioGroup @change 事件参数为 RadioChangeEvent，需取 e.target.value
        onChange: (e: any) => {
          const val = e?.target?.value ?? e;
          targetType.value = Number(val) || 1;
        },
      },
      rules: 'selectRequired',
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  // 抽屉宽度 75%
  class: 'w-[75%]! max-w-[75%]!',
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    // 默认确认按钮 = 保存（不发布）
    await handleSave(false);
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};

      // 同步发布信息（编辑模式）
      rowPublishStatus.value = row.publishStatus;
      rowPublisherName.value = row.publisherName || '';
      rowPublishTime.value = row.publishTime || '';
      rowRevokeTime.value = row.revokeTime || '';
      rowCreateTime.value = row.createTime || '';

      // 重置状态
      richContent.value = row.content || '';
      targetType.value = Number(row.targetType) || 1;
      targetUserIds.value = row.targetUserIds
        ? String(row.targetUserIds)
            .split(',')
            .map((s: string) => s.trim())
            .filter(Boolean)
        : [];

      // 设置表单数据
      baseFormApi.setValues({
        title: row.title,
        type: row.type ?? 2,
        level: row.level ?? 'normal',
        targetType: targetType.value,
      });

      // 加载同事列表
      if (colleagueOptions.value.length === 0) {
        loadColleagues();
      }

      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

// 统一保存：saveAndPublish=true 则保存后立即发布
async function handleSave(saveAndPublish: boolean) {
  // 已发布状态禁止编辑
  if (isReadonly.value) {
    message.warning('已发布的公告不可编辑，请先撤回');
    return;
  }

  const validate = await baseFormApi.validate();
  if (!validate.valid) {
    return;
  }

  // 校验指定用户
  if (targetType.value === 2 && targetUserIds.value.length === 0) {
    message.warning('请至少选择一个目标用户');
    return;
  }

  // 校验富文本内容
  const contentText = richContent.value?.replace(/<[^>]+>/g, '').trim() || '';
  if (!contentText) {
    message.warning('请输入公告内容');
    return;
  }

  setLoading(true);
  saving.value = true;

  const values = await baseFormApi.getValues();

  try {
    const payload = {
      title: values.title,
      type: values.type,
      level: values.level,
      targetType: targetType.value,
      targetUserIds:
        targetType.value === 2 ? targetUserIds.value.join(',') : '',
      content: richContent.value,
    };

    // 1. 保存（新建或更新）
    const isCreateMode = data.value?.create;
    let savedId: number | undefined;

    if (isCreateMode) {
      const res: any = await createNoticeApi(payload);
      savedId = typeof res === 'number' ? res : res?.id;
    } else {
      await updateNoticeApi(data.value.row.id, payload);
      savedId = data.value.row.id;
    }

    // 2. 如需发布，调用发布接口
    if (saveAndPublish && savedId) {
      await publishNoticeApi(savedId);
      message.success(isCreateMode ? '公告已创建并发布' : '公告已更新并发布');
    } else {
      message.success(
        isCreateMode
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
    }

    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch {
    // 错误由全局拦截器处理
  } finally {
    setLoading(false);
    saving.value = false;
  }
}

// 撤回
async function handleRevoke() {
  if (!data.value?.row?.id) return;
  setLoading(true);
  saving.value = true;
  try {
    await revokeNoticeApi(data.value.row.id);
    message.success('公告已撤回');
    rowPublishStatus.value = NOTICE_PUBLISH_STATUS.REVOKED;
    drawerApi.setData({ needRefresh: true });
  } catch {
    // 错误由全局拦截器处理
  } finally {
    setLoading(false);
    saving.value = false;
  }
}

// 发布状态徽标
const publishStatusMeta = computed(() => {
  switch (rowPublishStatus.value) {
    case NOTICE_PUBLISH_STATUS.PUBLISHED:
      return { text: '已发布', color: 'success', dot: '#52c41a' };
    case NOTICE_PUBLISH_STATUS.REVOKED:
      return { text: '已撤回', color: 'warning', dot: '#faad14' };
    default:
      return { text: '未发布', color: 'default', dot: '#bfbfbf' };
  }
});

// 等级徽标
const levelMeta = computed(() => {
  const values = baseFormApi.form.values;
  const level = values?.level;
  switch (level) {
    case 'urgent':
      return { text: '紧急', color: '#f5222d' };
    case 'high':
      return { text: '高', color: '#fa8c16' };
    case 'normal':
      return { text: '普通', color: '#1890ff' };
    case 'low':
      return { text: '低', color: '#8c8c8c' };
    default:
      return null;
  }
});

// 指定用户已选数量
const selectedUserCount = computed(() => targetUserIds.value.length);
</script>

<template>
  <Drawer :title="getTitle">
    <div class="notice-editor">
      <!-- 发布信息卡（仅编辑模式且已有发布记录时显示） -->
      <section
        v-if="!isCreate && rowPublishStatus !== undefined"
        class="info-card info-card--publish"
      >
        <header class="info-card__header">
          <div class="info-card__title">
            <span class="info-card__title-text">发布信息</span>
            <Tag :color="publishStatusMeta.color" class="info-card__status">
              <span class="status-dot" :style="{ background: publishStatusMeta.dot }"></span>
              {{ publishStatusMeta.text }}
            </Tag>
          </div>
        </header>
        <div class="info-card__body">
          <div class="info-item">
            <span class="info-item__label">发布人</span>
            <span class="info-item__value">
              {{ rowPublisherName || '—' }}
            </span>
          </div>
          <div class="info-item">
            <span class="info-item__label">发布时间</span>
            <span class="info-item__value">
              {{ rowPublishTime || '—' }}
            </span>
          </div>
          <div class="info-item" v-if="rowRevokeTime">
            <span class="info-item__label">撤回时间</span>
            <span class="info-item__value info-item__value--warning">
              {{ rowRevokeTime }}
            </span>
          </div>
          <div class="info-item">
            <span class="info-item__label">创建时间</span>
            <span class="info-item__value info-item__value--muted">
              {{ rowCreateTime || '—' }}
            </span>
          </div>
        </div>
      </section>

      <!-- 基础信息卡 -->
      <section class="info-card">
        <header class="info-card__header">
          <div class="info-card__title">
            <span class="info-card__title-text">基础信息</span>
            <span
              v-if="levelMeta"
              class="level-badge"
              :style="{ background: `${levelMeta.color}1a`, color: levelMeta.color, borderColor: `${levelMeta.color}40` }"
            >
              {{ levelMeta.text }}
            </span>
            <span v-if="isReadonly" class="readonly-hint">已发布，不可编辑</span>
          </div>
        </header>
        <div class="info-card__body info-card__body--form">
          <BaseForm />
        </div>

        <!-- 指定用户选择器（仅当 targetType=2 时显示） -->
        <transition name="slide-fade">
          <div v-if="targetType === 2" class="user-selector">
            <div class="user-selector__header">
              <span class="user-selector__label">
                <span class="required-mark">*</span> 目标用户
              </span>
              <span v-if="selectedUserCount > 0" class="user-selector__count">
                已选 {{ selectedUserCount }} 人
              </span>
            </div>
            <ASelect
              v-model:value="targetUserIds"
              mode="multiple"
              :options="colleagueOptions"
              :loading="colleagueLoading"
              placeholder="请选择目标用户（可多选，支持搜索）"
              :filter-option="(input: string, option: any) =>
                option.label.toLowerCase().includes(input.toLowerCase())
              "
              style="width: 100%"
              :max-tag-count="15"
              :disabled="isReadonly"
            />
          </div>
        </transition>
      </section>

      <!-- 公告内容卡 -->
      <section class="info-card">
        <header class="info-card__header">
          <div class="info-card__title">
            <span class="info-card__title-text">公告内容</span>
            <span class="info-card__hint">支持富文本格式</span>
          </div>
        </header>
        <div class="info-card__body info-card__body--editor">
          <RichTextEditor
            v-model="richContent"
            placeholder="请输入公告内容..."
            :height="480"
          />
        </div>
      </section>
    </div>

    <!-- 自定义底部操作区 -->
    <template #footer>
      <div class="drawer-footer">
        <div class="drawer-footer__left">
          <!-- 撤回按钮（仅已发布状态） -->
          <Popconfirm
            v-if="isPublished"
            title="确认撤回该公告？撤回后用户将无法查看"
            ok-text="撤回"
            cancel-text="取消"
            @confirm="handleRevoke"
          >
            <Button danger :loading="saving">
              撤回公告
            </Button>
          </Popconfirm>
        </div>

        <div class="drawer-footer__right">
          <Button @click="drawerApi.close()">取消</Button>

          <!-- 已发布状态只显示关闭 -->
          <Button v-if="isPublished" type="primary" @click="drawerApi.close()">
            关闭
          </Button>

          <!-- 未发布/已撤回状态：保存 + 保存并发布 -->
          <template v-else>
            <Button
              :loading="saving"
              @click="handleSave(false)"
            >
              {{ isCreate ? '保存草稿' : '保存' }}
            </Button>
            <Button
              type="primary"
              :loading="saving"
              @click="handleSave(true)"
            >
              {{ isRevoked ? '重新发布' : '保存并发布' }}
            </Button>
          </template>
        </div>
      </div>
    </template>
  </Drawer>
</template>

<style scoped>
.notice-editor {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 4px 0;
}

/* 卡片基础样式 */
.info-card {
  background: #ffffff;
  border: 1px solid #eef0f3;
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.2s ease;
}

.info-card:hover {
  border-color: #d9dde3;
}

/* 发布信息卡：用淡蓝底色区分 */
.info-card--publish {
  background: linear-gradient(180deg, #f8faff 0%, #ffffff 100%);
  border-color: #d6e4ff;
}

.info-card__header {
  padding: 14px 18px 8px;
  border-bottom: 1px solid #f5f5f5;
  background: transparent;
}

.info-card__title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.info-card__title-text {
  font-size: 14px;
  font-weight: 600;
  color: #1a1a1a;
  letter-spacing: 0.2px;
}

.info-card__hint {
  font-size: 12px;
  color: #8c8c8c;
  font-weight: 400;
}

.info-card__status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 10px;
  font-size: 12px;
  border-radius: 10px;
  margin-left: 4px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}

.level-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid;
  font-weight: 500;
}

.readonly-hint {
  font-size: 12px;
  color: #fa8c16;
  margin-left: auto;
  font-weight: 400;
}

.info-card__body {
  padding: 14px 18px 18px;
}

.info-card__body--form {
  /* vben form 内边距调整 */
}

.info-card__body--editor {
  padding: 14px 18px 18px;
}

/* 发布信息项 */
.info-item {
  display: flex;
  align-items: center;
  padding: 8px 0;
  font-size: 13px;
  border-bottom: 1px dashed #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-item__label {
  width: 80px;
  color: #8c8c8c;
  flex-shrink: 0;
}

.info-item__value {
  color: #262626;
  font-weight: 500;
}

.info-item__value--warning {
  color: #fa8c16;
}

.info-item__value--muted {
  color: #bfbfbf;
  font-weight: 400;
}

/* 指定用户选择器 */
.user-selector {
  margin: 12px 18px 18px;
  padding: 14px;
  background: #fafbfc;
  border: 1px dashed #d9dde3;
  border-radius: 6px;
}

.user-selector__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.user-selector__label {
  font-size: 13px;
  color: #262626;
  font-weight: 500;
}

.required-mark {
  color: #f5222d;
  margin-right: 2px;
}

.user-selector__count {
  font-size: 12px;
  color: #1890ff;
  background: #e6f7ff;
  padding: 2px 8px;
  border-radius: 10px;
}

/* 过渡动画 */
.slide-fade-enter-active {
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-fade-leave-active {
  transition: all 0.2s ease;
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
  max-height: 0;
}

.slide-fade-enter-to,
.slide-fade-leave-from {
  opacity: 1;
  transform: translateY(0);
  max-height: 500px;
}

/* 底部操作区 */
.drawer-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid #f0f0f0;
  background: #fafbfc;
}

.drawer-footer__left,
.drawer-footer__right {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
