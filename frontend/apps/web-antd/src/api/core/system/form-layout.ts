import { requestClient } from '#/api/request';

/** 布局元数据（与后端 mxx_system_form_layout 对应） */
export interface FormLayoutVO {
  id?: number;
  module: string;
  layoutType: number;
  layoutJson: any;
  version: number;
  status: number;
  updateTime?: string;
}

/** 取模块布局：管理端指定 roleKey 取该角色作用域布局；运行端（布局块）不传=按当前用户角色解析 */
export const getFormLayoutApi = async (
  module: string,
  layoutType: number,
  roleKey?: string,
) => {
  return requestClient.get('/api/system/form-layout/get', {
    params: { module, layoutType, roleKey: roleKey || undefined },
  });
};

/** 保存布局（upsert，version 乐观锁；roleKey 空=默认布局，否则角色专属布局） */
export const saveFormLayoutApi = async (data: {
  module: string;
  layoutType: number;
  layoutJson: any;
  version?: number;
  roleKey?: string;
}) => {
  return requestClient.post('/api/system/form-layout/save', data);
};

/** 恢复布局（roleKey 空=恢复默认布局，否则恢复该角色专属布局） */
export const resetFormLayoutApi = async (
  module: string,
  layoutType: number,
  roleKey?: string,
) => {
  return requestClient.post('/api/system/form-layout/reset', null, {
    params: { module, layoutType, roleKey: roleKey || undefined },
  });
};
