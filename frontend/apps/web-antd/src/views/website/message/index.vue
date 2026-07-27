<script lang="ts" setup>
import { Page } from '@vben/common-ui';
import { Empty } from 'ant-design-vue';
import { $t } from '#/locales';

defineOptions({ name: 'WebsiteMessage' });
</script>

<template>
  <Page>
    <div class="flex h-[60vh] items-center justify-center">
      <Empty :description="$t('page.website.messageTitle') + ' 开发中'" />
    </div>
  </Page>
</template>
