/**
 * 超级管理员业务操作拦截
 *
 * 设计原则（参照钉钉/飞书/用友/金蝶）：
 * - 超级管理员负责系统管理（用户/角色/部门/审批流配置等），不参与业务操作
 * - 超管可查看全部数据，但不能创建业务单据
 * - 基础数据（客户/联系人）超管可录入，但必须分配给业务人员
 *
 * 使用方式：
 *   import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
 *   const { isSuperAdmin, guardBusiness, guardAssignTo } = useSuperAdminGuard();
 *
 *   // 拦截业务操作（商机/合同/订单/报价单）
 *   function handleCreate() {
 *     if (guardBusiness('商机')) return;
 *     // 正常逻辑...
 *   }
 *
 *   // 基础数据允许创建，但提示分配负责人
 *   function handleSave() {
 *     if (guardAssignTo('客户')) return;
 *     // 正常逻辑...
 *   }
 */

import { computed } from 'vue';

import { useUserStore } from '@vben/stores';

import { message, Modal } from 'ant-design-vue';

export function useSuperAdminGuard() {
  const userStore = useUserStore();
  const userInfo = userStore.userInfo as any;

  /**
   * 是否是超级管理员
   * 判断条件：data_scope=1（全部数据权限）或 user_type=1
   * 兼容 camelCase / snake_case
   */
  const isSuperAdmin = computed(() => {
    const dataScope = userInfo?.dataScope ?? userInfo?.data_scope;
    const userType = userInfo?.userType ?? userInfo?.user_type;
    return dataScope === 1 || userType === 1;
  });

  /**
   * 拦截纯业务操作（商机、合同、订单、报价单、发货单等）
   * 超管完全不能创建，弹出 Modal 提示
   *
   * @param moduleName 模块名称，如"商机""合同"
   * @returns true 表示已拦截（应 return），false 表示放行
   */
  function guardBusiness(moduleName: string): boolean {
    if (!isSuperAdmin.value) return false;

    Modal.warning({
      title: `${moduleName}为业务单据`,
      content: `超级管理员负责系统管理，不参与业务操作。请使用业务账号（销售总监、销售经理、业务员等）登录后创建${moduleName}。`,
      okText: '我知道了',
    });
    return true;
  }

  /**
   * 基础数据（客户、联系人）允许超管创建，但提醒分配负责人
   * 如果超管自己作为负责人，弹出 message 提示并拦截
   *
   * @param moduleName 模块名称
   * @param assignedToId 当前设置的负责人ID
   * @param currentUserId 当前用户ID
   * @returns true 表示已拦截，false 表示放行
   */
  function guardAssignTo(
    moduleName: string,
    assignedToId: any,
    currentUserId: any,
  ): boolean {
    if (!isSuperAdmin.value) return false;

    // 超管可以创建，但不能自己是负责人
    if (
      !assignedToId ||
      String(assignedToId) === String(currentUserId) ||
      assignedToId === 0
    ) {
      message.warning(
        `${moduleName}需要分配给业务人员负责，超级管理员不参与业务。请选择业务人员作为负责人。`,
      );
      return true;
    }

    return false;
  }

  return {
    isSuperAdmin,
    guardBusiness,
    guardAssignTo,
  };
}
