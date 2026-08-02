<!--
  轻量页面使用说明组件（全局通用）
  - 默认收起：左侧图标+标题+一行简介，右侧展开按钮
  - 展开后：通过默认 slot 渲染步骤/说明内容（由调用方传入）
  - 用法：
    <PageUsageGuide :title="..." :brief="...">
      <div v-for="..." class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">{{ $t(...) }}</div>
          <div class="page-guide-step-desc">{{ $t(...) }}</div>
        </div>
      </div>
    </PageUsageGuide>
-->
<template>
  <div class="page-guide-wrapper">
    <!-- 头部：标题 + 简介 + 展开按钮（一行） -->
    <div class="page-guide-header">
      <div class="flex items-center gap-2 flex-shrink-0">
        <LucideInfo class="guide-icon" :style="{ color: 'hsl(var(--primary))' }" />
        <span class="guide-title">{{ title }}</span>
      </div>
      <div class="flex items-center flex-1 min-w-0 guide-brief-wrapper">
        <span v-if="brief" class="guide-brief">{{ brief }}</span>
      </div>
      <Button
        type="link"
        size="small"
        class="!p-0 !h-auto flex-shrink-0"
        @click="expanded = !expanded"
      >
        {{ expanded ? collapseText : expandText }}
        <component :is="expanded ? LucideChevronUp : LucideChevronDown" class="guide-arrow" />
      </Button>
    </div>

    <!-- 展开态：步骤说明（通过 slot 传入，支持任意结构） -->
    <div v-show="expanded" class="page-guide-body">
      <div class="page-guide-steps">
        <slot />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { Button } from 'ant-design-vue';
import { LucideInfo, LucideChevronDown, LucideChevronUp } from '@vben/icons';

withDefaults(
  defineProps<{
    title: string;
    brief?: string;
    expandText?: string;
    collapseText?: string;
  }>(),
  {
    brief: '',
    expandText: '展开说明',
    collapseText: '收起',
  },
);

const expanded = ref(false);
</script>

<style scoped>
.page-guide-wrapper {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  padding: 10px 14px;
  box-shadow: 0 1px 2px hsl(0 0% 0% / 0.03);
  margin-bottom: 12px;
}

.page-guide-header {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 28px;
}

.guide-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.guide-title {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
  white-space: nowrap;
}

.guide-brief-wrapper {
  overflow: hidden;
}

.guide-brief {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

.guide-arrow {
  width: 14px;
  height: 14px;
  display: inline-block;
  vertical-align: middle;
}

.page-guide-body {
  animation: guideFadeSlide 0.2s ease-out;
  padding-top: 10px;
  margin-top: 8px;
  border-top: 1px dashed hsl(var(--border));
}

@keyframes guideFadeSlide {
  from {
    opacity: 0;
    transform: translateY(-6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.page-guide-steps {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

@media (max-width: 768px) {
  .page-guide-header {
    flex-wrap: wrap;
  }
  .guide-brief-wrapper {
    order: 3;
    width: 100%;
    flex: 0 0 100%;
  }
  .guide-brief {
    white-space: normal;
  }
}

/* ===== 步骤项样式（供 slot 子元素复用，因 scoped 需用 :deep） ===== */
.page-guide-steps :deep(.page-guide-step-item) {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.page-guide-steps :deep(.page-guide-step-index) {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: hsl(var(--primary) / 0.1);
  color: hsl(var(--primary));
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.page-guide-steps :deep(.page-guide-step-content) {
  flex: 1;
  min-width: 0;
}

.page-guide-steps :deep(.page-guide-step-title) {
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
  line-height: 1.5;
}

.page-guide-steps :deep(.page-guide-step-desc) {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  line-height: 1.6;
  margin-top: 2px;
}
</style>
