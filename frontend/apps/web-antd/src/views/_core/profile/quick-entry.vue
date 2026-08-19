<script lang="ts" setup>
import { Card, Col, Row } from 'ant-design-vue';
import { useRouter } from 'vue-router';

import { $t } from '#/locales';

const router = useRouter();

const entries = [
  {
    key: 'payslip',
    title: $t('page.system.profile.entryPayslip'),
    desc: $t('page.system.profile.entryPayslipDesc'),
    path: '/finance/payslip',
  },
  {
    key: 'attendance',
    title: $t('page.system.profile.entryAttendance'),
    desc: $t('page.system.profile.entryAttendanceDesc'),
    path: '/finance/attendance',
  },
  {
    key: 'security',
    title: $t('page.system.profile.entrySecurity'),
    desc: $t('page.system.profile.entrySecurityDesc'),
    action: 'password' as const,
  },
];

function handleGo(entry: (typeof entries)[number]) {
  if (entry.action === 'password') {
    router.push({ path: '/profile', query: { tab: 'password' } });
    window.location.hash = '';
    // 触发父组件切到密码 tab：通过 hash 事件简单处理
    window.dispatchEvent(new CustomEvent('profile:switch-password'));
  } else if (entry.path) {
    router.push(entry.path);
  }
}
</script>

<template>
  <Row :gutter="16">
    <Col v-for="e in entries" :key="e.key" :xs="24" :md="8">
      <Card hoverable size="small" class="entry-card" @click="handleGo(e)">
        <div class="entry-title">{{ e.title }}</div>
        <div class="entry-desc">{{ e.desc }}</div>
      </Card>
    </Col>
  </Row>
</template>

<style scoped>
.entry-card {
  cursor: pointer;
  transition: all 0.2s;
}

.entry-title {
  font-size: 15px;
  font-weight: 600;
}

.entry-desc {
  margin-top: 4px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
}
</style>
