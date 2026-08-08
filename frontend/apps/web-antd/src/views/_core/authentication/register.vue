<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';
import type { Recordable } from '@vben/types';

import { computed, h, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { AuthenticationRegister, z } from '@vben/common-ui';
import { $t } from '@vben/locales';

import { getRegisterStatusApi } from '#/api';
import { checkUsernameApi, registerApi } from '#/api/core/auth';
import { useAuthStore } from '#/store';

defineOptions({ name: 'Register' });

const authStore = useAuthStore();
const router = useRouter();

const usernameChecking = ref(false);
const agreePolicy = ref(true);

// 注册开关：关闭时重定向回登录页
onMounted(async () => {
  try {
    const data = await getRegisterStatusApi();
    if (!data?.registerEnabled) {
      router.replace('/auth/login');
    }
  } catch {
    router.replace('/auth/login');
  }
});

const formSchema = computed((): VbenFormSchema[] => {
  return [
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: $t('authentication.usernameTip'),
        allowClear: true,
      },
      fieldName: 'username',
      label: $t('authentication.username'),
      rules: z.string()
        .min(3, { message: '用户名至少需要3个字符' })
        .max(20, { message: '用户名不能超过20个字符' })
        .regex(/^[a-zA-Z0-9_]+$/, { message: '用户名只能包含字母、数字和下划线，不能包含空格' }),
    },
    {
      component: 'VbenInputPassword',
      componentProps: {
        passwordStrength: true,
        placeholder: $t('authentication.password'),
      },
      fieldName: 'password',
      label: $t('authentication.password'),
      renderComponentContent() {
        return {
          strengthText: () => $t('authentication.passwordStrength'),
        };
      },
      rules: z.string()
        .min(6, { message: '密码至少需要6个字符' })
        .max(32, { message: '密码不能超过32个字符' }),
    },
    {
      component: 'VbenInputPassword',
      componentProps: {
        placeholder: $t('authentication.confirmPassword'),
      },
      fieldName: 'confirmPassword',
      label: $t('authentication.confirmPassword'),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入姓名',
        allowClear: true,
      },
      fieldName: 'nickName',
      label: '姓名',
      rules: z.string().min(1, { message: '姓名不能为空' }),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入手机号',
        allowClear: true,
      },
      fieldName: 'mobile',
      label: '手机号',
      rules: z.string().min(1, { message: '手机号不能为空' }).regex(/^1[3-9]\d{9}$/, { message: '请输入正确的手机号' }),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入申请部门',
        allowClear: true,
      },
      fieldName: 'deptName',
      label: '部门',
      rules: z.string().min(1, { message: '部门不能为空' }),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入申请岗位',
        allowClear: true,
      },
      fieldName: 'postName',
      label: '岗位',
      rules: z.string().min(1, { message: '岗位不能为空' }),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入邮箱（选填）',
        allowClear: true,
      },
      fieldName: 'email',
      label: '邮箱',
    },
    {
      component: 'VbenCheckbox',
      fieldName: 'agreePolicy',
      defaultValue: true,
      renderComponentContent: () => ({
        default: () =>
          h('span', [
            $t('authentication.agree'),
            h(
              'a',
              {
                class: 'vben-link ml-1 ',
                href: '',
              },
              `${$t('authentication.privacyPolicy')} & ${$t('authentication.terms')}`,
            ),
          ]),
      }),
    },
  ];
});

async function handleSubmit(value: Recordable<any>) {
  if (!value.agreePolicy) {
    return;
  }
  
  if (value.password !== value.confirmPassword) {
    return;
  }
  
  const username = value.username;
  if (username && username.length >= 3) {
    usernameChecking.value = true;
    try {
      const result = await checkUsernameApi(username);
      if (result.exists) {
        return;
      }
    } finally {
      usernameChecking.value = false;
    }
  }
  
  await registerApi(value);

  // 注册用户待审核（status=0），不自动登录，提示后跳转登录页
  window.$message?.success('注册成功，请等待管理员审核通过后登录');
  router.replace('/auth/login');
}
</script>

<template>
  <AuthenticationRegister
    :form-schema="formSchema"
    :loading="authStore.loginLoading || usernameChecking"
    :submit-button-disabled="!agreePolicy"
    @submit="handleSubmit"
  />
</template>
