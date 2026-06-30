import type { UserInfo } from '@vben/types';

import { requestClient } from '#/api/request';

/**
 * 获取用户信息
 */
export async function getUserInfoApi() {
  return requestClient.get<UserInfo>('/api/system/admin/userinfo');
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
