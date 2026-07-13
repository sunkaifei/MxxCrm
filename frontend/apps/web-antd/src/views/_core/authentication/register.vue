<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';
import type { Recordable } from '@vben/types';

import { computed, h, ref } from 'vue';

import { AuthenticationRegister, z } from '@vben/common-ui';
import { $t } from '@vben/locales';

import { checkUsernameApi, registerApi } from '#/api/core/auth';
import { useAuthStore } from '#/store';

defineOptions({ name: 'Register' });

const authStore = useAuthStore();

const usernameChecking = ref(false);
const agreePolicy = ref(true);

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
        .regex(/^[a-zA-Z0-9_]+$/, { message: '用户名只能包含字母、数字和下划线' }),
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
  
  await authStore.authLogin({
    username: value.username,
    password: value.password,
  });
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
