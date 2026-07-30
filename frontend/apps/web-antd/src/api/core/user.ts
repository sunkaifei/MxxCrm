import type { UserInfo } from '@vben/types';

import { requestClient } from '#/api/request';

/**
 * 获取用户信息
 *
 * 后端 UserLoginVO 返回字段为 id/nickname，
 * 前端 BasicUserInfo 期望 userId/realName，在此做字段映射。
 */
export async function getUserInfoApi() {
  const res: any = await requestClient.get('/api/system/admin/userinfo');
  return {
    ...res,
    userId: res?.userId ?? String(res?.id ?? ''),
    realName: res?.realName ?? res?.nickname ?? res?.username ?? '',
  } as UserInfo;
}

/**
 * 更新当前登录用户头像
 *
 * 头像文件上传成功后调用，将访问地址（含缓存破坏版本号）持久化到后端用户记录，
 * 使刷新页面后仍能读到最新头像。
 */
export async function updateAvatarApi(avatar: string) {
  return requestClient.put<string>('/api/system/admin/avatar', { avatar });
}
